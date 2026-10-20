//! Network map: a persistent, developer-editable floor-plan / satellite layout of the network,
//! merged at request time with live data (status, traffic, queue limits).
//!
//! * `map_store`  - what the developer authors: positions, icons, road waypoints, thresholds.
//! * this module  - REST + WebSocket glue, plus `compose_scene`, a *pure* function that merges
//!   topology + layout + counters into the scene the widget renders. It does no I/O, so it is
//!   unit-tested without a router.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use routeros_core::{build_command, Client};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::map_store::*;
use crate::routes::topology::{build_topology, TopologyEdge, TopologyNode};
use crate::state::{compute_tenant_hash, AppState, RouterTarget};

pub fn body_limit_layer() -> DefaultBodyLimit {
    DefaultBodyLimit::max(MAX_BG_BYTES + 1024)
}

// ------------------------------------------------------------------ rate model

#[derive(Clone, Debug, Default)]
pub struct IfaceCounters {
    pub running: bool,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub drops: u64,
}

#[derive(Clone, Debug, Default)]
pub struct IfaceRate {
    pub rx_bps: f64,
    pub tx_bps: f64,
    pub rx_pps: f64,
    pub tx_pps: f64,
    pub drop_delta: u64,
}

#[derive(Clone, Debug, Default)]
pub struct QueueInfo {
    pub name: String,
    /// (ipv4 as u32, prefix length)
    pub targets: Vec<(u32, u8)>,
    pub max_up: u64,
    pub max_down: u64,
    pub rate_up: u64,
    pub rate_down: u64,
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub dropped_up: u64,
    pub dropped_down: u64,
}

/// Counters remembered between two scene builds so rates can be derived from deltas.
#[derive(Clone, Debug, Default)]
pub struct RateSnapshot {
    pub at_ms: u64,
    pub ifaces: HashMap<String, IfaceCounters>,
    /// node id -> (bytes_in, bytes_out) for clients that expose cumulative counters.
    pub clients: HashMap<String, (u64, u64)>,
    pub iface_rates: HashMap<String, IfaceRate>,
    /// node id -> (up_bps, down_bps)
    pub client_rates: HashMap<String, (f64, f64)>,
}

pub fn mono_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

/// RouterOS rate/limit numbers, optionally with SI suffix (`10M`, `512k`).
pub fn parse_si(s: &str) -> u64 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    let (num, mult) = match s.chars().last() {
        Some('k') | Some('K') => (&s[..s.len() - 1], 1_000u64),
        Some('M') => (&s[..s.len() - 1], 1_000_000),
        Some('G') => (&s[..s.len() - 1], 1_000_000_000),
        _ => (s, 1),
    };
    num.trim().parse::<f64>().map(|v| (v * mult as f64) as u64).unwrap_or(0)
}

/// `"up/down"` -> (up, down)
pub fn parse_pair(s: &str) -> (u64, u64) {
    let mut it = s.split('/');
    let a = it.next().map(parse_si).unwrap_or(0);
    let b = it.next().map(parse_si).unwrap_or(0);
    (a, b)
}

pub fn parse_ipv4(s: &str) -> Option<u32> {
    let mut parts = s.trim().split('.');
    let mut v: u32 = 0;
    for _ in 0..4 {
        let p: u32 = parts.next()?.parse().ok()?;
        if p > 255 {
            return None;
        }
        v = (v << 8) | p;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(v)
}

pub fn parse_targets(s: &str) -> Vec<(u32, u8)> {
    s.split(',')
        .filter_map(|t| {
            let t = t.trim();
            let (ip, prefix) = match t.split_once('/') {
                Some((ip, p)) => (ip, p.parse::<u8>().ok()?),
                None => (t, 32),
            };
            if prefix > 32 {
                return None;
            }
            Some((parse_ipv4(ip)?, prefix))
        })
        .collect()
}

fn ip_in(ip: u32, net: u32, prefix: u8) -> bool {
    if prefix == 0 {
        return true;
    }
    let mask = u32::MAX << (32 - prefix as u32);
    (ip & mask) == (net & mask)
}

/// Most specific enabled queue that targets `ip`.
pub fn queue_for<'a>(queues: &'a [QueueInfo], ip: &str) -> Option<&'a QueueInfo> {
    let ip = parse_ipv4(ip)?;
    queues
        .iter()
        .filter_map(|q| {
            q.targets
                .iter()
                .filter(|(n, p)| ip_in(ip, *n, *p))
                .map(|(_, p)| *p)
                .max()
                .map(|p| (p, q))
        })
        .max_by_key(|(p, _)| *p)
        .map(|(_, q)| q)
}

pub fn fmt_rate_bytes(bps: f64) -> String {
    let b = (bps.max(0.0)) / 8.0;
    if b >= 1_073_741_824.0 {
        format!("{:.2} GB/s", b / 1_073_741_824.0)
    } else if b >= 1_048_576.0 {
        format!("{:.2} MB/s", b / 1_048_576.0)
    } else if b >= 1024.0 {
        format!("{:.1} KB/s", b / 1024.0)
    } else {
        format!("{:.0} B/s", b)
    }
}

pub fn fmt_bits(bps: f64) -> String {
    let b = bps.max(0.0);
    if b >= 1e9 {
        format!("{:.2} Gbps", b / 1e9)
    } else if b >= 1e6 {
        format!("{:.2} Mbps", b / 1e6)
    } else if b >= 1e3 {
        format!("{:.1} Kbps", b / 1e3)
    } else {
        format!("{:.0} bps", b)
    }
}

pub fn fmt_bytes(n: u64) -> String {
    let b = n as f64;
    if b >= 1_073_741_824.0 {
        format!("{:.2} GB", b / 1_073_741_824.0)
    } else if b >= 1_048_576.0 {
        format!("{:.2} MB", b / 1_048_576.0)
    } else if b >= 1024.0 {
        format!("{:.1} KB", b / 1024.0)
    } else {
        format!("{n} B")
    }
}

fn r4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

// ------------------------------------------------------------------ icon detection

fn tokens(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Human-friendly default name for the map: drops the leading emoji/symbols and the "AP:"-style prefix
/// the topology builder adds, and a trailing "(IP / MAC / ISP: ip)" suffix (the IP is a separate field).
/// Real names such as "ether1 (WAN / Internet In)" are kept. Falls back to the raw label if empty.
pub fn clean_label(raw: &str) -> String {
    let mut s = raw.trim_start_matches(|c: char| !c.is_alphanumeric()).trim();
    for p in ["AP: ", "Host: ", "WiFi: ", "PPPoE: "] {
        if let Some(rest) = s.strip_prefix(p) {
            s = rest.trim();
            break;
        }
    }
    if s.ends_with(')') {
        if let Some(open) = s.rfind(" (") {
            let inner = &s[open + 2..s.len() - 1];
            let addr_like = inner.chars().any(|c| c.is_ascii_digit() || c == ':' || c == '.')
                && inner.chars().all(|c| c.is_ascii_hexdigit() || ".:/ ".contains(c));
            if addr_like || inner.starts_with("ISP:") {
                s = s[..open].trim();
            }
        }
    }
    if s.is_empty() { raw.trim().to_string() } else { s.to_string() }
}

/// Best-effort icon from what the router knows (type, vendor, hostname). Heuristic - the user can
/// always override it per node. Only ids from [`ICONS`] are returned.
pub fn detect_icon(n: &TopologyNode, parent_type: Option<&str>) -> &'static str {
    match n.node_type.as_str() {
        "router" => return "router",
        "internet" => return "internet",
        "interface" => {
            return match n.sub_type.as_str() {
                "wan" => "wan-port",
                "bridge" => "switch",
                _ => "port",
            }
        }
        "ap" => return if parent_type == Some("ap") { "repeater" } else { "ap" },
        _ => {}
    }

    let hay = format!(
        "{} {} {} {}",
        n.label,
        n.vendor.as_deref().unwrap_or(""),
        n.extra.get("host_name").map(|s| s.as_str()).unwrap_or(""),
        n.extra.get("username").map(|s| s.as_str()).unwrap_or("")
    );
    let lower = hay.to_lowercase();
    let tok = tokens(&hay);
    let has_tok = |ws: &[&str]| tok.iter().any(|t| ws.contains(&t.as_str()));
    let has_sub = |ws: &[&str]| ws.iter().any(|w| lower.contains(w));

    if has_sub(&["hikvision", "dahua", "ezviz", "reolink", "imou", "cctv", "ipcam", "camera"]) || has_tok(&["cam", "ipc", "nvr", "dvr"]) {
        "camera"
    } else if has_tok(&["repeater", "extender"]) {
        "repeater"
    } else if has_tok(&["ap", "wap", "accesspoint"]) || has_sub(&["access point", "unifi", "tp-link-ap"]) {
        "ap"
    } else if has_sub(&["playstation", "xbox", "nintendo"]) || has_tok(&["ps4", "ps5"]) {
        "game-console"
    } else if has_sub(&["printer", "laserjet", "epson", "brother", "canon"]) {
        "printer"
    } else if has_sub(&["bravia", "webos", "tizen", "roku", "chromecast", "firetv", "appletv", "smarttv"]) || has_tok(&["tv"]) {
        "tv"
    } else if has_sub(&["ipad"]) || has_tok(&["tablet", "tab"]) {
        "tablet"
    } else if has_sub(&["macbook", "laptop", "thinkpad", "notebook", "surface", "ideapad", "vivobook", "zenbook"]) {
        "laptop"
    } else if has_tok(&["desktop", "pc", "workstation"]) || has_sub(&["desktop-"]) {
        "desktop"
    } else if has_sub(&["synology", "qnap", "proxmox", "vmware", "server"]) {
        "server"
    } else if has_sub(&["espressif", "esp32", "esp8266", "tuya", "shelly", "sonoff", "alios", "yeelight", "bardi", "iot", "smartplug", "smart plug"]) {
        "iot"
    } else if has_sub(&["iphone", "android", "galaxy", "samsung", "xiaomi", "redmi", "poco", "oppo", "vivo", "huawei", "realme", "pixel", "oneplus", "infinix", "tecno", "apple"]) || has_tok(&["phone"]) {
        "phone"
    } else {
        match n.sub_type.as_str() {
            "pppoe" => "pppoe-user",
            "hotspot" => "hotspot-user",
            _ => "unknown",
        }
    }
}

// ------------------------------------------------------------------ health model

#[derive(Clone, Debug)]
struct Reason {
    kind: &'static str,
    severity: &'static str, // "warning" | "critical"
    message: String,
}

fn health_of(reasons: &[Reason], base: &'static str) -> &'static str {
    if reasons.iter().any(|r| r.severity == "critical") {
        "critical"
    } else if reasons.iter().any(|r| r.severity == "warning") {
        "warning"
    } else {
        base
    }
}

fn worse(a: &'static str, b: &'static str) -> &'static str {
    fn rank(h: &str) -> u8 {
        match h {
            "critical" => 3,
            "warning" => 2,
            "ok" => 1,
            _ => 0,
        }
    }
    if rank(a) >= rank(b) { a } else { b }
}

/// Evaluate utilisation / packet-rate / drop conditions for a link or a queue.
#[allow(clippy::too_many_arguments)]
fn eval_traffic(
    th: &Thresholds,
    scope: &str,
    down: Option<f64>,
    up: Option<f64>,
    cap_down: Option<f64>,
    cap_up: Option<f64>,
    pps: f64,
    drops: u64,
) -> (Vec<Reason>, Option<f64>, Option<f64>) {
    let mut reasons = Vec::new();
    let ud = match (down, cap_down) {
        (Some(d), Some(c)) if c > 0.0 => Some(d / c * 100.0),
        _ => None,
    };
    let uu = match (up, cap_up) {
        (Some(u), Some(c)) if c > 0.0 => Some(u / c * 100.0),
        _ => None,
    };
    for (label, u) in [("download", ud), ("upload", uu)] {
        if let Some(u) = u {
            if u >= th.crit_pct {
                reasons.push(Reason {
                    kind: "limit_reached",
                    severity: "critical",
                    message: format!("{scope} {label} at {:.0}% of its limit - limit reached", u),
                });
            } else if u >= th.warn_pct {
                reasons.push(Reason {
                    kind: "limit_near",
                    severity: "warning",
                    message: format!("{scope} {label} at {:.0}% of its limit", u),
                });
            }
        }
    }
    if pps >= th.high_pps as f64 {
        reasons.push(Reason { kind: "high_pps", severity: "warning", message: format!("high packet rate: {:.0} packets/s", pps) });
    }
    if let Some(hb) = th.high_bps {
        let peak = down.unwrap_or(0.0).max(up.unwrap_or(0.0));
        if peak >= hb as f64 {
            reasons.push(Reason { kind: "high_traffic", severity: "warning", message: format!("high throughput: {}", fmt_bits(peak)) });
        }
    }
    if drops > 0 {
        reasons.push(Reason { kind: "drops", severity: "warning", message: format!("{drops} packets dropped since last sample") });
    }
    (reasons, ud, uu)
}

// ------------------------------------------------------------------ auto layout

/// Deterministic left-to-right tree layout used for nodes the user has not placed yet.
fn auto_layout(nodes: &[TopologyNode], parent: &HashMap<String, String>, edges: &[TopologyEdge]) -> HashMap<String, Point> {
    let ids: HashSet<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    for n in nodes {
        if let Some(p) = parent.get(&n.id) {
            if ids.contains(p.as_str()) {
                children.entry(p.as_str()).or_default().push(n.id.as_str());
            }
        }
    }
    for v in children.values_mut() {
        v.sort();
    }

    let internet = "node_internet";
    let mut roots: Vec<&str> = nodes
        .iter()
        .filter(|n| n.id != internet && !parent.get(&n.id).map(|p| ids.contains(p.as_str())).unwrap_or(false))
        .map(|n| n.id.as_str())
        .collect();
    roots.sort_by_key(|r| (*r != "node_router", *r));

    let mut depth: HashMap<&str, usize> = HashMap::new();
    let mut ypos: HashMap<&str, f64> = HashMap::new();
    let mut x_offset: HashMap<&str, f64> = HashMap::new();
    let mut slot = 0usize;
    let mut visited: HashSet<&str> = HashSet::new();

    // Assign hierarchy depths via BFS
    for root in &roots {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((root, 1usize));
        visited.insert(root);
        depth.insert(root, 1);

        while let Some((curr, d)) = queue.pop_front() {
            if let Some(kids) = children.get(curr) {
                for k in kids {
                    if visited.insert(k) {
                        depth.insert(k, d + 1);
                        queue.push_back((k, d + 1));
                    }
                }
            }
        }
    }

    // Now compute Y positions with multi-column fanout for dense child nodes
    visited.clear();
    for root in &roots {
        let mut stack: Vec<(&str, bool)> = vec![(root, false)];
        while let Some((id, done)) = stack.pop() {
            if done {
                let kids = children.get(id).cloned().unwrap_or_default();
                if kids.is_empty() {
                    let y = slot as f64;
                    slot += 1;
                    ypos.insert(id, y);
                } else if kids.len() > 5 {
                    // Multi-column fan-out for dense leaf devices (e.g. 10-30 connected clients on an AP/router)
                    let num_cols = if kids.len() > 16 { 3 } else { 2 };
                    let base_slot = slot;
                    let rows = (kids.len() + num_cols - 1) / num_cols;
                    slot += rows;

                    for (idx, kid) in kids.iter().enumerate() {
                        let c = idx % num_cols;
                        let r = idx / num_cols;
                        let y = base_slot as f64 + r as f64;
                        ypos.insert(kid, y);
                        let offset = (c as f64 - (num_cols - 1) as f64 / 2.0) * 0.10;
                        x_offset.insert(kid, offset);
                    }
                    let parent_y = base_slot as f64 + (rows as f64 - 1.0) / 2.0;
                    ypos.insert(id, parent_y);
                } else {
                    let ys: Vec<f64> = kids.iter().filter_map(|k| ypos.get(k).copied()).collect();
                    let y = if ys.is_empty() {
                        let y = slot as f64;
                        slot += 1;
                        y
                    } else {
                        ys.iter().sum::<f64>() / ys.len() as f64
                    };
                    ypos.insert(id, y);
                }
                continue;
            }
            if !visited.insert(id) {
                continue;
            }
            stack.push((id, true));
            if let Some(kids) = children.get(id) {
                if kids.len() <= 5 {
                    for k in kids.iter().rev() {
                        stack.push((k, false));
                    }
                }
            }
        }
    }

    // Internet sits one column to the left of the WAN interface row.
    if ids.contains(internet) {
        let target = edges.iter().find(|e| e.source == internet).map(|e| e.target.as_str());
        depth.insert(internet, 0);
        let y = target.and_then(|t| ypos.get(t).copied()).unwrap_or(0.0);
        ypos.insert(internet, y);
    }

    let max_depth = depth.values().copied().max().unwrap_or(1).max(1) as f64;
    let max_slot = (slot.max(1) - 1).max(1) as f64;
    let mut out = HashMap::new();
    for n in nodes {
        let d = depth.get(n.id.as_str()).copied().unwrap_or(1) as f64;
        let y = ypos.get(n.id.as_str()).copied().unwrap_or(0.0);
        let x_off = x_offset.get(n.id.as_str()).copied().unwrap_or(0.0);
        let raw_x = 0.08 + d * (0.82 / max_depth) + x_off;
        let raw_y = 0.08 + (y / max_slot).min(1.0) * 0.82;
        out.insert(
            n.id.clone(),
            Point {
                x: r4(raw_x.clamp(0.04, 0.96)),
                y: r4(raw_y.clamp(0.04, 0.96)),
            },
        );
    }
    out
}

// ------------------------------------------------------------------ scene composition

pub struct SceneInput {
    pub nodes: Vec<TopologyNode>,
    pub edges: Vec<TopologyEdge>,
    pub layout: MapLayout,
    pub layout_exists: bool,
    pub has_upload: bool,
    pub ifaces: HashMap<String, IfaceCounters>,
    pub queues: Vec<QueueInfo>,
    pub prev: Option<RateSnapshot>,
    pub now_ms: u64,
}

pub struct SceneOutput {
    pub scene: Value,
    pub snapshot: RateSnapshot,
}

fn wan_iface_node(nodes: &[TopologyNode]) -> Option<&TopologyNode> {
    nodes.iter().find(|n| n.node_type == "interface" && n.sub_type == "wan")
}

pub fn compose_scene(input: SceneInput) -> SceneOutput {
    let SceneInput { mut nodes, mut edges, layout, layout_exists, has_upload, ifaces, queues, prev, now_ms } = input;
    let th = layout.thresholds.clone();
    let mut warnings: Vec<String> = Vec::new();

    // ---- 0. merge manual / passive / custom infrastructure nodes from layout
    let existing_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    for (k, pl) in &layout.nodes {
        if !existing_ids.contains(k) {
            let is_passive = pl.icon.as_deref().map(|i| matches!(i, "odp" | "tiang" | "closure" | "unmanaged-switch" | "port" | "olt")).unwrap_or(false)
                || k.starts_with("odp_") || k.starts_with("tiang_") || k.starts_with("closure_") || k.starts_with("switch_");
            let node_type = if is_passive { "infrastructure" } else { "device" };
            let sub_type = pl.icon.clone().unwrap_or_else(|| "custom".to_string());
            let status = if is_passive { "passive" } else { "offline" };
            nodes.push(TopologyNode {
                id: k.clone(),
                label: pl.label.clone().unwrap_or_else(|| k.clone()),
                node_type: node_type.to_string(),
                sub_type,
                ip: None,
                mac: None,
                vendor: None,
                interface: None,
                parent_id: pl.parent_override.clone(),
                status: status.to_string(),
                extra: std::collections::HashMap::new(),
            });
            if let Some(p) = &pl.parent_override {
                edges.push(TopologyEdge {
                    id: format!("edge_{p}_{k}"),
                    source: p.clone(),
                    target: k.clone(),
                    edge_type: if is_passive { "physical".into() } else { "logical".into() },
                    speed: None,
                });
            }
        }
    }

    // ---- 1. relay / re-parenting overrides (estafet chains: AP -> AP2, client -> specific AP)
    let ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let ntype: HashMap<String, String> = nodes.iter().map(|n| (n.id.clone(), n.node_type.clone())).collect();
    let mut parent: HashMap<String, String> =
        nodes.iter().filter_map(|n| n.parent_id.clone().map(|p| (n.id.clone(), p))).collect();
    let order: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let mut overridden: HashSet<String> = HashSet::new();
    for id in &order {
        let Some(p) = layout.nodes.get(id).and_then(|pl| pl.parent_override.clone()) else { continue };
        let p_clean = p.trim();
        if p_clean.is_empty() || p_clean == "none" || p_clean == "disconnected" {
            // User explicitly disconnected this node from its parent
            parent.remove(id);
            overridden.insert(id.clone());
            continue;
        }
        if !ids.contains(&p) {
            warnings.push(format!("parent_override '{p}' of '{id}' is not on this router right now (ignored)"));
            continue;
        }
        let mut cur = p.clone();
        let mut cycle = false;
        for _ in 0..10_000 {
            if &cur == id {
                cycle = true;
                break;
            }
            match parent.get(&cur) {
                Some(pp) => cur = pp.clone(),
                None => break,
            }
        }
        if cycle {
            warnings.push(format!("parent_override '{p}' of '{id}' would create a loop (ignored)"));
            continue;
        }
        parent.insert(id.clone(), p);
        overridden.insert(id.clone());
    }
    for n in nodes.iter_mut() {
        if overridden.contains(&n.id) {
            n.parent_id = parent.get(&n.id).cloned();
        }
    }
    edges.retain(|e| !overridden.contains(&e.target));
    for id in &order {
        if overridden.contains(id) {
            if let Some(p) = parent.get(id) {
                let is_ap = ntype.get(id).map(|t| t == "ap").unwrap_or(false);
                edges.push(TopologyEdge {
                    id: format!("edge_{p}_{id}"),
                    source: p.clone(),
                    target: id.clone(),
                    edge_type: if is_ap { "relay".into() } else { "logical".into() },
                    speed: Some("Re-parented in map layout".into()),
                });
            }
        }
    }

    // ---- 2. hierarchy helpers
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for n in &nodes {
        if let Some(p) = parent.get(&n.id) {
            children.entry(p.clone()).or_default().push(n.id.clone());
        }
    }
    for v in children.values_mut() {
        v.sort();
    }
    let descendants = |id: &str| -> Vec<String> {
        let mut out = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = children.get(id).cloned().unwrap_or_default();
        while let Some(c) = stack.pop() {
            if !seen.insert(c.clone()) {
                continue;
            }
            if let Some(k) = children.get(&c) {
                stack.extend(k.iter().cloned());
            }
            out.push(c);
        }
        out
    };

    // ---- 3. interface rates from counter deltas
    let reuse_prev = prev.as_ref().map(|p| now_ms.saturating_sub(p.at_ms) < 500).unwrap_or(false);
    let mut snapshot = RateSnapshot { at_ms: now_ms, ifaces: ifaces.clone(), ..Default::default() };
    let mut rate_ready = false;
    if let Some(p) = prev.as_ref() {
        if reuse_prev {
            snapshot = p.clone();
            rate_ready = !p.iface_rates.is_empty() || !p.client_rates.is_empty();
        } else {
            let dt = (now_ms.saturating_sub(p.at_ms)) as f64 / 1000.0;
            for (name, cur) in &ifaces {
                if let Some(old) = p.ifaces.get(name) {
                    if cur.rx_bytes >= old.rx_bytes && cur.tx_bytes >= old.tx_bytes && dt > 0.0 {
                        snapshot.iface_rates.insert(
                            name.clone(),
                            IfaceRate {
                                rx_bps: (cur.rx_bytes - old.rx_bytes) as f64 * 8.0 / dt,
                                tx_bps: (cur.tx_bytes - old.tx_bytes) as f64 * 8.0 / dt,
                                rx_pps: cur.rx_packets.saturating_sub(old.rx_packets) as f64 / dt,
                                tx_pps: cur.tx_packets.saturating_sub(old.tx_packets) as f64 / dt,
                                drop_delta: cur.drops.saturating_sub(old.drops),
                            },
                        );
                    }
                }
            }
            rate_ready = true;
        }
    }

    // ---- 4. client rates: queue (instantaneous) first, cumulative-bytes delta as fallback
    let mut client_queue: HashMap<String, &QueueInfo> = HashMap::new();
    for n in nodes.iter().filter(|n| n.node_type == "device") {
        if let Some(ip) = n.ip.as_deref() {
            if let Some(q) = queue_for(&queues, ip) {
                client_queue.insert(n.id.clone(), q);
            }
        }
    }
    if !reuse_prev {
        let dt = prev.as_ref().map(|p| (now_ms.saturating_sub(p.at_ms)) as f64 / 1000.0).unwrap_or(0.0);
        for n in nodes.iter().filter(|n| n.node_type == "device") {
            if let Some(q) = client_queue.get(&n.id) {
                snapshot.client_rates.insert(n.id.clone(), (q.rate_up as f64, q.rate_down as f64));
                continue;
            }
            let bin = n.extra.get("bytes_in").and_then(|v| v.parse::<u64>().ok());
            let bout = n.extra.get("bytes_out").and_then(|v| v.parse::<u64>().ok());
            if let (Some(bi), Some(bo)) = (bin, bout) {
                snapshot.clients.insert(n.id.clone(), (bi, bo));
                if let Some((pi, po)) = prev.as_ref().and_then(|p| p.clients.get(&n.id)) {
                    if dt > 0.0 && bi >= *pi && bo >= *po {
                        // hotspot: bytes-in = from the user (upload), bytes-out = to the user (download)
                        snapshot.client_rates.insert(n.id.clone(), ((bi - pi) as f64 * 8.0 / dt, (bo - po) as f64 * 8.0 / dt));
                    }
                }
            }
        }
    }
    let client_rate = |id: &str| snapshot.client_rates.get(id).copied();

    // ---- 5. layout / positions
    let auto = auto_layout(&nodes, &parent, &edges);
    let pos_of = |id: &str| -> (Point, &'static str) {
        if let Some(p) = layout.nodes.get(id) {
            (Point { x: p.x, y: p.y }, "saved")
        } else {
            (auto.get(id).copied().unwrap_or(Point { x: 0.5, y: 0.5 }), "auto")
        }
    };
    let node_by_id: HashMap<&str, &TopologyNode> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // ---- 6. nodes
    let mut alerts: Vec<Value> = Vec::new();
    let mut node_health: HashMap<String, &'static str> = HashMap::new();
    let mut node_values: Vec<Value> = Vec::with_capacity(nodes.len());
    let wan_node = wan_iface_node(&nodes);

    for n in &nodes {
        let kids = children.get(&n.id).map(|v| v.len()).unwrap_or(0);
        let parent_type = parent.get(&n.id).and_then(|p| ntype.get(p)).map(|s| s.as_str());
        let auto_icon = detect_icon(n, parent_type);
        let placement = layout.nodes.get(&n.id);
        let (icon_id, icon_src) = match placement.and_then(|p| p.icon.as_deref()) {
            Some(i) if icon_exists(i) => (i.to_string(), "custom"),
            _ => (auto_icon.to_string(), "auto"),
        };
        let (pos, pos_src) = pos_of(&n.id);

        let mut reasons: Vec<Reason> = Vec::new();
        let mut base: &'static str = "ok";
        let mut traffic = json!(null);
        let mut queue_json = json!(null);

        match n.node_type.as_str() {
            "interface" => {
                let name = n.interface.clone().unwrap_or_default();
                let counters = ifaces.get(&name);
                let is_wan = n.sub_type == "wan";
                let down_link = n.status == "idle" || counters.map(|c| !c.running).unwrap_or(false);
                if down_link {
                    if is_wan || kids > 0 {
                        reasons.push(Reason { kind: "link_down", severity: "critical", message: format!("interface {name} is down (no link)") });
                    } else {
                        base = "idle";
                    }
                }
                let ratev = snapshot.iface_rates.get(&name);
                let edge_id = format!("edge_router_{name}");
                let (cap, cap_src) = link_capacity(&layout, &edge_id, n);
                let (down, up) = match ratev {
                    Some(r) if is_wan => (Some(r.rx_bps), Some(r.tx_bps)),
                    Some(r) => (Some(r.tx_bps), Some(r.rx_bps)),
                    None => (None, None),
                };
                let pps = ratev.map(|r| r.rx_pps.max(r.tx_pps)).unwrap_or(0.0);
                let drops = ratev.map(|r| r.drop_delta).unwrap_or(0);
                let (rs, ud, uu) = eval_traffic(&th, "link", down, up, cap, cap, pps, drops);
                reasons.extend(rs);
                let (tot_down, tot_up) = counters.map(|c| if is_wan { (c.rx_bytes, c.tx_bytes) } else { (c.tx_bytes, c.rx_bytes) }).unwrap_or((0, 0));
                traffic = traffic_json(
                    down, up, ratev.map(|r| r.rx_pps), ratev.map(|r| r.tx_pps), "interface_counter", cap, cap_src, ud, uu,
                    counters.map(|_| (tot_down, tot_up)),
                );
            }
            "internet" => {
                if n.status != "online" {
                    reasons.push(Reason { kind: "wan_down", severity: "critical", message: format!("WAN / ISP status: {}", n.status) });
                }
                if let Some(w) = wan_node {
                    let name = w.interface.clone().unwrap_or_default();
                    if let Some(r) = snapshot.iface_rates.get(&name) {
                        traffic = traffic_json(Some(r.rx_bps), Some(r.tx_bps), Some(r.rx_pps), Some(r.tx_pps), "interface_counter", None, "unknown", None, None, ifaces.get(&name).map(|c| (c.rx_bytes, c.tx_bytes)));
                    }
                }
            }
            "router" => {
                if let Some(w) = wan_node {
                    let name = w.interface.clone().unwrap_or_default();
                    if let Some(r) = snapshot.iface_rates.get(&name) {
                        traffic = traffic_json(Some(r.rx_bps), Some(r.tx_bps), Some(r.rx_pps), Some(r.tx_pps), "wan_interface_counter", None, "unknown", None, None, ifaces.get(&name).map(|c| (c.rx_bytes, c.tx_bytes)));
                    }
                }
            }
            "ap" => {
                let (up, down, cnt) = aggregate(&descendants(&n.id), &node_by_id, &snapshot);
                if cnt > 0 {
                    traffic = traffic_json(Some(down), Some(up), None, None, "client_aggregate", None, "unknown", None, None, None);
                }
            }
            _ => {
                // device
                match n.status.as_str() {
                    "online" | "active" | "bound" | "authorized" | "passive" => {}
                    "offline" => {
                        base = "critical";
                        reasons.push(Reason {
                            kind: "device_offline",
                            severity: "critical",
                            message: "Perangkat offline / kabel terputus (tidak terdeteksi di MikroTik)".into(),
                        });
                    }
                    _ => base = "idle",
                }
                let own = client_rate(&n.id);
                if let Some(q) = client_queue.get(&n.id) {
                    let (up, down) = own.unwrap_or((q.rate_up as f64, q.rate_down as f64));
                    let cap_up = if q.max_up > 0 { Some(q.max_up as f64) } else { None };
                    let cap_down = if q.max_down > 0 { Some(q.max_down as f64) } else { None };
                    let (rs, ud, uu) = eval_traffic(&th, "queue", Some(down), Some(up), cap_down, cap_up, 0.0, 0);
                    let limit_state = if rs.iter().any(|r| r.kind == "limit_reached") {
                        "reached"
                    } else if rs.iter().any(|r| r.kind == "limit_near") {
                        "near"
                    } else if cap_up.is_none() && cap_down.is_none() {
                        "unlimited"
                    } else {
                        "ok"
                    };
                    reasons.extend(rs);
                    queue_json = json!({
                        "name": q.name,
                        "max_down_bps": q.max_down, "max_up_bps": q.max_up,
                        "max_down_human": fmt_bits(q.max_down as f64), "max_up_human": fmt_bits(q.max_up as f64),
                        "down_util_pct": ud.map(|v| (v * 10.0).round() / 10.0),
                        "up_util_pct": uu.map(|v| (v * 10.0).round() / 10.0),
                        "limit_state": limit_state,
                        "dropped_down": q.dropped_down, "dropped_up": q.dropped_up,
                    });
                    traffic = traffic_json(Some(down), Some(up), None, None, "queue", cap_down, "queue_max_limit", ud, uu, Some((q.bytes_down, q.bytes_up)));
                } else if let Some((up, down)) = own {
                    traffic = traffic_json(Some(down), Some(up), None, None, "client_bytes_delta", None, "unknown", None, None, None);
                }
            }
        }

        let display_label = placement.and_then(|p| p.label.clone()).unwrap_or_else(|| clean_label(&n.label));
        let health = health_of(&reasons, base);
        node_health.insert(n.id.clone(), health);
        for r in &reasons {
            alerts.push(json!({
                "id": format!("{}:{}", n.id, r.kind), "severity": r.severity, "kind": r.kind,
                "node_id": n.id, "label": display_label, "message": r.message,
            }));
        }

        node_values.push(json!({
            "id": n.id,
            "label": n.label,
            "display_label": display_label,
            "node_type": n.node_type, "sub_type": n.sub_type,
            "ip": n.ip, "mac": n.mac, "vendor": n.vendor, "interface": n.interface,
            "parent_id": n.parent_id,
            "parent_source": if overridden.contains(&n.id) { "override" } else { "auto" },
            "status": n.status,
            "position": { "x": pos.x, "y": pos.y, "source": pos_src },
            "geo": {
                "lat": placement.and_then(|p| p.lat),
                "lng": placement.and_then(|p| p.lng),
                "address": placement.and_then(|p| p.address.clone()),
            },
            "icon": { "id": icon_id, "source": icon_src, "auto_id": auto_icon },
            "health": health,
            "reasons": reasons.iter().map(|r| json!({"kind": r.kind, "severity": r.severity, "message": r.message})).collect::<Vec<_>>(),
            "traffic": traffic,
            "queue": queue_json,
            "locked": placement.map(|p| p.locked).unwrap_or(false),
            "direct_children": kids,
            "extra": n.extra,
        }));
    }

    // subtree roll-up (how many descendants are warning/critical) for badges
    for v in node_values.iter_mut() {
        let id = v["id"].as_str().unwrap_or("").to_string();
        let d = descendants(&id);
        let w = d.iter().filter(|c| node_health.get(*c) == Some(&"warning")).count();
        let c = d.iter().filter(|c| node_health.get(*c) == Some(&"critical")).count();
        v["subtree"] = json!({ "total": d.len(), "warning": w, "critical": c });
    }

    // ---- 7. edges
    let mut edge_values: Vec<Value> = Vec::with_capacity(edges.len());
    for e in &edges {
        let (Some(src), Some(tgt)) = (node_by_id.get(e.source.as_str()), node_by_id.get(e.target.as_str())) else { continue };
        let (sp, _) = pos_of(&src.id);
        let (tp, _) = pos_of(&tgt.id);
        let path = layout.paths.get(&e.id);
        let waypoints: Vec<Point> = path.map(|p| p.waypoints.clone()).unwrap_or_default();
        let style = path.and_then(|p| p.style.clone()).unwrap_or_else(|| "straight".into());
        let mut polyline = vec![sp];
        polyline.extend(waypoints.iter().copied());
        polyline.push(tp);
        let length: f64 = polyline.windows(2).map(|w| ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt()).sum();

        // traffic of an edge = traffic of the node it leads to (interface counters or client aggregate)
        let tgt_traffic = node_values
            .iter()
            .find(|v| v["id"].as_str() == Some(tgt.id.as_str()))
            .map(|v| v["traffic"].clone())
            .unwrap_or(json!(null));
        let mut own_traffic = tgt_traffic.clone();

        // Direction: for a WAN interface the download enters through the port, i.e. flows toward the router.
        // Edges are stored router -> wan-iface, so "down" runs against the edge direction there.
        let wan_reverse = tgt.node_type == "interface" && tgt.sub_type == "wan" && src.node_type == "router";
        let internet_edge = src.node_type == "internet";
        if internet_edge {
            if let Some(w) = wan_node {
                if let Some(v) = node_values.iter().find(|v| v["id"].as_str() == Some(w.id.as_str())) {
                    own_traffic = v["traffic"].clone();
                }
            }
        }

        let tgt_health = node_health.get(&tgt.id).copied().unwrap_or("ok");
        let own_edge_health = if tgt.node_type == "interface" || internet_edge {
            let util = own_traffic["util_pct"]["down"].as_f64().unwrap_or(0.0).max(own_traffic["util_pct"]["up"].as_f64().unwrap_or(0.0));
            if util >= th.crit_pct { "critical" } else if util >= th.warn_pct { "warning" } else { "ok" }
        } else {
            "ok"
        };
        let health = if tgt.status == "offline" || src.status == "offline" {
            "critical"
        } else if tgt_health == "idle" {
            "idle"
        } else if tgt_health == "critical" && tgt.node_type != "device" {
            "critical"
        } else if tgt_health == "critical" || tgt_health == "warning" {
            // a limited / loaded client colours its own spur, never the trunk
            worse(own_edge_health, tgt_health)
        } else {
            own_edge_health
        };

        let is_edge_down = health == "critical" || health == "idle" || tgt.status == "offline" || src.status == "offline";
        let down_bps = if is_edge_down { 0.0 } else { own_traffic["down_bps"].as_f64().unwrap_or(0.0) };
        let up_bps = if is_edge_down { 0.0 } else { own_traffic["up_bps"].as_f64().unwrap_or(0.0) };
        let inten = |bps: f64| r4(((1.0 + bps).log10() / 9.0).clamp(0.0, 1.0));
        let (di, ui) = if own_traffic.is_null() || is_edge_down { (0.0, 0.0) } else { (inten(down_bps), inten(up_bps)) };

        edge_values.push(json!({
            "id": e.id, "source": e.source, "target": e.target,
            "edge_type": e.edge_type, "speed": e.speed,
            "health": health,
            "path": {
                "waypoints": waypoints, "polyline": polyline,
                "style": style, "length": r4(length),
                "source": if path.map(|p| !p.waypoints.is_empty()).unwrap_or(false) { "saved" } else { "auto" },
            },
            "traffic": own_traffic,
            "flow": {
                "down_intensity": di, "up_intensity": ui,
                "down_direction": if wan_reverse { "reverse" } else { "forward" },
            },
        }));
    }

    // ---- 8. summary
    let count = |h: &str| node_health.values().filter(|v| **v == h).count();
    let (wan_down, wan_up) = wan_node
        .and_then(|w| w.interface.as_ref())
        .and_then(|name| snapshot.iface_rates.get(name))
        .map(|r| (r.rx_bps, r.tx_bps))
        .unwrap_or((0.0, 0.0));

    let sev = |s: &str| alerts.iter().filter(|a| a["severity"] == s).count();
    let summary = json!({
        "nodes": node_values.len(), "edges": edge_values.len(),
        "health": { "ok": count("ok"), "idle": count("idle"), "warning": count("warning"), "critical": count("critical") },
        "alerts": { "critical": sev("critical"), "warning": sev("warning") },
        "wan": {
            "down_bps": wan_down.round(), "up_bps": wan_up.round(),
            "down_human": fmt_rate_bytes(wan_down), "up_human": fmt_rate_bytes(wan_up),
        },
    });

    let mut outages: Vec<Value> = Vec::new();
    for n in &node_values {
        if n["health"] == "critical" {
            outages.push(json!({
                "type": "node",
                "id": n["id"],
                "label": n["display_label"],
                "node_type": n["node_type"],
                "ip": n["ip"],
                "interface": n["interface"],
                "geo": n["geo"],
                "reasons": n["reasons"],
            }));
        }
    }
    for e in &edge_values {
        if e["health"] == "critical" {
            outages.push(json!({
                "type": "link",
                "id": e["id"],
                "source": e["source"],
                "target": e["target"],
                "speed": e["speed"],
                "health": e["health"],
            }));
        }
    }

    let scene = json!({
        "success": true,
        "generated_at": crate::current_iso_timestamp(),
        "rate_ready": rate_ready,
        "layout": {
            "id": layout.id, "name": layout.name, "description": layout.description,
            "exists": layout_exists, "version": layout.version, "updated_at": layout.updated_at,
            "background": {
                "kind": layout.background.kind, "url": layout.background.url,
                "aspect_ratio": layout.background.aspect_ratio, "opacity": layout.background.opacity,
                "has_upload": has_upload,
            },
            "placed_nodes": layout.nodes.len(),
        },
        "thresholds": th,
        "summary": summary,
        "outages": outages,
        "alerts": alerts,
        "warnings": warnings,
        "nodes": node_values,
        "edges": edge_values,
    });

    SceneOutput { scene, snapshot }
}

fn aggregate(ids: &[String], by_id: &HashMap<&str, &TopologyNode>, snap: &RateSnapshot) -> (f64, f64, usize) {
    let mut up = 0.0;
    let mut down = 0.0;
    let mut cnt = 0;
    for id in ids {
        if by_id.get(id.as_str()).map(|n| n.node_type == "device").unwrap_or(false) {
            if let Some((u, d)) = snap.client_rates.get(id) {
                up += u;
                down += d;
                cnt += 1;
            }
        }
    }
    (up, down, cnt)
}

fn link_capacity(layout: &MapLayout, edge_id: &str, n: &TopologyNode) -> (Option<f64>, &'static str) {
    if let Some(c) = layout.paths.get(edge_id).and_then(|p| p.capacity_bps) {
        return (Some(c as f64), "configured");
    }
    // Only plain ethernet ports get an *assumed* 1 Gbps. Everything else is unknown rather than guessed.
    if n.sub_type == "ether" {
        (Some(1_000_000_000.0), "assumed")
    } else {
        (None, "unknown")
    }
}

#[allow(clippy::too_many_arguments)]
fn traffic_json(
    down: Option<f64>,
    up: Option<f64>,
    rx_pps: Option<f64>,
    tx_pps: Option<f64>,
    source: &str,
    cap: Option<f64>,
    cap_source: &str,
    util_down: Option<f64>,
    util_up: Option<f64>,
    totals: Option<(u64, u64)>,
) -> Value {
    let d = down.unwrap_or(0.0);
    let u = up.unwrap_or(0.0);
    json!({
        "source": source,
        "down_bps": d.round(), "up_bps": u.round(),
        "down_human": fmt_rate_bytes(d), "up_human": fmt_rate_bytes(u),
        "down_bits_human": fmt_bits(d), "up_bits_human": fmt_bits(u),
        "pps_in": rx_pps.map(|v| v.round()), "pps_out": tx_pps.map(|v| v.round()),
        "capacity_bps": cap, "capacity_source": cap_source,
        "util_pct": {
            "down": util_down.map(|v| (v * 10.0).round() / 10.0),
            "up": util_up.map(|v| (v * 10.0).round() / 10.0),
        },
        "total_down_bytes": totals.map(|t| t.0), "total_up_bytes": totals.map(|t| t.1),
        "total_down_human": totals.map(|t| fmt_bytes(t.0)), "total_up_human": totals.map(|t| fmt_bytes(t.1)),
    })
}

// ------------------------------------------------------------------ live fetch

pub type StructCache = Option<(Instant, Vec<TopologyNode>, Vec<TopologyEdge>)>;

async fn fetch_counters(client: &Client) -> (HashMap<String, IfaceCounters>, Vec<QueueInfo>) {
    let iface_fut = client.run(build_command("/interface/print", std::iter::empty::<(&str, &str)>()));
    let queue_fut = client.run(build_command("/queue/simple/print", std::iter::empty::<(&str, &str)>()));
    let (ifr, qr) = tokio::join!(iface_fut, queue_fut);

    let mut ifaces = HashMap::new();
    if let Ok(rows) = ifr {
        for r in rows {
            let name = r.get("name").unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let g = |k: &str| r.get(k).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            ifaces.insert(
                name,
                IfaceCounters {
                    running: r.get("running").map(|v| v == "true").unwrap_or(false),
                    rx_bytes: g("rx-byte"),
                    tx_bytes: g("tx-byte"),
                    rx_packets: g("rx-packet"),
                    tx_packets: g("tx-packet"),
                    drops: g("rx-drop") + g("tx-drop"),
                },
            );
        }
    }

    let mut queues = Vec::new();
    if let Ok(rows) = qr {
        for r in rows {
            if r.get("disabled").map(|v| v == "true").unwrap_or(false) {
                continue;
            }
            let (max_up, max_down) = parse_pair(r.get("max-limit").unwrap_or("0/0"));
            let (rate_up, rate_down) = parse_pair(r.get("rate").unwrap_or("0/0"));
            let (bytes_up, bytes_down) = parse_pair(r.get("bytes").unwrap_or("0/0"));
            let (dropped_up, dropped_down) = parse_pair(r.get("dropped").unwrap_or("0/0"));
            queues.push(QueueInfo {
                name: r.get("name").unwrap_or("").to_string(),
                targets: parse_targets(r.get("target").unwrap_or("")),
                max_up, max_down, rate_up, rate_down, bytes_up, bytes_down, dropped_up, dropped_down,
            });
        }
    }
    (ifaces, queues)
}

/// Builds the full scene. `cache` lets a long-lived WebSocket reuse the (expensive) topology for a
/// few seconds and only re-read counters + queues each tick.
pub async fn fetch_scene(
    st: &AppState,
    tenant: &str,
    client: &Client,
    router_key: &str,
    layout_id: &str,
    filter: &str,
    cache: &mut StructCache,
) -> Result<Value, ApiError> {
    if !valid_layout_id(layout_id) {
        return Err(ApiError::BadRequest("layout_id must be 1-64 characters of a-z 0-9 _ -".into()));
    }
    let stored = st.maps.get(tenant, layout_id).await.map_err(store_err)?;
    let layout_exists = stored.is_some();
    let layout = stored.unwrap_or_else(|| MapLayout::new_empty(layout_id));
    let has_upload = layout.background.kind == "upload" && st.maps.get_background(tenant, layout_id).await.map_err(store_err)?.is_some();

    let stale = cache.as_ref().map(|(t, _, _)| t.elapsed().as_secs() >= 20).unwrap_or(true);
    let ((nodes, edges), (ifaces, queues)) = if stale {
        let (topo, counters) = tokio::join!(build_topology(client, filter), fetch_counters(client));
        *cache = Some((Instant::now(), topo.0.clone(), topo.1.clone()));
        (topo, counters)
    } else {
        let counters = fetch_counters(client).await;
        let (_, n, e) = cache.as_ref().cloned().unwrap();
        ((n, e), counters)
    };

    let key = format!("{tenant}|{router_key}");
    let prev = st.rates.lock().await.get(&key).cloned();
    let out = compose_scene(SceneInput { nodes, edges, layout, layout_exists, has_upload, ifaces, queues, prev, now_ms: mono_ms() });
    {
        let mut rates = st.rates.lock().await;
        if rates.len() >= 64 && !rates.contains_key(&key) {
            if let Some(k) = rates.keys().next().cloned() {
                rates.remove(&k);
            }
        }
        rates.insert(key, out.snapshot);
    }
    Ok(out.scene)
}

// ------------------------------------------------------------------ helpers

fn tenant_of(st: &AppState, headers: &HeaderMap) -> String {
    compute_tenant_hash(&st.token, headers.get("x-tenant-id").and_then(|v| v.to_str().ok()))
}

fn store_err(e: StoreError) -> ApiError {
    match e {
        StoreError::LimitReached(m) => ApiError::Conflict(m),
        other => {
            tracing::error!(error = %other, "map store failure");
            ApiError::Internal("map storage error (see gateway log)".into())
        }
    }
}

fn update_err(e: UpdateError) -> ApiError {
    match e {
        UpdateError::Invalid(m) if m.starts_with("__conflict__") => ApiError::Conflict(m.trim_start_matches("__conflict__").trim().to_string()),
        UpdateError::Invalid(m) => ApiError::BadRequest(m),
        UpdateError::Store(s) => store_err(s),
    }
}

fn check_id(id: &str) -> Result<(), ApiError> {
    if valid_layout_id(id) {
        Ok(())
    } else {
        Err(ApiError::BadRequest("layout id must be 1-64 characters of a-z 0-9 _ -".into()))
    }
}

fn check_key(k: &str) -> Result<(), ApiError> {
    if valid_key(k) {
        Ok(())
    } else {
        Err(ApiError::BadRequest("node/edge id may only contain letters, digits and _ - : .".into()))
    }
}

pub fn router_key(target: &Option<RouterTarget>, router_id: &Option<String>) -> String {
    target
        .as_ref()
        .and_then(|t| t.host.clone().map(|h| format!("{}:{}", h, t.port.unwrap_or(8728))))
        .or_else(|| router_id.clone())
        .unwrap_or_else(|| "main".into())
}

// ------------------------------------------------------------------ handlers

#[derive(Deserialize, Default)]
pub struct SceneReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub layout_id: Option<String>,
    pub filter_type: Option<String>,
    /// auto-place only: also move nodes that already have a saved position (locked ones never move).
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Deserialize, Default)]
pub struct SceneQuery {
    pub layout_id: Option<String>,
    pub filter: Option<String>,
}

/// GET|POST /api/v1/map/scene - the one call a map widget needs.
pub async fn scene(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    query: Option<Query<SceneQuery>>,
    req: Option<Json<SceneReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let q = query.map(|Query(q)| q).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let key = router_key(&target, &router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let layout_id = req.layout_id.or(q.layout_id).unwrap_or_else(|| "default".into());
    let filter = req.filter_type.or(q.filter).unwrap_or_else(|| "all".into());
    let tenant = tenant_of(&st, &headers);
    let mut cache: StructCache = None;
    Ok(Json(fetch_scene(&st, &tenant, &client, &key, &layout_id, &filter, &mut cache).await?))
}

/// GET /api/v1/map/icons
pub async fn icons() -> Json<Value> {
    Json(json!({
        "success": true,
        "count": ICONS.len(),
        "icons": ICONS.iter().map(|(id, label, cat)| json!({"id": id, "label": label, "category": cat})).collect::<Vec<_>>(),
    }))
}

/// GET /api/v1/map/layouts
pub async fn list_layouts(State(st): State<Arc<AppState>>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let tenant = tenant_of(&st, &headers);
    let items = st.maps.list(&tenant).await.map_err(store_err)?;
    Ok(Json(json!({ "success": true, "count": items.len(), "layouts": items })))
}

/// GET /api/v1/map/layouts/:id
pub async fn get_layout(State(st): State<Arc<AppState>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    let tenant = tenant_of(&st, &headers);
    match st.maps.get(&tenant, &id).await.map_err(store_err)? {
        Some(l) => Ok(Json(json!({ "success": true, "layout": l }))),
        None => Err(ApiError::NotFound(format!("layout '{id}' does not exist yet - PUT it, or place a node to create it"))),
    }
}

/// PUT /api/v1/map/layouts/:id - create or fully replace a layout (also usable as import).
pub async fn put_layout(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<MapLayout>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    let tenant = tenant_of(&st, &headers);
    let expected = body.version;
    let default_name = id.clone();
    let layout = st
        .maps
        .update(&tenant, &id, move |cur| {
            if expected != 0 && cur.version != expected {
                return Err(format!(
                    "__conflict__ layout is at version {} but you sent {expected}; reload it and re-apply your change",
                    cur.version
                ));
            }
            if body.background.kind == "upload" && cur.background.kind != "upload" {
                return Err("background.kind 'upload' can only be set by uploading an image (PUT /layouts/{id}/background)".into());
            }
            cur.name = if body.name.trim().is_empty() { default_name } else { body.name };
            cur.description = body.description;
            cur.router_id = body.router_id;
            cur.background = body.background;
            cur.nodes = body.nodes;
            cur.paths = body.paths;
            cur.thresholds = body.thresholds;
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "layout": layout })))
}

/// DELETE /api/v1/map/layouts/:id
pub async fn delete_layout(State(st): State<Arc<AppState>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    let tenant = tenant_of(&st, &headers);
    let existed = st.maps.delete(&tenant, &id).await.map_err(store_err)?;
    if !existed {
        return Err(ApiError::NotFound(format!("layout '{id}' does not exist")));
    }
    Ok(Json(json!({ "success": true, "deleted": id })))
}

#[derive(Deserialize, Default)]
pub struct NodePatch {
    pub x: Option<f64>,
    pub y: Option<f64>,
    /// "" resets to the auto-detected icon.
    pub icon: Option<String>,
    /// "" resets to the router-reported label.
    pub label: Option<String>,
    pub locked: Option<bool>,
    /// "" removes the override (back to the auto-detected parent).
    pub parent_override: Option<String>,
}

/// PUT /api/v1/map/layouts/:id/nodes/:node_id - place a node and/or change icon, label, lock, parent.
/// The layout is created on first use.
pub async fn patch_node(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, node_id)): Path<(String, String)>,
    Json(p): Json<NodePatch>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    check_key(&node_id)?;
    let tenant = tenant_of(&st, &headers);
    let nid = node_id.clone();
    let layout = st
        .maps
        .update(&tenant, &id, move |l| {
            let mut cur = match l.nodes.get(&nid) {
                Some(c) => c.clone(),
                None => match (p.x, p.y) {
                    (Some(x), Some(y)) => NodePlacement { x, y, ..Default::default() },
                    _ => return Err("x and y are required the first time a node is placed".into()),
                },
            };
            if let Some(x) = p.x { cur.x = x; }
            if let Some(y) = p.y { cur.y = y; }
            if let Some(i) = p.icon { cur.icon = if i.is_empty() { None } else { Some(i) }; }
            if let Some(t) = p.label { cur.label = if t.is_empty() { None } else { Some(t) }; }
            if let Some(k) = p.locked { cur.locked = k; }
            if let Some(po) = p.parent_override { cur.parent_override = if po.is_empty() { None } else { Some(po) }; }
            l.nodes.insert(nid, cur);
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "node": layout.nodes.get(&node_id), "version": layout.version })))
}

/// DELETE /api/v1/map/layouts/:id/nodes/:node_id - forget the placement (node goes back to auto position).
pub async fn delete_node(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, node_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    check_key(&node_id)?;
    let tenant = tenant_of(&st, &headers);
    if st.maps.get(&tenant, &id).await.map_err(store_err)?.is_none() {
        return Err(ApiError::NotFound(format!("layout '{id}' does not exist")));
    }
    let nid = node_id.clone();
    let layout = st
        .maps
        .update(&tenant, &id, move |l| {
            l.nodes.remove(&nid);
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "version": layout.version })))
}

/// PUT /api/v1/map/layouts/:id/paths/:edge_id - road waypoints, line style, capacity of one link.
pub async fn set_path(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, edge_id)): Path<(String, String)>,
    Json(p): Json<EdgePath>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    check_key(&edge_id)?;
    let tenant = tenant_of(&st, &headers);
    let eid = edge_id.clone();
    let layout = st
        .maps
        .update(&tenant, &id, move |l| {
            l.paths.insert(eid, p);
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "path": layout.paths.get(&edge_id), "version": layout.version })))
}

/// DELETE /api/v1/map/layouts/:id/paths/:edge_id
pub async fn delete_path(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((id, edge_id)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    check_key(&edge_id)?;
    let tenant = tenant_of(&st, &headers);
    if st.maps.get(&tenant, &id).await.map_err(store_err)?.is_none() {
        return Err(ApiError::NotFound(format!("layout '{id}' does not exist")));
    }
    let layout = st
        .maps
        .update(&tenant, &id, move |l| {
            l.paths.remove(&edge_id);
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "version": layout.version })))
}

/// POST /api/v1/map/layouts/:id/auto-place - saves the auto-layout positions of nodes that are not
/// placed yet (or all unlocked nodes with `overwrite:true`) so the developer starts from a sane map.
pub async fn auto_place(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    req: Option<Json<SceneReq>>,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let overwrite = req.overwrite;
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let key = router_key(&target, &router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;
    let tenant = tenant_of(&st, &headers);
    let filter = req.filter_type.unwrap_or_else(|| "all".into());
    let mut cache: StructCache = None;
    let scene = fetch_scene(&st, &tenant, &client, &key, &id, &filter, &mut cache).await?;

    let mut to_place: Vec<(String, f64, f64)> = Vec::new();
    for n in scene["nodes"].as_array().cloned().unwrap_or_default() {
        let src = n["position"]["source"].as_str().unwrap_or("auto");
        if src == "auto" || (overwrite && !n["locked"].as_bool().unwrap_or(false)) {
            if let (Some(nid), Some(x), Some(y)) = (n["id"].as_str(), n["position"]["x"].as_f64(), n["position"]["y"].as_f64()) {
                if valid_key(nid) {
                    to_place.push((nid.to_string(), x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)));
                }
            }
        }
    }
    let placed = to_place.len();
    let layout = st
        .maps
        .update(&tenant, &id, move |l| {
            for (nid, x, y) in to_place {
                if l.nodes.len() >= MAX_NODES && !l.nodes.contains_key(&nid) {
                    return Err(format!("too many placed nodes (max {MAX_NODES})"));
                }
                let e = l.nodes.entry(nid).or_default();
                e.x = x;
                e.y = y;
            }
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "placed": placed, "version": layout.version, "total_placed": layout.nodes.len() })))
}

/// PUT /api/v1/map/layouts/:id/background - raw PNG/JPEG/WebP bytes (max 4 MB).
pub async fn put_background(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    if body.len() > MAX_BG_BYTES {
        return Err(ApiError::PayloadTooLarge(format!("image is larger than {} MB", MAX_BG_BYTES / 1024 / 1024)));
    }
    let mime = sniff_image(&body).ok_or_else(|| ApiError::UnsupportedMedia("only PNG, JPEG and WebP images are accepted (SVG is refused because it can carry script)".into()))?;
    let tenant = tenant_of(&st, &headers);
    st.maps.put_background(&tenant, &id, &body).await.map_err(store_err)?;
    let layout = st
        .maps
        .update(&tenant, &id, |l| {
            l.background.kind = "upload".into();
            l.background.url = None;
            Ok(())
        })
        .await
        .map_err(update_err)?;
    Ok(Json(json!({ "success": true, "bytes": body.len(), "content_type": mime, "version": layout.version })))
}

/// GET /api/v1/map/layouts/:id/background
pub async fn get_background(State(st): State<Arc<AppState>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Response, ApiError> {
    check_id(&id)?;
    let tenant = tenant_of(&st, &headers);
    let bytes = st.maps.get_background(&tenant, &id).await.map_err(store_err)?.ok_or_else(|| ApiError::NotFound("this layout has no uploaded background".into()))?;
    let mime = sniff_image(&bytes).unwrap_or("application/octet-stream");
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "private, no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CONTENT_SECURITY_POLICY, "default-src 'none'; sandbox"),
        ],
        bytes,
    )
        .into_response())
}

/// DELETE /api/v1/map/layouts/:id/background
pub async fn delete_background(State(st): State<Arc<AppState>>, headers: HeaderMap, Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    check_id(&id)?;
    let tenant = tenant_of(&st, &headers);
    st.maps.delete_background(&tenant, &id).await.map_err(store_err)?;
    if st.maps.get(&tenant, &id).await.map_err(store_err)?.is_some() {
        st.maps
            .update(&tenant, &id, |l| {
                if l.background.kind == "upload" {
                    l.background.kind = "none".into();
                }
                Ok(())
            })
            .await
            .map_err(update_err)?;
    }
    Ok(Json(json!({ "success": true })))
}

// ------------------------------------------------------------------ static assets

const MAP_WIDGET_JS: &str = include_str!("assets/mikrotik-map.js");
const MAP_PAGE_JS: &str = include_str!("assets/map-page.js");
const MAP_PAGE_HTML: &str = include_str!("assets/map.html");

fn js_response(body: &'static str) -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

/// GET /sdk/mikrotik-map.js - the embeddable `<mikrotik-map>` web component.
pub async fn widget_js() -> Response {
    js_response(MAP_WIDGET_JS)
}

/// GET /sdk/map-page.js - script of the stand-alone /map page (separate file so CSP can forbid inline scripts).
pub async fn page_js() -> Response {
    js_response(MAP_PAGE_JS)
}

/// GET /map - stand-alone viewer/editor built on the same web component.
pub async fn page() -> Response {
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::REFERRER_POLICY, "no-referrer"),
            (
                header::CONTENT_SECURITY_POLICY,
                "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' https: http: data: blob:; connect-src 'self' ws: wss:; base-uri 'none'; form-action 'none'; frame-ancestors 'self'",
            ),
        ],
        MAP_PAGE_HTML,
    )
        .into_response()
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as Map;

    fn node(id: &str, ntype: &str, sub: &str, parent: Option<&str>, ip: Option<&str>, status: &str) -> TopologyNode {
        TopologyNode {
            id: id.into(), label: id.into(), node_type: ntype.into(), sub_type: sub.into(),
            ip: ip.map(|s| s.into()), mac: None, vendor: None,
            interface: if ntype == "interface" { Some(id.trim_start_matches("if_").into()) } else { None },
            parent_id: parent.map(|s| s.into()), status: status.into(), extra: Map::new(),
        }
    }
    fn edge(s: &str, t: &str) -> TopologyEdge {
        TopologyEdge { id: format!("edge_{s}_{t}"), source: s.into(), target: t.into(), edge_type: "physical".into(), speed: None }
    }

    fn base_net() -> (Vec<TopologyNode>, Vec<TopologyEdge>) {
        let nodes = vec![
            node("node_router", "router", "core", None, None, "online"),
            node("node_internet", "internet", "wan_cloud", None, None, "online"),
            node("if_ether1", "interface", "wan", Some("node_router"), None, "online"),
            node("if_ether5", "interface", "ether", Some("node_router"), None, "online"),
            node("ap_ether5_AA", "ap", "ap", Some("if_ether5"), Some("192.168.100.3"), "online"),
            node("ap_ether5_BB", "ap", "ap", Some("if_ether5"), Some("192.168.100.4"), "online"),
            node("client_dhcp_1", "device", "dhcp", Some("ap_ether5_AA"), Some("192.168.100.20"), "bound"),
            node("client_dhcp_2", "device", "dhcp", Some("ap_ether5_AA"), Some("192.168.100.21"), "bound"),
        ];
        let edges = vec![
            TopologyEdge { id: "edge_router_ether1".into(), source: "node_router".into(), target: "if_ether1".into(), edge_type: "wan".into(), speed: None },
            TopologyEdge { id: "edge_internet_wan".into(), source: "node_internet".into(), target: "if_ether1".into(), edge_type: "wan".into(), speed: None },
            TopologyEdge { id: "edge_router_ether5".into(), source: "node_router".into(), target: "if_ether5".into(), edge_type: "physical".into(), speed: None },
            edge("if_ether5", "ap_ether5_AA"),
            edge("if_ether5", "ap_ether5_BB"),
            edge("ap_ether5_AA", "client_dhcp_1"),
            edge("ap_ether5_AA", "client_dhcp_2"),
        ];
        (nodes, edges)
    }

    fn ifc(rx: u64, tx: u64) -> IfaceCounters {
        IfaceCounters { running: true, rx_bytes: rx, tx_bytes: tx, rx_packets: rx / 1000, tx_packets: tx / 1000, drops: 0 }
    }

    fn run(layout: MapLayout, ifaces: HashMap<String, IfaceCounters>, queues: Vec<QueueInfo>, prev: Option<RateSnapshot>, now: u64) -> SceneOutput {
        let (nodes, edges) = base_net();
        compose_scene(SceneInput { nodes, edges, layout, layout_exists: true, has_upload: false, ifaces, queues, prev, now_ms: now })
    }

    fn find<'a>(scene: &'a Value, key: &str, id: &str) -> &'a Value {
        scene[key].as_array().unwrap().iter().find(|v| v["id"] == id).unwrap_or_else(|| panic!("{id} missing"))
    }

    #[test]
    fn parsers() {
        assert_eq!(parse_si("10M"), 10_000_000);
        assert_eq!(parse_si("512k"), 512_000);
        assert_eq!(parse_si("1200"), 1200);
        assert_eq!(parse_pair("5000000/10000000"), (5_000_000, 10_000_000));
        assert_eq!(parse_pair("garbage"), (0, 0));
        assert_eq!(parse_targets("192.168.1.10/32,10.0.0.0/8,ether1"), vec![(parse_ipv4("192.168.1.10").unwrap(), 32), (parse_ipv4("10.0.0.0").unwrap(), 8)]);
        assert!(parse_ipv4("300.1.1.1").is_none());
    }

    #[test]
    fn most_specific_queue_wins() {
        let q = vec![
            QueueInfo { name: "all".into(), targets: parse_targets("192.168.100.0/24"), ..Default::default() },
            QueueInfo { name: "one".into(), targets: parse_targets("192.168.100.20/32"), ..Default::default() },
        ];
        assert_eq!(queue_for(&q, "192.168.100.20").unwrap().name, "one");
        assert_eq!(queue_for(&q, "192.168.100.99").unwrap().name, "all");
        assert!(queue_for(&q, "10.1.1.1").is_none());
    }

    #[test]
    fn human_formats() {
        assert_eq!(fmt_rate_bytes(8.0 * 512.0), "512 B/s");
        assert_eq!(fmt_rate_bytes(8.0 * 2048.0), "2.0 KB/s");
        assert_eq!(fmt_rate_bytes(8.0 * 3.0 * 1_048_576.0), "3.00 MB/s");
        assert_eq!(fmt_bytes(1_572_864), "1.50 MB");
        assert_eq!(fmt_bits(2_500_000.0), "2.50 Mbps");
    }

    #[test]
    fn icons_are_detected_and_always_valid() {
        let mut n = node("c", "device", "dhcp", None, None, "bound");
        n.label = "Galaxy-S23".into();
        assert_eq!(detect_icon(&n, None), "phone");
        n.label = "Hikvision-DS-2CD".into();
        assert_eq!(detect_icon(&n, None), "camera");
        n.label = "ESP_AABBCC".into();
        n.vendor = Some("Espressif IoT Device".into());
        assert_eq!(detect_icon(&n, None), "iot");
        n.label = "LivingRoom-TV".into();
        n.vendor = None;
        assert_eq!(detect_icon(&n, None), "tv");
        n.label = "weird".into();
        assert_eq!(detect_icon(&n, None), "unknown");
        n.label = "TP-Link-AP-Ruang-Tamu".into();
        assert_eq!(detect_icon(&n, None), "ap");
        n.label = "Apple-Watch".into();
        assert_eq!(detect_icon(&n, None), "phone", "'apple' must not be read as the token 'ap'");
        let ap = node("a", "ap", "ap", None, None, "online");
        assert_eq!(detect_icon(&ap, Some("interface")), "ap");
        assert_eq!(detect_icon(&ap, Some("ap")), "repeater");
        let (nodes, _) = base_net();
        for n in &nodes {
            assert!(icon_exists(detect_icon(n, None)));
        }
    }

    #[test]
    fn default_labels_are_cleaned_for_the_map() {
        assert_eq!(clean_label("📱 Host: Galaxy (192.168.88.20)"), "Galaxy");
        assert_eq!(clean_label("📡 AP: Rooftop (192.168.88.3)"), "Rooftop");
        assert_eq!(clean_label("🌐 Internet (ISP: 203.0.113.6)"), "Internet");
        assert_eq!(clean_label("📶 WiFi: Xiaomi (AA:BB:CC:DD:EE:FF)"), "Xiaomi");
        assert_eq!(clean_label("ether1 (WAN / Internet In)"), "ether1 (WAN / Internet In)");
        assert_eq!(clean_label("🎟️ voucher-01 (10.5.50.2)"), "voucher-01");
        assert_eq!(clean_label("📱"), "📱", "never returns an empty name");
        assert_eq!(clean_label("MikroTik hAP"), "MikroTik hAP");
    }

    #[test]
    fn unplaced_nodes_get_positions_inside_the_canvas() {
        let out = run(MapLayout::new_empty("t"), HashMap::new(), vec![], None, 1000);
        for n in out.scene["nodes"].as_array().unwrap() {
            let (x, y) = (n["position"]["x"].as_f64().unwrap(), n["position"]["y"].as_f64().unwrap());
            assert!((0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y), "{} at {x},{y}", n["id"]);
            assert_eq!(n["position"]["source"], "auto");
        }
        // internet is left of the router, router left of its interfaces
        let x = |id: &str| find(&out.scene, "nodes", id)["position"]["x"].as_f64().unwrap();
        assert!(x("node_internet") < x("node_router") && x("node_router") < x("if_ether5") && x("if_ether5") < x("ap_ether5_AA"));
    }

    #[test]
    fn saved_position_icon_label_and_waypoints_are_applied() {
        let mut l = MapLayout::new_empty("t");
        l.nodes.insert("ap_ether5_AA".into(), NodePlacement { x: 0.7, y: 0.2, icon: Some("server".into()), label: Some("Rooftop AP".into()), locked: true, parent_override: None, ..Default::default() });
        l.nodes.insert("if_ether5".into(), NodePlacement { x: 0.4, y: 0.5, ..Default::default() });
        l.paths.insert("edge_if_ether5_ap_ether5_AA".into(), EdgePath { waypoints: vec![Point { x: 0.4, y: 0.2 }, Point { x: 0.6, y: 0.2 }], style: Some("orthogonal".into()), capacity_bps: None });
        let out = run(l, HashMap::new(), vec![], None, 1000);
        let ap = find(&out.scene, "nodes", "ap_ether5_AA");
        assert_eq!(ap["position"]["source"], "saved");
        assert_eq!(ap["icon"]["id"], "server");
        assert_eq!(ap["icon"]["source"], "custom");
        assert_eq!(ap["icon"]["auto_id"], "ap");
        assert_eq!(ap["display_label"], "Rooftop AP");
        assert_eq!(ap["locked"], true);
        let e = find(&out.scene, "edges", "edge_if_ether5_ap_ether5_AA");
        assert_eq!(e["path"]["style"], "orthogonal");
        assert_eq!(e["path"]["polyline"].as_array().unwrap().len(), 4);
        assert_eq!(e["path"]["source"], "saved");
    }

    #[test]
    fn relay_chain_reparents_ap_and_creates_relay_edge() {
        let mut l = MapLayout::new_empty("t");
        l.nodes.insert("ap_ether5_BB".into(), NodePlacement { x: 0.8, y: 0.8, parent_override: Some("ap_ether5_AA".into()), ..Default::default() });
        let out = run(l, HashMap::new(), vec![], None, 1000);
        let b = find(&out.scene, "nodes", "ap_ether5_BB");
        assert_eq!(b["parent_id"], "ap_ether5_AA");
        assert_eq!(b["parent_source"], "override");
        assert_eq!(b["icon"]["id"], "repeater");
        let relay = find(&out.scene, "edges", "edge_ap_ether5_AA_ap_ether5_BB");
        assert_eq!(relay["edge_type"], "relay");
        assert!(out.scene["edges"].as_array().unwrap().iter().all(|e| e["id"] != "edge_if_ether5_ap_ether5_BB"), "old uplink edge must be gone");
        let a = find(&out.scene, "nodes", "ap_ether5_AA");
        assert_eq!(a["subtree"]["total"], 3);
    }

    #[test]
    fn relay_loop_is_refused() {
        let mut l = MapLayout::new_empty("t");
        l.nodes.insert("ap_ether5_AA".into(), NodePlacement { x: 0.1, y: 0.1, parent_override: Some("client_dhcp_1".into()), ..Default::default() });
        let out = run(l, HashMap::new(), vec![], None, 1000);
        assert_eq!(find(&out.scene, "nodes", "ap_ether5_AA")["parent_id"], "if_ether5");
        assert_eq!(out.scene["warnings"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn interface_rates_come_from_counter_deltas() {
        let mut first = HashMap::new();
        first.insert("ether1".to_string(), ifc(1_000_000, 100_000));
        first.insert("ether5".to_string(), ifc(0, 0));
        let a = run(MapLayout::new_empty("t"), first, vec![], None, 10_000);
        assert_eq!(a.scene["rate_ready"], false, "first sample has no rate yet");

        let mut second = HashMap::new();
        // 2 s later: WAN received +2 MiB (download), sent +256 KiB
        second.insert("ether1".to_string(), ifc(1_000_000 + 2_097_152, 100_000 + 262_144));
        second.insert("ether5".to_string(), ifc(0, 0));
        let b = run(MapLayout::new_empty("t"), second, vec![], Some(a.snapshot), 12_000);
        assert_eq!(b.scene["rate_ready"], true);
        let wan = find(&b.scene, "nodes", "if_ether1");
        let down = wan["traffic"]["down_bps"].as_f64().unwrap();
        assert!((down - 8.0 * 1_048_576.0).abs() < 2.0, "down_bps = {down}");
        assert_eq!(wan["traffic"]["down_human"], "1.00 MB/s");
        assert_eq!(wan["traffic"]["up_human"], "128.0 KB/s");
        assert_eq!(b.scene["summary"]["wan"]["down_human"], "1.00 MB/s");
        // WAN edge animation runs against the router->wan edge direction
        assert_eq!(find(&b.scene, "edges", "edge_router_ether1")["flow"]["down_direction"], "reverse");
        assert_eq!(find(&b.scene, "edges", "edge_internet_wan")["flow"]["down_direction"], "forward");
    }

    #[test]
    fn counter_reset_does_not_produce_negative_or_huge_rates() {
        let mut first = HashMap::new();
        first.insert("ether1".to_string(), ifc(9_000_000_000, 9_000_000_000));
        let a = run(MapLayout::new_empty("t"), first, vec![], None, 1000);
        let mut second = HashMap::new();
        second.insert("ether1".to_string(), ifc(10, 10)); // router rebooted
        let b = run(MapLayout::new_empty("t"), second, vec![], Some(a.snapshot), 3000);
        let wan = find(&b.scene, "nodes", "if_ether1");
        assert_eq!(wan["traffic"]["down_bps"].as_f64().unwrap_or(0.0), 0.0);
    }

    #[test]
    fn rapid_resampling_reuses_previous_rates() {
        let mut first = HashMap::new();
        first.insert("ether1".to_string(), ifc(0, 0));
        let a = run(MapLayout::new_empty("t"), first, vec![], None, 1000);
        let mut second = HashMap::new();
        second.insert("ether1".to_string(), ifc(1_000_000, 0));
        let b = run(MapLayout::new_empty("t"), second.clone(), vec![], Some(a.snapshot), 3000);
        let rate_b = find(&b.scene, "nodes", "if_ether1")["traffic"]["down_bps"].clone();
        // a second consumer asks 100 ms later: must not divide by ~0 and explode
        let c = run(MapLayout::new_empty("t"), second, vec![], Some(b.snapshot), 3100);
        assert_eq!(find(&c.scene, "nodes", "if_ether1")["traffic"]["down_bps"], rate_b);
    }

    #[test]
    fn queue_limit_turns_client_red_and_alerts() {
        let q = vec![QueueInfo {
            name: "client1".into(), targets: parse_targets("192.168.100.20/32"),
            max_down: 10_000_000, max_up: 2_000_000, rate_down: 9_800_000, rate_up: 100_000, ..Default::default()
        }];
        let out = run(MapLayout::new_empty("t"), HashMap::new(), q, None, 1000);
        let c = find(&out.scene, "nodes", "client_dhcp_1");
        assert_eq!(c["health"], "critical");
        assert_eq!(c["queue"]["limit_state"], "reached");
        assert_eq!(c["traffic"]["down_human"], "1.17 MB/s");
        assert!(out.scene["alerts"].as_array().unwrap().iter().any(|a| a["kind"] == "limit_reached" && a["node_id"] == "client_dhcp_1"));
        // a limited client colours its own spur, not the trunk and not its AP
        assert_eq!(find(&out.scene, "edges", "edge_ap_ether5_AA_client_dhcp_1")["health"], "critical");
        assert_eq!(find(&out.scene, "nodes", "ap_ether5_AA")["health"], "ok");
        assert_eq!(find(&out.scene, "nodes", "ap_ether5_AA")["subtree"]["critical"], 1);
        assert_eq!(find(&out.scene, "edges", "edge_router_ether5")["health"], "ok");
        // the other client is fine
        assert_eq!(find(&out.scene, "nodes", "client_dhcp_2")["health"], "ok");
    }

    #[test]
    fn queue_between_warn_and_crit_is_warning() {
        let q = vec![QueueInfo { name: "c".into(), targets: parse_targets("192.168.100.21/32"), max_down: 10_000_000, max_up: 0, rate_down: 8_500_000, ..Default::default() }];
        let out = run(MapLayout::new_empty("t"), HashMap::new(), q, None, 1000);
        let c = find(&out.scene, "nodes", "client_dhcp_2");
        assert_eq!(c["health"], "warning");
        assert_eq!(c["queue"]["limit_state"], "near");
    }

    #[test]
    fn down_link_with_children_is_red_without_children_is_grey() {
        let mut ifs = HashMap::new();
        ifs.insert("ether5".to_string(), IfaceCounters { running: false, ..Default::default() });
        ifs.insert("ether1".to_string(), IfaceCounters { running: false, ..Default::default() });
        let out = run(MapLayout::new_empty("t"), ifs, vec![], None, 1000);
        assert_eq!(find(&out.scene, "nodes", "if_ether5")["health"], "critical");
        assert_eq!(find(&out.scene, "nodes", "if_ether1")["health"], "critical", "WAN down is always critical");
        assert_eq!(find(&out.scene, "edges", "edge_router_ether5")["health"], "critical");
    }

    #[test]
    fn high_utilisation_uses_configured_capacity() {
        let mut l = MapLayout::new_empty("t");
        l.paths.insert("edge_router_ether1".into(), EdgePath { capacity_bps: Some(100_000_000), ..Default::default() });
        let mut first = HashMap::new();
        first.insert("ether1".to_string(), ifc(0, 0));
        let a = run(l.clone(), first, vec![], None, 0);
        let mut second = HashMap::new();
        // 97 Mbit in 1 s on a 100 Mbit link -> 97 % => critical (>= 95)
        second.insert("ether1".to_string(), ifc(12_125_000, 0));
        let b = run(l, second, vec![], Some(a.snapshot), 1000);
        let wan = find(&b.scene, "nodes", "if_ether1");
        assert_eq!(wan["traffic"]["capacity_source"], "configured");
        assert_eq!(wan["health"], "critical");
        assert_eq!(find(&b.scene, "edges", "edge_router_ether1")["health"], "critical");
    }

    #[test]
    fn assumed_capacity_is_only_for_ethernet_and_flagged() {
        let out = run(MapLayout::new_empty("t"), HashMap::new(), vec![], None, 0);
        assert_eq!(find(&out.scene, "nodes", "if_ether5")["traffic"]["capacity_source"], "assumed");
        assert_eq!(find(&out.scene, "nodes", "if_ether1")["traffic"]["capacity_source"], "unknown");
    }

    #[test]
    fn flow_intensity_scales_with_throughput() {
        let mut first = HashMap::new();
        first.insert("ether5".to_string(), ifc(0, 0));
        let a = run(MapLayout::new_empty("t"), first, vec![], None, 0);
        let mut second = HashMap::new();
        second.insert("ether5".to_string(), ifc(0, 12_500_000)); // 100 Mbit/s towards clients
        let b = run(MapLayout::new_empty("t"), second, vec![], Some(a.snapshot), 1000);
        let e = find(&b.scene, "edges", "edge_router_ether5");
        let di = e["flow"]["down_intensity"].as_f64().unwrap();
        assert!(di > 0.8 && di <= 1.0, "intensity {di}");
        assert_eq!(e["flow"]["up_intensity"].as_f64().unwrap(), 0.0);
    }

    #[test]
    fn ap_traffic_is_the_sum_of_its_clients() {
        let q = vec![
            QueueInfo { name: "1".into(), targets: parse_targets("192.168.100.20/32"), rate_down: 1_000_000, rate_up: 200_000, ..Default::default() },
            QueueInfo { name: "2".into(), targets: parse_targets("192.168.100.21/32"), rate_down: 3_000_000, rate_up: 400_000, ..Default::default() },
        ];
        let out = run(MapLayout::new_empty("t"), HashMap::new(), q, None, 0);
        let ap = find(&out.scene, "nodes", "ap_ether5_AA");
        assert_eq!(ap["traffic"]["down_bps"], 4_000_000.0);
        assert_eq!(ap["traffic"]["up_bps"], 600_000.0);
        assert_eq!(ap["traffic"]["source"], "client_aggregate");
        let none = find(&out.scene, "nodes", "ap_ether5_BB");
        assert!(none["traffic"].is_null(), "no known client rates -> no invented numbers");
    }
}
