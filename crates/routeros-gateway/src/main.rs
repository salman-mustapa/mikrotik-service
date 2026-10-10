pub mod error;
pub mod routes;
pub mod state;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use state::{AppState, Config};

fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

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

async fn auth(State(st): State<Arc<AppState>>, mut req: Request, next: Next) -> Response {
    let header_ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| ct_eq(t, &st.token));

    let mut query_params = std::collections::HashMap::new();
    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                query_params.insert(k.to_lowercase(), percent_decode(v));
            }
        }
    }

    let query_ok = if !header_ok {
        query_params.get("token").is_some_and(|t| ct_eq(t, &st.token))
    } else {
        false
    };

    if header_ok || query_ok {
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
        if !headers.contains_key("x-router-pass") {
            if let Some(p) = query_params.get("pass").or_else(|| query_params.get("password")).or_else(|| query_params.get("router_pass")) {
                if let Ok(val) = header::HeaderValue::from_str(p) {
                    headers.insert(header::HeaderName::from_static("x-router-pass"), val);
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

        let mut resp = next.run(req).await;
        resp.headers_mut().insert(
            header::HeaderName::from_static("x-content-type-options"),
            header::HeaderValue::from_static("nosniff"),
        );
        resp.headers_mut().insert(
            header::HeaderName::from_static("x-frame-options"),
            header::HeaderValue::from_static("SAMEORIGIN"),
        );
        resp
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "unauthorized: invalid or missing Bearer token" })),
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

async fn playground() -> Html<&'static str> {
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
            listen: "0.0.0.0:8080".into(),
            api_token: "change-me-to-a-long-random-string".into(),
            routers: vec![],
        }
    };

    if let Ok(env_listen) = std::env::var("GATEWAY_LISTEN") {
        cfg.listen = env_listen;
    }
    if let Ok(env_token) = std::env::var("GATEWAY_TOKEN") {
        cfg.api_token = env_token;
    }

    let listen_addr = cfg.listen.clone();

    let state = Arc::new(AppState::new(cfg));

    // Protected API router with bearer auth
    let protected = routes::build_api_router(state.clone())
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    // Combined router with web playground, health check, visualizer, websocket & permissive CORS
    let app = Router::new()
        .route("/", get(playground))
        .route("/health", get(health))
        .route("/docs", get(routes::docs::docs_page))
        .route("/documentation", get(routes::docs::docs_page))
        .route("/api/docs", get(routes::docs::docs_page))
        .route("/api/v1/spec", get(routes::docs::api_spec_json))
        .route("/topology", get(routes::visualizer::visualizer_page))
        .route("/visualizer", get(routes::visualizer::visualizer_page))
        .route("/sdk/mikrotik-widget.js", get(routes::visualizer::sdk_script))
        .route("/ws", get(routes::ws::ws_handler))
        .route("/metrics", get(routes::metrics::prometheus_metrics))
        .merge(protected)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("MikroTik Universal Gateway listening on http://{}", listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
