use std::collections::HashMap;
use axum::http::HeaderMap;
use routeros_core::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::error::ApiError;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    pub api_token: String,
    #[serde(default, rename = "router")]
    pub routers: Vec<RouterCfg>,
}

fn default_listen() -> String {
    "127.0.0.1:8080".into()
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
    pub configured_routers: HashMap<String, Slot>,
    /// Global persistent connection pool: key = "user@host:port:password"
    pub dynamic_pool: Mutex<HashMap<String, Client>>,
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

        Self {
            token: cfg.api_token,
            configured_routers: configured,
            dynamic_pool: Mutex::new(HashMap::new()),
        }
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
                let key = format!("{user}@{host}:{port}:{password}");

                let mut pool = self.dynamic_pool.lock().await;
                if let Some(c) = pool.get(&key).filter(|c| !c.is_closed()) {
                    return Ok(c.clone());
                }

                let addr = format!("{host}:{port}");
                tracing::info!(addr = %addr, user = %user, "establishing dynamic connection");
                let client = Client::connect(&addr, user, password).await?;
                pool.insert(key, client.clone());
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
