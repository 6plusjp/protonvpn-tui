//! VPN data types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
    pub features: Vec<String>, // e.g., ["P2P", "Secure"]
}

impl City {
    pub fn new(name: String) -> Self {
        Self {
            name,
            features: Vec::new(),
        }
    }

    pub fn with_features(name: String, features: Vec<String>) -> Self {
        Self { name, features }
    }
}

/// VPN server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub code: String,        // Country code (e.g., "JP", "US")
    pub country: String,   // Full country name
    pub cities: Vec<City>, // City information with features
}

/// Connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub server_ip: String,
    pub protocol: String,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: chrono::Utc::now(),
            server_ip: String::new(),
            protocol: String::from("Unknown"),
        }
    }
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
            // Last part is the code
            let code = parts.last().unwrap();
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

/// Parse connect output to extract server ID, IP, city and country
pub fn parse_connect_output(
    output: &str,
) -> (String, Option<String>, Option<String>, Option<String>) {
    let mut server_id = String::new();
    let mut ip = None;
    let mut city = None;
    let mut country = None;

    // Find "Connected to" line first - all info is in this single line
    let connected_line = output
        .lines()
        .find(|l| l.trim().starts_with("Connected to "));

    if let Some(line) = connected_line {
        let line = line.trim();

        // Extract server_id, city, country
        if let Some(rest) = line.strip_prefix("Connected to ") {
            if let Some(end_idx) = rest.find(" in ") {
                server_id = rest[..end_idx].to_string();
                let after_server = &rest[end_idx + 4..];

                if let Some(period_idx) = after_server.find('.') {
                    let location_part = &after_server[..period_idx];
                    if let Some(last_comma_idx) = location_part.rfind(", ") {
                        city = Some(location_part[..last_comma_idx].to_string());
                        country = Some(location_part[last_comma_idx + 2..].to_string());
                    }
                } else if let Some(last_comma_idx) = after_server.rfind(", ") {
                    city = Some(after_server[..last_comma_idx].to_string());
                    country = Some(after_server[last_comma_idx + 2..].to_string());
                }
            }

            // Extract IP from same line
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                let part_clean = part.trim_end_matches(':');
                if (part_clean == "is" || part_clean == "IP" || part_clean == "address")
                    && i + 1 < parts.len()
                {
                    let potential_ip = parts[i + 1].trim_end_matches('.');
                    if potential_ip.contains('.')
                        && potential_ip.chars().filter(|&c| c == '.').count() == 3
                        && !potential_ip.starts_with("10.")
                        && !potential_ip.starts_with("172.")
                        && !potential_ip.starts_with("192.168")
                    {
                        ip = Some(potential_ip.to_string());
                        break;
                    }
                }
            }
        }
    }

    (server_id, ip, city, country)
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
        let (server_id, ip, city, country) = parse_connect_output(output);

        assert_eq!(server_id, "DE#200");
        assert_eq!(ip, None);
        assert_eq!(city, Some("Berlin".to_string()));
        assert_eq!(country, Some("Germany".to_string()));
    }

    #[test]
    fn test_parse_connect_output_private_ip_ignored() {
        let output = r#"Connected to JP#379 in Tokyo, Japan.
IP address: 10.0.0.1"#;
        let (server_id, ip, city, country) = parse_connect_output(output);

        assert_eq!(server_id, "JP#379");
        assert_eq!(ip, None);
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
    }

    #[test]
    fn test_parse_connect_output_no_server() {
        let output = "Connection failed. Please try again.";
        let (server_id, ip, city, country) = parse_connect_output(output);

        assert!(server_id.is_empty());
        assert_eq!(ip, None);
        assert_eq!(city, None);
        assert_eq!(country, None);
    }

    #[test]
    fn test_parse_connect_output_same_line_ip() {
        let output =
            r#"Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144."#;
        let (server_id, ip, city, country) = parse_connect_output(output);

        assert_eq!(server_id, "JP#374");
        assert_eq!(ip, Some("159.26.119.144".to_string()));
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
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
        let (server_id, ip, city, country) = parse_connect_output(output);

        assert_eq!(server_id, "JP#374");
        assert_eq!(ip, Some("159.26.119.144".to_string()));
        assert_eq!(city, Some("Tokyo".to_string()));
        assert_eq!(country, Some("Japan".to_string()));
    }
}
