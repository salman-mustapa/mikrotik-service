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

async fn auth(State(st): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    let header_ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| ct_eq(t, &st.token));

    let query_ok = if !header_ok {
        req.uri().query().and_then(|q| {
            q.split('&').find_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                if parts.next()? == "token" {
                    parts.next()
                } else {
                    None
                }
            })
        }).is_some_and(|t| ct_eq(t, &st.token))
    } else {
        false
    };

    if header_ok || query_ok {
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
        .merge(protected)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("MikroTik Universal Gateway listening on http://{}", listen_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
