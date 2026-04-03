//! VPN connect/disconnect and refresh operations

use crate::state::ConnectionState;
use crate::state::NotificationType;

impl crate::state::AppState {
    pub fn refresh_servers(&mut self) {
        tracing::info!("Refreshing server list");
        let cached = self.vpn_state.servers();
        if !cached.is_empty() {
            self.set_servers(cached);
        }

        self.show_notification(
            "Refreshing servers...".to_string(),
            NotificationType::Info,
            Some("servers".to_string()),
        );

        self.connection_manager.async_manager.spawn_refresh_servers(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect(&mut self) {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
                None,
            );
            return;
        }
        let Some(idx) = self.ui_state.selected_server else {
            self.show_notification(
                "No server selected".to_string(),
                NotificationType::Error,
                None,
            );
            return;
        };

        let filtered = self.filtered_servers();
        let server = match filtered.get(idx) {
            Some(server) => server,
            None => {
                self.show_notification(
                    "No server selected".to_string(),
                    NotificationType::Error,
                    None,
                );
                return;
            }
        };
        let server_id = server.code.clone();
        let server_country = server.country.clone();

        tracing::info!("Connecting to server: {}", server_id);
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            format!("Connecting to {}...", server_country),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect(
            self.vpn_state.clone(),
            server_id,
            self.connection_manager.async_notifier.clone(),
        );
    }

    /// Connect to a special server type (random, fastest, p2p, tor, securecore)
    fn connect_special<F>(&mut self, log_msg: &str, notification_msg: &str, spawn_fn: F)
    where
        F: FnOnce(
            &crate::vpn::AsyncTaskManager,
            std::sync::Arc<crate::vpn::VpnClient>,
            std::sync::Arc<crate::state::AsyncNotifier>,
        ),
    {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
                None,
            );
            return;
        }

        tracing::info!("{}", log_msg);
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            notification_msg.to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        let vpn_state = self.vpn_state.clone();
        let notifier = self.connection_manager.async_notifier.clone();
        spawn_fn(&self.connection_manager.async_manager, vpn_state, notifier);
    }

    pub fn connect_random(&mut self) {
        self.connect_special(
            "Connecting to random server",
            "Connecting to random server...",
            |am, vpn, n| am.spawn_connect_random(vpn, n),
        );
    }

    pub fn connect_fastest(&mut self) {
        self.connect_special(
            "Connecting to fastest server",
            "Connecting to fastest server...",
            |am, vpn, n| am.spawn_connect_fastest(vpn, n),
        );
    }

    pub fn connect_p2p(&mut self) {
        self.connect_special(
            "Connecting to P2P server",
            "Connecting to P2P server...",
            |am, vpn, n| am.spawn_connect_p2p(vpn, n),
        );
    }

    pub fn connect_tor(&mut self) {
        self.connect_special(
            "Connecting to Tor server",
            "Connecting to Tor server...",
            |am, vpn, n| am.spawn_connect_tor(vpn, n),
        );
    }

    pub fn connect_securecore(&mut self) {
        self.connect_special(
            "Connecting to SecureCore server",
            "Connecting to SecureCore server...",
            |am, vpn, n| am.spawn_connect_securecore(vpn, n),
        );
    }

    pub fn disconnect(&mut self) {
        if self.connection_manager.connection.is_disconnected() {
            return;
        }

        let server_info = match &self.connection_manager.connection {
            ConnectionState::Connected { server, .. } => server.clone(),
            _ => String::from("VPN"),
        };

        tracing::info!("Disconnecting from {}", server_info);
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Disconnecting;
        self.show_notification(
            format!("Disconnecting from {}...", server_info),
            NotificationType::Info,
            Some("disconnect".to_string()),
        );

        self.connection_manager.async_manager.spawn_disconnect(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect_city(&mut self, city: &str) {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
                None,
            );
            return;
        }

        let city = city.to_string();
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            format!("Connecting to {}...", city),
            NotificationType::Info,
            Some("connect:city".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect_city(
            self.vpn_state.clone(),
            city,
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn spawn_config_set(&mut self, key: String, value: String) {
        self.connection_manager.async_manager.spawn_config_set(
            self.vpn_state.clone(),
            key,
            value,
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn sync_connection_from_vpn(&mut self) {
        if self.vpn_state.is_connected() {
            if let Some((server, ip)) = self.vpn_state.get_connected_server_info() {
                let ip_clone = ip.clone();
                self.connection_manager.connection = ConnectionState::Connected {
                    server: server.clone(),
                    ip,
                    city: None,
                    country: None,
                    via: None,
                    load: None,
                };
                self.vpn_state.update_connected_at(&server, &ip_clone);
                tracing::info!("Synced connection from VPN: {}", server);
            }
        }
    }
}
