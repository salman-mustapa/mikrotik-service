use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::Html;
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
pub struct TelegramSendReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub bot_token: String,
    pub chat_id: String,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct WhatsAppSendReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub gowa_url: String, // e.g. "http://127.0.0.1:3000/send/message" or "https://wa.myisp.com/api/send"
    pub api_key: Option<String>,
    pub phone: String,    // Target phone number, e.g. "628123456789"
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct MultiNotifyReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub title: String,
    pub message: String,
    pub severity: Option<String>, // "INFO", "WARNING", "CRITICAL"
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_id: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WhatsAppConfig {
    pub gowa_url: String,
    pub api_key: Option<String>,
    pub phone: String,
}

#[derive(Deserialize, Debug)]
pub struct SetupNetwatchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub host: String,                  // IP to monitor, e.g. "172.16.10.2"
    pub device_name: Option<String>,   // e.g. "FAUJIA HOTSPOT 1"
    pub interval: Option<String>,      // default "5s"
    pub timeout: Option<String>,       // default "1000ms"
    pub comment: Option<String>,
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
}

#[derive(Deserialize, Debug)]
pub struct BatchNetwatchTarget {
    pub host: String,
    pub device_name: String,
    pub interval: Option<String>,
    pub timeout: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct BatchSetupNetwatchReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub targets: Vec<BatchNetwatchTarget>,
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
}

#[derive(Deserialize, Debug)]
pub struct IdReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct ToggleMonitorReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub id: String,
    pub disabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct SetupReportSchedulerReq {
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub interval: Option<String>, // default "1d"
    pub start_time: Option<String>, // default "21:00:00"
    pub bot_token: String,
    pub chat_id: String,
    pub report_title: Option<String>, // default "LAPORAN NOC HARIAN"
}

#[derive(Deserialize, Debug, Default)]
pub struct HotspotChatSnippetQuery {
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub hotspot_name: Option<String>,
    pub wa_phone: Option<String>,
    pub gowa_url: Option<String>,
}

/// Helper function to URL-encode text for RouterOS /tool fetch query parameters
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

/// POST /api/v1/integrations/telegram/send - Send instant message to Telegram via MikroTik /tool/fetch
pub async fn telegram_send(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<TelegramSendReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let clean_msg = url_encode(&req.message);
    let fetch_url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
        req.bot_token, req.chat_id, clean_msg
    );

    client.run(build_command(
        "/tool/fetch",
        [
            ("url", fetch_url.as_str()),
            ("keep-result", "no"),
            ("http-method", "get"),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "channel": "telegram",
        "chat_id": req.chat_id,
        "message": "Pesan Telegram berhasil dipicu via RouterOS /tool/fetch"
    })))
}

/// POST /api/v1/integrations/whatsapp/send - Send WhatsApp message via Gowa WhatsApp API or MikroTik /tool/fetch
pub async fn whatsapp_send(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<WhatsAppSendReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let clean_phone = req.phone.replace(['+', '-', ' '], "");
    let clean_msg = url_encode(&req.message);

    // Support both Gowa GET query endpoint and webhook dispatcher
    let gowa_target_url = if req.gowa_url.contains('?') {
        format!("{}&phone={}&message={}", req.gowa_url, clean_phone, clean_msg)
    } else {
        format!("{}/send?phone={}&message={}", req.gowa_url.trim_end_matches('/'), clean_phone, clean_msg)
    };

    client.run(build_command(
        "/tool/fetch",
        [
            ("url", gowa_target_url.as_str()),
            ("keep-result", "no"),
            ("http-method", "get"),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "channel": "whatsapp_gowa",
        "recipient": clean_phone,
        "message": "Pesan WhatsApp (Gowa) berhasil dipicu via RouterOS /tool/fetch"
    })))
}

/// POST /api/v1/integrations/notify - Multi-Channel Notification Dispatcher (Telegram + WhatsApp)
pub async fn multi_notify(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MultiNotifyReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let severity = req.severity.as_deref().unwrap_or("INFO");
    let icon = match severity {
        "CRITICAL" => "🚨 [CRITICAL ALERT]",
        "WARNING" => "⚠️ [WARNING]",
        _ => "ℹ️ [NOTIFICATION]",
    };

    let formatted_message = format!("{}\n*{}*\n{}", icon, req.title, req.message);
    let mut sent_channels = Vec::new();

    // 1. Dispatch to Telegram
    if let Some(tg) = req.telegram {
        let clean_msg = url_encode(&formatted_message);
        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
            tg.bot_token, tg.chat_id, clean_msg
        );
        let _ = client.run(build_command("/tool/fetch", [("url", url.as_str()), ("keep-result", "no")])).await;
        sent_channels.push("telegram");
    }

    // 2. Dispatch to WhatsApp (Gowa)
    if let Some(wa) = req.whatsapp {
        let clean_phone = wa.phone.replace(['+', '-', ' '], "");
        let clean_msg = url_encode(&formatted_message);
        let url = format!("{}/send?phone={}&message={}", wa.gowa_url.trim_end_matches('/'), clean_phone, clean_msg);
        let _ = client.run(build_command("/tool/fetch", [("url", url.as_str()), ("keep-result", "no")])).await;
        sent_channels.push("whatsapp_gowa");
    }

    Ok(Json(json!({
        "success": true,
        "sent_channels": sent_channels,
        "title": req.title,
        "severity": severity
    })))
}

/// POST /api/v1/integrations/netwatch/setup - Sets up Netwatch host monitor with automated Telegram/WhatsApp alerts
pub async fn netwatch_setup(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupNetwatchReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let dev_label = req.device_name.as_deref().unwrap_or(req.host.as_str());
    let interval = req.interval.as_deref().unwrap_or("5s");
    let timeout = req.timeout.as_deref().unwrap_or("1000ms");
    let comment = req.comment.unwrap_or_else(|| format!("Netwatch: {}", dev_label));

    let mut up_scripts = Vec::new();
    let mut down_scripts = Vec::new();

    // Add Telegram alert action
    if let Some(ref tg) = req.telegram {
        let down_msg = url_encode(&format!("🚨 ALERT: Perangkat '{}' ({}) DOWN / Terputus!", dev_label, req.host));
        let up_msg = url_encode(&format!("✅ RECOVERED: Perangkat '{}' ({}) UP / Normal Kembali!", dev_label, req.host));
        down_scripts.push(format!(
            "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
            tg.bot_token, tg.chat_id, down_msg
        ));
        up_scripts.push(format!(
            "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
            tg.bot_token, tg.chat_id, up_msg
        ));
    }

    // Add WhatsApp (Gowa) alert action
    if let Some(ref wa) = req.whatsapp {
        let clean_phone = wa.phone.replace(['+', '-', ' '], "");
        let down_msg = url_encode(&format!("🚨 *ALERT*: Perangkat *{}* ({}) *DOWN*!", dev_label, req.host));
        let up_msg = url_encode(&format!("✅ *RECOVERED*: Perangkat *{}* ({}) *UP*!", dev_label, req.host));
        let base_url = wa.gowa_url.trim_end_matches('/');
        down_scripts.push(format!(
            "/tool fetch url=\"{}/send?phone={}&message={}\" keep-result=no",
            base_url, clean_phone, down_msg
        ));
        up_scripts.push(format!(
            "/tool fetch url=\"{}/send?phone={}&message={}\" keep-result=no",
            base_url, clean_phone, up_msg
        ));
    }

    let up_script_str = up_scripts.join("\n");
    let down_script_str = down_scripts.join("\n");

    client.run(build_command(
        "/tool/netwatch/add",
        [
            ("host", req.host.as_str()),
            ("interval", interval),
            ("timeout", timeout),
            ("up-script", up_script_str.as_str()),
            ("down-script", down_script_str.as_str()),
            ("comment", comment.as_str()),
        ],
    )).await?;

    Ok(Json(json!({
        "success": true,
        "monitored_host": req.host,
        "device_name": dev_label,
        "interval": interval,
        "timeout": timeout,
        "has_telegram": req.telegram.is_some(),
        "has_whatsapp": req.whatsapp.is_some(),
        "message": format!("Netwatch untuk host {} ({}) berhasil di-setup!", req.host, dev_label)
    })))
}

/// POST /api/v1/integrations/netwatch/batch-setup - Provisions multiple Netwatch monitors in one API call
pub async fn netwatch_batch_setup(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BatchSetupNetwatchReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let mut configured = Vec::new();

    for t in req.targets {
        let interval = t.interval.as_deref().unwrap_or("5s");
        let timeout = t.timeout.as_deref().unwrap_or("1000ms");
        let comment = format!("Netwatch: {}", t.device_name);

        let mut up_scripts = Vec::new();
        let mut down_scripts = Vec::new();

        if let Some(ref tg) = req.telegram {
            let down_msg = url_encode(&format!("🚨 ALERT: '{}' ({}) DOWN!", t.device_name, t.host));
            let up_msg = url_encode(&format!("✅ RECOVERED: '{}' ({}) UP!", t.device_name, t.host));
            down_scripts.push(format!(
                "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
                tg.bot_token, tg.chat_id, down_msg
            ));
            up_scripts.push(format!(
                "/tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}\" keep-result=no",
                tg.bot_token, tg.chat_id, up_msg
            ));
        }

        if let Some(ref wa) = req.whatsapp {
            let clean_phone = wa.phone.replace(['+', '-', ' '], "");
            let down_msg = url_encode(&format!("🚨 *ALERT*: *{}* ({}) *DOWN*!", t.device_name, t.host));
            let up_msg = url_encode(&format!("✅ *RECOVERED*: *{}* ({}) *UP*!", t.device_name, t.host));
            let base_url = wa.gowa_url.trim_end_matches('/');
            down_scripts.push(format!(
                "/tool fetch url=\"{}/send?phone={}&message={}\" keep-result=no",
                base_url, clean_phone, down_msg
            ));
            up_scripts.push(format!(
                "/tool fetch url=\"{}/send?phone={}&message={}\" keep-result=no",
                base_url, clean_phone, up_msg
            ));
        }

        let up_script_str = up_scripts.join("\n");
        let down_script_str = down_scripts.join("\n");

        let res = client.run(build_command(
            "/tool/netwatch/add",
            [
                ("host", t.host.as_str()),
                ("interval", interval),
                ("timeout", timeout),
                ("up-script", up_script_str.as_str()),
                ("down-script", down_script_str.as_str()),
                ("comment", comment.as_str()),
            ],
        )).await;

        if res.is_ok() {
            configured.push(json!({
                "host": t.host,
                "device_name": t.device_name,
                "status": "configured"
            }));
        }
    }

    Ok(Json(json!({
        "success": true,
        "total_configured": configured.len(),
        "devices": configured
    })))
}

/// GET & POST /api/v1/integrations/netwatch/monitors - Lists all configured Netwatch monitors with live statuses
pub async fn netwatch_list(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    req: Option<Json<BaseReq>>,
) -> Result<Json<Value>, ApiError> {
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let rows = client.run(build_command("/tool/netwatch/print", std::iter::empty::<(&str, &str)>())).await?;
    let data: Vec<HashMap<String, String>> = rows.into_iter().map(|r| r.attrs).collect();

    Ok(Json(json!({
        "success": true,
        "count": data.len(),
        "monitors": data
    })))
}

/// POST /api/v1/integrations/netwatch/toggle - Enables or disables a Netwatch monitor
pub async fn netwatch_toggle(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ToggleMonitorReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let cmd = if req.disabled { "/tool/netwatch/disable" } else { "/tool/netwatch/enable" };
    client.run(build_command(cmd, [(".id", req.id.as_str())])).await?;

    Ok(Json(json!({
        "success": true,
        "id": req.id,
        "disabled": req.disabled,
        "message": format!("Netwatch monitor status diubah menjadi: {}", if req.disabled { "Disabled" } else { "Enabled" })
    })))
}

/// POST /api/v1/integrations/netwatch/remove - Removes a Netwatch monitor by ID
pub async fn netwatch_remove(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<IdReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    client.run(build_command("/tool/netwatch/remove", [(".id", req.id.as_str())])).await?;

    Ok(Json(json!({
        "success": true,
        "message": "Netwatch monitor berhasil dihapus dari router"
    })))
}

/// POST /api/v1/integrations/reports/setup-scheduler - Installs automated NOC Daily Report Script & Scheduler
pub async fn setup_report_scheduler(
    State(st): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SetupReportSchedulerReq>,
) -> Result<Json<Value>, ApiError> {
    let (target, router_id) = AppState::parse_target(&headers, req.router, req.router_id);
    let client = st.resolve_client(target.as_ref(), router_id.as_deref()).await?;

    let interval = req.interval.as_deref().unwrap_or("1d");
    let start_time = req.start_time.as_deref().unwrap_or("21:00:00");
    let title = req.report_title.as_deref().unwrap_or("LAPORAN NOC HARIAN");

    // Bulletproof RouterOS script that queries identity, active hotspot, and resource metrics
    let script_source = format!(
        ":local id [/system identity get name];\n\
         :local hsCount [:len [/ip hotspot active find]];\n\
         :local pppCount [:len [/ppp active find]];\n\
         :local cpu [/system resource get cpu-load];\n\
         :local mem ([/system resource get free-memory] / 1048576);\n\
         :local upt [/system resource get uptime];\n\
         :local text \"*{}*%0A%0A*Router:* $id%0A*Hotspot Aktif:* $hsCount user%0A*PPPoE Aktif:* $pppCount user%0A*CPU Load:* $cpu%25%0A*RAM Bebas:* $mem MB%0A*Uptime:* $upt\";\n\
         /tool fetch url=\"https://api.telegram.org/bot{}/sendMessage?chat_id={}&text=$text\" keep-result=no;",
        url_encode(title),
        req.bot_token,
        req.chat_id
    );

    let script_name = "noc_daily_report";
    let sched_name = "sched_noc_daily_report";

    // 1. Add or update script
    let _ = client.run(build_command(
        "/system/script/add",
        [
            ("name", script_name),
            ("source", script_source.as_str()),
            ("policy", "read,write,test,policy"),
            ("comment", "Automated NOC Daily Report Dispatcher"),
        ],
    )).await;

    // 2. Add or update scheduler
    let _ = client.run(build_command(
        "/system/scheduler/add",
        [
            ("name", sched_name),
            ("on-event", script_name),
            ("interval", interval),
            ("start-time", start_time),
            ("comment", "Automated NOC Scheduler"),
        ],
    )).await;

    Ok(Json(json!({
        "success": true,
        "script_name": script_name,
        "scheduler_name": sched_name,
        "interval": interval,
        "start_time": start_time,
        "message": "NOC Automated Daily Report Scheduler berhasil diinstal pada MikroTik!"
    })))
}

/// GET /api/v1/integrations/hotspot-chat/snippet - Generates drop-in HTML/JS Live Chat widget snippet for Hotspot login & status page
pub async fn hotspot_chat_snippet(
    Query(q): Query<HotspotChatSnippetQuery>,
) -> Html<String> {
    let hotspot = q.hotspot_name.as_deref().unwrap_or("HOTSPOT CUSTOMER CARE");
    let bot_id = q.chat_id.as_deref().unwrap_or("YOUR_TELEGRAM_CHAT_ID");
    let wa_number = q.wa_phone.as_deref().unwrap_or("628123456789");

    let snippet_code = format!(
        r#"<!-- ================================================================= -->
<!-- HOTSPOT LIVE CHAT & WHATSAPP FLOATING WIDGET (Intergram & Gowa)    -->
<!-- Masukkan kode ini tepat sebelum tag </body> pada login.html/status.html -->
<!-- ================================================================= -->

<!-- 1. Intergram Live Chat Widget (Bridged to Telegram Bot) -->
<script>
  window.intergramId = "{bot_id}";
  window.intergramCustomizations = {{
    titleClosed: 'Live Chat Support',
    titleOpen: 'Live Chat {hotspot}',
    introMessage: 'Halo! Selamat datang di {hotspot}. Ada yang bisa kami bantu seputar voucher / koneksi?',
    autoResponse: 'Pesan Anda telah diteruskan ke Tim NOC. Mohon tunggu respon kami.',
    autoNoResponse: 'Customer service kami sedang memproses permintaan Anda, terima kasih atas kesabaran Anda.',
    mainColor: '#1d4ed8',
    alwaysUseFloatingButton: true
  }};
</script>
<script id="intergram" src="https://www.intergram.xyz/js/widget.js"></script>

<!-- 2. WhatsApp Direct Helpdesk Floating Button (Gowa Compatible) -->
<a href="https://wa.me/{wa_number}?text=Halo%20Admin%20{hotspot}%2C%20saya%20butuh%20bantuan%20koneksi%20wifi" 
   target="_blank" 
   style="position: fixed; bottom: 85px; right: 20px; z-index: 9999; background: #25d366; color: white; border-radius: 50px; padding: 10px 16px; display: flex; align-items: center; gap: 8px; font-family: sans-serif; font-size: 13px; font-weight: bold; text-decoration: none; box-shadow: 0 4px 14px rgba(0,0,0,0.25);">
  <svg width="20" height="20" fill="currentColor" viewBox="0 0 24 24"><path d="M.057 24l1.687-6.163c-1.041-1.804-1.588-3.849-1.587-5.946.003-6.556 5.338-11.891 11.893-11.891 3.181.001 6.167 1.24 8.413 3.488 2.245 2.248 3.481 5.236 3.48 8.414-.003 6.557-5.338 11.892-11.893 11.892-1.99-.001-3.951-.5-5.688-1.448l-6.305 1.654zm6.597-3.807c1.676.995 3.276 1.591 5.392 1.592 5.448 0 9.886-4.434 9.889-9.885.002-5.462-4.415-9.89-9.881-9.892-5.452 0-9.887 4.434-9.889 9.884-.001 2.225.651 3.891 1.746 5.634l-.999 3.648 3.742-.981z"/></svg>
  <span>Bantuan WhatsApp</span>
</a>
"#,
        hotspot = hotspot,
        bot_id = bot_id,
        wa_number = wa_number
    );

    let html_page = format!(
        r#"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <title>Hotspot Chat Snippet Generator</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; padding: 24px; background: #f8fafc; color: #0f172a; line-height: 1.5; }}
    pre {{ background: #1e293b; color: #f8fafc; padding: 18px; border-radius: 8px; overflow-x: auto; font-family: monospace; font-size: 13px; }}
    .badge {{ background: #eff6ff; color: #1d4ed8; padding: 3px 8px; border-radius: 6px; font-weight: 700; font-size: 12px; }}
    button {{ background: #2563eb; color: white; border: none; padding: 8px 16px; border-radius: 6px; font-weight: 700; cursor: pointer; }}
    button:hover {{ background: #1d4ed8; }}
  </style>
</head>
<body>
  <h2>🌐 Hotspot Live Chat & WhatsApp Helpdesk Embed Code</h2>
  <p>Salin snippet di bawah ini dan tempelkan ke template hotspot MikroTik Anda (<code>login.html</code> atau <code>status.html</code>) tepat sebelum tag penutup <code>&lt;/body&gt;</code>:</p>
  <button onclick="navigator.clipboard.writeText(document.getElementById('code').innerText); alert('Kode berhasil disalin!');">📋 Salin Kode Embed</button>
  <pre id="code">{}</pre>
</body>
</html>"#,
        snippet_code
    );

    Html(html_page)
}
