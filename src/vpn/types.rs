//! VPN data types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// VPN connection result
#[derive(Debug, Clone)]
pub struct ConnectResult {
    pub server_id: String,
    pub ip: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub via: Option<String>,
}

/// Server features
///
/// Note: `protonvpn-cli` does not provide server feature information
/// (secure_core, p2p, tor, streaming). These fields will always be `false`
/// unless the CLI adds support for a server info command.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerFeatures {
    pub secure_core: bool,
    pub p2p: bool,
    pub tor: bool,
    pub streaming: bool,
}

/// City information with features
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct City {
    pub name: String,
    #[serde(default)]
    pub name_lower: String,
    pub features: Vec<String>, // e.g., ["P2P", "Secure"]
}

impl City {
    pub fn new(name: String) -> Self {
        let name_lower = name.to_lowercase();
        Self {
            name,
            name_lower,
            features: Vec::new(),
        }
    }

    pub fn with_features(name: String, features: Vec<String>) -> Self {
        let name_lower = name.to_lowercase();
        Self {
            name,
            name_lower,
            features,
        }
    }

    pub fn ensure_name_lower(&mut self) {
        if self.name_lower.is_empty() {
            self.name_lower = self.name.to_lowercase();
        }
    }
}

/// VPN server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub code: String,
    pub code_lower: String,
    pub country: String,
    pub country_lower: String,
    pub cities: Vec<City>,
}

// ============================================================================
// Parsing functions - moved from client.rs
// ============================================================================

/// Parse countries output from protonvpn CLI
/// Format: "Country Name             XX"
pub fn parse_countries(output: &str) -> HashMap<String, String> {
    let mut countries = HashMap::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Skip header lines
        if line.starts_with("Country") || line.starts_with('-') {
            continue;
        }
        // Skip update messages
        if line.starts_with("Server list") {
            continue;
        }

        // Parse: "Country Name             XX" (code at the end)
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // SAFETY: parts.len() >= 2 guarantees at least one element exists.
            // This is checked immediately above, so unwrap is safe here.
            let code = parts
                .last()
                .expect("parts has at least 2 elements due to length check");
            // Everything before is the country name
            let name = parts[..parts.len() - 1].join(" ");
            if !code.is_empty() && !name.is_empty() && code.len() <= 3 {
                countries.insert(code.to_string(), name.to_string());
            }
        }
    }

    countries
}

/// Parse cities output with features from protonvpn CLI
pub fn parse_cities_with_features(output: &str) -> Vec<City> {
    let mut cities: Vec<City> = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if !line.starts_with(|c: char| c.is_alphabetic()) {
            continue;
        }

        if line.starts_with("Cities") || line.starts_with("City") {
            continue;
        }

        if line.starts_with("Server list") {
            continue;
        }

        let mut chars = line.char_indices().peekable();
        let mut name_end = None;

        while let Some((start, c)) = chars.next() {
            if c.is_whitespace() {
                let mut consecutive = 1;
                while let Some(&(_, next_c)) = chars.peek() {
                    if next_c.is_whitespace() {
                        consecutive += 1;
                        chars.next();
                    } else {
                        break;
                    }
                }
                if consecutive >= 2 {
                    name_end = Some(start);
                    break;
                }
            }
        }

        let (name, features_str) = match name_end {
            Some(pos) => (line[..pos].to_string(), line[pos..].trim()),
            None => (line.to_string(), ""),
        };

        let features: Vec<String> = features_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(existing) = cities.iter_mut().find(|c| c.name == name) {
            if !features.is_empty() && existing.features.is_empty() {
                existing.features = features;
            }
            continue;
        }

        cities.push(City::with_features(name, features));
    }

    cities
}

/// Extract IP address from a line
fn extract_ip_from_line(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, part) in parts.iter().enumerate() {
        let part_clean = part.trim_end_matches(':');
        if (part_clean == "is"
            || part_clean == "IP"
            || part_clean == "address"
            || part_clean == "address.")
            && i + 1 < parts.len()
        {
            let potential_ip = parts[i + 1].trim_end_matches('.');
            if potential_ip.contains('.')
                && potential_ip.chars().filter(|&c| c == '.').count() == 3
                && !potential_ip.starts_with("10.")
                && !potential_ip.starts_with("172.")
                && !potential_ip.starts_with("192.168")
            {
                return Some(potential_ip.to_string());
            }
        }
    }
    None
}

/// Parse connect output to extract server ID, IP, city and country
pub fn parse_connect_output(output: &str) -> ConnectResult {
    let mut server_id = String::new();
    let mut ip = None;
    let mut city = None;
    let mut country = None;
    let mut via = None;

    // Collect all lines
    let lines: Vec<&str> = output.lines().collect();

    // Find "Connected to" line - skip any preceding error/traceback lines
    let connected_idx = lines
        .iter()
        .position(|l| l.trim().starts_with("Connected to "));

    if let Some(idx) = connected_idx {
        let line = lines[idx].trim();

        // Extract server_id, city, country
        if let Some(rest) = line.strip_prefix("Connected to ") {
            if let Some(end_idx) = rest.find(" in ") {
                server_id = rest[..end_idx].to_string();
                let after_server = &rest[end_idx + 4..];

                // Check for Secure Core format: "in Tokyo, via Switzerland."
                if let Some(via_idx) = after_server.find(", via ") {
                    // Secure Core: city, via entry_country
                    if let Some(period_idx) = after_server.find('.') {
                        city = Some(after_server[..via_idx].to_string());
                        via = Some(after_server[via_idx + 6..period_idx].to_string());
                    } else {
                        city = Some(after_server[..via_idx].to_string());
                        via = Some(after_server[via_idx + 6..].to_string());
                    }
                    country = None;
                } else if let Some(period_idx) = after_server.find('.') {
                    let location_part = &after_server[..period_idx];
                    if let Some(last_comma_idx) = location_part.rfind(", ") {
                        city = Some(location_part[..last_comma_idx].to_string());
                        country = Some(location_part[last_comma_idx + 2..].to_string());
                    } else {
                        city = None;
                        country = Some(location_part.to_string());
                    }
                } else if let Some(last_comma_idx) = after_server.rfind(", ") {
                    city = Some(after_server[..last_comma_idx].to_string());
                    country = Some(after_server[last_comma_idx + 2..].to_string());
                }
            }

            // Try to extract IP from same line first
            ip = extract_ip_from_line(line);
        }

        // If IP not on same line, look at subsequent lines
        if ip.is_none() && idx + 1 < lines.len() {
            // Check next line for "Your new IP address is X.X.X.X."
            let next_line = lines[idx + 1].trim();
            ip = extract_ip_from_line(next_line);
        }
    }

    ConnectResult {
        server_id,
        ip,
        city,
        country,
        via,
    }
}

/// Status information parsed from `protonvpn status` output
#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    pub server: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub protocol: Option<String>,
    pub uptime: Option<chrono::Duration>,
    pub load: Option<u8>, // Server load percentage (0-100)
}

/// Parse `protonvpn status` output for location information
///
/// Supports multiple CLI output formats:
/// - Format 1: `Location: Tokyo, Japan`
/// - Format 2: `City: Tokyo` + `Country: Japan`
/// - Format 3: `Server: JP#374` (fallback: extract country code from server ID)
///
/// Returns None if server line is missing (indicates not connected)
pub fn parse_status_output(output: &str) -> Option<StatusInfo> {
    let mut info = StatusInfo::default();
    let mut has_server = false;

    for line in output.lines() {
        let line = line.trim();

        // Parse Server field
        if let Some(val) = line.strip_prefix("Server:") {
            let server = val.trim().to_string();
            if !server.is_empty() {
                info.server = Some(server);
                has_server = true;
            }
        }
        // Parse Location field (Format 1: "Location: Tokyo, Japan")
        else if let Some(val) = line.strip_prefix("Location:") {
            let val = val.trim();
            if let Some(comma_idx) = val.find(", ") {
                info.city = Some(val[..comma_idx].trim().to_string());
                info.country = Some(val[comma_idx + 2..].trim().to_string());
            } else if !val.is_empty() {
                // No comma - determine if it's a city or country
                // Country codes are typically 2-3 characters
                if val.len() <= 3 && val.chars().all(|c| c.is_ascii_uppercase()) {
                    info.country = Some(val.to_string());
                } else {
                    info.city = Some(val.to_string());
                }
            }
        }
        // Parse City field (Format 2)
        else if let Some(val) = line.strip_prefix("City:") {
            let city = val.trim().to_string();
            if !city.is_empty() {
                info.city = Some(city);
            }
        }
        // Parse Country field (Format 2)
        else if let Some(val) = line.strip_prefix("Country:") {
            let country = val.trim().to_string();
            if !country.is_empty() {
                info.country = Some(country);
            }
        }
        // Parse Protocol field
        else if let Some(val) = line.strip_prefix("Protocol:") {
            let protocol = val.trim().to_string();
            if !protocol.is_empty() {
                info.protocol = Some(protocol);
            }
        }
        // Parse Load field
        else if let Some(val) = line.strip_prefix("Load:") {
            let val = val.trim();
            if let Some(percent_idx) = val.find('%') {
                let load_str = &val[..percent_idx];
                if let Ok(load) = load_str.parse::<u8>() {
                    info.load = Some(load);
                }
            }
        }
        // Parse Uptime/Time field
        else if let Some(time_str) = line
            .strip_prefix("Uptime:")
            .or_else(|| line.strip_prefix("Time:"))
        {
            let time_str = time_str.trim();
            let parts: Vec<&str> = time_str.split(':').collect();
            if parts.len() == 3 {
                if let (Ok(hours), Ok(mins), Ok(secs)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<i64>(),
                    parts[2].parse::<i64>(),
                ) {
                    let total_seconds = hours * 3600 + mins * 60 + secs;
                    info.uptime = Some(chrono::Duration::seconds(total_seconds));
                }
            }
        }
    }

    // Fallback: extract country code from server ID if country is missing
    if info.country.is_none() {
        if let Some(ref server) = info.server {
            info.country = extract_country_code(server);
        }
    }

    if has_server {
        Some(info)
    } else {
        None
    }
}

/// Extract country code from server ID
///
/// Examples:
/// - "JP#374" → "JP"
/// - "CH-JP#2" → "CH" (entry country for Secure Core)
fn extract_country_code(server_id: &str) -> Option<String> {
    server_id
        .split(['#', '-'])
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Parse uptime from protonvpn status output
///
/// Supports both "Uptime:" and "Time:" field names (varies by CLI version)
/// Expected format: "Uptime: 00:15:32" or "Time: 00:15:32"
/// Returns None if uptime line is missing or malformed
pub fn parse_status_uptime(output: &str) -> Option<chrono::Duration> {
    for line in output.lines() {
        let time_str = line
            .strip_prefix("Uptime:")
            .or_else(|| line.strip_prefix("Time:"));
        if let Some(time_str) = time_str {
            let time_str = time_str.trim();
            let parts: Vec<&str> = time_str.split(':').collect();
            if parts.len() == 3 {
                let hours: i64 = parts[0].parse().ok()?;
                let mins: i64 = parts[1].parse().ok()?;
                let secs: i64 = parts[2].parse().ok()?;
                let total_seconds = hours * 3600 + mins * 60 + secs;
                return Some(chrono::Duration::seconds(total_seconds));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_countries_single() {
        let output = "Japan               JP";
        let countries = parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_countries_multiple() {
        let output = r#"Japan               JP
United States       US
Germany             DE
United Kingdom      GB"#;
        let countries = parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
        assert_eq!(countries.get("US"), Some(&"United States".to_string()));
        assert_eq!(countries.get("DE"), Some(&"Germany".to_string()));
        assert_eq!(countries.get("GB"), Some(&"United Kingdom".to_string()));
    }

    #[test]
    fn test_parse_countries_skips_header() {
        let output = r#"Country             Code
------------------  ----
Japan               JP"#;
        let countries = parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
        assert!(!countries.contains_key("Country"));
        assert!(!countries.contains_key("Code"));
    }

    #[test]
    fn test_parse_countries_skips_dashes() {
        let output = r#"------------------  ----
Japan               JP"#;
        let countries = parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_countries_empty() {
        let output = "";
        let countries = parse_countries(output);

        assert!(countries.is_empty());
    }

    #[test]
    fn test_parse_countries_with_server_list_message() {
        let output = r#"Server list is currently being updated. Please try again later.
Japan               JP"#;
        let countries = parse_countries(output);

        assert_eq!(countries.get("JP"), Some(&"Japan".to_string()));
    }

    #[test]
    fn test_parse_cities_single() {
        let output = "Tokyo";
        let cities = parse_cities_with_features(output);

        assert_eq!(cities.len(), 1);
        assert_eq!(cities[0].name, "Tokyo");
    }

    #[test]
    fn test_parse_cities_multiple() {
        let output = "Tokyo\nOsaka\nKyoto\nSapporo";
        let cities = parse_cities_with_features(output);

        assert_eq!(cities.len(), 4);
        assert_eq!(cities[0].name, "Tokyo");
        assert_eq!(cities[1].name, "Osaka");
        assert_eq!(cities[2].name, "Kyoto");
        assert_eq!(cities[3].name, "Sapporo");
    }

    #[test]
    fn test_parse_cities_with_whitespace() {
        let output = "  Tokyo  \n  Osaka  \n  ";
        let cities = parse_cities_with_features(output);

        assert_eq!(cities.len(), 2);
        assert_eq!(cities[0].name, "Tokyo");
        assert_eq!(cities[1].name, "Osaka");
    }

    #[test]
    fn test_parse_cities_multi_word_with_features() {
        let output = "Tel Aviv  P2P, Secure Core\nOsaka  P2P";
        let cities = parse_cities_with_features(output);

        assert_eq!(cities.len(), 2);
        assert_eq!(cities[0].name, "Tel Aviv");
        assert_eq!(cities[0].features, vec!["P2P", "Secure Core"]);
        assert_eq!(cities[1].name, "Osaka");
        assert_eq!(cities[1].features, vec!["P2P"]);
    }

    #[test]
    fn test_parse_cities_empty() {
        let output = "";
        let cities = parse_cities_with_features(output);

        assert!(cities.is_empty());
    }

    #[test]
    fn test_parse_cities_with_duplicates() {
        let output = "Cities in Canada:\n\
            City       Features\n\
            ---------  ----------\n\
            Montreal   P2P\n\
            Vancouver\n\
            Montreal   P2P\n\
            Toronto    P2P\n\
            Vancouver  P2P\n\
            Toronto    P2P\n\
            Vancouver  P2P\n\
            Toronto    P2P\n\
            Vancouver  P2P\n\
            Toronto    P2P\n\
            Vancouver  P2P\n\
            Montreal   P2P\n\
            Toronto\n\
            Vancouver";
        let cities = parse_cities_with_features(output);

        assert_eq!(cities.len(), 3);
        assert_eq!(cities[0].name, "Montreal");
        assert_eq!(cities[0].features, vec!["P2P"]);
        assert_eq!(cities[1].name, "Vancouver");
        assert_eq!(cities[1].features, vec!["P2P"]);
        assert_eq!(cities[2].name, "Toronto");
        assert_eq!(cities[2].features, vec!["P2P"]);
    }

    #[test]
    fn test_parse_connect_output_no_ip() {
        let output = r#"Connected to DE#200 in Berlin, Germany.
No IP address found."#;
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "DE#200");
        assert_eq!(result.ip, None);
        assert_eq!(result.city, Some("Berlin".to_string()));
        assert_eq!(result.country, Some("Germany".to_string()));
    }

    #[test]
    fn test_parse_connect_output_private_ip_ignored() {
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 10.0.0.1"#;
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "JP#379");
        assert_eq!(result.ip, None);
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_no_server() {
        let output = "Connection failed. Please try again.";
        let result = parse_connect_output(output);

        assert!(result.server_id.is_empty());
        assert_eq!(result.ip, None);
        assert_eq!(result.city, None);
        assert_eq!(result.country, None);
    }

    #[test]
    fn test_parse_connect_output_same_line_ip() {
        let output =
            r#"Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144."#;
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "JP#374");
        assert_eq!(result.ip, Some("159.26.119.144".to_string()));
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_with_error_traceback() {
        let output = r#"2026-03-06T06:48:22.914403+00:00 | concurrent.futures:336 | ERROR | exception calling callback
Traceback (most recent call last):
  File "/usr/lib/python3.14/site-packages/proton/vpn/backend/networkmanager/core/local_agent/listener.py", line 51, in connect
    certificate)
    ^^^^^^^^^^^^
local_agent.LocalAgentError: Tokio(Custom { kind: InvalidData, error: InvalidCertificate(NotValidForName) })

Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144."#;
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "JP#374");
        assert_eq!(result.ip, Some("159.26.119.144".to_string()));
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_secure_core() {
        let output =
            "Connected to CH-JP#2 in Tokyo, via Switzerland. Your new IP address is 37.19.205.233.";
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "CH-JP#2");
        assert_eq!(result.ip, Some("37.19.205.233".to_string()));
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, None);
        assert_eq!(result.via, Some("Switzerland".to_string()));
    }

    #[test]
    fn test_parse_connect_output_secure_core_no_ip() {
        let output = "Connected to CH-JP#2 in Tokyo, via Switzerland.";
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "CH-JP#2");
        assert_eq!(result.ip, None);
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, None);
        assert_eq!(result.via, Some("Switzerland".to_string()));
    }

    #[test]
    fn test_parse_connect_output_country_only() {
        let output = "Connected to JP#374 in Japan. Your new IP address is 159.26.119.144.";
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "JP#374");
        assert_eq!(result.ip, Some("159.26.119.144".to_string()));
        assert_eq!(result.city, None);
        assert_eq!(result.country, Some("Japan".to_string()));
        assert_eq!(result.via, None);
    }

    #[test]
    fn test_parse_connect_output_ip_on_separate_line() {
        let output = "Connected to JP#423 in Tokyo, Japan.\nYour new IP address is 159.26.119.172.";
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "JP#423");
        assert_eq!(result.ip, Some("159.26.119.172".to_string()));
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_secure_core_ip_on_separate_line() {
        let output = "Connected to CH-JP#2 in Tokyo, via Switzerland.\nYour new IP address is 103.155.232.232.";
        let result = parse_connect_output(output);

        assert_eq!(result.server_id, "CH-JP#2");
        assert_eq!(result.ip, Some("103.155.232.232".to_string()));
        assert_eq!(result.city, Some("Tokyo".to_string()));
        assert_eq!(result.country, None);
        assert_eq!(result.via, Some("Switzerland".to_string()));
    }

    #[test]
    fn test_parse_status_uptime() {
        let output = r#"Status: Connected
Server: JP#374
Country: Japan
City: Tokyo
IP: 159.26.119.144
Uptime: 00:15:32"#;
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_some());
        let uptime = uptime.unwrap();
        assert_eq!(uptime.num_seconds(), 15 * 60 + 32);
    }

    #[test]
    fn test_parse_status_uptime_hours() {
        let output = "Uptime: 02:30:45";
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_some());
        let uptime = uptime.unwrap();
        assert_eq!(uptime.num_seconds(), 2 * 3600 + 30 * 60 + 45);
    }

    #[test]
    fn test_parse_status_uptime_missing() {
        let output = "Status: Connected\nServer: JP#374";
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_none());
    }

    #[test]
    fn test_parse_status_uptime_malformed() {
        let output = "Uptime: invalid";
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_none());
    }

    #[test]
    fn test_parse_status_time_field() {
        let output = r#"Status:       Connected
Time:         1:23:45
IP:           192.168.1.1
Server:       JP#374
Country:      Japan
City:         Tokyo"#;
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_some());
        let uptime = uptime.unwrap();
        assert_eq!(uptime.num_seconds(), 1 * 3600 + 23 * 60 + 45);
    }

    #[test]
    fn test_parse_status_time_field_simple() {
        let output = "Time: 00:05:30";
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_some());
        let uptime = uptime.unwrap();
        assert_eq!(uptime.num_seconds(), 5 * 60 + 30);
    }

    #[test]
    fn test_parse_status_uptime_takes_priority() {
        let output = r#"Uptime: 01:00:00
Time: 00:30:00"#;
        let uptime = parse_status_uptime(output);
        assert!(uptime.is_some());
        let uptime = uptime.unwrap();
        assert_eq!(uptime.num_seconds(), 1 * 3600);
    }

    #[test]
    fn test_parse_status_output_location_format() {
        let output = r#"Server: JP#374
Location: Tokyo, Japan
Protocol: WireGuard
Uptime: 00:15:32"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("JP#374".to_string()));
        assert_eq!(info.city, Some("Tokyo".to_string()));
        assert_eq!(info.country, Some("Japan".to_string()));
        assert_eq!(info.protocol, Some("WireGuard".to_string()));
        assert_eq!(info.uptime, Some(chrono::Duration::seconds(15 * 60 + 32)));
    }

    #[test]
    fn test_parse_status_output_city_country_format() {
        let output = r#"Status: Connected
Server: JP#374
Country: Japan
City: Tokyo
IP: 159.26.119.144
Uptime: 00:15:32"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("JP#374".to_string()));
        assert_eq!(info.city, Some("Tokyo".to_string()));
        assert_eq!(info.country, Some("Japan".to_string()));
        assert_eq!(info.uptime, Some(chrono::Duration::seconds(15 * 60 + 32)));
    }

    #[test]
    fn test_parse_status_output_location_city_only() {
        let output = r#"Server: JP#374
Location: Tokyo
Protocol: WireGuard"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("JP#374".to_string()));
        assert_eq!(info.city, Some("Tokyo".to_string()));
        assert_eq!(info.country, Some("JP".to_string()));
    }

    #[test]
    fn test_parse_status_output_fallback_country_from_server_id() {
        let output = r#"Server: JP#374
Protocol: WireGuard"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("JP#374".to_string()));
        assert_eq!(info.city, None);
        assert_eq!(info.country, Some("JP".to_string()));
    }

    #[test]
    fn test_parse_status_output_secure_core_server_id() {
        let output = r#"Server: CH-JP#2
Location: Tokyo
Protocol: WireGuard"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("CH-JP#2".to_string()));
        assert_eq!(info.city, Some("Tokyo".to_string()));
        assert_eq!(info.country, Some("CH".to_string()));
    }

    #[test]
    fn test_parse_status_output_not_connected() {
        let output = "Status: Disconnected";
        let info = parse_status_output(output);
        assert!(info.is_none());
    }

    #[test]
    fn test_parse_status_output_empty() {
        let output = "";
        let info = parse_status_output(output);
        assert!(info.is_none());
    }

    #[test]
    fn test_extract_country_code_simple() {
        assert_eq!(extract_country_code("JP#374"), Some("JP".to_string()));
        assert_eq!(extract_country_code("US#100"), Some("US".to_string()));
    }

    #[test]
    fn test_extract_country_code_secure_core() {
        assert_eq!(extract_country_code("CH-JP#2"), Some("CH".to_string()));
        assert_eq!(extract_country_code("US-DE#5"), Some("US".to_string()));
    }

    #[test]
    fn test_extract_country_code_no_hash() {
        assert_eq!(extract_country_code("JP"), Some("JP".to_string()));
    }

    #[test]
    fn test_extract_country_code_empty() {
        assert_eq!(extract_country_code(""), None);
    }

    #[test]
    fn test_parse_status_output_with_load() {
        let output = r#"Status: Connected
Server: JP#374 in Tokyo, Japan
Load: 5%
Protocol: WireGuard
Uptime: 00:15:32"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.server, Some("JP#374 in Tokyo, Japan".to_string()));
        assert_eq!(info.load, Some(5));
    }

    #[test]
    fn test_parse_status_output_load_100_percent() {
        let output = r#"Status: Connected
Server: JP#374
Load: 100%"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.load, Some(100));
    }

    #[test]
    fn test_parse_status_output_missing_load() {
        let output = r#"Status: Connected
Server: JP#374
Protocol: WireGuard"#;
        let info = parse_status_output(output);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.load, None);
    }
}
