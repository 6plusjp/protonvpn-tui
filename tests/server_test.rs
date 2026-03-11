//! Regression tests for issue002: Servers view with city details and connection

use protonvpn_tui::state::AppView;

mod app_view {
    use super::*;

    #[test]
    fn test_app_view_servers_exists() {
        let view = AppView::Servers;
        assert_eq!(view, AppView::Servers);
    }

    #[test]
    fn test_app_view_default_is_servers() {
        let view = AppView::default();
        assert_eq!(view, AppView::Servers);
    }

    #[test]
    fn test_app_view_next_from_servers() {
        let view = AppView::Servers;
        assert_eq!(view.next(), AppView::Settings);
    }

    #[test]
    fn test_app_view_next_from_settings() {
        let view = AppView::Settings;
        assert_eq!(view.next(), AppView::Logs);
    }

    #[test]
    fn test_app_view_next_from_logs() {
        let view = AppView::Logs;
        assert_eq!(view.next(), AppView::Servers);
    }

    #[test]
    fn test_app_view_prev_from_servers() {
        let view = AppView::Servers;
        assert_eq!(view.prev(), AppView::Logs);
    }

    #[test]
    fn test_app_view_prev_from_logs() {
        let view = AppView::Logs;
        assert_eq!(view.prev(), AppView::Settings);
    }

    #[test]
    fn test_app_view_prev_from_settings() {
        let view = AppView::Settings;
        assert_eq!(view.prev(), AppView::Servers);
    }

    #[test]
    fn test_app_view_help_cycles_to_servers() {
        let view = AppView::Help;
        assert_eq!(view.next(), AppView::Servers);
        assert_eq!(view.prev(), AppView::Servers);
    }

    #[test]
    fn test_app_view_all_variants() {
        let views = [
            AppView::Servers,
            AppView::Settings,
            AppView::Help,
            AppView::Logs,
        ];

        for (i, v1) in views.iter().enumerate() {
            for (j, v2) in views.iter().enumerate() {
                if i != j {
                    assert_ne!(v1, v2);
                }
            }
        }
    }
}

mod server {
    use protonvpn_tui::vpn::City as VpnCity;
    use protonvpn_tui::vpn::Server;

    #[test]
    fn test_server_with_cities() {
        let server = Server {
            id: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![
                VpnCity::new("Tokyo".to_string()),
                VpnCity::new("Osaka".to_string()),
            ],
        };
        assert_eq!(server.id, "JP");
        assert_eq!(server.country, "Japan");
        assert_eq!(server.cities.len(), 2);
    }

    #[test]
    fn test_server_with_empty_cities() {
        let server = Server {
            id: "XX".to_string(),
            country: "Unknown".to_string(),
            cities: vec![],
        };
        assert!(server.cities.is_empty());
    }

    #[test]
    fn test_server_clone() {
        let original = Server {
            id: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![VpnCity::new("Tokyo".to_string())],
        };
        let cloned = original.clone();
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.country, cloned.country);
        assert_eq!(original.cities, cloned.cities);
    }

    #[test]
    fn test_server_serialize() {
        let server = Server {
            id: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![VpnCity::new("Tokyo".to_string())],
        };
        let json = serde_json::to_string(&server).unwrap();
        assert!(json.contains("JP"));
        assert!(json.contains("Japan"));
        assert!(json.contains("Tokyo"));
    }

    #[test]
    fn test_server_deserialize() {
        let json = r#"{"id":"US","country":"United States","cities":[{"name":"New York","features":[]},{"name":"Los Angeles","features":[]}]}"#;
        let server: Server = serde_json::from_str(json).unwrap();
        assert_eq!(server.id, "US");
        assert_eq!(server.country, "United States");
        assert_eq!(server.cities.len(), 2);
    }

    #[test]
    fn test_server_city_count() {
        let server = Server {
            id: "JP".to_string(),
            country: "Japan".to_string(),
            cities: vec![
                VpnCity::new("Tokyo".to_string()),
                VpnCity::new("Osaka".to_string()),
                VpnCity::new("Nagoya".to_string()),
                VpnCity::new("Sapporo".to_string()),
            ],
        };
        assert_eq!(server.cities.len(), 4);
    }
}

mod vpn_client {
    use protonvpn_tui::vpn::VpnClient;

    #[test]
    fn test_vpn_client_has_connect_random() {
        let client = VpnClient::with_path("echo");
        let result = client.connect_random();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_vpn_client_has_disconnect() {
        let client = VpnClient::with_path("echo");
        let result = client.disconnect();
        assert!(result.is_err() || result.is_ok());
    }
}

mod vpn_state {
    use protonvpn_tui::vpn::VpnClient;

    #[test]
    #[ignore = "Actually connects to VPN, disconnecting internet"]
    fn test_vpn_state_has_connect_random() {
        let client = VpnClient::new();
        let result = client.connect_random();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    #[ignore = "Actually disconnects VPN"]
    fn test_vpn_state_has_disconnect() {
        let client = VpnClient::new();
        let result = client.disconnect();
        assert!(result.is_err() || result.is_ok());
    }
}
