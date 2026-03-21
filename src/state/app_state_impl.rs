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

    pub fn connect_random(&mut self) {
        tracing::info!("Connecting to random server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to random server...".to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect_random(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect_fastest(&mut self) {
        tracing::info!("Connecting to fastest server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to fastest server...".to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect_fastest(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect_p2p(&mut self) {
        tracing::info!("Connecting to P2P server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to P2P server...".to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect_p2p(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect_tor(&mut self) {
        tracing::info!("Connecting to Tor server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to Tor server...".to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager.async_manager.spawn_connect_tor(
            self.vpn_state.clone(),
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn connect_securecore(&mut self) {
        tracing::info!("Connecting to SecureCore server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to SecureCore server...".to_string(),
            NotificationType::Info,
            Some("connect".to_string()),
        );

        self.connection_manager
            .async_manager
            .spawn_connect_securecore(
                self.vpn_state.clone(),
                self.connection_manager.async_notifier.clone(),
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
}
