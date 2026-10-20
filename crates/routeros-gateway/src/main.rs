pub mod error;
pub mod map_store;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use state::{AppState, Config};

fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let mut chars = s.bytes();
    while let Some(b) = chars.next() {
        match b {
            b'+' => bytes.push(b' '),
            b'%' => {
                let h1 = chars.next();
                let h2 = chars.next();
                if let (Some(h1), Some(h2)) = (h1, h2) {
                    let hex_str = [h1, h2];
                    if let Ok(val) = u8::from_str_radix(std::str::from_utf8(&hex_str).unwrap_or(""), 16) {
                        bytes.push(val);
                        continue;
                    }
                }
                bytes.push(b'%');
            }
            _ => bytes.push(b),
        }
    }
    String::from_utf8_lossy(&bytes).to_string()
}

fn current_iso_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = (days as i64) + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", y, m, d, hours, mins, seconds, millis)
}

/// Paths a read-only viewer token may GET. Mirrors the allow-list enforced for `/ws` in `routes::ws`.
fn viewer_may_access(method: &axum::http::Method, path: &str) -> bool {
    if path == "/ws" {
        return method == axum::http::Method::GET;
    }
    if method != axum::http::Method::GET {
        return false;
    }
    path == "/api/v1/map/scene"
        || path == "/api/v1/map/icons"
        || path == "/api/v1/map/layouts"
        || path.starts_with("/api/v1/map/layouts/")
}

fn get_client_ip(req: &Request, trust_forwarded: bool) -> std::net::IpAddr {
    if trust_forwarded {
        if let Some(h) = req.headers().get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
            if let Some(first) = h.split(',').next() {
                if let Ok(ip) = first.trim().parse::<std::net::IpAddr>() {
                    return ip;
                }
            }
        }
        if let Some(h) = req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()) {
            if let Ok(ip) = h.trim().parse::<std::net::IpAddr>() {
                return ip;
            }
        }
    }

    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))
}

async fn global_security_headers(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();

    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        header::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        header::HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        header::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        header::HeaderValue::from_static("accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"),
    );

    // Prevent caching of sensitive API responses and tokens
    if path.starts_with("/api/") || path == "/metrics" || path == "/ws" {
        headers.insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
        );
        headers.insert(
            header::PRAGMA,
            header::HeaderValue::from_static("no-cache"),
        );
    }

    resp
}

async fn auth(State(st): State<Arc<AppState>>, mut req: Request, next: Next) -> Response {
    let client_ip = get_client_ip(&req, st.trust_forwarded_for);

    // Rate-limit failed authentication attempts per IP (Finding 2)
    if let Err(retry_after) = st.auth_rate_limiter.check(client_ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "success": false,
                "error": format!("Too many failed authentication attempts from this IP. Please retry after {} seconds.", retry_after),
                "code": "RATE_LIMITED",
                "retry_after_secs": retry_after
            })),
        )
            .into_response();
    }

    let header_role = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .and_then(|t| st.role_for_token(t));

    let mut query_params = std::collections::HashMap::new();
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                query_params.insert(k.to_lowercase(), percent_decode(v));
            }
        }
    }

    // Token via query string is strictly restricted to streaming/SSE endpoints (where EventSource cannot set headers)
    let (is_streaming_endpoint, is_ticket_endpoint) = {
        let p = req.uri().path();
        (
            p.starts_with("/api/v1/listen")
                || p.starts_with("/api/v1/logs/stream")
                || p.starts_with("/api/v1/interfaces/stream")
                || p == "/ws",
            p.starts_with("/api/v1/ticket")
                || p.starts_with("/api/v1/sessions/connect"),
        )
    };
    let query_role = if header_role.is_none() && is_streaming_endpoint {
        query_params.get("token").and_then(|t| st.role_for_token(t))
    } else {
        None
    };

    // Check Ephemeral Connection Ticket in header or query (?ticket=)
    let ticket_str = req
        .headers()
        .get("x-connection-ticket")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| query_params.get("ticket").cloned());

    let mut ticket_target = None;
    if let Some(t) = &ticket_str {
        if let Some(target) = st.resolve_ticket(t).await {
            ticket_target = Some(target);
        }
    }

    // Inject ticket target into headers if ticket resolved
    if let Some(t) = &ticket_target {
        let headers = req.headers_mut();
        if let Some(h) = &t.host {
            if let Ok(val) = header::HeaderValue::from_str(h) {
                headers.insert(header::HeaderName::from_static("x-router-host"), val);
            }
        }
        if let Some(p) = t.port {
            if let Ok(val) = header::HeaderValue::from_str(&p.to_string()) {
                headers.insert(header::HeaderName::from_static("x-router-port"), val);
            }
        }
        if let Some(u) = &t.user {
            if let Ok(val) = header::HeaderValue::from_str(u) {
                headers.insert(header::HeaderName::from_static("x-router-user"), val);
            }
        }
        if let Some(p) = &t.password {
            if let Ok(val) = header::HeaderValue::from_str(p) {
                headers.insert(header::HeaderName::from_static("x-router-pass"), val);
            }
        }
    }

    // Populate router headers from query if not already in headers
    let headers = req.headers_mut();
    if !headers.contains_key("x-router-host") {
        if let Some(h) = query_params.get("host").or_else(|| query_params.get("router_host")) {
            if let Ok(val) = header::HeaderValue::from_str(h) {
                headers.insert(header::HeaderName::from_static("x-router-host"), val);
            }
        }
    }
    if !headers.contains_key("x-router-port") {
        if let Some(p) = query_params.get("port").or_else(|| query_params.get("router_port")) {
            if let Ok(val) = header::HeaderValue::from_str(p) {
                headers.insert(header::HeaderName::from_static("x-router-port"), val);
            }
        }
    }
    if !headers.contains_key("x-router-user") {
        if let Some(u) = query_params.get("user").or_else(|| query_params.get("router_user")) {
            if let Ok(val) = header::HeaderValue::from_str(u) {
                headers.insert(header::HeaderName::from_static("x-router-user"), val);
            }
        }
    }
    if !headers.contains_key("x-router-id") {
        if let Some(id) = query_params.get("router_id").or_else(|| query_params.get("id")) {
            if let Ok(val) = header::HeaderValue::from_str(id) {
                headers.insert(header::HeaderName::from_static("x-router-id"), val);
            }
        }
    }

    // Determine authorization role:
    // 1. Explicit Bearer token match
    // 2. Ephemeral ticket match -> Role::Admin
    // 3. Ticket issuing endpoint -> Role::Admin (anyone can request ticket for their router)
    // 4. Direct router credentials provided (X-Router-Host & X-Router-User) -> Role::Admin
    // 5. Configured router target provided (X-Router-Id) -> Role::Admin
    let has_direct_router = req.headers().contains_key("x-router-host") && req.headers().contains_key("x-router-user");
    let has_configured_router = req.headers().contains_key("x-router-id");

    let role = header_role
        .or(query_role)
        .or_else(|| {
            if ticket_target.is_some() || is_ticket_endpoint || has_direct_router || has_configured_router {
                Some(state::Role::Admin)
            } else {
                None
            }
        });

    // A viewer token is read-only: GET on the map read endpoints (or the /ws upgrade, which enforces
    // its own per-action allow-list). Everything else is refused before it reaches any handler.
    if role == Some(state::Role::Viewer) && !viewer_may_access(req.method(), req.uri().path()) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({ "success": false, "error": "forbidden: this read-only viewer token can only read the network map", "code": "FORBIDDEN" })),
        )
            .into_response();
    }

    if role.is_some() {
        // Reset failure record on successful auth
        st.auth_rate_limiter.record_success(client_ip).await;

        if role == Some(state::Role::Viewer) {
            // A viewer may only use routers defined in config.toml (router_id), never ad-hoc host + credentials.
            let headers = req.headers_mut();
            for name in ["x-router-host", "x-router-port", "x-router-user", "x-router-pass"] {
                headers.remove(name);
            }
        }

        let start_time = std::time::Instant::now();
        let method = req.method().to_string();
        let endpoint = req.uri().path().to_string();
        let tenant_ctx = req.headers().get("x-tenant-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let tenant_hash = state::compute_tenant_hash(&st.token, tenant_ctx.as_deref());
        let router_target = req
            .headers()
            .get("x-router-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| {
                req.headers()
                    .get("x-router-host")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "main".to_string());
        let client_ip_str = Some(client_ip.to_string());

        let mut resp = next.run(req).await;
        let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let status_code = resp.status().as_u16();
        let success = resp.status().is_success();

        resp.headers_mut().insert(
            header::HeaderName::from_static("x-response-time"),
            header::HeaderValue::from_str(&format!("{:.2}ms", elapsed_ms)).unwrap_or_else(|_| header::HeaderValue::from_static("0ms")),
        );
        resp.headers_mut().insert(
            header::HeaderName::from_static("x-tenant-scope"),
            header::HeaderValue::from_str(&tenant_hash).unwrap_or_else(|_| header::HeaderValue::from_static("unknown")),
        );

        // Record audit log entry in segregated ring buffer if not the audit inspection endpoint itself
        if !endpoint.starts_with("/api/v1/audit") {
            let log_id = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
            let entry = state::AuditLogEntry {
                id: log_id,
                timestamp: current_iso_timestamp(),
                tenant_hash,
                router_target,
                method,
                endpoint,
                duration_ms: elapsed_ms,
                status_code,
                success,
                client_ip: client_ip_str,
            };
            st.record_audit_log(entry).await;
        }

        resp
    } else {
        // Record failed authentication attempt for rate limiting (Finding 2)
        st.auth_rate_limiter.record_failure(client_ip).await;

        let client_ip_str = Some(client_ip.to_string());
        let endpoint = req.uri().path().to_string();
        let method = req.method().to_string();
        let log_id = format!("{:016x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
        let entry = state::AuditLogEntry {
            id: log_id,
            timestamp: current_iso_timestamp(),
            tenant_hash: "unauthorized".into(),
            router_target: "none".into(),
            method,
            endpoint,
            duration_ms: 0.0,
            status_code: 401,
            success: false,
            client_ip: client_ip_str,
        };
        st.record_audit_log(entry).await;

        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "unauthorized: please provide router connection headers (X-Router-Host, X-Router-User) or connection ticket (?ticket=)",
                "code": "MISSING_ROUTER_AUTH"
            })),
        )
            .into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "mikrotik-universal-rust-gateway",
        "version": "0.2.0"
    }))
}

async fn playground() -> Response {
    routes::docs::console_page().await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".into());
    let mut cfg: Config = if std::path::Path::new(&path).exists() {
        toml::from_str(&std::fs::read_to_string(&path)?)?
    } else {
        Config {
            listen: "127.0.0.1:8080".into(),
            api_token: "change-me-to-a-long-random-string".into(),
            data_dir: state::default_data_dir(),
            viewer_token: None,
            routers: vec![],
            allow_dynamic_routers: true,
            allow_raw_command: false,
            allow_terminal_exec: false,
            allow_audit_clear: false,
            trust_forwarded_for: false,
            cors_allowed_origins: vec![],
            protect_docs: false,
            allow_system_reset: false,
            allow_system_reboot: true,
        }
    };

    if let Ok(env_listen) = std::env::var("GATEWAY_LISTEN") {
        cfg.listen = env_listen;
    }
    if let Ok(env_data) = std::env::var("GATEWAY_DATA_DIR") {
        if !env_data.trim().is_empty() {
            cfg.data_dir = env_data;
        }
    }
    if let Ok(env_viewer) = std::env::var("GATEWAY_VIEWER_TOKEN") {
        if !env_viewer.trim().is_empty() {
            cfg.viewer_token = Some(env_viewer);
        }
    }
    if let Ok(env_token) = std::env::var("GATEWAY_TOKEN") {
        cfg.api_token = env_token;
    }
    if let Ok(env_cors) = std::env::var("GATEWAY_CORS_ORIGINS") {
        cfg.cors_allowed_origins = env_cors
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    if let Ok(env_dyn) = std::env::var("GATEWAY_ALLOW_DYNAMIC_ROUTERS") {
        cfg.allow_dynamic_routers = env_dyn == "1" || env_dyn == "true";
    } else {
        cfg.allow_dynamic_routers = true;
    }
    if let Ok(env_raw) = std::env::var("GATEWAY_ALLOW_RAW_COMMAND") {
        cfg.allow_raw_command = env_raw == "1" || env_raw == "true";
    }
    if let Ok(env_term) = std::env::var("GATEWAY_ALLOW_TERMINAL_EXEC") {
        cfg.allow_terminal_exec = env_term == "1" || env_term == "true";
    }
    if let Ok(env_clear) = std::env::var("GATEWAY_ALLOW_AUDIT_CLEAR") {
        cfg.allow_audit_clear = env_clear == "1" || env_clear == "true";
    }
    if let Ok(env_fwd) = std::env::var("GATEWAY_TRUST_FORWARDED_FOR") {
        cfg.trust_forwarded_for = env_fwd == "1" || env_fwd == "true";
    }
    if let Ok(env_pdoc) = std::env::var("GATEWAY_PROTECT_DOCS") {
        cfg.protect_docs = env_pdoc == "1" || env_pdoc == "true";
    }
    if let Ok(env_reset) = std::env::var("GATEWAY_ALLOW_SYSTEM_RESET") {
        cfg.allow_system_reset = env_reset == "1" || env_reset == "true";
    }
    if let Ok(env_reboot) = std::env::var("GATEWAY_ALLOW_SYSTEM_REBOOT") {
        cfg.allow_system_reboot = env_reboot == "1" || env_reboot == "true";
    }

    let is_public_mode = cfg.api_token == "change-me-to-a-long-random-string" || cfg.api_token.trim().len() < 32;
    if is_public_mode {
        tracing::info!(
            "🌐 PUBLIC SERVICE MODE: Default or blank master token detected. Public dynamic router access, testing UI, and ephemeral session tickets are fully active."
        );
    }

    if let Some(v) = &cfg.viewer_token {
        if v.trim().len() < 16 || v.trim() == cfg.api_token {
            tracing::warn!("viewer_token ignored: it must be at least 16 characters and different from api_token");
        } else {
            tracing::info!("read-only viewer_token enabled (network map GET + /ws subscribe_map only)");
        }
    }

    let listen_addr = cfg.listen.clone();

    // CORS configuration (Finding 11)
    let cors = if cfg.cors_allowed_origins.iter().any(|o| o == "*") || (cfg.cors_allowed_origins.is_empty() && is_public_mode) {
        tower_http::cors::CorsLayer::permissive()
    } else if cfg.cors_allowed_origins.is_empty() {
        tower_http::cors::CorsLayer::new()
            .allow_origin([
                "http://localhost:8080".parse().unwrap(),
                "http://127.0.0.1:8080".parse().unwrap(),
                "http://localhost:3000".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
            ])
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                axum::http::HeaderName::from_static("x-router-host"),
                axum::http::HeaderName::from_static("x-router-port"),
                axum::http::HeaderName::from_static("x-router-user"),
                axum::http::HeaderName::from_static("x-router-pass"),
                axum::http::HeaderName::from_static("x-router-id"),
                axum::http::HeaderName::from_static("x-tenant-id"),
                axum::http::HeaderName::from_static("x-connection-ticket"),
            ])
    } else {
        let mut origins = Vec::new();
        for o in &cfg.cors_allowed_origins {
            if let Ok(hv) = o.parse::<axum::http::HeaderValue>() {
                origins.push(hv);
            }
        }
        tower_http::cors::CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers([
                header::AUTHORIZATION,
                header::CONTENT_TYPE,
                axum::http::HeaderName::from_static("x-router-host"),
                axum::http::HeaderName::from_static("x-router-port"),
                axum::http::HeaderName::from_static("x-router-user"),
                axum::http::HeaderName::from_static("x-router-pass"),
                axum::http::HeaderName::from_static("x-router-id"),
                axum::http::HeaderName::from_static("x-tenant-id"),
                axum::http::HeaderName::from_static("x-connection-ticket"),
            ])
    };

    let protect_docs = cfg.protect_docs;
    let state = Arc::new(AppState::new(cfg));

    // Base API router
    let mut api_router = routes::build_api_router(state.clone());
    if protect_docs {
        api_router = api_router
            .route("/docs", get(routes::docs::docs_page))
            .route("/documentation", get(routes::docs::docs_page))
            .route("/api/docs", get(routes::docs::docs_page))
            .route("/api/v1/spec", get(routes::docs::api_spec_json));
    }

    // Protected API router with bearer auth
    let protected = api_router.layer(middleware::from_fn_with_state(state.clone(), auth));

    // Base combined router
    let mut app = Router::new()
        .route("/", get(playground))
        .route("/health", get(health))
        .route("/topology", get(routes::visualizer::visualizer_page))
        .route("/visualizer", get(routes::visualizer::visualizer_page))
        .route("/sdk/mikrotik-widget.js", get(routes::visualizer::sdk_script))
        .route("/map", get(routes::map::page))
        .route("/sdk/mikrotik-map.js", get(routes::map::widget_js))
        .route("/sdk/map-page.js", get(routes::map::page_js))
        .route("/ws", get(routes::ws::ws_handler))
        .route("/metrics", get(routes::metrics::prometheus_metrics));

    if !protect_docs {
        app = app
            .route("/docs", get(routes::docs::docs_page))
            .route("/documentation", get(routes::docs::docs_page))
            .route("/api/docs", get(routes::docs::docs_page))
            .route("/api/v1/spec", get(routes::docs::api_spec_json));
    }

    let app = app
        .merge(protected)
        .layer(middleware::from_fn(global_security_headers))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("MikroTik Universal Gateway listening on http://{}", listen_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_builds_without_panic() {
        let cfg = Config {
            listen: "127.0.0.1:8080".into(),
            api_token: "test-token-012345678901234567890123456789".into(),
            data_dir: std::env::temp_dir().join("ros-gw-main-test").to_string_lossy().into_owned(),
            viewer_token: None,
            routers: vec![],
            allow_dynamic_routers: false,
            allow_raw_command: false,
            allow_terminal_exec: false,
            allow_audit_clear: false,
            trust_forwarded_for: false,
            cors_allowed_origins: vec![],
            protect_docs: false,
            allow_system_reset: false,
            allow_system_reboot: true,
        };
        let state = Arc::new(AppState::new(cfg));
        let protected = routes::build_api_router(state.clone());
        let _app = Router::new()
            .route("/", get(playground))
            .route("/health", get(health))
            .route("/docs", get(routes::docs::docs_page))
            .route("/documentation", get(routes::docs::docs_page))
            .route("/api/docs", get(routes::docs::docs_page))
            .route("/api/v1/spec", get(routes::docs::api_spec_json))
            .route("/topology", get(routes::visualizer::visualizer_page))
            .route("/visualizer", get(routes::visualizer::visualizer_page))
            .route("/sdk/mikrotik-widget.js", get(routes::visualizer::sdk_script))
            .route("/map", get(routes::map::page))
            .route("/sdk/mikrotik-map.js", get(routes::map::widget_js))
            .route("/sdk/map-page.js", get(routes::map::page_js))
            .route("/ws", get(routes::ws::ws_handler))
            .route("/metrics", get(routes::metrics::prometheus_metrics))
            .merge(protected);
    }

    #[test]
    fn test_constant_time_equality() {
        assert!(state::ct_eq("secure_token_123", "secure_token_123"));
        assert!(!state::ct_eq("secure_token_123", "secure_token_124"));
        assert!(!state::ct_eq("short", "longer_token"));
        assert!(!state::ct_eq("", "token"));
        assert!(state::ct_eq("", ""));
    }

    #[test]
    fn test_tenant_hash_isolation() {
        let h1 = state::compute_tenant_hash("token_a", Some("tenant_1"));
        let h2 = state::compute_tenant_hash("token_a", Some("tenant_2"));
        let h3 = state::compute_tenant_hash("token_b", Some("tenant_1"));
        let h1_again = state::compute_tenant_hash("token_a", Some("tenant_1"));

        assert_eq!(h1, h1_again);
        assert_ne!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn test_viewer_role_is_read_only() {
        use axum::http::Method;
        let mk = |viewer: Option<&str>| {
            AppState::new(Config {
                listen: "127.0.0.1:8080".into(),
                api_token: "admin-token-012345678901234567890123456789".into(),
                data_dir: std::env::temp_dir().join("ros-gw-role-test").to_string_lossy().into_owned(),
                viewer_token: viewer.map(|s| s.to_string()),
                routers: vec![],
                allow_dynamic_routers: false,
                allow_raw_command: false,
                allow_terminal_exec: false,
                allow_audit_clear: false,
                trust_forwarded_for: false,
                cors_allowed_origins: vec![],
                protect_docs: false,
                allow_system_reset: false,
                allow_system_reboot: true,
            })
        };
        let st = mk(Some("viewer-token-01234567890123456789"));
        assert_eq!(st.role_for_token("admin-token-012345678901234567890123456789"), Some(state::Role::Admin));
        assert_eq!(st.role_for_token("viewer-token-01234567890123456789"), Some(state::Role::Viewer));
        assert_eq!(st.role_for_token("nope"), None);
        assert_eq!(st.role_for_token(""), None);

        // unusable viewer tokens are ignored instead of weakening security
        assert!(mk(Some("short")).viewer_token.is_none());
        assert!(mk(Some("admin-token-012345678901234567890123456789")).viewer_token.is_none());
        assert!(mk(None).role_for_token("viewer-token-01234567890123456789").is_none());

        assert!(viewer_may_access(&Method::GET, "/api/v1/map/scene"));
        assert!(viewer_may_access(&Method::GET, "/api/v1/map/layouts/home"));
        assert!(viewer_may_access(&Method::GET, "/api/v1/map/layouts/home/background"));
        assert!(viewer_may_access(&Method::GET, "/ws"));
        assert!(!viewer_may_access(&Method::POST, "/api/v1/map/scene"));
        assert!(!viewer_may_access(&Method::PUT, "/api/v1/map/layouts/home"));
        assert!(!viewer_may_access(&Method::DELETE, "/api/v1/map/layouts/home"));
        assert!(!viewer_may_access(&Method::POST, "/api/v1/command"));
        assert!(!viewer_may_access(&Method::GET, "/api/v1/system/reboot"));
        assert!(!viewer_may_access(&Method::GET, "/api/v1/audit/logs"));
        assert!(!viewer_may_access(&Method::GET, "/api/v1/map/scene-evil"));
    }

    #[test]
    fn test_percent_decoding() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("user%2Bpass"), "user+pass");
        assert_eq!(percent_decode("clean_token_123"), "clean_token_123");
    }

    #[test]
    fn test_ssrf_host_blocking() {
        assert!(state::is_blocked_host("localhost"));
        assert!(state::is_blocked_host("127.0.0.1"));
        assert!(state::is_blocked_host("127.0.0.2"));
        assert!(state::is_blocked_host("169.254.169.254"));
        assert!(state::is_blocked_host("169.254.1.1"));
        assert!(state::is_blocked_host("metadata.google.internal"));
        assert!(state::is_blocked_host("instance-data"));
        assert!(state::is_blocked_host("::1"));

        // Normal private or public IP addresses must be allowed (when dynamic targets are enabled)
        assert!(!state::is_blocked_host("192.168.88.1"));
        assert!(!state::is_blocked_host("10.0.0.1"));
        assert!(!state::is_blocked_host("172.16.0.1"));
    }

    #[tokio::test]
    async fn test_auth_rate_limiter() {
        let limiter = state::AuthRateLimiter::new();
        let ip: std::net::IpAddr = "192.0.2.1".parse().unwrap();

        // 4 failed attempts should still be allowed
        for _ in 0..4 {
            assert!(limiter.check(ip).await.is_ok());
            limiter.record_failure(ip).await;
        }

        // 5th attempt: recorded failure
        limiter.record_failure(ip).await;

        // 6th attempt: should be blocked with rate limit error
        assert!(limiter.check(ip).await.is_err());

        // Successful auth resets the failure counter
        limiter.record_success(ip).await;
        assert!(limiter.check(ip).await.is_ok());
    }

    #[test]
    fn test_streaming_endpoints_detection() {
        let is_stream = |path: &str| {
            path.starts_with("/api/v1/listen")
                || path.starts_with("/api/v1/logs/stream")
                || path.starts_with("/api/v1/interfaces/stream")
                || path == "/ws"
        };

        assert!(is_stream("/api/v1/listen"));
        assert!(is_stream("/api/v1/logs/stream"));
        assert!(is_stream("/api/v1/interfaces/stream"));
        assert!(is_stream("/ws"));
        assert!(!is_stream("/api/v1/command"));
        assert!(!is_stream("/api/v1/batch"));
        assert!(!is_stream("/api/v1/system/reboot"));
        assert!(!is_stream("/api/v1/hotspot/users"));
    }
}
