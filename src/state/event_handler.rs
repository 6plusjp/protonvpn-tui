//! Async event processing and handling

use crate::state::AsyncEvent;
use crate::state::ConnectionState;
use crate::state::NotificationType;
use crate::paths;
use crate::ui::{notify_connect_failed, notify_connected, notify_disconnected};
use crate::vpn::torrent_sync;

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
                AsyncEvent::Connected(result, needs_refresh) => {
                    self.show_notification(
                        format!("Connected to {}", &result.server),
                        NotificationType::Success,
                        Some("connect".to_string()),
                    );
                    if self.user_config.ui.system_notifications {
                        notify_connected(
                            &result.server,
                            result.city.as_deref(),
                            result.via.as_deref(),
                        );
                    }
                    tracing::info!("Successfully connected to server: {}", result.server);
                    self.connection_manager.connection = ConnectionState::Connected {
                        server: result.server,
                        ip: result.ip.unwrap_or_default(),
                        city: result.city,
                        country: result.country,
                        via: result.via,
                        load: None,
                    };
                    self.connection_manager.previous_connection = None;
                    notification_shown = true;

                    std::thread::spawn({
                        let vpn_state = self.vpn_state.clone();
                        move || {
                            let port = paths::proton_forwarded_port_path()
                                .and_then(|p| std::fs::read_to_string(p).ok())
                                .and_then(|c| c.trim().parse().ok());
                            if let Some(p) = port {
                                torrent_sync::sync_forwarded_port(p);
                                vpn_state.set_forwarded_port(Some(p));
                            }
                        }
                    });

                    if needs_refresh {
                        tracing::info!("CLI server list was outdated, triggering refresh");
                        self.refresh_servers();
                    }
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
                    if self.user_config.ui.system_notifications {
                        notify_connect_failed(&e);
                    }
                    notification_shown = true;
                }
                AsyncEvent::Disconnected => {
                    self.show_notification(
                        "Disconnected".to_string(),
                        NotificationType::Info,
                        Some("disconnect".to_string()),
                    );
                    if self.user_config.ui.system_notifications {
                        notify_disconnected();
                    }
                    tracing::info!("Disconnected from VPN");
                    self.connection_manager.connection = ConnectionState::Disconnected;
                    self.connection_manager.previous_connection = None;
                    self.vpn_state.set_forwarded_port(None);
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
                AsyncEvent::CitiesLoaded(country_code, cities, needs_refresh) => {
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

                    if needs_refresh {
                        tracing::info!("CLI server list was outdated, triggering refresh");
                        self.refresh_servers();
                    }
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
                AsyncEvent::ConnectCityResult(result, needs_refresh) => {
                    self.show_notification(
                        format!("Connected to {}", result.server),
                        NotificationType::Success,
                        Some("connect:city".to_string()),
                    );
                    if self.user_config.ui.system_notifications {
                        notify_connected(
                            &result.server,
                            result.city.as_deref(),
                            result.via.as_deref(),
                        );
                    }
                    tracing::info!(
                        "Successfully connected to server (connect_city): {}",
                        result.server
                    );
                    self.connection_manager.connection = ConnectionState::Connected {
                        server: result.server,
                        ip: result.ip.unwrap_or_default(),
                        city: result.city,
                        country: result.country,
                        via: result.via,
                        load: None,
                    };
                    self.connection_manager.previous_connection = None;
                    notification_shown = true;

                    if needs_refresh {
                        tracing::info!("CLI server list was outdated, triggering refresh");
                        self.refresh_servers();
                    }
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
                    if self.user_config.ui.system_notifications {
                        notify_connect_failed(&e);
                    }
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
                                .map(|(_, _, ip)| ip)
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
            let (server, server_id, ip, city, country, load) =
                match self.vpn_state.get_status_info() {
                    Some(status) => {
                        let server = status.server.unwrap_or_else(|| "Unknown".to_string());
                        let server_id = self
                            .vpn_state
                            .get_connected_server_info()
                            .map(|(sid, _, _)| sid)
                            .unwrap_or_default();
                        let ip = self
                            .vpn_state
                            .get_connected_server_info()
                            .map(|(_, _, ip)| ip)
                            .unwrap_or_default();

                        if let Some(uptime) = status.uptime {
                            self.vpn_state.adjust_connected_at_from_uptime(uptime);
                        }

                        (
                            server,
                            server_id,
                            ip,
                            status.city,
                            status.country,
                            status.load,
                        )
                    }
                    None => {
                        let server = String::new();
                        let server_id = String::new();
                        let ip = String::new();
                        (server, server_id, ip, None, None, None)
                    }
                };

            self.vpn_state
                .sync_cache_with_connection(&server, &server_id, &ip);

            self.connection_manager.connection = ConnectionState::Connected {
                server,
                ip,
                city,
                country,
                via: None,
                load,
            };

            let cached_server_id = self.vpn_state.get_connected_server_id();
            if cached_server_id.as_ref() == Some(&server_id) {
                if let Some(port) = self.vpn_state.get_cached_forwarded_port() {
                    tracing::debug!(
                        "Restored forwarded port {} for reconnected server {}",
                        port,
                        server_id
                    );
                }
            } else {
                self.vpn_state.set_forwarded_port(None);
                std::thread::spawn({
                    let vpn_state = self.vpn_state.clone();
                    move || {
                        let port = paths::proton_forwarded_port_path()
                            .and_then(|p| std::fs::read_to_string(p).ok())
                            .and_then(|c| c.trim().parse().ok());
                        if let Some(p) = port {
                            torrent_sync::sync_forwarded_port(p);
                            vpn_state.set_forwarded_port(Some(p));
                        }
                    }
                });
            }

            return true;
        }
        false
    }
}
