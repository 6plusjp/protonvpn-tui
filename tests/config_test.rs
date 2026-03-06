use protonvpn_tui::config::{ProtonCustomDns, ProtonDnsIp, ProtonFeatures, ProtonSettings};

mod proton_settings {
    use super::*;

    #[test]
    fn test_proton_settings_default() {
        let settings = ProtonSettings::default();
        assert!(settings.protocol.is_none());
        assert!(settings.killswitch.is_none());
    }

    #[test]
    fn test_proton_settings_count_empty() {
        let settings = ProtonSettings::default();
        assert_eq!(settings.settings_count(), 0);
    }

    #[test]
    fn test_proton_settings_count_killswitch() {
        let settings = ProtonSettings {
            killswitch: Some(1),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_ipv6() {
        let settings = ProtonSettings {
            ipv6: Some(true),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_custom_dns_enabled() {
        let settings = ProtonSettings {
            custom_dns: ProtonCustomDns {
                enabled: true,
                ip_list: vec![ProtonDnsIp {
                    ip: "1.1.1.1".to_string(),
                    enabled: true,
                }],
            },
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_custom_dns_disabled() {
        let settings = ProtonSettings {
            custom_dns: ProtonCustomDns {
                enabled: false,
                ip_list: vec![ProtonDnsIp {
                    ip: "1.1.1.1".to_string(),
                    enabled: true,
                }],
            },
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 0);
    }

    #[test]
    fn test_proton_settings_count_netshield() {
        let settings = ProtonSettings {
            features: Some(ProtonFeatures {
                netshield: Some(1),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_moderate_nat() {
        let settings = ProtonSettings {
            features: Some(ProtonFeatures {
                moderate_nat: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_vpn_accelerator() {
        let settings = ProtonSettings {
            features: Some(ProtonFeatures {
                vpn_accelerator: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_port_forwarding() {
        let settings = ProtonSettings {
            features: Some(ProtonFeatures {
                port_forwarding: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(settings.settings_count(), 1);
    }

    #[test]
    fn test_proton_settings_count_multiple() {
        let settings = ProtonSettings {
            protocol: None,
            killswitch: Some(1),
            ipv6: Some(true),
            custom_dns: ProtonCustomDns {
                enabled: true,
                ip_list: vec![],
            },
            features: Some(ProtonFeatures {
                netshield: Some(1),
                moderate_nat: Some(true),
                vpn_accelerator: Some(true),
                port_forwarding: Some(true),
                split_tunneling: None,
            }),
        };
        assert_eq!(settings.settings_count(), 7);
    }
}
