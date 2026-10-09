use axum::response::Html;
use axum::Json;
use serde::Deserialize;

use crate::error::ApiError;

#[derive(Deserialize, Debug, Clone)]
pub struct VoucherItem {
    pub username: String,
    pub password: String,
    pub profile: Option<String>,
    pub timelimit: Option<String>,
    pub datalimit: Option<String>,
    pub price: Option<String>,
    pub comment: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RenderVoucherReq {
    pub vouchers: Vec<VoucherItem>,
    #[serde(default = "default_template_type")]
    pub template_type: String, // "thermal", "card_grid", "custom"
    pub hotspot_name: Option<String>,
    pub dns_name: Option<String>,
    pub custom_html_template: Option<String>,
    pub currency: Option<String>,
}

fn default_template_type() -> String {
    "card_grid".into()
}

/// POST /api/v1/hotspot/voucher-template/render - Render ready-to-print HTML vouchers (Thermal 58/80mm or Grid Cards)
pub async fn render_template(
    Json(req): Json<RenderVoucherReq>,
) -> Result<Html<String>, ApiError> {
    let hotspot_title = req.hotspot_name.as_deref().unwrap_or("WIFI HOTSPOT");
    let dns_url = req.dns_name.as_deref().unwrap_or("login.wifi");
    let currency = req.currency.as_deref().unwrap_or("Rp");

    let html = match req.template_type.as_str() {
        "thermal" => render_thermal(&req.vouchers, hotspot_title, dns_url, currency),
        "custom" if req.custom_html_template.is_some() => {
            render_custom(&req.vouchers, req.custom_html_template.as_deref().unwrap(), hotspot_title, dns_url, currency)
        }
        _ => render_grid(&req.vouchers, hotspot_title, dns_url, currency),
    };

    Ok(Html(html))
}

fn render_grid(vouchers: &[VoucherItem], title: &str, dns: &str, curr: &str) -> String {
    let mut cards = String::new();
    for (i, v) in vouchers.iter().enumerate() {
        let profile = v.profile.as_deref().unwrap_or("Regular");
        let validity = v.timelimit.as_deref().unwrap_or(v.datalimit.as_deref().unwrap_or("1 Hari"));
        let price = v.price.as_deref().unwrap_or("3.000");

        cards.push_str(&format!(
            r#"<div class="voucher-card">
              <div class="card-header">
                <h3>{title}</h3>
                <span class="badge">#{idx}</span>
              </div>
              <div class="card-body">
                <div class="code-box">
                  <span class="label">KODE VOUCHER</span>
                  <div class="code-val">{user}</div>
                </div>
                <div class="info-row">
                  <span>Paket:</span><strong>{profile}</strong>
                </div>
                <div class="info-row">
                  <span>Durasi:</span><strong>{validity}</strong>
                </div>
                <div class="info-row price-row">
                  <span>Tarif:</span><span class="price-val">{curr} {price}</span>
                </div>
              </div>
              <div class="card-footer">
                <span>Login: http://{dns}</span>
              </div>
            </div>"#,
            title = title,
            idx = i + 1,
            user = v.username,
            profile = profile,
            validity = validity,
            curr = curr,
            price = price,
            dns = dns
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Cetak Voucher Hotspot</title>
  <style>
    @page {{ size: A4 portrait; margin: 10mm; }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
    body {{ background: #f8fafc; color: #1e293b; padding: 15px; }}
    .grid-container {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }}
    @media print {{
      body {{ background: white; padding: 0; }}
      .no-print {{ display: none !important; }}
    }}
    .voucher-card {{
      border: 1.5px dashed #0284c7; border-radius: 8px; background: white;
      padding: 10px; page-break-inside: avoid;
    }}
    .card-header {{ display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #e2e8f0; padding-bottom: 5px; }}
    .card-header h3 {{ font-size: 11px; font-weight: 800; color: #0369a1; text-transform: uppercase; }}
    .badge {{ font-size: 9px; background: #e0f2fe; color: #0369a1; padding: 2px 5px; border-radius: 4px; }}
    .code-box {{ background: #f0fdf4; border: 1px solid #86efac; border-radius: 4px; text-align: center; padding: 6px; margin: 6px 0; }}
    .code-box .label {{ font-size: 8px; color: #15803d; font-weight: bold; letter-spacing: 0.5px; }}
    .code-val {{ font-size: 15px; font-weight: 900; letter-spacing: 2px; color: #14532d; }}
    .info-row {{ display: flex; justify-content: space-between; font-size: 9px; margin-bottom: 3px; color: #64748b; }}
    .info-row strong {{ color: #0f172a; }}
    .price-row {{ border-top: 1px dashed #e2e8f0; padding-top: 4px; margin-top: 4px; }}
    .price-val {{ font-weight: 900; color: #ea580c; font-size: 11px; }}
    .card-footer {{ text-align: center; font-size: 8px; color: #94a3b8; border-top: 1px solid #f1f5f9; padding-top: 4px; margin-top: 4px; }}
    .print-btn {{
      position: fixed; bottom: 20px; right: 20px; background: #0284c7; color: white;
      border: none; padding: 12px 24px; border-radius: 999px; font-weight: bold; cursor: pointer;
      box-shadow: 0 4px 12px rgba(0,0,0,0.15); font-size: 14px;
    }}
  </style>
</head>
<body>
  <button class="print-btn no-print" onclick="window.print()">🖨️ Cetak Seluruh Voucher ({total})</button>
  <div class="grid-container">
    {cards}
  </div>
</body>
</html>"#,
        total = vouchers.len(),
        cards = cards
    )
}

fn render_thermal(vouchers: &[VoucherItem], title: &str, dns: &str, curr: &str) -> String {
    let mut receipts = String::new();
    for (i, v) in vouchers.iter().enumerate() {
        let profile = v.profile.as_deref().unwrap_or("Regular");
        let validity = v.timelimit.as_deref().unwrap_or(v.datalimit.as_deref().unwrap_or("1 Hari"));
        let price = v.price.as_deref().unwrap_or("3.000");

        receipts.push_str(&format!(
            r#"<div class="thermal-receipt">
              <div class="t-center bold">{title}</div>
              <div class="t-center text-xs">Login: http://{dns}</div>
              <div class="divider">================================</div>
              <div class="t-center bold big" style="margin: 8px 0;">{user}</div>
              <div class="divider">--------------------------------</div>
              <div class="row"><span>Paket:</span><span>{profile}</span></div>
              <div class="row"><span>Masa Aktif:</span><span>{validity}</span></div>
              <div class="row bold"><span>Tarif:</span><span>{curr} {price}</span></div>
              <div class="divider">================================</div>
              <div class="t-center text-xs">Simpan struk ini sebagai bukti voucher. #{idx}</div>
            </div>"#,
            title = title,
            dns = dns,
            user = v.username,
            profile = profile,
            validity = validity,
            curr = curr,
            price = price,
            idx = i + 1
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Cetak Struk Thermal</title>
  <style>
    @page {{ size: 58mm auto; margin: 0; }}
    * {{ font-family: "Courier New", Courier, monospace; box-sizing: border-box; }}
    body {{ width: 58mm; margin: 0; padding: 4px; font-size: 11px; }}
    .thermal-receipt {{ page-break-after: always; padding: 6px 0; }}
    .t-center {{ text-align: center; }}
    .bold {{ font-weight: bold; }}
    .big {{ font-size: 16px; letter-spacing: 2px; }}
    .text-xs {{ font-size: 9px; }}
    .divider {{ overflow: hidden; text-align: center; font-size: 9px; margin: 4px 0; }}
    .row {{ display: flex; justify-content: space-between; }}
    @media screen {{
      body {{ background: #334155; padding: 20px; }}
      .thermal-receipt {{ background: white; margin: 0 auto 15px auto; padding: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.3); }}
    }}
  </style>
</head>
<body>
  {receipts}
</body>
</html>"#,
        receipts = receipts
    )
}

fn render_custom(vouchers: &[VoucherItem], tpl: &str, title: &str, dns: &str, curr: &str) -> String {
    let mut out = String::new();
    for (i, v) in vouchers.iter().enumerate() {
        let rendered = tpl
            .replace("{{username}}", &v.username)
            .replace("{{password}}", &v.password)
            .replace("{{profile}}", v.profile.as_deref().unwrap_or("Regular"))
            .replace("{{timelimit}}", v.timelimit.as_deref().unwrap_or(""))
            .replace("{{datalimit}}", v.datalimit.as_deref().unwrap_or(""))
            .replace("{{price}}", v.price.as_deref().unwrap_or(""))
            .replace("{{currency}}", curr)
            .replace("{{hotspot_name}}", title)
            .replace("{{login_url}}", &format!("http://{dns}"))
            .replace("{{serial}}", &(i + 1).to_string());
        out.push_str(&rendered);
    }
    out
}
