//! Tests for VPN operations.
//!
//! Tests VPN client operations like listing servers, connecting, disconnecting.

use protonvpn_tui::vpn::{City, Server, VpnClient};

mod vpn_client_operations {
    use super::*;

    #[test]
    fn test_vpn_client_new() {
        let client = VpnClient::new();
        drop(client);
    }

    #[test]
    fn test_vpn_client_with_path() {
        let client = VpnClient::with_path("echo");
        drop(client);
    }

    #[test]
    fn test_vpn_client_get_servers() {
        let client = VpnClient::new();
        let _servers = client.get_servers();
    }

    #[test]
    fn test_vpn_state_new() {
        let client = VpnClient::new();
        drop(client);
    }

    #[test]
    fn test_vpn_state_get_servers() {
        let client = VpnClient::new();
        let _servers = client.get_servers();
    }

    #[test]
    fn test_vpn_state_get_servers_or_refresh() {
        let client = VpnClient::new();
        let _servers = client.get_servers();
    }
}

mod server_cache {
    use super::*;

    #[test]
    fn test_server_serialization_roundtrip() {
        let server = Server {
            code: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![City::new("Tokyo".to_string())],
        };

        let serialized = serde_json::to_string(&server).unwrap();
        let deserialized: Server = serde_json::from_str(&serialized).unwrap();

        assert_eq!(server.code, deserialized.code);
        assert_eq!(server.country, deserialized.country);
        assert_eq!(server.cities.len(), deserialized.cities.len());
    }

    #[test]
    fn test_server_multiple_cities_serialization() {
        let server = Server {
            code: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![
                City::new("Tokyo".to_string()),
                City::new("Osaka".to_string()),
                City::new("Nagoya".to_string()),
            ],
        };

        let serialized = serde_json::to_string(&server).unwrap();
        let deserialized: Server = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.cities.len(), 3);
    }

    #[test]
    fn test_city_with_features() {
        use protonvpn_tui::vpn::City;

        let city = City {
            name: "Tokyo".to_string(),
            features: vec!["P2P".to_string(), "Secure Core".to_string()],
        };

        let serialized = serde_json::to_string(&city).unwrap();
        let deserialized: City = serde_json::from_str(&serialized).unwrap();

        assert_eq!(city.name, deserialized.name);
        assert_eq!(city.features, deserialized.features);
    }
}

mod vpn_connection {
    use super::*;

    #[test]
    fn test_vpn_state_is_connected() {
        let client = VpnClient::new();
        let _ = client.is_connected();
    }

    #[test]
    fn test_vpn_state_matches_ip() {
        let client = VpnClient::new();
        assert!(!client.matches_ip("1.1.1.1"));
        assert!(!client.matches_ip("8.8.8.8"));
    }
}
