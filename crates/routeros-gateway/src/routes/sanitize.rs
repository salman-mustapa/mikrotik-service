use crate::error::ApiError;

/// Validate RouterOS time duration / validity strings (e.g., "1d", "12h", "30m", "7d", "1d12h").
/// Only permits ASCII digits and unit characters [smhdw].
pub fn validate_validity(val: &str) -> Result<(), ApiError> {
    let s = val.trim();
    if s.is_empty() || s.len() > 32 {
        return Err(ApiError::BadRequest("Validity duration must be 1-32 characters".into()));
    }
    // Must contain only digits and valid duration unit identifiers
    let is_valid = s.chars().all(|c| c.is_ascii_digit() || matches!(c, 's' | 'm' | 'h' | 'd' | 'w'));
    if !is_valid {
        return Err(ApiError::BadRequest(format!(
            "Invalid validity format '{}'. Expected duration format like '1d', '12h', '30m'",
            s
        )));
    }
    Ok(())
}

/// Validate numeric price string (e.g., "5000", "10000.50").
pub fn validate_price(val: &str) -> Result<(), ApiError> {
    let s = val.trim();
    if s.is_empty() || s.len() > 20 {
        return Err(ApiError::BadRequest("Price must be 1-20 characters".into()));
    }
    let is_num = s.chars().all(|c| c.is_ascii_digit() || c == '.');
    if !is_num || s.parse::<f64>().is_err() {
        return Err(ApiError::BadRequest(format!("Invalid price '{}'. Must be numeric.", s)));
    }
    Ok(())
}

/// Validate safe identifier (e.g., profile name, interface name).
pub fn validate_safe_identifier(val: &str, field_name: &str) -> Result<(), ApiError> {
    let s = val.trim();
    if s.is_empty() || s.len() > 64 {
        return Err(ApiError::BadRequest(format!("{} must be 1-64 characters", field_name)));
    }
    let is_safe = s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '@'));
    if !is_safe {
        return Err(ApiError::BadRequest(format!(
            "Invalid characters in {}. Only alphanumeric, '_', '-', '.', and '@' are allowed.",
            field_name
        )));
    }
    Ok(())
}

/// Validate Telegram bot token format (e.g. 123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ_1234567)
pub fn validate_telegram_bot_token(token: &str) -> Result<(), ApiError> {
    let s = token.trim();
    if s.is_empty() || s.len() < 20 || s.len() > 80 {
        return Err(ApiError::BadRequest("Invalid Telegram bot token length".into()));
    }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 || parts[0].is_empty() || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest("Invalid Telegram bot token format".into()));
    }
    if !parts[1].chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-')) {
        return Err(ApiError::BadRequest("Invalid Telegram bot token characters".into()));
    }
    Ok(())
}

/// Validate Telegram chat ID (numeric, may start with '-')
pub fn validate_telegram_chat_id(chat_id: &str) -> Result<(), ApiError> {
    let s = chat_id.trim();
    if s.is_empty() || s.len() > 30 {
        return Err(ApiError::BadRequest("Invalid Telegram chat ID length".into()));
    }
    let check = s.strip_prefix('-').unwrap_or(s);
    if check.is_empty() || !check.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::BadRequest("Invalid Telegram chat ID. Must be numeric.".into()));
    }
    Ok(())
}

/// Validate safe HTTP/HTTPS URL to prevent script injection in url="..." parameters
pub fn validate_safe_url(url: &str) -> Result<(), ApiError> {
    let s = url.trim();
    if !s.starts_with("http://") && !s.starts_with("https://") {
        return Err(ApiError::BadRequest("URL must start with http:// or https://".into()));
    }
    // Block quotes, semicolons, script delimiters, brackets, and newlines
    if s.contains(['"', '\'', ';', '$', '[', ']', '`', '\r', '\n', ' ']) {
        return Err(ApiError::BadRequest("URL contains illegal characters for RouterOS script execution".into()));
    }
    Ok(())
}

/// Escape string for safe embedding into RouterOS script double quotes
pub fn escape_routeros_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '$' => out.push_str("\\$"),
            '[' => out.push_str("\\["),
            ']' => out.push_str("\\]"),
            ';' => out.push_str("\\;"),
            '\r' | '\n' => out.push(' '),
            _ => out.push(c),
        }
    }
    out
}

/// Validate safe filename on RouterOS storage to prevent directory traversal and command injection.
/// Rejects path traversal sequences (".."), leading slashes, null bytes, control characters,
/// and permits only alphanumeric characters, underscores, dashes, dots, and internal single slashes.
pub fn validate_safe_filename(name: &str) -> Result<(), ApiError> {
    let s = name.trim();
    if s.is_empty() || s.len() > 128 {
        return Err(ApiError::BadRequest("Filename must be between 1 and 128 characters".into()));
    }
    if s.contains("..") || s.contains('\\') || s.starts_with('/') || s.starts_with('.') {
        return Err(ApiError::BadRequest("Filename contains forbidden path traversal patterns (e.g. '..', leading slashes, or hidden files)".into()));
    }
    // Only permit safe characters for router storage paths
    let is_safe = s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/'));
    if !is_safe {
        return Err(ApiError::BadRequest(format!(
            "Filename '{}' contains invalid characters. Only alphanumeric, '.', '_', '-', and '/' are permitted.",
            s
        )));
    }
    Ok(())
}

/// Standard HTML entity encoder to prevent Cross-Site Scripting (XSS)
pub fn html_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// Validate comma-separated port numbers (e.g. "80", "8291,22", "80,443,8080")
pub fn validate_ports_list(ports: &str) -> Result<(), ApiError> {
    let s = ports.trim();
    if s.is_empty() || s.len() > 64 {
        return Err(ApiError::BadRequest("Ports list must be 1-64 characters".into()));
    }
    for p in s.split(',') {
        let p_trimmed = p.trim();
        match p_trimmed.parse::<u16>() {
            Ok(port) if port > 0 => {}
            _ => return Err(ApiError::BadRequest(format!("Invalid port '{}' in ports list", p_trimmed))),
        }
    }
    Ok(())
}

/// Validate 24-hour time format (HH:MM:SS or HH:MM)
pub fn validate_time_format(time: &str) -> Result<(), ApiError> {
    let s = time.trim();
    let parts: Vec<&str> = s.split(':').collect();
    if (parts.len() != 2 && parts.len() != 3) || s.len() > 8 {
        return Err(ApiError::BadRequest("Invalid time format. Expected 'HH:MM:SS' or 'HH:MM'".into()));
    }
    for (idx, p) in parts.iter().enumerate() {
        if p.len() != 2 || !p.chars().all(|c| c.is_ascii_digit()) {
            return Err(ApiError::BadRequest("Time components must be 2 digits (e.g. 04:00:00)".into()));
        }
        let val: u32 = p.parse().unwrap_or(99);
        if idx == 0 && val > 23 {
            return Err(ApiError::BadRequest("Hour component must be 00-23".into()));
        }
        if (idx == 1 || idx == 2) && val > 59 {
            return Err(ApiError::BadRequest("Minute/Second component must be 00-59".into()));
        }
    }
    Ok(())
}

/// Validate bandwidth rate strings (e.g. "5M", "512k", "1G", "2M/10M", "100k/500k")
pub fn validate_bandwidth_rate(rate: &str) -> Result<(), ApiError> {
    let s = rate.trim();
    if s.is_empty() || s.len() > 32 {
        return Err(ApiError::BadRequest("Bandwidth rate must be 1-32 characters".into()));
    }
    let is_valid = s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '/' || matches!(c, 'k' | 'K' | 'm' | 'M' | 'g' | 'G' | 'b' | 'B'));
    if !is_valid {
        return Err(ApiError::BadRequest(format!("Invalid bandwidth rate format '{}'. Expected format like '5M', '512k', '2M/10M'", s)));
    }
    Ok(())
}

/// Validate IP address (IPv4 or IPv6) or CIDR subnet (e.g. "192.168.1.1", "10.0.0.0/24", "::1")
pub fn validate_ip_or_cidr(val: &str) -> Result<(), ApiError> {
    let s = val.trim();
    if s.is_empty() || s.len() > 64 {
        return Err(ApiError::BadRequest("IP or CIDR string must be 1-64 characters".into()));
    }
    if let Some((ip_part, prefix_part)) = s.split_once('/') {
        ip_part.parse::<std::net::IpAddr>().map_err(|_| ApiError::BadRequest(format!("Invalid IP in CIDR '{}'", s)))?;
        let prefix: u8 = prefix_part.parse().map_err(|_| ApiError::BadRequest(format!("Invalid prefix length in CIDR '{}'", s)))?;
        if prefix > 128 {
            return Err(ApiError::BadRequest(format!("Prefix length out of range in CIDR '{}'", s)));
        }
    } else {
        s.parse::<std::net::IpAddr>().map_err(|_| ApiError::BadRequest(format!("Invalid IP address '{}'", s)))?;
    }
    Ok(())
}

/// Validate MAC address format (e.g. "AA:BB:CC:DD:EE:FF" or "aa:bb:cc:dd:ee:ff" or with '-')
pub fn validate_mac_address(mac: &str) -> Result<(), ApiError> {
    let s = mac.trim();
    if s.len() != 17 {
        return Err(ApiError::BadRequest("MAC address must be 17 characters (e.g. 00:11:22:33:44:55)".into()));
    }
    let delimiter = if s.contains(':') { ':' } else if s.contains('-') { '-' } else {
        return Err(ApiError::BadRequest("MAC address must use ':' or '-' as delimiters".into()));
    };
    let parts: Vec<&str> = s.split(delimiter).collect();
    if parts.len() != 6 {
        return Err(ApiError::BadRequest("MAC address must have 6 octets".into()));
    }
    for p in parts {
        if p.len() != 2 || !p.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiError::BadRequest(format!("Invalid hex octet '{}' in MAC address", p)));
        }
    }
    Ok(())
}

/// Validate safe host name (FQDN) or IP address for ping, traceroute, and network tools
pub fn validate_host_or_ip(val: &str) -> Result<(), ApiError> {
    if val.contains(['\r', '\n', ' ', '\t']) {
        return Err(ApiError::BadRequest("Host or IP cannot contain whitespace or newlines".into()));
    }
    let s = val.trim();
    if s.is_empty() || s.len() > 128 {
        return Err(ApiError::BadRequest("Host or IP must be between 1 and 128 characters".into()));
    }
    let is_safe = s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'));
    if !is_safe {
        return Err(ApiError::BadRequest(format!("Host or IP '{}' contains invalid characters", s)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validity_validation() {
        assert!(validate_validity("1d").is_ok());
        assert!(validate_validity("12h").is_ok());
        assert!(validate_validity("30m").is_ok());
        assert!(validate_validity("1d12h30m").is_ok());
        assert!(validate_validity("1d; /user add evil").is_err());
        assert!(validate_validity("1d\"").is_err());
        assert!(validate_validity("").is_err());
    }

    #[test]
    fn test_price_validation() {
        assert!(validate_price("5000").is_ok());
        assert!(validate_price("10000.50").is_ok());
        assert!(validate_price("0").is_ok());
        assert!(validate_price("5000; evil").is_err());
        assert!(validate_price("abc").is_err());
    }

    #[test]
    fn test_telegram_validation() {
        assert!(validate_telegram_bot_token("123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ_1234567").is_ok());
        assert!(validate_telegram_bot_token("123:abc").is_err());
        assert!(validate_telegram_bot_token("invalid_token_without_colon").is_err());
        assert!(validate_telegram_bot_token("123456789:evil;script").is_err());

        assert!(validate_telegram_chat_id("123456789").is_ok());
        assert!(validate_telegram_chat_id("-1001234567890").is_ok());
        assert!(validate_telegram_chat_id("12345; evil").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_safe_url("https://example.com/webhook").is_ok());
        assert!(validate_safe_url("http://192.168.1.1:8080/api").is_ok());
        assert!(validate_safe_url("javascript:alert(1)").is_err());
        assert!(validate_safe_url("https://example.com/\" keep-result=no ; /user add").is_err());
    }

    #[test]
    fn test_routeros_escape() {
        assert_eq!(escape_routeros_string("clean"), "clean");
        assert_eq!(escape_routeros_string("foo\"bar"), "foo\\\"bar");
        assert_eq!(escape_routeros_string("foo$var"), "foo\\$var");
        assert_eq!(escape_routeros_string("foo;bar"), "foo\\;bar");
        assert_eq!(escape_routeros_string("foo\nbar"), "foo bar");
    }

    #[test]
    fn test_safe_filename() {
        assert!(validate_safe_filename("backup.backup").is_ok());
        assert!(validate_safe_filename("export_2026.rsc").is_ok());
        assert!(validate_safe_filename("disk1/backup.backup").is_ok());
        assert!(validate_safe_filename("../etc/passwd").is_err());
        assert!(validate_safe_filename("..\\boot.ini").is_err());
        assert!(validate_safe_filename("/root/evil").is_err());
        assert!(validate_safe_filename(".hidden").is_err());
        assert!(validate_safe_filename("file;evil").is_err());
        assert!(validate_safe_filename("").is_err());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>alert(1)</script>"), "&lt;script&gt;alert(1)&lt;/script&gt;");
        assert_eq!(html_escape("Ben & Jerry's \"Ice Cream\""), "Ben &amp; Jerry&#x27;s &quot;Ice Cream&quot;");
    }

    #[test]
    fn test_ports_list() {
        assert!(validate_ports_list("80").is_ok());
        assert!(validate_ports_list("8291,22").is_ok());
        assert!(validate_ports_list("80,443,8080").is_ok());
        assert!(validate_ports_list("0").is_err());
        assert!(validate_ports_list("70000").is_err());
        assert!(validate_ports_list("80;evil").is_err());
    }

    #[test]
    fn test_time_format() {
        assert!(validate_time_format("04:00:00").is_ok());
        assert!(validate_time_format("21:30").is_ok());
        assert!(validate_time_format("24:00:00").is_err());
        assert!(validate_time_format("04:60:00").is_err());
        assert!(validate_time_format("04:00;evil").is_err());
    }

    #[test]
    fn test_bandwidth_rate() {
        assert!(validate_bandwidth_rate("5M").is_ok());
        assert!(validate_bandwidth_rate("512k").is_ok());
        assert!(validate_bandwidth_rate("2M/10M").is_ok());
        assert!(validate_bandwidth_rate("1G").is_ok());
        assert!(validate_bandwidth_rate("").is_err());
        assert!(validate_bandwidth_rate("5M; evil").is_err());
    }

    #[test]
    fn test_ip_or_cidr() {
        assert!(validate_ip_or_cidr("192.168.1.1").is_ok());
        assert!(validate_ip_or_cidr("10.0.0.0/24").is_ok());
        assert!(validate_ip_or_cidr("::1").is_ok());
        assert!(validate_ip_or_cidr("2001:db8::/32").is_ok());
        assert!(validate_ip_or_cidr("999.999.999.999").is_err());
        assert!(validate_ip_or_cidr("10.0.0.0/200").is_err());
        assert!(validate_ip_or_cidr("192.168.1.1; evil").is_err());
    }

    #[test]
    fn test_mac_address() {
        assert!(validate_mac_address("00:1A:2B:3C:4D:5E").is_ok());
        assert!(validate_mac_address("00-1a-2b-3c-4d-5e").is_ok());
        assert!(validate_mac_address("00:11:22:33:44").is_err());
        assert!(validate_mac_address("00:11:22:33:44:GG").is_err());
        assert!(validate_mac_address("00:11:22:33:44:55;evil").is_err());
    }

    #[test]
    fn test_host_or_ip() {
        assert!(validate_host_or_ip("192.168.1.1").is_ok());
        assert!(validate_host_or_ip("google.com").is_ok());
        assert!(validate_host_or_ip("one.one.one.one").is_ok());
        assert!(validate_host_or_ip("::1").is_ok());
        assert!(validate_host_or_ip("8.8.8.8; evil").is_err());
        assert!(validate_host_or_ip("8.8.8.8\n").is_err());
        assert!(validate_host_or_ip("").is_err());
    }
}
