//! Persistent storage for network-map layouts.
//!
//! A *layout* is the developer/operator-authored part of a map: where each device sits on the
//! floor plan / satellite image, which icon it uses, the waypoints that make a link follow a
//! road or cross a roof, and the alert thresholds. Live data (status, traffic, queues) is never
//! stored here - it is merged in at request time by `routes::map`.
//!
//! Storage layout (one JSON file per layout, scoped per tenant):
//! `<data_dir>/<tenant_hash>/<layout_id>.json` and optional `<layout_id>.bg` (floor-plan image).
//!
//! Safety properties:
//! * ids are validated against a strict charset, so a request can never escape `data_dir`;
//! * writes are atomic (temp file + rename) and serialised by a mutex;
//! * every number is checked for finiteness and range, collection sizes are capped;
//! * uploaded images are accepted by *magic bytes only* (PNG/JPEG/WebP). SVG is refused because it
//!   can carry script.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub const MAX_NODES: usize = 3000;
pub const MAX_PATHS: usize = 4000;
pub const MAX_WAYPOINTS: usize = 64;
pub const MAX_BG_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_LAYOUTS_PER_TENANT: usize = 100;

/// Icon ids a client may pick. `auto` detection only ever returns ids from this list.
pub const ICONS: &[(&str, &str, &str)] = &[
    ("internet", "Internet / ISP", "infrastructure"),
    ("router", "Router", "infrastructure"),
    ("switch", "Switch / Bridge", "infrastructure"),
    ("port", "Ethernet port", "infrastructure"),
    ("wan-port", "WAN port", "infrastructure"),
    ("ap", "Access Point", "infrastructure"),
    ("repeater", "Repeater / Relay AP", "infrastructure"),
    ("server", "Server", "infrastructure"),
    ("phone", "Smartphone", "client"),
    ("tablet", "Tablet", "client"),
    ("laptop", "Laptop", "client"),
    ("desktop", "Desktop PC", "client"),
    ("tv", "Smart TV", "client"),
    ("printer", "Printer", "client"),
    ("camera", "CCTV / IP camera", "client"),
    ("iot", "IoT device", "client"),
    ("game-console", "Game console", "client"),
    ("hotspot-user", "Hotspot user", "client"),
    ("pppoe-user", "PPPoE subscriber", "client"),
    ("unknown", "Unknown device", "client"),
];

pub fn icon_exists(id: &str) -> bool {
    ICONS.iter().any(|(i, _, _)| *i == id)
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Background {
    /// `none` | `url` (remote http/https image) | `upload` (image stored by this gateway).
    #[serde(default = "d_bg_kind")]
    pub kind: String,
    #[serde(default)]
    pub url: Option<String>,
    /// width / height, used only while no image is loaded.
    #[serde(default = "d_aspect")]
    pub aspect_ratio: f64,
    #[serde(default = "d_opacity")]
    pub opacity: f64,
}

fn d_bg_kind() -> String {
    "none".into()
}
fn d_aspect() -> f64 {
    16.0 / 9.0
}
fn d_opacity() -> f64 {
    1.0
}

impl Default for Background {
    fn default() -> Self {
        Self { kind: d_bg_kind(), url: None, aspect_ratio: d_aspect(), opacity: d_opacity() }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NodePlacement {
    /// Normalised 0..1 across the background image (left -> right).
    pub x: f64,
    /// Normalised 0..1 across the background image (top -> bottom).
    pub y: f64,
    /// Overrides the auto-detected icon. Must be one of [`ICONS`].
    #[serde(default)]
    pub icon: Option<String>,
    /// Overrides the label shown on the map.
    #[serde(default)]
    pub label: Option<String>,
    /// A locked node cannot be dragged in the editor.
    #[serde(default)]
    pub locked: bool,
    /// Re-parents the node (relay / "estafet" chains such as AP -> AP2, or client -> specific AP).
    #[serde(default)]
    pub parent_override: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EdgePath {
    /// Intermediate points only; the endpoints are the node positions.
    #[serde(default)]
    pub waypoints: Vec<Point>,
    /// `straight` | `orthogonal` | `smooth`.
    #[serde(default)]
    pub style: Option<String>,
    /// Link capacity (bits/s) used to compute utilisation. Overrides auto/assumed values.
    #[serde(default)]
    pub capacity_bps: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Thresholds {
    /// Utilisation (% of capacity / queue limit) at which a link or queue turns amber.
    #[serde(default = "d_warn")]
    pub warn_pct: f64,
    /// Utilisation at which it turns red ("limit reached").
    #[serde(default = "d_crit")]
    pub crit_pct: f64,
    /// Packets per second considered "high packet rate".
    #[serde(default = "d_pps")]
    pub high_pps: u64,
    /// Optional absolute throughput (bits/s) that raises a warning regardless of capacity.
    #[serde(default)]
    pub high_bps: Option<u64>,
}

fn d_warn() -> f64 {
    80.0
}
fn d_crit() -> f64 {
    95.0
}
fn d_pps() -> u64 {
    50_000
}

impl Default for Thresholds {
    fn default() -> Self {
        Self { warn_pct: d_warn(), crit_pct: d_crit(), high_pps: d_pps(), high_bps: None }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MapLayout {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// Informational: which configured router this layout was drawn for.
    #[serde(default)]
    pub router_id: Option<String>,
    #[serde(default)]
    pub background: Background,
    #[serde(default)]
    pub nodes: BTreeMap<String, NodePlacement>,
    #[serde(default)]
    pub paths: BTreeMap<String, EdgePath>,
    #[serde(default)]
    pub thresholds: Thresholds,
    /// Monotonic counter for optimistic concurrency (send it back on PUT to detect lost updates).
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub updated_at: String,
}

impl MapLayout {
    pub fn new_empty(id: &str) -> Self {
        Self { id: id.to_string(), name: id.to_string(), ..Default::default() }
    }
}

#[derive(Serialize, Debug)]
pub struct LayoutSummary {
    pub id: String,
    pub name: String,
    pub nodes: usize,
    pub paths: usize,
    pub has_background: bool,
    pub version: u64,
    pub updated_at: String,
}

// ---------------------------------------------------------------- validation

/// Layout ids: 1..=64 of `[a-z0-9_-]`.
pub fn valid_layout_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Node / edge keys come from the topology (`if_ether1`, `client_dhcp_AA:BB:..`, `edge_a_b`).
pub fn valid_key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b':' | b'.'))
}

fn unit(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}

fn clean_text(s: &str, max: usize) -> bool {
    s.chars().count() <= max && !s.chars().any(|c| c.is_control())
}

pub fn validate_layout(l: &MapLayout) -> Result<(), String> {
    if !clean_text(&l.name, 80) {
        return Err("name must be <= 80 characters without control characters".into());
    }
    if let Some(d) = &l.description {
        if !clean_text(d, 500) {
            return Err("description must be <= 500 characters without control characters".into());
        }
    }
    if let Some(r) = &l.router_id {
        if !clean_text(r, 64) {
            return Err("router_id is invalid".into());
        }
    }
    validate_background(&l.background)?;
    if l.nodes.len() > MAX_NODES {
        return Err(format!("too many placed nodes (max {MAX_NODES})"));
    }
    if l.paths.len() > MAX_PATHS {
        return Err(format!("too many paths (max {MAX_PATHS})"));
    }
    for (k, n) in &l.nodes {
        if !valid_key(k) {
            return Err(format!("invalid node id '{k}'"));
        }
        validate_placement(k, n)?;
    }
    for (k, p) in &l.paths {
        if !valid_key(k) {
            return Err(format!("invalid edge id '{k}'"));
        }
        validate_path(k, p)?;
    }
    validate_thresholds(&l.thresholds)
}

pub fn validate_background(b: &Background) -> Result<(), String> {
    match b.kind.as_str() {
        "none" | "upload" => {}
        "url" => {
            let u = b.url.as_deref().unwrap_or("");
            let lower = u.to_ascii_lowercase();
            if !(lower.starts_with("https://") || lower.starts_with("http://")) {
                return Err("background.url must start with http:// or https://".into());
            }
            if u.len() > 2048 || u.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"' || c == '\'' || c == '<' || c == '>') {
                return Err("background.url contains forbidden characters or is too long".into());
            }
        }
        other => return Err(format!("background.kind '{other}' is invalid (none|url|upload)")),
    }
    if !(b.aspect_ratio.is_finite() && (0.2..=5.0).contains(&b.aspect_ratio)) {
        return Err("background.aspect_ratio must be between 0.2 and 5".into());
    }
    if !(b.opacity.is_finite() && (0.0..=1.0).contains(&b.opacity)) {
        return Err("background.opacity must be between 0 and 1".into());
    }
    Ok(())
}

pub fn validate_placement(key: &str, n: &NodePlacement) -> Result<(), String> {
    if !unit(n.x) || !unit(n.y) {
        return Err(format!("node '{key}': x and y must be numbers between 0 and 1"));
    }
    if let Some(i) = &n.icon {
        if !icon_exists(i) {
            return Err(format!("node '{key}': unknown icon '{i}' (see GET /api/v1/map/icons)"));
        }
    }
    if let Some(l) = &n.label {
        if !clean_text(l, 80) {
            return Err(format!("node '{key}': label must be <= 80 characters without control characters"));
        }
    }
    if let Some(p) = &n.parent_override {
        if !valid_key(p) {
            return Err(format!("node '{key}': invalid parent_override"));
        }
        if p == key {
            return Err(format!("node '{key}': parent_override cannot point to itself"));
        }
    }
    Ok(())
}

pub fn validate_path(key: &str, p: &EdgePath) -> Result<(), String> {
    if p.waypoints.len() > MAX_WAYPOINTS {
        return Err(format!("path '{key}': too many waypoints (max {MAX_WAYPOINTS})"));
    }
    if p.waypoints.iter().any(|w| !unit(w.x) || !unit(w.y)) {
        return Err(format!("path '{key}': every waypoint x and y must be between 0 and 1"));
    }
    if let Some(s) = &p.style {
        if !matches!(s.as_str(), "straight" | "orthogonal" | "smooth") {
            return Err(format!("path '{key}': style must be straight|orthogonal|smooth"));
        }
    }
    if let Some(c) = p.capacity_bps {
        if c == 0 || c > 1_000_000_000_000 {
            return Err(format!("path '{key}': capacity_bps must be between 1 and 1e12"));
        }
    }
    Ok(())
}

pub fn validate_thresholds(t: &Thresholds) -> Result<(), String> {
    if !(t.warn_pct.is_finite() && t.crit_pct.is_finite() && t.warn_pct > 0.0 && t.warn_pct < t.crit_pct && t.crit_pct <= 200.0) {
        return Err("thresholds: require 0 < warn_pct < crit_pct <= 200".into());
    }
    if t.high_pps == 0 {
        return Err("thresholds.high_pps must be > 0".into());
    }
    Ok(())
}

/// Detects the image type from magic bytes. Never trusts a client supplied content type.
pub fn sniff_image(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        Some("image/png")
    } else if bytes.len() >= 3 && bytes[..3] == [0xFF, 0xD8, 0xFF] {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

// ------------------------------------------------------------------- storage

pub struct MapStore {
    dir: PathBuf,
    write_lock: Mutex<()>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(String),
    Corrupt(String),
    LimitReached(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Io(m) => write!(f, "storage error: {m}"),
            StoreError::Corrupt(m) => write!(f, "stored layout is corrupt: {m}"),
            StoreError::LimitReached(m) => write!(f, "{m}"),
        }
    }
}

impl MapStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into(), write_lock: Mutex::new(()) }
    }

    /// `tenant` is the 16-hex tenant scope; anything else is refused so it cannot be a path trick.
    fn tenant_dir(&self, tenant: &str) -> Result<PathBuf, StoreError> {
        if tenant.len() != 16 || !tenant.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(StoreError::Io("invalid tenant scope".into()));
        }
        Ok(self.dir.join(tenant))
    }

    fn file(&self, tenant: &str, id: &str, ext: &str) -> Result<PathBuf, StoreError> {
        if !valid_layout_id(id) {
            return Err(StoreError::Io("invalid layout id".into()));
        }
        Ok(self.tenant_dir(tenant)?.join(format!("{id}.{ext}")))
    }

    pub async fn list(&self, tenant: &str) -> Result<Vec<LayoutSummary>, StoreError> {
        let dir = self.tenant_dir(tenant)?;
        let mut out = Vec::new();
        let mut rd = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(StoreError::Io(e.to_string())),
        };
        while let Ok(Some(entry)) = rd.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(id) = path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()) else { continue };
            if !valid_layout_id(&id) {
                continue;
            }
            if let Ok(Some(l)) = self.get(tenant, &id).await {
                let has_bg = l.background.kind == "upload" && self.file(tenant, &id, "bg").map(|p| p.exists()).unwrap_or(false);
                out.push(LayoutSummary {
                    id: l.id.clone(),
                    name: l.name.clone(),
                    nodes: l.nodes.len(),
                    paths: l.paths.len(),
                    has_background: has_bg || l.background.kind == "url",
                    version: l.version,
                    updated_at: l.updated_at.clone(),
                });
            }
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    pub async fn get(&self, tenant: &str, id: &str) -> Result<Option<MapLayout>, StoreError> {
        let path = self.file(tenant, id, "json")?;
        match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice::<MapLayout>(&bytes)
                .map(|mut l| {
                    l.id = id.to_string();
                    Some(l)
                })
                .map_err(|e| StoreError::Corrupt(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StoreError::Io(e.to_string())),
        }
    }

    /// Atomic write. The caller must already have validated the layout.
    pub async fn put(&self, tenant: &str, layout: &MapLayout) -> Result<(), StoreError> {
        let _g = self.write_lock.lock().await;
        self.put_locked(tenant, layout).await
    }

    async fn put_locked(&self, tenant: &str, layout: &MapLayout) -> Result<(), StoreError> {
        let path = self.file(tenant, &layout.id, "json")?;
        let dir = self.tenant_dir(tenant)?;
        tokio::fs::create_dir_all(&dir).await.map_err(|e| StoreError::Io(e.to_string()))?;
        if !path.exists() {
            let count = self.list(tenant).await.map(|v| v.len()).unwrap_or(0);
            if count >= MAX_LAYOUTS_PER_TENANT {
                return Err(StoreError::LimitReached(format!("layout limit reached (max {MAX_LAYOUTS_PER_TENANT} per tenant)")));
            }
        }
        let json = serde_json::to_vec_pretty(layout).map_err(|e| StoreError::Io(e.to_string()))?;
        let tmp = dir.join(format!(".{}.json.tmp", layout.id));
        tokio::fs::write(&tmp, &json).await.map_err(|e| StoreError::Io(e.to_string()))?;
        tokio::fs::rename(&tmp, &path).await.map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }

    /// Read-modify-write under the store lock so concurrent editors cannot lose each other's
    /// changes between the read and the write. `f` receives the current layout (or a fresh one).
    pub async fn update<F>(&self, tenant: &str, id: &str, f: F) -> Result<MapLayout, UpdateError>
    where
        F: FnOnce(&mut MapLayout) -> Result<(), String>,
    {
        let _g = self.write_lock.lock().await;
        let mut layout = match self.get(tenant, id).await.map_err(UpdateError::Store)? {
            Some(l) => l,
            None => MapLayout::new_empty(id),
        };
        f(&mut layout).map_err(UpdateError::Invalid)?;
        validate_layout(&layout).map_err(UpdateError::Invalid)?;
        layout.id = id.to_string();
        layout.version = layout.version.saturating_add(1);
        layout.updated_at = crate::current_iso_timestamp();
        self.put_locked(tenant, &layout).await.map_err(UpdateError::Store)?;
        Ok(layout)
    }

    pub async fn delete(&self, tenant: &str, id: &str) -> Result<bool, StoreError> {
        let _g = self.write_lock.lock().await;
        let json = self.file(tenant, id, "json")?;
        let bg = self.file(tenant, id, "bg")?;
        let existed = json.exists();
        for p in [json, bg] {
            match tokio::fs::remove_file(&p).await {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(StoreError::Io(e.to_string())),
            }
        }
        Ok(existed)
    }

    pub async fn put_background(&self, tenant: &str, id: &str, bytes: &[u8]) -> Result<(), StoreError> {
        let path = self.file(tenant, id, "bg")?;
        let dir = self.tenant_dir(tenant)?;
        tokio::fs::create_dir_all(&dir).await.map_err(|e| StoreError::Io(e.to_string()))?;
        let tmp = dir.join(format!(".{id}.bg.tmp"));
        tokio::fs::write(&tmp, bytes).await.map_err(|e| StoreError::Io(e.to_string()))?;
        tokio::fs::rename(&tmp, &path).await.map_err(|e| StoreError::Io(e.to_string()))
    }

    pub async fn get_background(&self, tenant: &str, id: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let path = self.file(tenant, id, "bg")?;
        match tokio::fs::read(&path).await {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(StoreError::Io(e.to_string())),
        }
    }

    pub async fn delete_background(&self, tenant: &str, id: &str) -> Result<(), StoreError> {
        let path = self.file(tenant, id, "bg")?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StoreError::Io(e.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum UpdateError {
    Invalid(String),
    Store(StoreError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_store(tag: &str) -> MapStore {
        let dir = std::env::temp_dir().join(format!("ros-gw-map-test-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        MapStore::new(dir)
    }

    const T: &str = "0123456789abcdef";

    #[test]
    fn ids_and_keys_are_strict() {
        assert!(valid_layout_id("home-1"));
        assert!(!valid_layout_id("../etc"));
        assert!(!valid_layout_id("Home"));
        assert!(!valid_layout_id(""));
        assert!(valid_key("client_dhcp_AA:BB:CC:DD:EE:FF"));
        assert!(!valid_key("a/b"));
        assert!(!valid_key("a b"));
    }

    #[test]
    fn placement_range_and_icon_checks() {
        let ok = NodePlacement { x: 0.5, y: 1.0, icon: Some("router".into()), ..Default::default() };
        assert!(validate_placement("n", &ok).is_ok());
        let bad_x = NodePlacement { x: 1.01, y: 0.5, ..Default::default() };
        assert!(validate_placement("n", &bad_x).is_err());
        let nan = NodePlacement { x: f64::NAN, y: 0.5, ..Default::default() };
        assert!(validate_placement("n", &nan).is_err());
        let bad_icon = NodePlacement { x: 0.1, y: 0.1, icon: Some("<script>".into()), ..Default::default() };
        assert!(validate_placement("n", &bad_icon).is_err());
        let selfp = NodePlacement { x: 0.1, y: 0.1, parent_override: Some("n".into()), ..Default::default() };
        assert!(validate_placement("n", &selfp).is_err());
    }

    #[test]
    fn background_url_must_be_http() {
        let mut b = Background { kind: "url".into(), url: Some("javascript:alert(1)".into()), ..Default::default() };
        assert!(validate_background(&b).is_err());
        b.url = Some("https://tiles.example.com/plan.png".into());
        assert!(validate_background(&b).is_ok());
        b.url = Some("https://x.com/a\" onerror=\"x".into());
        assert!(validate_background(&b).is_err());
    }

    #[test]
    fn image_sniffing_rejects_svg_and_html() {
        assert_eq!(sniff_image(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0]), Some("image/png"));
        assert_eq!(sniff_image(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("image/jpeg"));
        assert_eq!(sniff_image(b"<svg xmlns='http://www.w3.org/2000/svg'><script/></svg>"), None);
        assert_eq!(sniff_image(b"<html>"), None);
    }

    #[test]
    fn thresholds_must_be_ordered() {
        assert!(validate_thresholds(&Thresholds::default()).is_ok());
        assert!(validate_thresholds(&Thresholds { warn_pct: 90.0, crit_pct: 80.0, ..Default::default() }).is_err());
    }

    #[tokio::test]
    async fn store_roundtrip_versioning_and_isolation() {
        let s = tmp_store("rt");
        assert!(s.get(T, "home").await.unwrap().is_none());

        let l = s
            .update(T, "home", |l| {
                l.name = "Home".into();
                l.nodes.insert("if_ether1".into(), NodePlacement { x: 0.2, y: 0.3, ..Default::default() });
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(l.version, 1);

        let l2 = s.update(T, "home", |l| { l.name = "Home 2".into(); Ok(()) }).await.unwrap();
        assert_eq!(l2.version, 2);
        assert_eq!(l2.nodes.len(), 1, "earlier placement must survive a later update");

        // another tenant never sees it
        assert!(s.get("fedcba9876543210", "home").await.unwrap().is_none());
        assert_eq!(s.list(T).await.unwrap().len(), 1);

        // invalid update is rejected and does not change anything
        let bad = s.update(T, "home", |l| { l.nodes.insert("x".into(), NodePlacement { x: 5.0, y: 0.0, ..Default::default() }); Ok(()) }).await;
        assert!(matches!(bad, Err(UpdateError::Invalid(_))));
        assert_eq!(s.get(T, "home").await.unwrap().unwrap().version, 2);

        assert!(s.delete(T, "home").await.unwrap());
        assert!(s.get(T, "home").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn tenant_scope_cannot_be_a_path() {
        let s = tmp_store("tp");
        assert!(s.get("../../etc/pass", "home").await.is_err());
        assert!(s.get(T, "../home").await.is_err());
    }
}
