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
}

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
                json!({ "success": false, "error": format!("router '{id}' not found in configuration") }),
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
                StatusCode::BAD_GATEWAY,
                json!({
                    "success": false,
                    "error": format!("router authentication failed: {m}"),
                    "code": "AUTH_FAILED"
                }),
            ),
            ApiError::RouterFatal(m) => (
                StatusCode::BAD_GATEWAY,
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
        };
        (status, Json(body)).into_response()
    }
}
