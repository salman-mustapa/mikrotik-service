use std::sync::Arc;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::ApiError;
use crate::state::{is_blocked_host, AppState, RouterTarget};

#[derive(Deserialize)]
pub struct IssueTicketReq {
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: Option<String>,
    #[serde(default)]
    pub pass: Option<String>,
    #[serde(default)]
    pub ttl_secs: Option<u64>,
}

#[derive(Serialize)]
pub struct IssueTicketResp {
    pub success: bool,
    pub ticket: String,
    pub expires_in_secs: u64,
    pub host: String,
    pub user: String,
    pub message: String,
}

pub async fn issue_ticket(
    State(st): State<Arc<AppState>>,
    Json(req): Json<IssueTicketReq>,
) -> Result<Json<IssueTicketResp>, ApiError> {
    let host = req.host.trim();
    if host.is_empty() {
        return Err(ApiError::BadRequest("Missing router host".into()));
    }
    if is_blocked_host(host) {
        return Err(ApiError::Forbidden(
            "Target router host is blocked by SSRF security policy (loopback, link-local, or cloud metadata target).".into()
        ));
    }
    let user = req.user.trim();
    if user.is_empty() {
        return Err(ApiError::BadRequest("Missing router username".into()));
    }
    let password = req.password.or(req.pass);
    let port = req.port.unwrap_or(8728);
    let ttl = req.ttl_secs.unwrap_or(3600);

    let target = RouterTarget {
        host: Some(host.to_string()),
        port: Some(port),
        user: Some(user.to_string()),
        password,
    };

    let ticket = st.issue_ticket(target, ttl).await;

    Ok(Json(IssueTicketResp {
        success: true,
        ticket,
        expires_in_secs: ttl.clamp(60, 86400),
        host: host.to_string(),
        user: user.to_string(),
        message: "Ephemeral connection ticket created in RAM. Embed this ticket in frontend or WebSocket without leaking router password.".into(),
    }))
}

#[derive(Deserialize)]
pub struct VerifyTicketQuery {
    pub ticket: String,
}

pub async fn verify_ticket(
    State(st): State<Arc<AppState>>,
    Query(q): Query<VerifyTicketQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if let Some(target) = st.resolve_ticket(&q.ticket).await {
        Ok(Json(json!({
            "valid": true,
            "host": target.host,
            "port": target.port,
            "user": target.user,
        })))
    } else {
        Err(ApiError::NotFound("Ticket invalid, expired, or already revoked".into()))
    }
}
