use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Path, Query, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::{Stream, StreamExt};
use routeros_core::{build_command, Client, Error};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct Config {
    #[serde(default = "default_listen")]
    listen: String,
    api_token: String,
    #[serde(default, rename = "router")]
    routers: Vec<RouterCfg>,
}

fn default_listen() -> String {
    "127.0.0.1:8080".into()
}

#[derive(Deserialize, Clone)]
pub struct RouterCfg {
    id: String,
    addr: String,
    user: String,
    #[serde(default)]
    password: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DynamicTarget {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub user: String,
    #[serde(default)]
    pub password: String,
}

fn default_port() -> u16 {
    8728
}

struct Slot {
    cfg: RouterCfg,
    client: Mutex<Option<Client>>,
}

struct AppState {
    token: String,
    routers: HashMap<String, Slot>,
    /// Global dynamic connection pool: key = "user@host:port:password"
    dynamic_pool: Mutex<HashMap<String, Client>>,
}

impl AppState {
    /// Return a live client for a configured router ID.
    async fn client(&self, id: &str) -> Result<Client, ApiError> {
        let slot = self.routers.get(id).ok_or(ApiError::UnknownRouter)?;
        let mut guard = slot.client.lock().await;
        if let Some(c) = guard.as_ref().filter(|c| !c.is_closed()) {
            return Ok(c.clone());
        }
        tracing::info!(router = id, "connecting configured router");
        let c = Client::connect(&slot.cfg.addr, &slot.cfg.user, &slot.cfg.password).await?;
        *guard = Some(c.clone());
        Ok(c)
    }

    /// Return a live client from the dynamic connection pool (or connect and cache it).
    async fn dynamic_client(&self, target: &DynamicTarget) -> Result<Client, ApiError> {
        let key = format!("{}@{}:{}:{}", target.user, target.host, target.port, target.password);
        let mut pool = self.dynamic_pool.lock().await;
        if let Some(c) = pool.get(&key).filter(|c| !c.is_closed()) {
            return Ok(c.clone());
        }
        let addr = format!("{}:{}", target.host, target.port);
        tracing::info!(addr = %addr, user = %target.user, "establishing dynamic pooled connection");
        let c = Client::connect(&addr, &target.user, &target.password).await?;
        pool.insert(key, c.clone());
        Ok(c)
    }
}

enum ApiError {
    UnknownRouter,
    BadRequest(String),
    Router(Error),
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError::Router(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::UnknownRouter => (StatusCode::NOT_FOUND, json!({ "error": "unknown router" })),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, json!({ "error": m })),
            ApiError::Router(Error::Trap { message, category }) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": message, "category": category }),
            ),
            ApiError::Router(Error::Login(m)) => (
                StatusCode::BAD_GATEWAY,
                json!({ "error": format!("router login failed: {m}") }),
            ),
            ApiError::Router(e) => (StatusCode::BAD_GATEWAY, json!({ "error": e.to_string() })),
        };
        (status, Json(body)).into_response()
    }
}

fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn auth(State(st): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|t| ct_eq(t, &st.token));
    if ok {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, Json(json!({ "error": "unauthorized" }))).into_response()
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[derive(Deserialize)]
struct StaticCommandReq {
    command: String,
    #[serde(default)]
    args: Map<String, Value>,
}

#[derive(Deserialize)]
struct DynamicCommandReq {
    /// Optional: target router definition. If omitted, falls back to "main" router from config.
    router: Option<DynamicTarget>,
    command: String,
    #[serde(default)]
    args: Map<String, Value>,
}

/// POST /routers/:id/command (Static config)
async fn run_static_command(
    State(st): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<StaticCommandReq>,
) -> Result<Json<Vec<HashMap<String, String>>>, ApiError> {
    if !req.command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }
    let client = st.client(&id).await?;
    let words = build_command(&req.command, req.args.iter().map(|(k, v)| (k.as_str(), value_to_string(v))));
    let rows = client.run(words).await?;
    Ok(Json(rows.into_iter().map(|r| r.attrs).collect()))
}

/// POST /api/v1/command (Universal Dynamic Endpoint)
async fn run_dynamic_command(
    State(st): State<Arc<AppState>>,
    Json(req): Json<DynamicCommandReq>,
) -> Result<Json<Vec<HashMap<String, String>>>, ApiError> {
    if !req.command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }
    let client = match &req.router {
        Some(target) => st.dynamic_client(target).await?,
        None => st.client("main").await?,
    };
    let words = build_command(&req.command, req.args.iter().map(|(k, v)| (k.as_str(), value_to_string(v))));
    let rows = client.run(words).await?;
    Ok(Json(rows.into_iter().map(|r| r.attrs).collect()))
}

/// GET /routers/:id/listen?command=/interface/monitor-traffic&interface=ether1
async fn static_listen(
    State(st): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(mut q): Query<Vec<(String, String)>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let pos = q
        .iter()
        .position(|(k, _)| k == "command")
        .ok_or_else(|| ApiError::BadRequest("missing `command` query parameter".into()))?;
    let (_, command) = q.remove(pos);
    if !command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }

    let client = st.client(&id).await?;
    let sub = client.listen(build_command(&command, q)).await?;

    let stream = sub.map(|item| {
        Ok(match item {
            Ok(s) => Event::default().json_data(&s.attrs).unwrap_or_default(),
            Err(e) => Event::default().event("error").data(e.to_string()),
        })
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// GET /api/v1/listen?host=...&port=...&user=...&password=...&command=/interface/monitor-traffic&interface=ether1
async fn dynamic_listen(
    State(st): State<Arc<AppState>>,
    Query(mut q): Query<Vec<(String, String)>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    // Extract dynamic connection params if present
    let extract = |key: &str, q: &mut Vec<(String, String)>| {
        q.iter().position(|(k, _)| k == key).map(|pos| q.remove(pos).1)
    };

    let host = extract("host", &mut q);
    let port = extract("port", &mut q).and_then(|p| p.parse::<u16>().ok()).unwrap_or(8728);
    let user = extract("user", &mut q);
    let password = extract("password", &mut q).unwrap_or_default();

    let pos = q
        .iter()
        .position(|(k, _)| k == "command")
        .ok_or_else(|| ApiError::BadRequest("missing `command` query parameter".into()))?;
    let (_, command) = q.remove(pos);
    if !command.starts_with('/') {
        return Err(ApiError::BadRequest("command must start with '/'".into()));
    }

    let client = match (host, user) {
        (Some(host), Some(user)) => {
            st.dynamic_client(&DynamicTarget {
                host,
                port,
                user,
                password,
            })
            .await?
        }
        _ => st.client("main").await?,
    };

    let sub = client.listen(build_command(&command, q)).await?;

    let stream = sub.map(|item| {
        Ok(match item {
            Ok(s) => Event::default().json_data(&s.attrs).unwrap_or_default(),
            Err(e) => Event::default().event("error").data(e.to_string()),
        })
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "service": "mikrotik-rust-gateway" }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let path = std::env::args().nth(1).unwrap_or_else(|| "config.toml".into());
    let cfg: Config = toml::from_str(&std::fs::read_to_string(&path)?)?;

    let state = Arc::new(AppState {
        token: cfg.api_token,
        routers: cfg
            .routers
            .into_iter()
            .map(|r| (r.id.clone(), Slot { cfg: r, client: Mutex::new(None) }))
            .collect(),
        dynamic_pool: Mutex::new(HashMap::new()),
    });

    let protected = Router::new()
        // Universal dynamic endpoints:
        .route("/api/v1/command", post(run_dynamic_command))
        .route("/api/v1/listen", get(dynamic_listen))
        // Static configured endpoints:
        .route("/routers/:id/command", post(run_static_command))
        .route("/routers/:id/listen", get(static_listen))
        .layer(middleware::from_fn_with_state(state.clone(), auth));

    let app = Router::new()
        .route("/health", get(health))
        .merge(protected)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.listen).await?;
    tracing::info!("listening on http://{}", cfg.listen);
    axum::serve(listener, app).await?;
    Ok(())
}
