use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
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

#[derive(Deserialize, Debug, Default)]
pub struct ArchPackageUrlReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub target_version: Option<String>, // e.g. "7.15.3" or "6.49.13", defaults to router's version or latest
}

#[derive(Deserialize, Debug)]
pub struct ResetConfigReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub keep_users: Option<bool>,
    pub no_defaults: Option<bool>,
    pub skip_backup: Option<bool>,
    pub run_after_reset: Option<String>, // .rsc script filename
}

/// POST /api/v1/maintenance/netinstall-prep - Prepares the router to boot directly into Netinstall
/// mode on the next reboot without needing physical access to hold the hardware reset button.
pub async fn netinstall_prep(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    // Check routerboard availability
    let rb_rows = client.run(build_command("/system/routerboard/print", std::iter::empty::<(&str, &str)>())).await?;
    let model = rb_rows.first().and_then(|r| r.get("model")).unwrap_or("MikroTik");

    // Configure RouterBOARD boot device
    client.run(build_command(
        "/system/routerboard/settings/set",
        [("boot-device", "try-ethernet-once-then-nand")]
    )).await?;

    Ok(Json(json!({
        "success": true,
        "device_model": model,
        "boot_device_setting": "try-ethernet-once-then-nand",
        "message": "RouterBOARD berhasil disetel untuk boot Etherboot (Netinstall) pada reboot berikutnya.",
        "instructions": [
            "1. Hubungkan port ether1 / boot port router langsung ke kartu jaringan PC menggunakan kabel LAN.",
            "2. Jalankan aplikasi Netinstall di PC dan atur Net booting (Client IP e.g. 192.168.88.2).",
            "3. Lakukan reboot router (/api/v1/system/reboot). Router akan otomatis mencari server Netinstall selama 1 kali siklus boot.",
            "4. Jika tidak ada Netinstall, router akan kembali boot normal dari penyimpanan NAND."
        ]
    })))
}

/// POST /api/v1/maintenance/pre-upgrade-snapshot - Automatically saves both binary backup (.backup)
/// and human-readable text config (.rsc) with timestamp before any dangerous OS upgrade.
pub async fn pre_upgrade_snapshot(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BaseReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let prefix = format!("pre_upgrade_snapshot_{}", now);

    let backup_name = format!("{}.backup", prefix);
    let rsc_name = format!("{}.rsc", prefix);

    // 1. Create binary backup
    client.run(build_command("/system/backup/save", [("name", prefix.as_str())])).await?;

    // 2. Export config text script
    client.run(build_command("/export", [("file", prefix.as_str())])).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Snapshot ganda (Binary .backup & Text .rsc) berhasil dibuat sebelum operasi kritis",
        "files_created": {
            "binary_backup": backup_name,
            "export_script": rsc_name
        },
        "timestamp_unix": now
    })))
}

/// POST /api/v1/maintenance/architecture-package-url - Resolves hardware CPU architecture and builds
/// official direct CDN download URLs for RouterOS packages and Netinstall tools.
pub async fn architecture_package_url(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ArchPackageUrlReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let res_rows = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>())).await?;
    let first = res_rows.first();
    let current_version = first.and_then(|r| r.get("version")).unwrap_or("7.15.2");
    let arch_name = first.and_then(|r| r.get("architecture-name")).unwrap_or("mipsbe");
    let board_name = first.and_then(|r| r.get("board-name")).unwrap_or("RouterBOARD");

    // Clean version string e.g. "7.15.2 (stable)" -> "7.15.2"
    let clean_current_ver = current_version.split(' ').next().unwrap_or(current_version);
    let target_ver = req.target_version.as_deref().unwrap_or(clean_current_ver);

    // MikroTik package CDN naming conventions
    let npk_url = format!(
        "https://download.mikrotik.com/routeros/{ver}/routeros-{ver}-{arch}.npk",
        ver = target_ver,
        arch = arch_name
    );
    let extra_pkgs_url = format!(
        "https://download.mikrotik.com/routeros/{ver}/all_packages-{arch}-{ver}.zip",
        ver = target_ver,
        arch = arch_name
    );
    let netinstall_url = format!(
        "https://download.mikrotik.com/routeros/{ver}/netinstall-{ver}.zip",
        ver = target_ver
    );

    Ok(Json(json!({
        "success": true,
        "hardware": {
            "board_name": board_name,
            "architecture": arch_name,
            "installed_version": clean_current_ver
        },
        "resolved_target_version": target_ver,
        "official_downloads": {
            "main_package_npk": npk_url,
            "extra_packages_zip": extra_pkgs_url,
            "netinstall_tool_zip": netinstall_url
        },
        "architecture_notes": match arch_name {
            "smips" => "Peringatan Flash 16MB: Hanya gunakan package routeros utama tanpa paket ekstra yang tidak diperlukan.",
            "arm64" => "Arsitektur 64-bit bertenaga tinggi, mendukung WireGuard hardware acceleration dan MikroTik Container.",
            "tile" => "Multi-core Tilera CPU (CCR10xx series).",
            _ => "Arsitektur standar didukung penuh."
        }
    })))
}

/// POST /api/v1/maintenance/reset-configuration - Hard reset / factory reset router configuration
pub async fn reset_configuration(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ResetConfigReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut args = Vec::new();
    let keep_usr_str = req.keep_users.map(|v| if v { "yes" } else { "no" });
    if let Some(ku) = keep_usr_str.as_deref() {
        args.push(("keep-users", ku));
    }
    let no_def_str = req.no_defaults.map(|v| if v { "yes" } else { "no" });
    if let Some(nd) = no_def_str.as_deref() {
        args.push(("no-defaults", nd));
    }
    let skip_bk_str = req.skip_backup.map(|v| if v { "yes" } else { "no" });
    if let Some(sb) = skip_bk_str.as_deref() {
        args.push(("skip-backup", sb));
    }
    if let Some(script) = req.run_after_reset.as_deref() {
        args.push(("run-after-reset", script));
    }

    client.run(build_command("/system/reset-configuration", args)).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Reset configuration command executed. Router will reboot to factory/blank state."
    })))
}
