use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use routeros_core::Error as CoreError;
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    UnknownRouter(String),
    RouterTrap {
        message: String,
        category: Option<String>,
    },
    RouterLogin(String),
    RouterFatal(String),
    Internal(String),
    NotFound(String),
    Conflict(String),
    PayloadTooLarge(String),
    UnsupportedMedia(String),
    Forbidden(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::BadRequest(m) => write!(f, "Bad request: {m}"),
            ApiError::UnknownRouter(id) => write!(f, "Unknown router: {id}"),
            ApiError::RouterTrap { message, .. } => write!(f, "Router trap: {message}"),
            ApiError::RouterLogin(m) => write!(f, "Router login error: {m}"),
            ApiError::RouterFatal(m) => write!(f, "Router fatal error: {m}"),
            ApiError::Internal(m) => write!(f, "Internal error: {m}"),
            ApiError::NotFound(m) => write!(f, "Not found: {m}"),
            ApiError::Conflict(m) => write!(f, "Conflict: {m}"),
            ApiError::PayloadTooLarge(m) => write!(f, "Payload too large: {m}"),
            ApiError::UnsupportedMedia(m) => write!(f, "Unsupported media: {m}"),
            ApiError::Forbidden(m) => write!(f, "Forbidden: {m}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<CoreError> for ApiError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Trap { message, category } => ApiError::RouterTrap { message, category },
            CoreError::Login(m) => ApiError::RouterLogin(m),
            CoreError::Fatal(m) => ApiError::RouterFatal(m),
            CoreError::Closed => ApiError::RouterFatal("connection closed".into()),
            CoreError::Protocol(m) => ApiError::BadRequest(format!("protocol error: {m}")),
            CoreError::Io(e) => ApiError::Internal(format!("network error: {e}")),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::BadRequest(m) => (
                StatusCode::BAD_REQUEST,
                json!({ "success": false, "error": m }),
            ),
            ApiError::UnknownRouter(id) => (
                StatusCode::NOT_FOUND,
                json!({
                    "success": false,
                    "error": format!("router '{id}' not found in configuration. Sertakan target router via HTTP Headers (X-Router-Host, X-Router-Port, X-Router-User, X-Router-Pass), Query Params (?host=..&user=..), atau daftarkan router slot di config.toml"),
                    "code": "ROUTER_NOT_FOUND"
                }),
            ),
            ApiError::RouterTrap { message, category } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "success": false,
                    "error": message,
                    "category": category,
                    "code": "ROUTER_TRAP"
                }),
            ),
            ApiError::RouterLogin(m) => (
                StatusCode::UNAUTHORIZED,
                json!({
                    "success": false,
                    "error": format!("router authentication failed: {m}"),
                    "code": "AUTH_FAILED"
                }),
            ),
            ApiError::RouterFatal(m) => (
                StatusCode::SERVICE_UNAVAILABLE,
                json!({
                    "success": false,
                    "error": format!("router connection dropped: {m}"),
                    "code": "CONNECTION_LOST"
                }),
            ),
            ApiError::Internal(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success": false, "error": m, "code": "INTERNAL_ERROR" }),
            ),
            ApiError::NotFound(m) => (
                StatusCode::NOT_FOUND,
                json!({ "success": false, "error": m, "code": "NOT_FOUND" }),
            ),
            ApiError::Conflict(m) => (
                StatusCode::CONFLICT,
                json!({ "success": false, "error": m, "code": "VERSION_CONFLICT" }),
            ),
            ApiError::PayloadTooLarge(m) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                json!({ "success": false, "error": m, "code": "PAYLOAD_TOO_LARGE" }),
            ),
            ApiError::UnsupportedMedia(m) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                json!({ "success": false, "error": m, "code": "UNSUPPORTED_MEDIA" }),
            ),
            ApiError::Forbidden(m) => (
                StatusCode::FORBIDDEN,
                json!({ "success": false, "error": m, "code": "FORBIDDEN" }),
            ),
        };
        (status, Json(body)).into_response()
    }
}
