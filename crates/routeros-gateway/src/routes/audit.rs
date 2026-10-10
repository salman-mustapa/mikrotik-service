use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{compute_tenant_hash, AppState, AuditLogEntry};

#[derive(Deserialize, Debug, Default)]
pub struct AuditFilterQuery {
    pub router_id: Option<String>,
    pub status: Option<String>, // "all", "success", "failed"
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub min_duration_ms: Option<f64>,
}

/// GET /api/v1/audit/logs - Strictly isolated audit logs for the caller's tenant
pub async fn get_audit_logs(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuditFilterQuery>,
) -> Result<Json<Value>, ApiError> {
    let tenant_ctx = headers.get("x-tenant-id").and_then(|v| v.to_str().ok());
    let caller_tenant_hash = compute_tenant_hash(&st.token, tenant_ctx);

    let logs_guard = st.audit_logs.lock().await;

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0);

    let mut matched: Vec<AuditLogEntry> = logs_guard
        .iter()
        // Strict security boundary: Zero-leak multi-tenant segregation
        .filter(|e| e.tenant_hash == caller_tenant_hash)
        .filter(|e| {
            if let Some(ref rid) = query.router_id {
                e.router_target == *rid
            } else {
                true
            }
        })
        .filter(|e| match query.status.as_deref() {
            Some("success") => e.success,
            Some("failed") => !e.success,
            _ => true,
        })
        .filter(|e| {
            if let Some(min_dur) = query.min_duration_ms {
                e.duration_ms >= min_dur
            } else {
                true
            }
        })
        .cloned()
        .collect();

    // Reverse order: Most recent executions first
    matched.reverse();

    let total_matched = matched.len();
    let page: Vec<AuditLogEntry> = matched
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(json!({
        "success": true,
        "tenant_scope": caller_tenant_hash,
        "total_matched": total_matched,
        "returned": page.len(),
        "limit": limit,
        "offset": offset,
        "logs": page
    })))
}

/// GET /api/v1/audit/performance-stats - Latency and throughput analytics per tenant
pub async fn get_performance_stats(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let tenant_ctx = headers.get("x-tenant-id").and_then(|v| v.to_str().ok());
    let caller_tenant_hash = compute_tenant_hash(&st.token, tenant_ctx);

    let logs_guard = st.audit_logs.lock().await;

    let tenant_logs: Vec<&AuditLogEntry> = logs_guard
        .iter()
        .filter(|e| e.tenant_hash == caller_tenant_hash)
        .collect();

    if tenant_logs.is_empty() {
        return Ok(Json(json!({
            "success": true,
            "tenant_scope": caller_tenant_hash,
            "total_requests": 0,
            "message": "Belum ada riwayat eksekusi untuk scope tenant ini"
        })));
    }

    let total_requests = tenant_logs.len();
    let success_count = tenant_logs.iter().filter(|e| e.success).count();
    let error_count = total_requests - success_count;
    let success_rate = (success_count as f64 / total_requests as f64) * 100.0;

    let mut durations: Vec<f64> = tenant_logs.iter().map(|e| e.duration_ms).collect();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min_duration_ms = durations.first().copied().unwrap_or(0.0);
    let max_duration_ms = durations.last().copied().unwrap_or(0.0);
    let sum_duration: f64 = durations.iter().sum();
    let avg_duration_ms = sum_duration / total_requests as f64;

    let p95_index = ((total_requests as f64 * 0.95).ceil() as usize).saturating_sub(1);
    let p95_duration_ms = durations.get(p95_index).copied().unwrap_or(max_duration_ms);

    let mut routers_set = std::collections::HashSet::new();
    for l in &tenant_logs {
        routers_set.insert(l.router_target.clone());
    }
    let mut active_routers: Vec<String> = routers_set.into_iter().collect();
    active_routers.sort();

    Ok(Json(json!({
        "success": true,
        "tenant_scope": caller_tenant_hash,
        "total_requests": total_requests,
        "success_count": success_count,
        "error_count": error_count,
        "success_rate_percent": format!("{:.2}%", success_rate),
        "latency_metrics": {
            "avg_ms": format!("{:.2} ms", avg_duration_ms),
            "p95_ms": format!("{:.2} ms", p95_duration_ms),
            "min_ms": format!("{:.2} ms", min_duration_ms),
            "max_ms": format!("{:.2} ms", max_duration_ms),
        },
        "active_routers_count": active_routers.len(),
        "active_routers": active_routers
    })))
}

/// POST /api/v1/audit/clear - Clear execution logs exclusively for the caller's tenant
pub async fn clear_audit_logs(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let tenant_ctx = headers.get("x-tenant-id").and_then(|v| v.to_str().ok());
    let caller_tenant_hash = compute_tenant_hash(&st.token, tenant_ctx);

    let mut logs_guard = st.audit_logs.lock().await;
    let initial_count = logs_guard.len();

    // Retain only logs that DO NOT belong to caller's tenant
    logs_guard.retain(|e| e.tenant_hash != caller_tenant_hash);

    let purged_count = initial_count - logs_guard.len();

    Ok(Json(json!({
        "success": true,
        "tenant_scope": caller_tenant_hash,
        "purged_records": purged_count,
        "message": format!("Berhasil menghapus {} catatan log audit milik tenant!", purged_count)
    })))
}
