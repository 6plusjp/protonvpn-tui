//! Async event processing and handling

use crate::error::categorize_error;
use crate::state::AsyncEvent;
use crate::state::ConnectionState;
use crate::state::NotificationType;

impl crate::state::AppState {
    pub fn wait_for_async_events(&mut self, timeout: std::time::Duration) -> bool {
        let events = self.connection_manager.async_notifier.wait_timeout(timeout);
        if events.is_empty() {
            return false;
        }
        self.process_async_events()
    }

    pub fn process_async_events(&mut self) -> bool {
        let events = self.connection_manager.async_notifier.try_recv_all();
        if events.is_empty() {
            return false;
        }
        let mut notification_shown = false;
        for event in events {
            match event {
                AsyncEvent::ServersRefreshed(servers) => {
                    self.set_servers(servers);
                    tracing::info!("Server list refreshed: {} servers", self.servers.len());
                    self.is_initialized = true;
                    if self.vpn_state.is_cli_unavailable() {
                        self.show_notification(
                            "ProtonVPN CLI unavailable. VPN functionality disabled.".to_string(),
                            NotificationType::Error,
                            Some("servers".to_string()),
                        );
                    } else {
                        self.show_notification(
                            format!("Refreshed {} servers", self.servers.len()),
                            NotificationType::Success,
                            Some("servers".to_string()),
                        );
                    }
                    notification_shown = true;
                    self.connection_manager.pending_refresh.remove(&());
                }
                AsyncEvent::ServersRefreshFailed(e) => {
                    tracing::warn!("Server list refresh failed: {}", e);
                    self.is_initialized = true;
                    self.show_notification(
                        format!("Refresh failed: {}", e),
                        NotificationType::Error,
                        Some("servers".to_string()),
                    );
                    notification_shown = true;
                    self.connection_manager.pending_refresh.remove(&());
                }
                AsyncEvent::Connected(result) => {
                    self.show_notification(
                        format!("Connected to {}", &result.server_id),
                        NotificationType::Success,
                        Some("connect".to_string()),
                    );
                    tracing::info!("Successfully connected to server: {}", result.server_id);
                    self.connection_manager.connection = ConnectionState::Connected {
                        server: result.server_id,
                        ip: result.ip.unwrap_or_default(),
                        city: result.city,
                        country: result.country,
                        via: result.via,
                    };
                    self.connection_manager.previous_connection = None;
                    self.connection_manager.pending_connect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::ConnectFailed(e) => {
                    if let Some(prev) = self.connection_manager.previous_connection.take() {
                        self.connection_manager.connection = prev;
                    } else {
                        self.connection_manager.connection = ConnectionState::Disconnected;
                    }
                    tracing::warn!("Connection failed: {}", e);
                    self.show_notification(
                        format!("Connection failed ({}):", e),
                        NotificationType::Error,
                        Some("connect".to_string()),
                    );
                    self.connection_manager.pending_connect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::Disconnected => {
                    self.show_notification(
                        "Disconnected".to_string(),
                        NotificationType::Info,
                        Some("disconnect".to_string()),
                    );
                    tracing::info!("Disconnected from VPN");
                    self.connection_manager.connection = ConnectionState::Disconnected;
                    self.connection_manager.previous_connection = None;
                    self.connection_manager.pending_disconnect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::DisconnectFailed(e) => {
                    tracing::warn!("Disconnect failed: {}", e);
                    self.show_notification(
                        format!("Disconnect failed: {}", e),
                        NotificationType::Error,
                        Some("disconnect".to_string()),
                    );
                    if let Some(prev) = self.connection_manager.previous_connection.take() {
                        self.connection_manager.connection = prev;
                    }
                    self.connection_manager.pending_disconnect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::CitiesLoaded(_, _) => {}
                AsyncEvent::ConnectCityResult(result) => {
                    self.show_notification(
                        format!("Connected to {}", result.server_id),
                        NotificationType::Success,
                        Some("connect:city".to_string()),
                    );
                    self.connection_manager.connection = ConnectionState::Connected {
                        server: result.server_id,
                        ip: result.ip.unwrap_or_default(),
                        city: result.city,
                        country: result.country,
                        via: result.via,
                    };
                    self.connection_manager.previous_connection = None;
                    self.connection_manager.pending_connect_city.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::ConnectCityFailed(e) => {
                    if let Some(prev) = self.connection_manager.previous_connection.take() {
                        self.connection_manager.connection = prev;
                    } else {
                        self.connection_manager.connection = ConnectionState::Disconnected;
                    }
                    self.show_notification(
                        format!("Connection failed ({}):", e),
                        NotificationType::Error,
                        Some("connect:city".to_string()),
                    );
                    self.connection_manager.pending_connect_city.remove(&());
                    notification_shown = true;
                }
            }
        }
        notification_shown
    }

    pub fn check_pending_async_events(&mut self) -> bool {
        let mut notification_shown = false;
        notification_shown |= self.process_async_events();
        notification_shown |= self.check_pending_refresh();
        notification_shown |= self.check_pending_connect();
        notification_shown |= self.check_pending_disconnect();
        notification_shown |= self.check_pending_cities();
        notification_shown |= self.check_pending_connect_city();
        notification_shown |= self.check_pending_config_set();
        notification_shown |= self.sync_connection_state();
        notification_shown
    }

    fn check_pending_refresh(&mut self) -> bool {
        let mut notification_shown = false;
        if let Some(rx) = self.connection_manager.pending_refresh.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(servers) => {
                        self.set_servers(servers);
                        tracing::info!("Server list refreshed: {} servers", self.servers.len());
                        self.is_initialized = true;

                        if self.vpn_state.is_cli_unavailable() {
                            self.show_notification(
                                "ProtonVPN CLI unavailable. VPN functionality disabled."
                                    .to_string(),
                                NotificationType::Error,
                                Some("servers".to_string()),
                            );
                        } else {
                            self.show_notification(
                                format!("Refreshed {} servers", self.servers.len()),
                                NotificationType::Success,
                                Some("servers".to_string()),
                            );
                        }
                        notification_shown = true;
                    }
                    Err(e) => {
                        tracing::warn!("Server list refresh failed: {}", e);
                        self.is_initialized = true;
                        self.show_notification(
                            format!("Refresh failed: {}", e),
                            NotificationType::Error,
                            Some("servers".to_string()),
                        );
                        notification_shown = true;
                    }
                }
                self.connection_manager.pending_refresh.remove(&());
            }
        }
        notification_shown
    }

    fn check_pending_connect(&mut self) -> bool {
        if !self.connection_manager.connection.is_connecting() {
            return false;
        }
        if let Some(rx) = self.connection_manager.pending_connect.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(conn_result) => {
                        self.show_notification(
                            format!("Connected to {}", &conn_result.server_id),
                            NotificationType::Success,
                            Some("connect".to_string()),
                        );
                        tracing::info!(
                            "Successfully connected to server: {}",
                            conn_result.server_id
                        );
                        self.connection_manager.connection = ConnectionState::Connected {
                            server: conn_result.server_id,
                            ip: conn_result.ip.unwrap_or_default(),
                            city: conn_result.city,
                            country: conn_result.country,
                            via: conn_result.via,
                        };
                        self.connection_manager.previous_connection = None;
                        self.connection_manager.pending_connect.remove(&());
                        return true;
                    }
                    Err(e) => {
                        if let Some(prev) = self.connection_manager.previous_connection.take() {
                            self.connection_manager.connection = prev;
                        } else {
                            self.connection_manager.connection = ConnectionState::Disconnected;
                        }
                        tracing::warn!("Connection failed: {}", e);
                        self.show_notification(
                            format!("Connection failed ({}):", categorize_error(&e)),
                            NotificationType::Error,
                            Some("connect".to_string()),
                        );
                        self.connection_manager.pending_connect.remove(&());
                        return true;
                    }
                }
            }
        }
        false
    }

    fn check_pending_disconnect(&mut self) -> bool {
        let mut notification_shown = false;
        if !self.connection_manager.connection.is_disconnecting() {
            return false;
        }
        if let Some(rx) = self.connection_manager.pending_disconnect.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                let server_info = match &self.connection_manager.connection {
                    ConnectionState::Connecting => Some("unknown server".to_string()),
                    ConnectionState::Connected { server, .. } => Some(server.clone()),
                    _ => None,
                };
                match result {
                    Ok(()) => {
                        self.connection_manager.connection = ConnectionState::Disconnected;
                        tracing::info!("Successfully disconnected from VPN");
                        let msg = server_info
                            .map(|s| format!("Disconnected from {}", s))
                            .unwrap_or_else(|| "Disconnected".to_string());
                        self.show_notification(
                            msg,
                            NotificationType::Info,
                            Some("disconnect".to_string()),
                        );
                        notification_shown = true;
                        self.connection_manager.previous_connection = None;
                        self.connection_manager.pending_disconnect.remove(&());
                    }
                    Err(e) => {
                        if let Some(prev) = self.connection_manager.previous_connection.take() {
                            self.connection_manager.connection = prev;
                        } else {
                            self.connection_manager.connection = ConnectionState::Disconnected;
                        }
                        tracing::warn!("Disconnect failed: {}", e);
                        self.show_notification(
                            format!("Disconnect failed ({}):", categorize_error(&e)),
                            NotificationType::Error,
                            Some("disconnect".to_string()),
                        );
                        notification_shown = true;
                        self.connection_manager.pending_disconnect.remove(&());
                    }
                }
            }
        }
        notification_shown
    }

    fn check_pending_cities(&mut self) -> bool {
        let mut notification_shown = false;
        let mut results_to_process = Vec::new();
        for (country_code, rx) in self.connection_manager.pending_cities.iter_mut() {
            if let Ok(result) = rx.try_recv() {
                results_to_process.push((country_code.clone(), result));
            }
        }

        for (country_code, result) in results_to_process {
            match result {
                Ok(cities) => {
                    let city_count = cities.len();
                    if self.current_country_code.as_deref() == Some(&country_code) {
                        self.current_cities.clear();
                        self.current_cities = cities;
                    }
                    self.show_notification(
                        format!("Loaded {} cities for {}", city_count, country_code),
                        NotificationType::Success,
                        Some(format!("cities:{}", country_code)),
                    );
                    notification_shown = true;
                }
                Err(e) => {
                    self.show_notification(
                        format!("Failed to load cities ({}):", categorize_error(&e)),
                        NotificationType::Error,
                        Some(format!("cities:{}", country_code)),
                    );
                    notification_shown = true;
                }
            }
            self.connection_manager.pending_cities.remove(&country_code);
        }

        if !self.connection_manager.pending_cities.is_empty() {
            self.server_cache.invalidate();
        }
        notification_shown
    }

    fn check_pending_connect_city(&mut self) -> bool {
        let mut notification_shown = false;
        if let Some(rx) = self.connection_manager.pending_connect_city.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(conn_result) => {
                        self.show_notification(
                            format!("Connected to {}", &conn_result.server_id),
                            NotificationType::Success,
                            Some("connect:city".to_string()),
                        );
                        notification_shown = true;
                        tracing::info!(
                            "Successfully connected to server (connect_city): {}",
                            conn_result.server_id
                        );
                        self.connection_manager.connection = ConnectionState::Connected {
                            server: conn_result.server_id,
                            ip: conn_result.ip.unwrap_or_default(),
                            city: conn_result.city,
                            country: conn_result.country,
                            via: conn_result.via,
                        };
                        self.connection_manager.previous_connection = None;
                        self.connection_manager.pending_connect_city.remove(&());
                    }
                    Err(e) => {
                        if let Some(prev) = self.connection_manager.previous_connection.take() {
                            self.connection_manager.connection = prev;
                        } else {
                            self.connection_manager.connection = ConnectionState::Disconnected;
                        }
                        tracing::warn!("Connection failed (connect_city): {}", e);
                        self.show_notification(
                            format!("Connection failed ({}):", e),
                            NotificationType::Error,
                            Some("connect:city".to_string()),
                        );
                        notification_shown = true;
                        self.connection_manager.pending_connect_city.remove(&());
                    }
                }
            }
        }
        notification_shown
    }

    fn check_pending_config_set(&mut self) -> bool {
        let notification_shown = false;
        if let Some(rx) = self.connection_manager.pending_config_set.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(msg) => {
                        self.show_notification(
                            format!("Setting updated: {}", msg),
                            NotificationType::Success,
                            None,
                        );
                        self.clear_settings_cache();
                    }
                    Err(e) => {
                        tracing::warn!("Config set failed: {}", e);
                        self.show_notification(
                            format!("Failed to update setting ({}):", categorize_error(&e)),
                            NotificationType::Error,
                            None,
                        );
                    }
                }
                self.connection_manager.pending_config_set.remove(&());
            }
        }
        notification_shown
    }

    fn sync_connection_state(&mut self) -> bool {
        if self.connection_manager.connection.is_connected() {
            return false;
        }
        if self.connection_manager.connection == ConnectionState::Disconnected
            && self.vpn_state.is_connected()
        {
            let (server, ip) = self
                .vpn_state
                .get_connected_server_info()
                .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
            self.connection_manager.connection = ConnectionState::Connected {
                server,
                ip,
                city: None,
                country: None,
                via: None,
            };
            return false;
        }
        false
    }
}
