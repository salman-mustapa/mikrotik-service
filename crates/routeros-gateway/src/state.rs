use std::collections::{HashMap, VecDeque};
use axum::http::HeaderMap;
use routeros_core::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use md5::{Digest, Md5};

use crate::error::ApiError;
use crate::map_store::MapStore;
use crate::routes::map::RateSnapshot;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuditLogEntry {
    pub id: String,
    pub timestamp: String,
    pub tenant_hash: String,
    pub router_target: String,
    pub method: String,
    pub endpoint: String,
    pub duration_ms: f64,
    pub status_code: u16,
    pub success: bool,
    pub client_ip: Option<String>,
}

/// Constant-time string equality check to prevent timing attacks on tokens
pub fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// What a presented token is allowed to do.
/// `Admin` = the full gateway API. `Viewer` = read-only network map (see `viewer_token`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Admin,
    Viewer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Viewer => "viewer",
        }
    }
}

/// Compute a salted one-way 16-hex hash from API token + optional tenant context
pub fn compute_tenant_hash(token: &str, tenant_context: Option<&str>) -> String {
    let mut hasher = Md5::new();
    hasher.update(b"ros-gw-salt-v1:");
    hasher.update(token.as_bytes());
    if let Some(ctx) = tenant_context {
        hasher.update(b":tenant:");
        hasher.update(ctx.as_bytes());
    }
    let res = hex::encode(hasher.finalize());
    res[..16].to_string()
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    pub api_token: String,
    /// Optional read-only token for browsers / dashboards. It can only call the *GET* map endpoints
    /// (`/api/v1/map/scene|icons|layouts`) and `subscribe_map` over `/ws`, can never choose a router
    /// by host/credentials (only by `router_id` from this file) and cannot edit or run commands.
    /// Override with `GATEWAY_VIEWER_TOKEN`. Must be >= 16 chars and different from `api_token`.
    #[serde(default)]
    pub viewer_token: Option<String>,
    /// Directory where network-map layouts (and uploaded floor-plan images) are persisted.
    /// Override with `GATEWAY_DATA_DIR`. Mount it as a volume in Docker to keep layouts.
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default, rename = "router")]
    pub routers: Vec<RouterCfg>,
}

fn default_listen() -> String {
    "127.0.0.1:8080".into()
}

pub fn default_data_dir() -> String {
    "./data".into()
}

#[derive(Deserialize, Clone, Debug)]
pub struct RouterCfg {
    pub id: String,
    pub addr: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RouterTarget {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

pub struct Slot {
    pub cfg: RouterCfg,
    pub client: Mutex<Option<Client>>,
}

pub struct AppState {
    pub token: String,
    /// Validated read-only token (see [`Config::viewer_token`]); `None` when disabled.
    pub viewer_token: Option<String>,
    pub configured_routers: HashMap<String, Slot>,
    /// Global persistent connection pool: key = "user@host:port:pass_hash"
    pub dynamic_pool: Mutex<HashMap<String, Client>>,
    /// Multi-tenant segregated ring buffer audit log (max 5,000 entries)
    pub audit_logs: Mutex<VecDeque<AuditLogEntry>>,
    /// Persistent network-map layouts (positions, icons, road waypoints, thresholds).
    pub maps: MapStore,
    /// Last interface/queue counter sample per `tenant|router`, used to turn counters into rates.
    pub rates: Mutex<HashMap<String, RateSnapshot>>,
}

impl AppState {
    pub fn new(cfg: Config) -> Self {
        let configured = cfg
            .routers
            .into_iter()
            .map(|r| {
                (
                    r.id.clone(),
                    Slot {
                        cfg: r,
                        client: Mutex::new(None),
                    },
                )
            })
            .collect();

        let viewer_token = cfg
            .viewer_token
            .clone()
            .map(|t| t.trim().to_string())
            .filter(|t| t.len() >= 16 && !ct_eq(t, &cfg.api_token));

        Self {
            token: cfg.api_token,
            viewer_token,
            configured_routers: configured,
            dynamic_pool: Mutex::new(HashMap::new()),
            audit_logs: Mutex::new(VecDeque::with_capacity(5000)),
            maps: MapStore::new(cfg.data_dir),
            rates: Mutex::new(HashMap::new()),
        }
    }

    /// Resolve a presented bearer token to a role, in constant time. `None` = unknown token.
    pub fn role_for_token(&self, presented: &str) -> Option<Role> {
        if ct_eq(presented, &self.token) {
            return Some(Role::Admin);
        }
        match &self.viewer_token {
            Some(v) if ct_eq(presented, v) => Some(Role::Viewer),
            _ => None,
        }
    }

    /// Record an execution log entry to the in-memory ring buffer with LRU eviction
    pub async fn record_audit_log(&self, entry: AuditLogEntry) {
        let mut logs = self.audit_logs.lock().await;
        if logs.len() >= 5000 {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    /// Resolve a live, authenticated Client instance using pool or configured slots
    pub async fn resolve_client(
        &self,
        target: Option<&RouterTarget>,
        router_id: Option<&str>,
    ) -> Result<Client, ApiError> {
        // 1. Check if dynamic target host + user are provided
        if let Some(t) = target {
            if let (Some(host), Some(user)) = (&t.host, &t.user) {
                let port = t.port.unwrap_or(8728);
                let password = t.password.as_deref().unwrap_or_default();
                let mut pass_hasher = Md5::new();
                pass_hasher.update(password.as_bytes());
                let pass_hash = hex::encode(pass_hasher.finalize());
                let key = format!("{user}@{host}:{port}:{pass_hash}");

                // 1.1 Fast check without holding lock during network connect
                {
                    let pool = self.dynamic_pool.lock().await;
                    if let Some(c) = pool.get(&key).filter(|c| !c.is_closed()) {
                        return Ok(c.clone());
                    }
                }

                // 1.2 Connect outside lock (has 10s connect & login timeout)
                let addr = format!("{host}:{port}");
                tracing::info!(addr = %addr, user = %user, "establishing dynamic connection");
                let client = Client::connect(&addr, user, password).await?;

                // 1.3 Cache in pool with max capacity of 256
                {
                    let mut pool = self.dynamic_pool.lock().await;
                    if pool.len() >= 256 {
                        pool.retain(|_, c| !c.is_closed());
                        if pool.len() >= 256 {
                            if let Some(first_key) = pool.keys().next().cloned() {
                                pool.remove(&first_key);
                            }
                        }
                    }
                    pool.insert(key, client.clone());
                }
                return Ok(client);
            }
        }

        // 2. Check by configured router ID (fallback to "main")
        let id = router_id.unwrap_or("main");
        let slot = self
            .configured_routers
            .get(id)
            .ok_or_else(|| ApiError::UnknownRouter(id.to_string()))?;

        let mut guard = slot.client.lock().await;
        if let Some(c) = guard.as_ref().filter(|c| !c.is_closed()) {
            return Ok(c.clone());
        }

        tracing::info!(router = id, addr = %slot.cfg.addr, "connecting configured router");
        let client = Client::connect(&slot.cfg.addr, &slot.cfg.user, &slot.cfg.password).await?;
        *guard = Some(client.clone());
        Ok(client)
    }

    /// Parse router target from HTTP headers or fallback to body params
    pub fn parse_target(
        headers: &HeaderMap,
        body_target: Option<RouterTarget>,
        body_id: Option<String>,
    ) -> (Option<RouterTarget>, Option<String>) {
        let h_host = headers.get("X-Router-Host").and_then(|v| v.to_str().ok());
        let h_port = headers
            .get("X-Router-Port")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u16>().ok());
        let h_user = headers.get("X-Router-User").and_then(|v| v.to_str().ok());
        let h_pass = headers.get("X-Router-Pass").and_then(|v| v.to_str().ok());
        let h_id = headers
            .get("X-Router-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if let (Some(host), Some(user)) = (h_host, h_user) {
            return (
                Some(RouterTarget {
                    host: Some(host.to_string()),
                    port: h_port,
                    user: Some(user.to_string()),
                    password: h_pass.map(|s| s.to_string()),
                }),
                h_id.or(body_id),
            );
        }

        (body_target, h_id.or(body_id))
    }
}
