//! Main application state

use crate::config::Settings;
use crate::vpn::Server;
use crate::vpn::VpnState;
use super::{AppView, ConnectionState};

/// Notification popup
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Error,
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current connection state
    pub connection: ConnectionState,
    /// Current UI view
    pub current_view: AppView,
    /// Search query for server filtering
    pub search_query: String,
    /// Application configuration
    pub config: Settings,
    /// Server list
    pub servers: Vec<Server>,
    /// Selected server index
    pub selected_server: Option<usize>,
    /// VPN state
    pub vpn_state: VpnState,
    /// Notification popup message
    pub notification: Option<Notification>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            current_view: AppView::Connect,
            search_query: String::new(),
            config: Settings::default(),
            servers: Vec::new(),
            selected_server: None,
            vpn_state: VpnState::new(),
            notification: None,
        }
    }

    pub fn with_config(mut self, config: Settings) -> Self {
        self.config = config;
        self
    }

    pub fn switch_view(&mut self) {
        self.current_view = self.current_view.next();
    }

    /// Show a notification popup
    pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notification = Some(Notification {
            message,
            notification_type,
        });
    }

    /// Clear notification
    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    /// Sync connection state from system (check proton0 interface)
    pub fn sync_connection_state(&mut self) {
        if self.vpn_state.is_connected() {
            // Get VPN IP from proton0
            let ip = self.vpn_state.get_vpn_ip();
            match ip {
                Some(current_ip) => {
                    // Check if IP matches cached connection
                    let server = if self.vpn_state.matches_ip(&current_ip) {
                        self.vpn_state.get_connected_server().unwrap_or_else(|| "Unknown".to_string())
                    } else {
                        // IP doesn't match - connection was made outside our TUI
                        "Unknown".to_string()
                    };
                    self.connection = ConnectionState::Connected { server, ip: current_ip };
                }
                None => {
                    // Can't get IP but interface exists
                    self.connection = ConnectionState::Connected {
                        server: "Unknown".to_string(),
                        ip: String::new(),
                    };
                }
            }
        } else {
            self.connection = ConnectionState::Disconnected;
        }
    }

    /// Refresh server list
    pub fn refresh_servers(&mut self) {
        match self.vpn_state.list_servers() {
            Ok(servers) => {
                self.servers = servers;
                self.show_notification(
                    format!("Loaded {} servers", self.servers.len()),
                    NotificationType::Success,
                );
            }
            Err(e) => {
                self.show_notification(format!("Failed: {}", e), NotificationType::Error);
            }
        }
    }

    /// Connect to selected server
    pub fn connect(&mut self) {
        if let Some(idx) = self.selected_server {
            if let Some(server) = self.servers.get(idx) {
                match self.vpn_state.connect(&server.id) {
                    Ok(()) => {
                        self.connection = ConnectionState::Connected {
                            server: server.id.clone(),
                            ip: String::new(),
                        };
                        self.show_notification(
                            format!("Connected to {}", server.name),
                            NotificationType::Success,
                        );
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Connection failed: {}", e),
                            NotificationType::Error,
                        );
                    }
                }
                self.show_notification("No server selected".to_string(), NotificationType::Error);
            }
        } else {
            self.show_notification("No server selected".to_string(), NotificationType::Error);
        }
    }

    /// Disconnect from VPN
    pub fn disconnect(&mut self) {
        match self.vpn_state.disconnect() {
            Ok(()) => {
                self.connection = ConnectionState::Disconnected;
                self.show_notification("Disconnected".to_string(), NotificationType::Info);
            }
            Err(e) => {
                self.show_notification(
                    format!("Disconnect failed: {}", e),
                    NotificationType::Error,
                );
            }
    }
    }
    /// Get filtered servers based on search query
    pub fn filtered_servers(&self) -> Vec<&Server> {
        if self.search_query.is_empty() {
            self.servers.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.servers
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&query)
                        || s.country.to_lowercase().contains(&query)
                        || s.city.to_lowercase().contains(&query)
                })
                .collect()
        }
    }

    /// Select next server
    pub fn select_next(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }

        match self.selected_server {
            Some(idx) => {
                self.selected_server = Some((idx + 1).min(filtered.len() - 1));
            }
            None => {
                self.selected_server = Some(0);
            }
        }
    }

    /// Select previous server
    pub fn select_prev(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }

        match self.selected_server {
            Some(idx) => {
                self.selected_server = Some(idx.saturating_sub(1));
            }
            None => {
                self.selected_server = Some(filtered.len() - 1);
            }
        }
    }
}
