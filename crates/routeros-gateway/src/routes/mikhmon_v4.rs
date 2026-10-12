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
pub struct LockMacProfileReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile_name: String,
}

#[derive(Deserialize, Debug)]
pub struct BindUserMacReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub username: String,
    pub mac_address: String,
}

#[derive(Deserialize, Debug)]
pub struct ExpiryModeSetupReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub profile_name: String,
    /// Mode expired: "remove", "notice", "remove_and_record"
    pub expiry_mode: String,
    /// Masa berlaku kuota/waktu voucher (misal: "1d", "7d", "30d", "3h")
    pub validity: String,
    /// Profile isolir tujuan jika menggunakan mode "notice" (default: "EXPIRED_NOTICE")
    pub notice_profile: Option<String>,
    /// Harga voucher untuk pencatatan log jika mode "remove_and_record"
    pub price: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SetupTrialReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    /// Nama hotspot server profile (default: "hsprof1" atau cari yang aktif)
    pub server_profile: Option<String>,
    /// Aktifkan login trial gratisan
    pub enable_trial: bool,
    /// Batas durasi trial (default: "15m", misal "30m", "1h")
    #[serde(default = "default_trial_uptime")]
    pub trial_uptime_limit: String,
    /// Waktu reset uji coba per MAC (default: "1d")
    #[serde(default = "default_trial_reset")]
    pub trial_uptime_reset: String,
    /// User profile yang digunakan trial
    pub trial_user_profile: Option<String>,
}

fn default_trial_uptime() -> String { "15m".into() }
fn default_trial_reset() -> String { "1d".into() }

/// POST /api/v1/hotspot/mikhmon-v4/lock-mac/enable - Mengaktifkan Auto-Lock MAC Address pada User Profile Hotspot
/// Mencegah pembeli voucher membagikan voucher ke teman/perangkat lain setelah login pertama
pub async fn enable_lock_mac(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LockMacProfileReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let profiles = client.run(build_command(
        "/ip/hotspot/user/profile/print",
        [("name", req.profile_name.as_str())],
    )).await?;

    let p_row = profiles.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Profile '{}' tidak ditemukan pada router", req.profile_name))
    })?;

    let id = p_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID Profile tidak ditemukan".into()))?;

    // Skrip On-Login MikroTik: Saat login pertama, otomatis ikat MAC Address klien ke akun voucher
    let lock_mac_script = 
        ":local uMac [/ip hotspot active get [find user=$user] mac-address];\n\
         :local boundMac [/ip hotspot user get [find name=$user] mac-address];\n\
         :if ($boundMac = \"\" and $uMac != \"\") do={\n\
           /ip hotspot user set [find name=$user] mac-address=$uMac comment=([/ip hotspot user get [find name=$user] comment] . \" [LOCKED-MAC:\" . $uMac . \"]\");\n\
           :log info (\"[Mikhmon-V4] User \" . $user . \" dikunci ke MAC \" . $uMac);\n\
         }";

    client.run(build_command(
        "/ip/hotspot/user/profile/set",
        [
            (".id", id),
            ("on-login", lock_mac_script),
            ("shared-users", "1"),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "profile": req.profile_name,
        "shared_users": 1,
        "message": format!("Fitur Lock MAC (Anti-Tukar HP) berhasil disuntikkan ke Profile '{}'!", req.profile_name)
    })))
}

/// POST /api/v1/hotspot/mikhmon-v4/lock-mac/bind-now - Manual bind MAC address ke voucher tertentu
pub async fn bind_user_mac_now(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BindUserMacReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let users = client.run(build_command(
        "/ip/hotspot/user/print",
        [("name", req.username.as_str())],
    )).await?;

    let u_row = users.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("User '{}' tidak ditemukan di database Hotspot", req.username))
    })?;

    let id = u_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID User tidak valid".into()))?;

    client.run(build_command(
        "/ip/hotspot/user/set",
        [
            (".id", id),
            ("mac-address", req.mac_address.as_str()),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "username": req.username,
        "mac_address": req.mac_address,
        "message": format!("User '{}' berhasil dikunci ke MAC address '{}'", req.username, req.mac_address)
    })))
}

/// POST /api/v1/hotspot/mikhmon-v4/expiry-mode/setup - Mengonfigurasi 3-Mode Expiry Voucher (Remove / Notice / Record)
pub async fn setup_expiry_mode(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ExpiryModeSetupReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let profiles = client.run(build_command(
        "/ip/hotspot/user/profile/print",
        [("name", req.profile_name.as_str())],
    )).await?;

    let p_row = profiles.into_iter().next().ok_or_else(|| {
        ApiError::BadRequest(format!("Profile '{}' tidak ditemukan", req.profile_name))
    })?;

    let id = p_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID Profile tidak valid".into()))?;
    let mode_lower = req.expiry_mode.to_lowercase();
    let notice_prof = req.notice_profile.as_deref().unwrap_or("EXPIRED_NOTICE");
    let price_str = req.price.as_deref().unwrap_or("0");

    let on_login_script = match mode_lower.as_str() {
        "notice" => {
            // Mode Notice: Saat masa aktif habis, user dialihkan ke profile isolir
            format!(
                ":local validity \"{}\";\n\
                 :local nProf \"{}\";\n\
                 :local uComment [/ip hotspot user get [find name=$user] comment];\n\
                 :if ([:find $uComment \"EXP:\"] = nil) do={{\n\
                   :local date [/system clock get date];\n\
                   :local time [/system clock get time];\n\
                   /ip hotspot user set [find name=$user] comment=($uComment . \" [EXP:\" . $date . \" \" . $time . \"|VAL:\" . $validity . \"|MODE:NOTICE]\");\n\
                 }}",
                req.validity, notice_prof
            )
        }
        "remove_and_record" => {
            // Mode Remove & Record: Rekam transaksi kasir ke log sebelum user dihapus
            format!(
                ":local validity \"{}\";\n\
                 :local price \"{}\";\n\
                 :local uComment [/ip hotspot user get [find name=$user] comment];\n\
                 :if ([:find $uComment \"EXP:\"] = nil) do={{\n\
                   :local date [/system clock get date];\n\
                   :local time [/system clock get time];\n\
                   :log info (\"[HOTSPOT-SALE] USER:\" . $user . \" | PRICE:\" . $price . \" | DATE:\" . $date);\n\
                   /ip hotspot user set [find name=$user] comment=($uComment . \" [EXP:\" . $date . \" \" . $time . \"|VAL:\" . $validity . \"|REC:\" . $price . \"]\");\n\
                 }}",
                req.validity, price_str
            )
        }
        _ => {
            // Mode Remove (Standar)
            format!(
                ":local validity \"{}\";\n\
                 :local uComment [/ip hotspot user get [find name=$user] comment];\n\
                 :if ([:find $uComment \"EXP:\"] = nil) do={{\n\
                   :local date [/system clock get date];\n\
                   :local time [/system clock get time];\n\
                   /ip hotspot user set [find name=$user] comment=($uComment . \" [EXP:\" . $date . \" \" . $time . \"|VAL:\" . $validity . \"|MODE:REMOVE]\");\n\
                 }}",
                req.validity
            )
        }
    };

    client.run(build_command(
        "/ip/hotspot/user/profile/set",
        [
            (".id", id),
            ("on-login", on_login_script.as_str()),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "profile": req.profile_name,
        "expiry_mode": mode_lower,
        "validity": req.validity,
        "message": format!("Skrip Expiry Mode '{}' berhasil diinjeksikan ke Profile '{}'!", mode_lower, req.profile_name)
    })))
}

/// GET or POST /api/v1/hotspot/mikhmon-v4/trial/status - Cek status login gratisan (Trial) di Hotspot Server Profile
pub async fn trial_status(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let profiles = client.run(build_command(
        "/ip/hotspot/profile/print",
        std::iter::empty::<(&str, &str)>(),
    )).await?;

    let data: Vec<HashMap<String, String>> = profiles.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "server_profiles": data
    })))
}

/// POST /api/v1/hotspot/mikhmon-v4/trial/setup - Mengonfigurasi parameter Trial Pengguna Gratisan
pub async fn setup_trial(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupTrialReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let profiles = client.run(build_command(
        "/ip/hotspot/profile/print",
        std::iter::empty::<(&str, &str)>(),
    )).await?;

    let target_row = if let Some(sp) = req.server_profile.as_deref() {
        profiles.into_iter().find(|p| p.get("name").map(|n| n == sp).unwrap_or(false))
    } else {
        profiles.into_iter().next()
    }.ok_or_else(|| ApiError::BadRequest("Hotspot Server Profile tidak ditemukan".into()))?;

    let id = target_row.get(".id").ok_or_else(|| ApiError::BadRequest("ID Server Profile tidak valid".into()))?;
    let trial_enabled_str = if req.enable_trial { "yes" } else { "no" };

    let mut args = vec![
        (".id", id),
        ("trial-uptime-limit", req.trial_uptime_limit.as_str()),
        ("trial-uptime-reset", req.trial_uptime_reset.as_str()),
    ];

    // Di RouterOS, trial diaktifkan melalui login-by atau use-trial
    let current_login_by = target_row.get("login-by").unwrap_or("http-chap,cookie");
    let updated_login_by = if req.enable_trial {
        if !current_login_by.contains("trial") {
            format!("{},trial", current_login_by)
        } else {
            current_login_by.to_string()
        }
    } else {
        current_login_by.replace(",trial", "").replace("trial,", "").replace("trial", "")
    };
    args.push(("login-by", updated_login_by.as_str()));

    if let Some(up) = req.trial_user_profile.as_deref() {
        args.push(("trial-user-profile", up));
    }

    client.run(build_command("/ip/hotspot/profile/set", args)).await?;

    Ok(Json(json!({
        "success": true,
        "trial_enabled": req.enable_trial,
        "login_by": updated_login_by,
        "trial_uptime_limit": req.trial_uptime_limit,
        "trial_uptime_reset": req.trial_uptime_reset,
        "message": format!("Hotspot Trial pengguna gratisan berhasil di-setting (Aktif: {})!", trial_enabled_str)
    })))
}

/// POST /api/v1/hotspot/mikhmon-v4/trial/reset-cookies - Reset daftar MAC trial agar pengguna bisa coba lagi
pub async fn reset_trial_cookies(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let cookies = client.run(build_command(
        "/ip/hotspot/cookie/print",
        std::iter::empty::<(&str, &str)>(),
    )).await.unwrap_or_default();

    let mut removed = 0;
    for c in cookies {
        if let Some(user) = c.get("user") {
            if user.starts_with("T-") || user.to_lowercase().contains("trial") {
                if let Some(id) = c.get(".id") {
                    let _ = client.run(build_command("/ip/hotspot/cookie/remove", [(".id", id)])).await;
                    removed += 1;
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "cleared_trial_cookies": removed,
        "message": format!("{} session cookie trial berhasil direset. Pengguna dapat mencoba paket trial kembali.", removed)
    })))
}
