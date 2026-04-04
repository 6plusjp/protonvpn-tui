//! Async event processing and handling

use crate::state::AsyncEvent;
use crate::state::ConnectionState;
use crate::state::NotificationType;

impl crate::state::AppState {
    pub fn wait_for_async_events(&mut self, timeout: std::time::Duration) -> bool {
        let events = self.connection_manager.async_notifier.wait_timeout(timeout);
        if !events.is_empty() {
            self.handle_async_events(events);
            return true;
        }
        self.sync_connection_state()
    }

    pub fn process_async_events(&mut self) -> bool {
        let events = self.connection_manager.async_notifier.try_recv_all();
        if events.is_empty() {
            return false;
        }
        self.handle_async_events(events)
    }

    fn handle_async_events(&mut self, events: Vec<AsyncEvent>) -> bool {
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
                        load: None,
                    };
                    self.connection_manager.previous_connection = None;
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
                        format!("Connection failed: {}", e),
                        NotificationType::Error,
                        Some("connect".to_string()),
                    );
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
                    notification_shown = true;
                }
                AsyncEvent::CitiesLoaded(country_code, cities) => {
                    let city_count = cities.len();

                    if self.current_country_code.as_deref() == Some(&country_code) {
                        self.current_cities.clear();
                        self.current_cities = cities.clone();
                    }

                    for server in self.servers.iter_mut() {
                        if server.code == country_code {
                            server.cities = cities.clone();
                            break;
                        }
                    }
                    self.server_cache.invalidate();

                    self.connection_manager.loading_cities.remove(&country_code);
                    self.show_notification(
                        format!("Loaded {} cities for {}", city_count, country_code),
                        NotificationType::Success,
                        Some(format!("cities:{}", country_code)),
                    );
                    notification_shown = true;
                }
                AsyncEvent::CitiesLoadFailed(country_code, e) => {
                    self.connection_manager.loading_cities.remove(&country_code);
                    self.show_notification(
                        format!("Failed to load cities: {}", e),
                        NotificationType::Error,
                        Some(format!("cities:{}", country_code)),
                    );
                    notification_shown = true;
                }
                AsyncEvent::ConnectCityResult(result) => {
                    self.show_notification(
                        format!("Connected to {}", result.server_id),
                        NotificationType::Success,
                        Some("connect:city".to_string()),
                    );
                    tracing::info!(
                        "Successfully connected to server (connect_city): {}",
                        result.server_id
                    );
                    self.connection_manager.connection = ConnectionState::Connected {
                        server: result.server_id,
                        ip: result.ip.unwrap_or_default(),
                        city: result.city,
                        country: result.country,
                        via: result.via,
                        load: None,
                    };
                    self.connection_manager.previous_connection = None;
                    notification_shown = true;
                }
                AsyncEvent::ConnectCityFailed(e) => {
                    if let Some(prev) = self.connection_manager.previous_connection.take() {
                        self.connection_manager.connection = prev;
                    } else {
                        self.connection_manager.connection = ConnectionState::Disconnected;
                    }
                    tracing::warn!("Connection failed (connect_city): {}", e);
                    self.show_notification(
                        format!("Connection failed: {}", e),
                        NotificationType::Error,
                        Some("connect:city".to_string()),
                    );
                    notification_shown = true;
                }
                AsyncEvent::ConfigSetResult(msg) => {
                    self.show_notification(
                        format!("Setting updated: {}", msg),
                        NotificationType::Success,
                        None,
                    );
                    self.clear_settings_cache();
                    notification_shown = true;
                }
                AsyncEvent::ConfigSetFailed(e) => {
                    tracing::warn!("Config set failed: {}", e);
                    self.show_notification(
                        format!("Failed to update setting: {}", e),
                        NotificationType::Error,
                        None,
                    );
                    notification_shown = true;
                }
                AsyncEvent::StatusInfoLoaded(status) => {
                    if let ConnectionState::Connected { .. } = &self.connection_manager.connection {
                        if let Some(server) = status.server {
                            let server_ip = self
                                .vpn_state
                                .get_connected_server_info()
                                .map(|(_, ip)| ip)
                                .unwrap_or_default();

                            self.connection_manager.connection = ConnectionState::Connected {
                                server: server.clone(),
                                ip: server_ip,
                                city: status.city,
                                country: status.country,
                                via: None,
                                load: status.load,
                            };

                            if let Some(uptime) = status.uptime {
                                self.vpn_state.adjust_connected_at_from_uptime(uptime);
                            }

                            tracing::info!("Updated connection state from status info: {}", server);
                        }
                    }
                }
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
            let (server, ip, city, country, load) = match self.vpn_state.get_status_info() {
                Some(status) => {
                    let server = status.server.unwrap_or_else(|| "Unknown".to_string());
                    // Get IP from persistence file (status doesn't include IP in all versions)
                    let ip = self
                        .vpn_state
                        .get_connected_server_info()
                        .map(|(_, ip)| ip)
                        .unwrap_or_default();

                    // Adjust connected_at from uptime if available
                    if let Some(uptime) = status.uptime {
                        self.vpn_state.adjust_connected_at_from_uptime(uptime);
                    }

                    (server, ip, status.city, status.country, status.load)
                }
                None => {
                    let (server, ip) = self
                        .vpn_state
                        .get_connected_server_info()
                        .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
                    (server, ip, None, None, None)
                }
            };

            // Sync cache with actual connection info (e.g., after reboot with auto-connect)
            self.vpn_state.sync_cache_with_connection(&server, &ip);

            self.connection_manager.connection = ConnectionState::Connected {
                server,
                ip,
                city,
                country,
                via: None,
                load,
            };
            return true;
        }
        false
    }
}
