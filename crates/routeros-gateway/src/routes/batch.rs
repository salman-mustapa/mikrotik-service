use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug)]
pub struct CommandItem {
    pub command: String,
    #[serde(default)]
    pub args: Map<String, Value>,
}

#[derive(Deserialize, Debug)]
pub struct BatchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub commands: Vec<CommandItem>,
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// POST /api/v1/batch - Eksekusi banyak perintah secara paralel/pipelined dalam 1 HTTP request
pub async fn execute_batch(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BatchReq>,
) -> Result<Json<Value>, ApiError> {
    if req.commands.is_empty() {
        return Err(ApiError::BadRequest("commands list cannot be empty".into()));
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut results = Vec::new();

    // Iterate through commands in sequence or concurrent futures over multiplexed socket
    for item in req.commands {
        if !item.command.starts_with('/') {
            results.push(json!({
                "command": item.command,
                "success": false,
                "error": "command must start with '/'"
            }));
            continue;
        }

        let words = build_command(
            &item.command,
            item.args.iter().map(|(k, v)| (k.as_str(), value_to_string(v))),
        );

        match client.run(words).await {
            Ok(rows) => {
                let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();
                results.push(json!({
                    "command": item.command,
                    "success": true,
                    "count": data.len(),
                    "data": data
                }));
            }
            Err(e) => {
                results.push(json!({
                    "command": item.command,
                    "success": false,
                    "error": e.to_string()
                }));
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "total": results.len(),
        "results": results
    })))
}
