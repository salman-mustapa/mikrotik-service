use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug, Default)]
pub struct BaseReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GenerateBatchVouchersReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub qty: usize,
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default = "default_len")]
    pub length: usize,
    #[serde(default = "default_charset")]
    pub character_set: String, // "numeric", "alphanumeric_lower", "alphanumeric_upper", "alphanumeric_mixed"
    #[serde(default = "default_mode")]
    pub user_mode: String,     // "user_is_pass" or "user_and_pass"
    #[serde(default = "default_profile")]
    pub profile: String,
    pub time_limit: Option<String>,  // e.g. "1h", "3h", "1d", "7d"
    pub validity: Option<String>,    // e.g. "1h", "3h", "1d", "7d" (calendar validity)
    pub data_limit: Option<String>,  // e.g. "500M", "1G", "5G"
    pub price: Option<String>,       // e.g. "3000", "5000", "10000"
    pub server: Option<String>,      // default "all"
    pub comment_tag: Option<String>, // custom batch tag, e.g. "BATCH-OKT"
    pub dns_name: Option<String>,    // e.g. "hotspot.local" or "login.wifi"
}

#[derive(Deserialize, Debug)]
pub struct CreateSingleVoucherReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Username voucher (jika kosong, otomatis digenerate dengan prefix)
    pub username: Option<String>,
    /// Password voucher (jika kosong atau user_mode == "user_is_pass", sama dengan username)
    pub password: Option<String>,
    #[serde(default = "default_mode")]
    pub user_mode: String, // "user_is_pass" atau "user_and_pass"
    #[serde(default = "default_profile")]
    pub profile: String,
    pub time_limit: Option<String>, // e.g. "1h", "3h", "1d"
    pub validity: Option<String>,   // e.g. "1d", "7d", "30d"
    pub data_limit: Option<String>, // e.g. "500M", "2G"
    pub price: Option<String>,      // e.g. "3000", "5000"
    pub server: Option<String>,
    pub comment: Option<String>,
    pub dns_name: Option<String>,
}

fn default_prefix() -> String { "V-".into() }
fn default_len() -> usize { 6 }
fn default_charset() -> String { "alphanumeric_lower".into() }
fn default_mode() -> String { "user_is_pass".into() }
fn default_profile() -> String { "default".into() }

#[derive(Deserialize, Debug)]
pub struct FilterVouchersReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile: Option<String>,
    pub status: Option<String>, // "all", "available", "online", "used", "expired"
    pub batch_tag: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ThermalPrintReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub vouchers: Vec<String>, // Array of usernames
    #[serde(default = "default_paper_width")]
    pub paper_width: String,   // "58mm" or "80mm"
    pub hotspot_name: Option<String>,
    pub dns_name: Option<String>,
}

fn default_paper_width() -> String { "58mm".into() }

#[derive(Deserialize, Debug)]
pub struct SellVoucherReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub voucher_code: String,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>, // WhatsApp / Phone number, e.g. "628123456789"
    pub cashier: Option<String>,
    pub webhook_url: Option<String>,    // Generic external webhook (e.g. to Laravel / WhatsApp service)
    pub hotspot_name: Option<String>,
    pub dns_name: Option<String>,
}

pub type SellAndSendReq = SellVoucherReq;

#[derive(Deserialize, Debug)]
pub struct CleanExpiredReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile: Option<String>,
}

/// Helper function to parse RouterOS duration string (e.g. "1w2d3h4m5s", "01:30:00", "2h30m") into seconds
fn parse_ros_duration_seconds(raw: &str) -> u64 {
    let s = raw.trim();
    if s.is_empty() || s == "0" {
        return 0;
    }

    // Handle "HH:MM:SS" format
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 3 {
            let h: u64 = parts[0].parse().unwrap_or(0);
            let m: u64 = parts[1].parse().unwrap_or(0);
            let sec: u64 = parts[2].parse().unwrap_or(0);
            return h * 3600 + m * 60 + sec;
        } else if parts.len() == 2 {
            let m: u64 = parts[0].parse().unwrap_or(0);
            let sec: u64 = parts[1].parse().unwrap_or(0);
            return m * 60 + sec;
        }
    }

    // Handle "1w2d3h4m5s" format
    let mut total_secs: u64 = 0;
    let mut current_num: u64 = 0;

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num = current_num * 10 + (c as u64 - '0' as u64);
        } else {
            match c {
                'w' => total_secs += current_num * 7 * 86400,
                'd' => total_secs += current_num * 86400,
                'h' => total_secs += current_num * 3600,
                'm' => total_secs += current_num * 60,
                's' => total_secs += current_num,
                _ => {}
            }
            current_num = 0;
        }
    }

    total_secs
}

/// Helper function to format seconds into readable duration (e.g. "2h 30m 15s")
fn format_duration_seconds(secs: u64) -> String {
    if secs == 0 {
        return "0s".into();
    }
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;

    let mut res = String::new();
    if d > 0 { res.push_str(&format!("{}d ", d)); }
    if h > 0 { res.push_str(&format!("{}h ", h)); }
    if m > 0 { res.push_str(&format!("{}m ", m)); }
    if s > 0 || res.is_empty() { res.push_str(&format!("{}s", s)); }
    res.trim().to_string()
}

/// Helper function to parse human-readable byte limit string (e.g. "500M", "1G", "2G") into bytes
fn parse_byte_str(raw: &str) -> u64 {
    let s = raw.trim().to_uppercase();
    if s.is_empty() {
        return 0;
    }

    if s.ends_with('G') || s.ends_with("GB") {
        let num: f64 = s.trim_end_matches("GB").trim_end_matches('G').parse().unwrap_or(0.0);
        return (num * 1024.0 * 1024.0 * 1024.0) as u64;
    }
    if s.ends_with('M') || s.ends_with("MB") {
        let num: f64 = s.trim_end_matches("MB").trim_end_matches('M').parse().unwrap_or(0.0);
        return (num * 1024.0 * 1024.0) as u64;
    }
    if s.ends_with('K') || s.ends_with("KB") {
        let num: f64 = s.trim_end_matches("KB").trim_end_matches('K').parse().unwrap_or(0.0);
        return (num * 1024.0) as u64;
    }

    s.parse::<u64>().unwrap_or(0)
}

/// Helper function to format bytes into readable string (e.g. "1.5 GB", "420.3 MB")
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Helper function to parse month name (e.g. "oct", "nov") to 1..12
fn parse_ros_month(m: &str) -> Option<u32> {
    match m.to_lowercase().as_str() {
        "jan" => Some(1), "feb" => Some(2), "mar" => Some(3), "apr" => Some(4),
        "may" => Some(5), "jun" => Some(6), "jul" => Some(7), "aug" => Some(8),
        "sep" => Some(9), "oct" => Some(10), "nov" => Some(11), "dec" => Some(12),
        _ => None,
    }
}

fn is_leap_year(y: u32) -> bool {
    y.is_multiple_of(4) && !y.is_multiple_of(100) || y.is_multiple_of(400)
}

/// Convert (year, month, day, hour, min, sec) to seconds since UNIX epoch (1970-01-01 00:00:00 UTC)
fn date_time_to_unix(year: u32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Option<u64> {
    if year < 1970 || !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || min > 59 || sec > 59 {
        return None;
    }

    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut days: u64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }

    let is_current_leap = is_leap_year(year);
    for m in 1..month {
        if m == 2 && is_current_leap {
            days += 29;
        } else {
            days += days_in_months[(m - 1) as usize];
        }
    }

    days += (day.saturating_sub(1)) as u64;
    Some(days * 86400 + (hour as u64) * 3600 + (min as u64) * 60 + (sec as u64))
}

/// Parse date string in either RouterOS v6 format ("mmm/dd/yyyy HH:MM:SS") or RouterOS v7 format ("yyyy-mm-dd HH:MM:SS")
fn parse_ros_timestamp(date_str: &str, time_str: &str) -> Option<u64> {
    let t_parts: Vec<&str> = time_str.split(':').collect();
    if t_parts.len() < 3 {
        return None;
    }
    let hour: u32 = t_parts[0].parse().ok()?;
    let min: u32 = t_parts[1].parse().ok()?;
    let sec: u32 = t_parts[2].parse().ok()?;

    if date_str.contains('/') {
        // ROS v6: "mmm/dd/yyyy"
        let d_parts: Vec<&str> = date_str.split('/').collect();
        if d_parts.len() >= 3 {
            let month = parse_ros_month(d_parts[0])?;
            let day: u32 = d_parts[1].parse().ok()?;
            let year: u32 = d_parts[2].parse().ok()?;
            return date_time_to_unix(year, month, day, hour, min, sec);
        }
    } else if date_str.contains('-') {
        // ROS v7: "yyyy-mm-dd"
        let d_parts: Vec<&str> = date_str.split('-').collect();
        if d_parts.len() >= 3 {
            let year: u32 = d_parts[0].parse().ok()?;
            let month: u32 = d_parts[1].parse().ok()?;
            let day: u32 = d_parts[2].parse().ok()?;
            return date_time_to_unix(year, month, day, hour, min, sec);
        }
    }

    None
}

/// Helper function to check if a voucher comment has expired based on [EXP:...|VAL:...]
pub fn is_voucher_comment_expired(comment: &str, now_unix: u64) -> bool {
    if !comment.contains("EXP:") {
        return false;
    }

    let exp_part = match comment.split("EXP:").nth(1) {
        Some(p) => p,
        None => return false,
    };

    let date_time_segment = exp_part.split('|').next().unwrap_or("").trim_end_matches(']');
    let mut dt_tokens = date_time_segment.split_whitespace();
    let date_str = match dt_tokens.next() {
        Some(d) => d,
        None => return false,
    };
    let time_str = match dt_tokens.next() {
        Some(t) => t,
        None => return false,
    };

    let start_unix = match parse_ros_timestamp(date_str, time_str) {
        Some(ts) => ts,
        None => return false,
    };

    let validity_secs = if comment.contains("VAL:") {
        let val_part = comment.split("VAL:").nth(1).unwrap_or("").split('|').next().unwrap_or("").trim_end_matches(']');
        parse_ros_duration_seconds(val_part)
    } else {
        86400
    };

    if validity_secs == 0 {
        return false;
    }

    now_unix >= start_unix + validity_secs
}

/// POST /api/v1/hotspot/vouchers/generate - High-speed parallel voucher batch creation
pub async fn generate_vouchers(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<GenerateBatchVouchersReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.profile, "profile")?;
    if !req.prefix.is_empty() {
        crate::routes::sanitize::validate_safe_identifier(&req.prefix, "prefix")?;
    }
    if let Some(ref t) = req.time_limit {
        crate::routes::sanitize::validate_validity(t)?;
    }
    if let Some(ref v) = req.validity {
        crate::routes::sanitize::validate_validity(v)?;
    }
    if let Some(ref d) = req.data_limit {
        crate::routes::sanitize::validate_bandwidth_rate(d)?;
    }
    if let Some(ref p) = req.price {
        crate::routes::sanitize::validate_price(p)?;
    }
    if let Some(ref s) = req.server {
        crate::routes::sanitize::validate_safe_identifier(s, "server")?;
    }
    if let Some(ref tag) = req.comment_tag {
        crate::routes::sanitize::validate_safe_identifier(tag, "comment_tag")?;
    }

    let qty = req.qty.clamp(1, 1000);
    let len = req.length.clamp(4, 16);
    let dns = req.dns_name.unwrap_or_else(|| "hotspot.local".into());
    let price_str = req.price.unwrap_or_else(|| "3000".into());

    let charset: &'static [u8] = match req.character_set.as_str() {
        "numeric" => b"1234567890",
        "alphanumeric_upper" => b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789",
        "alphanumeric_mixed" => b"abcdefghjkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789",
        _ => b"abcdefghjkmnpqrstuvwxyz23456789", // alphanumeric_lower (safe no confusing 0/O, 1/l)
    };

    let seed_base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1234567890);

    let batch_tag = req.comment_tag.unwrap_or_else(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        format!("VC-{}", t)
    });

    let val_label = req.validity.as_deref().or(req.time_limit.as_deref()).unwrap_or("-");

    let mut generated_items = Vec::with_capacity(qty);
    let mut futures = Vec::with_capacity(qty);

    for i in 0..qty {
        let mut code = String::with_capacity(len);
        let mut val = seed_base.wrapping_add((i as u128).wrapping_mul(7890123456789));
        for _ in 0..len {
            val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let idx = (val as usize) % charset.len();
            code.push(charset[idx] as char);
        }

        let username = format!("{}{}", req.prefix, code);
        let password = if req.user_mode == "user_and_pass" {
            // Generate separate 4-char numeric pin
            let pin_val = (val >> 16) % 9000 + 1000;
            pin_val.to_string()
        } else {
            username.clone()
        };

        let structured_comment = format!("[VOUCHER|{}|Rp {}|{}|VAL:{}]", req.profile, price_str, batch_tag, val_label);

        let mut args = vec![
            ("name".to_string(), username.clone()),
            ("password".to_string(), password.clone()),
            ("profile".to_string(), req.profile.clone()),
            ("comment".to_string(), structured_comment),
        ];

        if let Some(ref t) = req.time_limit {
            args.push(("limit-uptime".to_string(), t.clone()));
        }
        if let Some(ref d) = req.data_limit {
            args.push(("limit-bytes-total".to_string(), d.clone()));
        }
        if let Some(ref s) = req.server {
            args.push(("server".to_string(), s.clone()));
        }

        let words: Vec<String> = build_command(
            "/ip/hotspot/user/add",
            args.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        );
        futures.push(client.run(words));

        let login_url = format!("http://{}/login?username={}&password={}", dns, username, password);
        let qr_url = format!("https://api.qrserver.com/v1/create-qr-code/?size=120x120&data={}", login_url);

        generated_items.push(json!({
            "username": username,
            "password": password,
            "profile": req.profile,
            "time_limit": req.time_limit,
            "validity": req.validity,
            "data_limit": req.data_limit,
            "price": price_str,
            "batch_tag": batch_tag,
            "login_url": login_url,
            "qr_code_url": qr_url,
        }));
    }

    let results = futures::future::join_all(futures).await;
    let success_count = results.into_iter().filter(|r| r.is_ok()).count();

    Ok(Json(json!({
        "success": true,
        "total_requested": qty,
        "total_created": success_count,
        "batch_tag": batch_tag,
        "profile": req.profile,
        "vouchers": generated_items,
    })))
}

/// POST /api/v1/hotspot/vouchers/create-single - Buat 1 Voucher Fleksibel (Custom Username/Password atau Auto-Generate)
pub async fn create_single_voucher(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateSingleVoucherReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    crate::routes::sanitize::validate_safe_identifier(&req.profile, "profile")?;
    if let Some(ref t) = req.time_limit {
        crate::routes::sanitize::validate_validity(t)?;
    }
    if let Some(ref v) = req.validity {
        crate::routes::sanitize::validate_validity(v)?;
    }
    let price_str = req.price.as_deref().unwrap_or("0");
    crate::routes::sanitize::validate_price(price_str)?;

    let username = match req.username {
        Some(u) if !u.trim().is_empty() => {
            crate::routes::sanitize::validate_safe_identifier(&u, "username")?;
            u.trim().to_string()
        }
        _ => {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(987654321);
            let chars = b"23456789abcdefghjkmnpqrstuvwxyz";
            let mut rnd = String::with_capacity(6);
            let mut val = seed;
            for _ in 0..6 {
                val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let idx = (val as usize) % chars.len();
                rnd.push(chars[idx] as char);
            }
            format!("V-{}", rnd)
        }
    };

    let password = match req.password {
        Some(p) if !p.trim().is_empty() && req.user_mode != "user_is_pass" => {
            if p.contains(['\r', '\n', '"', ';']) {
                return Err(ApiError::BadRequest("Password contains invalid characters".into()));
            }
            p.trim().to_string()
        }
        _ => username.clone(),
    };

    let mut comment_parts = vec![format!("[VOUCHER|{}|Rp {}]", req.profile, price_str)];
    if let Some(ref v) = req.validity {
        comment_parts.push(format!("[VAL:{}]", v));
    }
    if let Some(ref c) = req.comment {
        comment_parts.push(c.clone());
    }
    let comment_final = comment_parts.join(" ");

    let mut args = vec![
        ("name", username.as_str()),
        ("password", password.as_str()),
        ("profile", req.profile.as_str()),
        ("comment", comment_final.as_str()),
    ];

    if let Some(ref t) = req.time_limit {
        args.push(("limit-uptime", t.as_str()));
    }
    if let Some(ref d) = req.data_limit {
        args.push(("limit-bytes-total", d.as_str()));
    }
    let srv = req.server.as_deref().unwrap_or("all");
    args.push(("server", srv));

    client.run(build_command("/ip/hotspot/user/add", args)).await?;

    let dns = req.dns_name.as_deref().unwrap_or("hotspot.local");
    let login_url = format!("http://{}/login?username={}&password={}", dns, username, password);
    let qr_url = format!("https://api.qrserver.com/v1/create-qr-code/?size=150x150&data={}", url_encode(&login_url));

    Ok(Json(json!({
        "success": true,
        "voucher": {
            "username": username,
            "password": password,
            "profile": req.profile,
            "time_limit": req.time_limit,
            "validity": req.validity,
            "data_limit": req.data_limit,
            "price": price_str,
            "login_url": login_url,
            "qr_code_url": qr_url,
            "comment": comment_final
        },
        "message": format!("Voucher '{}' berhasil dibuat!", username)
    })))
}

/// GET or POST /api/v1/hotspot/vouchers/history - Riwayat Lengkap User yang Pernah Aktif, Pernah Terpakai & Total Konsumsi Kuota
pub async fn voucher_usage_history(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterVouchersReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or(FilterVouchersReq {
        router: None,
        router_id: None,
        profile: None,
        status: None,
        batch_tag: None,
    });
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let user_fut = client.run(build_command("/ip/hotspot/user/print", std::iter::empty::<(&str, &str)>()));
    let act_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));

    let (user_res, act_res) = tokio::join!(user_fut, act_fut);
    let user_rows = user_res?;
    let act_rows = act_res.unwrap_or_default();

    let mut active_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for row in act_rows {
        if let Some(user) = row.get("user") {
            active_map.insert(user.to_string(), row.attrs);
        }
    }

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut online_list = Vec::new();
    let mut previously_used_list = Vec::new();
    let mut expired_list = Vec::new();
    let mut unused_list = Vec::new();

    let mut total_bytes_all: u64 = 0;
    let mut total_uptime_secs_all: u64 = 0;

    for row in user_rows {
        let name = row.get("name").unwrap_or("").to_string();
        if name.is_empty() || name == "default-trial" {
            continue;
        }

        let profile = row.get("profile").unwrap_or("default").to_string();
        if let Some(ref p_filter) = req.profile {
            if &profile != p_filter {
                continue;
            }
        }

        let comment = row.get("comment").unwrap_or("").to_string();
        let uptime_str = row.get("uptime").unwrap_or("0s").to_string();
        let limit_uptime_str = row.get("limit-uptime").unwrap_or("").to_string();
        let bytes_in: u64 = row.get("bytes-in").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_out: u64 = row.get("bytes-out").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_total = bytes_in + bytes_out;
        let limit_bytes_str = row.get("limit-bytes-total").unwrap_or("").to_string();
        let limit_bytes: u64 = limit_bytes_str.parse().unwrap_or_else(|_| parse_byte_str(&limit_bytes_str));

        let used_uptime_secs = parse_ros_duration_seconds(&uptime_str);
        let limit_uptime_secs = parse_ros_duration_seconds(&limit_uptime_str);

        total_bytes_all += bytes_total;
        total_uptime_secs_all += used_uptime_secs;

        let is_online = active_map.contains_key(&name);
        let is_calendar_expired = is_voucher_comment_expired(&comment, now_ts);
        let is_limit_expired = (limit_uptime_secs > 0 && used_uptime_secs >= limit_uptime_secs) || (limit_bytes > 0 && bytes_total >= limit_bytes);

        let voucher_info = json!({
            "username": name,
            "profile": profile,
            "comment": comment,
            "uptime": uptime_str,
            "uptime_seconds": used_uptime_secs,
            "bytes_in": bytes_in,
            "bytes_out": bytes_out,
            "bytes_total": bytes_total,
            "bytes_total_formatted": format_bytes_human(bytes_total),
            "limit_uptime": limit_uptime_str,
            "limit_bytes": limit_bytes_str,
            "is_online": is_online,
            "active_session": active_map.get(&name)
        });

        if is_online {
            online_list.push(voucher_info);
        } else if is_calendar_expired || is_limit_expired {
            expired_list.push(voucher_info);
        } else if used_uptime_secs > 0 || bytes_total > 0 {
            previously_used_list.push(voucher_info);
        } else {
            unused_list.push(voucher_info);
        }
    }

    Ok(Json(json!({
        "success": true,
        "summary": {
            "total_vouchers": online_list.len() + previously_used_list.len() + expired_list.len() + unused_list.len(),
            "online_now": online_list.len(),
            "previously_used": previously_used_list.len(),
            "unused_inventory": unused_list.len(),
            "expired": expired_list.len(),
            "total_bandwidth_consumed": total_bytes_all,
            "total_bandwidth_consumed_formatted": format_bytes_human(total_bytes_all),
            "total_uptime_seconds": total_uptime_secs_all
        },
        "online_vouchers": online_list,
        "previously_used_vouchers": previously_used_list,
        "expired_vouchers": expired_list,
        "unused_vouchers": unused_list
    })))
}

fn format_bytes_human(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// POST /api/v1/hotspot/vouchers/tracking - Realtime voucher lifecycle, quota, and uptime tracker
pub async fn voucher_tracking(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<FilterVouchersReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or(FilterVouchersReq {
        router: None,
        router_id: None,
        profile: None,
        status: None,
        batch_tag: None,
    });
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Concurrently fetch users and active sessions
    let user_fut = client.run(build_command("/ip/hotspot/user/print", std::iter::empty::<(&str, &str)>()));
    let act_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));

    let (user_res, act_res) = tokio::join!(user_fut, act_fut);
    let user_rows = user_res?;
    let act_rows = act_res.unwrap_or_default();

    // Map active users: username -> active session details
    let mut active_map: HashMap<String, HashMap<String, String>> = HashMap::new();
    for row in act_rows {
        if let Some(user) = row.get("user") {
            active_map.insert(user.to_string(), row.attrs);
        }
    }

    let mut vouchers = Vec::new();
    let mut stats = json!({
        "total": 0,
        "available": 0,
        "online": 0,
        "used": 0,
        "expired": 0,
        "total_revenue_potential": 0,
    });

    for row in user_rows {
        let name = row.get("name").unwrap_or("").to_string();
        if name.is_empty() || name == "default-trial" {
            continue;
        }

        let profile = row.get("profile").unwrap_or("default").to_string();
        if let Some(ref p_filter) = req.profile {
            if &profile != p_filter {
                continue;
            }
        }

        let comment = row.get("comment").unwrap_or("").to_string();
        if let Some(ref tag_filter) = req.batch_tag {
            if !comment.contains(tag_filter) {
                continue;
            }
        }

        // Parse metrics
        let uptime_str = row.get("uptime").unwrap_or("0s").to_string();
        let limit_uptime_str = row.get("limit-uptime").unwrap_or("").to_string();
        let bytes_in: u64 = row.get("bytes-in").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_out: u64 = row.get("bytes-out").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_total = bytes_in + bytes_out;
        let limit_bytes_str = row.get("limit-bytes-total").unwrap_or("").to_string();
        let limit_bytes: u64 = limit_bytes_str.parse().unwrap_or_else(|_| parse_byte_str(&limit_bytes_str));

        let used_uptime_secs = parse_ros_duration_seconds(&uptime_str);
        let limit_uptime_secs = parse_ros_duration_seconds(&limit_uptime_str);

        let is_online = active_map.contains_key(&name);

        // Determine accurate status
        let status = if is_online {
            "online"
        } else if (limit_uptime_secs > 0 && used_uptime_secs >= limit_uptime_secs) || (limit_bytes > 0 && bytes_total >= limit_bytes) {
            "expired"
        } else if used_uptime_secs > 0 || bytes_total > 0 {
            "used"
        } else {
            "available"
        };

        if let Some(ref s_filter) = req.status {
            if s_filter != "all" && status != s_filter {
                continue;
            }
        }

        // Remaining calculations
        let remaining_uptime_secs = limit_uptime_secs.saturating_sub(used_uptime_secs);
        let remaining_bytes = limit_bytes.saturating_sub(bytes_total);
        let quota_percent_left = if limit_bytes > 0 {
            format!("{:.1}%", (remaining_bytes as f64 / limit_bytes as f64) * 100.0)
        } else {
            "Unlimited".into()
        };

        // Extract price from structured comment [VOUCHER|Profile|Rp 3000|...]
        let price = if comment.contains("Rp ") {
            comment.split("Rp ").nth(1).and_then(|p| p.split('|').next()).unwrap_or("0").trim().to_string()
        } else {
            "0".to_string()
        };

        let price_num: u64 = price.replace(['.', ','], "").parse().unwrap_or(0);

        // Update stats
        let total = stats["total"].as_u64().unwrap_or(0) + 1;
        stats["total"] = json!(total);
        match status {
            "available" => {
                let count = stats["available"].as_u64().unwrap_or(0) + 1;
                stats["available"] = json!(count);
                let rev = stats["total_revenue_potential"].as_u64().unwrap_or(0) + price_num;
                stats["total_revenue_potential"] = json!(rev);
            }
            "online" => {
                let count = stats["online"].as_u64().unwrap_or(0) + 1;
                stats["online"] = json!(count);
            }
            "used" => {
                let count = stats["used"].as_u64().unwrap_or(0) + 1;
                stats["used"] = json!(count);
            }
            "expired" => {
                let count = stats["expired"].as_u64().unwrap_or(0) + 1;
                stats["expired"] = json!(count);
            }
            _ => {}
        }

        let mut voucher_entry = json!({
            "id": row.get(".id").unwrap_or(""),
            "username": name,
            "password": row.get("password").unwrap_or(""),
            "profile": profile,
            "status": status,
            "is_online": is_online,
            "uptime_used": uptime_str,
            "uptime_used_seconds": used_uptime_secs,
            "uptime_limit": limit_uptime_str,
            "uptime_remaining": format_duration_seconds(remaining_uptime_secs),
            "bytes_used": format_bytes(bytes_total),
            "bytes_used_raw": bytes_total,
            "bytes_limit": format_bytes(limit_bytes),
            "bytes_remaining": format_bytes(remaining_bytes),
            "quota_percent_left": quota_percent_left,
            "price": price,
            "comment": comment,
        });

        if let Some(act) = active_map.get(&name) {
            voucher_entry["active_session"] = json!({
                "ip_address": act.get("address").map(|s| s.as_str()).unwrap_or(""),
                "mac_address": act.get("mac-address").map(|s| s.as_str()).unwrap_or(""),
                "session_uptime": act.get("uptime").map(|s| s.as_str()).unwrap_or(""),
                "bytes_in": act.get("bytes-in").map(|s| s.as_str()).unwrap_or("0"),
                "bytes_out": act.get("bytes-out").map(|s| s.as_str()).unwrap_or("0"),
            });
        }

        vouchers.push(voucher_entry);
    }

    Ok(Json(json!({
        "success": true,
        "summary": stats,
        "count": vouchers.len(),
        "vouchers": vouchers
    })))
}

/// POST /api/v1/hotspot/vouchers/thermal-print - Generates ESC/POS 58mm/80mm thermal receipt payload
pub async fn thermal_print(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ThermalPrintReq>,
) -> Result<Json<Value>, ApiError> {
    if req.vouchers.len() > 500 {
        return Err(ApiError::BadRequest("Cannot print more than 500 vouchers at once".into()));
    }
    for v in &req.vouchers {
        crate::routes::sanitize::validate_safe_identifier(v, "voucher username")?;
    }

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let hotspot = req.hotspot_name.as_deref().unwrap_or("WIFI HOTSPOT");
    let dns = req.dns_name.as_deref().unwrap_or("hotspot.local");
    let is_80mm = req.paper_width == "80mm";
    let line_width = if is_80mm { 42 } else { 32 };
    let separator = "-".repeat(line_width);

    let rows = client.run(build_command("/ip/hotspot/user/print", std::iter::empty::<(&str, &str)>())).await?;
    let mut vouchers_map = HashMap::new();
    for r in rows {
        if let Some(name) = r.get("name") {
            vouchers_map.insert(name.to_string(), r.attrs);
        }
    }

    let mut receipts = Vec::new();

    for user_code in req.vouchers {
        if let Some(data) = vouchers_map.get(&user_code) {
            let pass = data.get("password").map(|s| s.as_str()).unwrap_or(user_code.as_str());
            let profile = data.get("profile").map(|s| s.as_str()).unwrap_or("Regular");
            let timelimit = data.get("limit-uptime").map(|s| s.as_str()).unwrap_or("1 Hari");
            let comment = data.get("comment").map(|s| s.as_str()).unwrap_or("");
            let price = if comment.contains("Rp ") {
                comment.split("Rp ").nth(1).and_then(|p| p.split('|').next()).unwrap_or("3.000").trim()
            } else {
                "3.000"
            };

            let login_url = format!("http://{}/login?username={}&password={}", dns, user_code, pass);

            // ESC/POS Formatted Plain Text representation
            let formatted_text = format!(
                "{sep}\n\
                 {center_title}\n\
                 {sep}\n\
                 KODE VOUCHER : {user}\n\
                 PASSWORD     : {pass}\n\
                 PAKET        : {profile}\n\
                 MASA AKTIF   : {limit}\n\
                 TARIF        : Rp {price}\n\
                 {sep}\n\
                 Login URL    : http://{dns}\n\
                 Scan QR Code di bawah untuk Auto-Login\n\
                 {sep}\n",
                sep = separator,
                center_title = hotspot,
                user = user_code,
                pass = pass,
                profile = profile,
                limit = timelimit,
                price = price,
                dns = dns
            );

            receipts.push(json!({
                "username": user_code,
                "password": pass,
                "profile": profile,
                "price": price,
                "time_limit": timelimit,
                "login_url": login_url,
                "qr_code_url": format!("https://api.qrserver.com/v1/create-qr-code/?size=100x100&data={}", login_url),
                "paper_width": req.paper_width,
                "thermal_text": formatted_text,
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "count": receipts.len(),
        "receipts": receipts
    })))
}

/// POST /api/v1/hotspot/vouchers/sell - Mark voucher as SOLD and return structured data + receipt template
pub async fn sell_voucher(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SellVoucherReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_safe_identifier(&req.voucher_code, "voucher_code")?;

    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let hotspot = req.hotspot_name.as_deref().unwrap_or("WIFI HOTSPOT");
    let dns = req.dns_name.as_deref().unwrap_or("hotspot.local");
    let cust_name = req.customer_name.as_deref().unwrap_or("Pelanggan");
    if cust_name.len() > 64 || cust_name.contains(['|', ']', '[', '\r', '\n']) {
        return Err(ApiError::BadRequest("customer_name contains invalid characters or exceeds 64 characters".into()));
    }
    let clean_phone = if let Some(p) = req.customer_phone.as_deref() {
        let cp: String = p.chars().filter(|c| c.is_ascii_digit()).collect();
        if cp.len() > 20 {
            return Err(ApiError::BadRequest("customer_phone must be 20 digits or fewer".into()));
        }
        cp
    } else {
        String::new()
    };
    let cashier = req.cashier.as_deref().unwrap_or("Admin Kasir");
    if cashier.len() > 64 || cashier.contains(['|', ']', '[', '\r', '\n']) {
        return Err(ApiError::BadRequest("cashier contains invalid characters or exceeds 64 characters".into()));
    }

    // Query voucher data from router
    let rows = client.run(build_command(
        "/ip/hotspot/user/print",
        [("name", req.voucher_code.as_str())],
    )).await?;

    let v_row = rows.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Voucher '{}' tidak ditemukan pada router", req.voucher_code))
    })?;

    let pass = v_row.get("password").unwrap_or(req.voucher_code.as_str());
    let profile = v_row.get("profile").unwrap_or("Regular");
    let limit = v_row.get("limit-uptime").unwrap_or("Aktif");
    let comment = v_row.get("comment").unwrap_or("");
    let price = if comment.contains("Rp ") {
        comment.split("Rp ").nth(1).and_then(|p| p.split('|').next()).unwrap_or("3.000").trim()
    } else {
        "3.000"
    };

    let login_url = format!("http://{}/login?username={}&password={}", dns, req.voucher_code, pass);

    // Update comment on router to mark as SOLD
    let now_ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let sold_comment = format!("[SOLD|{}|Rp {}|{}|TS-{}]", cust_name, price, clean_phone, now_ts);
    if let Some(id) = v_row.get(".id") {
        let _ = client.run(build_command(
            "/ip/hotspot/user/set",
            [(".id", id), ("comment", sold_comment.as_str())],
        )).await;
    }

    // Format ready-to-send receipt text for WhatsApp/SMS messaging
    let formatted_receipt = format!(
        "🎉 *STRUK PEMBELIAN VOUCHER WIFI*\n\
         *{}*\n\
         ----------------------------------\n\
         👤 *Nama:* {}\n\
         🎫 *Kode Voucher:* `{}`\n\
         🔑 *Password:* `{}`\n\
         📦 *Paket:* {}\n\
         ⏱️ *Masa Aktif:* {}\n\
         💰 *Tarif:* Rp {}\n\
         ----------------------------------\n\
         🔗 *Klik Link Auto-Login:*\n\
         {}\n\n\
         Terima kasih telah menggunakan layanan wifi kami!",
        hotspot, cust_name, req.voucher_code, pass, profile, limit, price, login_url
    );

    // Optional webhook forwarder (e.g. if caller provides webhook URL to Laravel, Next.js, or WhatsApp bridge)
    let mut webhook_forwarded = false;
    if let Some(ref wh) = req.webhook_url {
        crate::routes::sanitize::validate_safe_url(wh)?;
        let host_part = wh.split("://").nth(1).unwrap_or("").split('/').next().unwrap_or("").split(':').next().unwrap_or("");
        if crate::state::is_blocked_host(host_part) {
            return Err(ApiError::Forbidden("Webhook URL points to blocked host/network".into()));
        }

        let encoded_msg = url_encode(&formatted_receipt);
        let target_url = if wh.contains('?') {
            format!("{}&phone={}&message={}", wh, clean_phone, encoded_msg)
        } else {
            format!("{}/send?phone={}&message={}", wh.trim_end_matches('/'), clean_phone, encoded_msg)
        };
        let res = client.run(build_command(
            "/tool/fetch",
            [("url", target_url.as_str()), ("keep-result", "no")],
        )).await;
        webhook_forwarded = res.is_ok();
    }

    Ok(Json(json!({
        "success": true,
        "voucher": {
            "code": req.voucher_code,
            "password": pass,
            "profile": profile,
            "price": price,
            "time_limit": limit,
            "login_url": login_url,
            "qr_code_url": format!("https://api.qrserver.com/v1/create-qr-code/?size=100x100&data={}", login_url),
        },
        "sale": {
            "customer_name": cust_name,
            "customer_phone": clean_phone,
            "cashier": cashier,
            "sold_at": now_ts,
        },
        "receipt_text": formatted_receipt,
        "webhook_forwarded": webhook_forwarded,
        "message": "Voucher berhasil ditandai terjual di router. Data siap diolah oleh aplikasi bisnis (Laravel/Next.js)!"
    })))
}

/// Alias for backward compatibility
pub async fn sell_and_send(
    st: State<Arc<AppState>>,
    headers: HeaderMap,
    req: Json<SellVoucherReq>,
) -> Result<Json<Value>, ApiError> {
    sell_voucher(st, headers, req).await
}

/// POST /api/v1/hotspot/vouchers/clean-expired - Safely purges expired vouchers from MikroTik
pub async fn clean_expired(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CleanExpiredReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/hotspot/user/print", std::iter::empty::<(&str, &str)>())).await?;

    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut removed_count = 0;
    for r in rows {
        let name = r.get("name").unwrap_or("");
        if name.is_empty() || name == "default-trial" { continue; }

        let profile = r.get("profile").unwrap_or("default");
        if let Some(ref p_filter) = req.profile {
            if profile != p_filter { continue; }
        }

        let uptime_str = r.get("uptime").unwrap_or("0s");
        let limit_uptime_str = r.get("limit-uptime").unwrap_or("");
        let comment = r.get("comment").unwrap_or("");
        let bytes_in: u64 = r.get("bytes-in").and_then(|v| v.parse().ok()).unwrap_or(0);
        let bytes_out: u64 = r.get("bytes-out").and_then(|v| v.parse().ok()).unwrap_or(0);
        let limit_bytes: u64 = r.get("limit-bytes-total").and_then(|v| v.parse().ok()).unwrap_or(0);

        let used_secs = parse_ros_duration_seconds(uptime_str);
        let limit_secs = parse_ros_duration_seconds(limit_uptime_str);

        let is_expired = (limit_secs > 0 && used_secs >= limit_secs)
            || (limit_bytes > 0 && (bytes_in + bytes_out) >= limit_bytes)
            || is_voucher_comment_expired(comment, now_unix);

        if is_expired {
            if let Some(id) = r.get(".id") {
                let _ = client.run(build_command("/ip/hotspot/user/remove", [(".id", id)])).await;
                // Also disconnect active session if user is currently online
                let _ = client.run(build_command("/ip/hotspot/active/remove", [("?user", name)])).await;
                // Also remove scheduler if any
                let _ = client.run(build_command("/system/scheduler/remove", [("?name", name)])).await;
                removed_count += 1;
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "removed_vouchers_count": removed_count,
        "message": format!("Berhasil membersihkan {} voucher yang telah kedaluwarsa", removed_count)
    })))
}

#[derive(Deserialize, Debug, Default)]
pub struct SalesReportReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile: Option<String>,
    pub cashier: Option<String>,
}

/// GET /api/v1/hotspot/vouchers/sales-report - Financial & Sales Revenue Summary
pub async fn sales_report(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<SalesReportReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/ip/hotspot/user/print", std::iter::empty::<(&str, &str)>())).await?;

    let mut total_vouchers = 0;
    let mut total_sold = 0;
    let mut total_available = 0;
    let mut total_revenue: u64 = 0;
    let mut potential_revenue: u64 = 0;

    let mut profile_stats: HashMap<String, (u64, u64)> = HashMap::new();
    let mut recent_sales = Vec::new();

    for r in rows {
        let name = r.get("name").unwrap_or("");
        if name.is_empty() || name == "default-trial" { continue; }

        let profile = r.get("profile").unwrap_or("default").to_string();
        if let Some(ref p_filt) = req.profile {
            if &profile != p_filt { continue; }
        }

        let comment = r.get("comment").unwrap_or("");
        total_vouchers += 1;

        // Parse price from comment, e.g. "Rp 3.000" or metadata tag
        let price: u64 = if comment.contains("Rp ") {
            comment.split("Rp ").nth(1)
                .and_then(|p| p.split('|').next())
                .unwrap_or("0")
                .replace('.', "")
                .trim()
                .parse()
                .unwrap_or(0)
        } else {
            0
        };

        let is_sold = comment.contains("[SOLD");
        if is_sold {
            total_sold += 1;
            total_revenue += price;

            let entry = profile_stats.entry(profile.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += price;

            if recent_sales.len() < 25 {
                recent_sales.push(json!({
                    "username": name,
                    "profile": profile,
                    "price": price,
                    "comment": comment
                }));
            }
        } else {
            total_available += 1;
            potential_revenue += price;
        }
    }

    let breakdown: Vec<Value> = profile_stats.into_iter().map(|(prof, (count, rev))| {
        json!({
            "profile": prof,
            "vouchers_sold": count,
            "revenue": rev,
            "revenue_formatted": format!("Rp {}", rev)
        })
    }).collect();

    Ok(Json(json!({
        "success": true,
        "financial_summary": {
            "total_vouchers_in_router": total_vouchers,
            "vouchers_sold": total_sold,
            "vouchers_available": total_available,
            "total_revenue": total_revenue,
            "total_revenue_formatted": format!("Rp {}", total_revenue),
            "potential_unrealized_revenue": potential_revenue,
            "potential_unrealized_revenue_formatted": format!("Rp {}", potential_revenue),
        },
        "breakdown_by_profile": breakdown,
        "recent_sales": recent_sales
    })))
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}

// ============================================================================
// Autonomous Router Sweeper Engine (1-Job RouterOS Background Sweeper)
// ============================================================================

#[derive(Deserialize, Debug, Default)]
pub struct DeployCleanerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    #[serde(default = "default_cleaner_interval")]
    pub interval: String, // e.g. "5m", "10m"
}

fn default_cleaner_interval() -> String {
    "5m".into()
}

/// POST /api/v1/hotspot/sweeper/deploy - Pasang autonomous sweeper script & scheduler di RouterOS
/// Berjalan mandiri di RouterOS (cukup 1 task saja untuk ribuan voucher, zero bloat, 0-1% CPU)
pub async fn deploy_auto_cleaner(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<DeployCleanerReq>,
) -> Result<Json<Value>, ApiError> {
    crate::routes::sanitize::validate_validity(&req.interval)?;
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let cleaner_script_name = "hotspot_auto_sweeper";
    let cleaner_source = ":local uList [/ip hotspot user find where comment~\"EXP:\"];\n\
        :foreach u in=$uList do={\n\
          :local uName [/ip hotspot user get $u name];\n\
          :local uLimit [/ip hotspot user get $u limit-uptime];\n\
          :local uUp [/ip hotspot user get $u uptime];\n\
          :if ($uLimit != \"\" and $uUp >= $uLimit) do={\n\
            /ip hotspot user remove $u;\n\
            /ip hotspot active remove [find user=$uName];\n\
            :log info (\"[HOTSPOT-SWEEPER] User \" . $uName . \" kadaluwarsa & dibersihkan otomatis\");\n\
          }\n\
        }";

    // 1. Remove existing script if exists
    let existing_scripts = client.run(build_command(
        "/system/script/print",
        [("name", cleaner_script_name)],
    )).await.unwrap_or_default();
    for s in existing_scripts {
        if let Some(id) = s.get(".id") {
            let _ = client.run(build_command("/system/script/remove", [(".id", id)])).await;
        }
    }

    // 2. Add /system/script
    client.run(build_command(
        "/system/script/add",
        [
            ("name", cleaner_script_name),
            ("source", cleaner_source),
            ("comment", "Autonomous Hotspot Voucher Sweeper"),
        ],
    )).await?;

    // 3. Remove existing scheduler if exists
    let sched_name = "hotspot_auto_sweeper_job";
    let existing_sched = client.run(build_command(
        "/system/scheduler/print",
        [("name", sched_name)],
    )).await.unwrap_or_default();
    for s in existing_sched {
        if let Some(id) = s.get(".id") {
            let _ = client.run(build_command("/system/scheduler/remove", [(".id", id)])).await;
        }
    }

    // 4. Add /system/scheduler
    client.run(build_command(
        "/system/scheduler/add",
        [
            ("name", sched_name),
            ("interval", req.interval.as_str()),
            ("on-event", cleaner_script_name),
            ("comment", "Autonomous Hotspot Sweeper Recurring Scheduler"),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "script": cleaner_script_name,
        "scheduler": sched_name,
        "interval": req.interval,
        "message": format!("Autonomous Sweeper berhasil diinstal di RouterOS! Berjalan setiap {}.", req.interval)
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_time_to_unix() {
        let ts = date_time_to_unix(2026, 10, 18, 0, 0, 0);
        assert!(ts.is_some());
        assert!(ts.unwrap() > 1_700_000_000);
    }

    #[test]
    fn test_is_voucher_comment_expired_ros_v6() {
        let comment = "[VOUCHER|default|Rp 5000|VC-1 [EXP:oct/18/2026 10:00:00|VAL:1h|MODE:REMOVE]]";
        let start_ts = date_time_to_unix(2026, 10, 18, 10, 0, 0).unwrap();
        
        assert!(!is_voucher_comment_expired(comment, start_ts + 1800));
        assert!(is_voucher_comment_expired(comment, start_ts + 3600));
        assert!(is_voucher_comment_expired(comment, start_ts + 7200));
    }

    #[test]
    fn test_is_voucher_comment_expired_ros_v7() {
        let comment = "[EXP:2026-10-18 10:00:00|VAL:1d|MODE:REMOVE]";
        let start_ts = date_time_to_unix(2026, 10, 18, 10, 0, 0).unwrap();
        
        assert!(!is_voucher_comment_expired(comment, start_ts + 43200));
        assert!(is_voucher_comment_expired(comment, start_ts + 86400));
    }
}

