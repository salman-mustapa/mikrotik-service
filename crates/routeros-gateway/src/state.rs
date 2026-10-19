use std::collections::{HashMap, VecDeque};
use axum::http::HeaderMap;
use routeros_core::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use sha2::{Digest as Sha2Digest, Sha256};

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

/// Constant-time string equality check using SHA-256 digest comparison.
/// Both inputs are hashed before comparison so timing is independent of string length.
pub fn ct_eq(a: &str, b: &str) -> bool {
    let hash_a = Sha256::digest(a.as_bytes());
    let hash_b = Sha256::digest(b.as_bytes());
    hash_a.iter().zip(hash_b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Check if an IP address is in a blocked/dangerous range (loopback, link-local, cloud metadata, multicast, broadcast)
pub fn is_blocked_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // 127.0.0.0/8 (Loopback)
            ipv4.is_loopback()
                // 0.0.0.0/8 ("This" network)
                || octets[0] == 0
                // 169.254.0.0/16 (Link-local & AWS/GCP/Azure Cloud Metadata 169.254.169.254)
                || (octets[0] == 169 && octets[1] == 254)
                // 224.0.0.0/4 (Multicast)
                || ipv4.is_multicast()
                // 255.255.255.255 (Broadcast)
                || ipv4.is_broadcast()
        }
        std::net::IpAddr::V6(ipv6) => {
            ipv6.is_loopback()
                || ipv6.is_multicast()
                || (ipv6.segments()[0] & 0xffc0) == 0xfe80
                || match ipv6.to_ipv4() {
                    Some(v4) => is_blocked_ip(std::net::IpAddr::V4(v4)),
                    None => false,
                }
        }
    }
}

/// Check if a host string is a blocked destination (localhost, cloud metadata, loopback)
pub fn is_blocked_host(host: &str) -> bool {
    let h = host.trim().to_lowercase();
    if h == "localhost"
        || h == "127.0.0.1"
        || h == "::1"
        || h == "169.254.169.254"
        || h == "metadata.google.internal"
        || h == "instance-data"
        || h.ends_with(".localhost")
    {
        return true;
    }

    if let Ok(ip) = h.parse::<std::net::IpAddr>() {
        return is_blocked_ip(ip);
    }
    false
}

/// Rate limiter for tracking failed authentications per IP to mitigate brute force attacks
#[derive(Debug)]
pub struct AuthRateLimiter {
    failures: Mutex<HashMap<std::net::IpAddr, (u32, std::time::Instant)>>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
        }
    }

    /// Check if IP has exceeded failure threshold. If locked out, returns Err(retry_after_secs).
    pub async fn check(&self, ip: std::net::IpAddr) -> Result<(), u64> {
        let mut map = self.failures.lock().await;
        let now = std::time::Instant::now();
        map.retain(|_, (_, last)| now.duration_since(*last).as_secs() < 300);

        if let Some((count, last)) = map.get(&ip) {
            let elapsed = now.duration_since(*last).as_secs();
            if *count >= 5 && elapsed < 60 {
                return Err(60 - elapsed);
            }
        }
        Ok(())
    }

    /// Record a failed authentication attempt
    pub async fn record_failure(&self, ip: std::net::IpAddr) {
        let mut map = self.failures.lock().await;
        let now = std::time::Instant::now();
        let entry = map.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1).as_secs() > 60 {
            entry.0 = 1;
        } else {
            entry.0 = entry.0.saturating_add(1);
        }
        entry.1 = now;
    }

    /// Record a successful authentication attempt (clears failure history)
    pub async fn record_success(&self, ip: std::net::IpAddr) {
        let mut map = self.failures.lock().await;
        map.remove(&ip);
    }
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new()
    }
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

/// Compute a salted one-way 16-hex hash from API token + optional tenant context using SHA-256
pub fn compute_tenant_hash(token: &str, tenant_context: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"ros-gw-salt-v2:");
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

    /// Security: Allow targeting arbitrary routers via X-Router-Host / body.
    /// Default: false (SSRF protection). When false, only routers configured in config.toml can be targeted.
    #[serde(default)]
    pub allow_dynamic_routers: bool,

    /// Security: Allow arbitrary raw command execution via /api/v1/command.
    /// Default: false (prevents unauthorized raw shell execution).
    #[serde(default)]
    pub allow_raw_command: bool,

    /// Security: Allow multi-line terminal script execution via /api/v1/scripts/terminal-exec.
    /// Default: false.
    #[serde(default)]
    pub allow_terminal_exec: bool,

    /// Security: Allow clearing audit logs via /api/v1/audit/clear.
    /// Default: false.
    #[serde(default)]
    pub allow_audit_clear: bool,

    /// Security: Trust X-Forwarded-For / X-Real-IP headers for client IP extraction.
    /// Default: false (only use direct socket connection IP).
    #[serde(default)]
    pub trust_forwarded_for: bool,

    /// Security: Whitelisted CORS origins. If empty, defaults to same-origin / localhost.
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,

    /// Security: Require authentication to access /docs and /api/v1/spec.
    /// Default: false.
    #[serde(default)]
    pub protect_docs: bool,

    /// Security: Allow hard factory reset of router via /api/v1/maintenance/reset-configuration.
    /// Default: false (prevents accidental or unauthorized router wipe).
    #[serde(default)]
    pub allow_system_reset: bool,

    /// Security: Allow remote reboot and shutdown via /api/v1/system/reboot and /api/v1/system/shutdown.
    /// Default: true (can be disabled via GATEWAY_ALLOW_SYSTEM_REBOOT=false).
    #[serde(default = "default_true")]
    pub allow_system_reboot: bool,
}

fn default_true() -> bool {
    true
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

    pub allow_dynamic_routers: bool,
    pub allow_raw_command: bool,
    pub allow_terminal_exec: bool,
    pub allow_audit_clear: bool,
    pub trust_forwarded_for: bool,
    pub cors_allowed_origins: Vec<String>,
    pub protect_docs: bool,
    pub allow_system_reset: bool,
    pub allow_system_reboot: bool,
    pub auth_rate_limiter: AuthRateLimiter,
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
            allow_dynamic_routers: cfg.allow_dynamic_routers,
            allow_raw_command: cfg.allow_raw_command,
            allow_terminal_exec: cfg.allow_terminal_exec,
            allow_audit_clear: cfg.allow_audit_clear,
            trust_forwarded_for: cfg.trust_forwarded_for,
            cors_allowed_origins: cfg.cors_allowed_origins,
            protect_docs: cfg.protect_docs,
            allow_system_reset: cfg.allow_system_reset,
            allow_system_reboot: cfg.allow_system_reboot,
            auth_rate_limiter: AuthRateLimiter::new(),
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
                if !self.allow_dynamic_routers {
                    return Err(ApiError::Forbidden(
                        "Dynamic router targets via X-Router-Host are disabled by gateway security policy. Configure router slots in config.toml or enable GATEWAY_ALLOW_DYNAMIC_ROUTERS=true.".into()
                    ));
                }

                if is_blocked_host(host) {
                    return Err(ApiError::Forbidden(
                        "Target router host is blocked by SSRF security policy (loopback, link-local, or cloud metadata target).".into()
                    ));
                }

                let port = t.port.unwrap_or(8728);
                if port == 0 {
                    return Err(ApiError::BadRequest("Invalid router port: 0".into()));
                }

                let password = t.password.as_deref().unwrap_or_default();
                let mut pass_hasher = Sha256::new();
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
                let client = Client::connect(&addr, user, password).await.map_err(|e| {
                    tracing::warn!(addr = %addr, error = %e, "dynamic router connection failed");
                    ApiError::RouterFatal("Unable to establish connection to router target (connection refused or timeout)".into())
                })?;

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
