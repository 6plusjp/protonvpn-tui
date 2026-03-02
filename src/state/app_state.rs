//! Application state management

use crate::config::Settings;
use crate::state::ConnectionState;
use crate::state::ServerFilter;
use crate::vpn::Server;
use crate::vpn::VpnState;
use std::collections::HashMap;

/// Notification type for UI feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Error,
}

/// Notification popup
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
}

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub connection: ConnectionState,
    pub current_view: crate::state::AppView,
    pub search_query: String,
    pub filter: ServerFilter,
    pub config: Settings,
    pub servers: Vec<Server>,
    pub selected_server: Option<usize>,
    pub vpn_state: VpnState,
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
            current_view: crate::state::AppView::Connect,
            search_query: String::new(),
            filter: ServerFilter::default(),
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

    pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notification = Some(Notification {
            message,
            notification_type,
        });
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    pub fn sync_connection_state(&mut self) {
        if self.vpn_state.is_connected() {
            // On startup, don't trust cache - show as unknown
            // User must explicitly connect to get server info
            self.connection = ConnectionState::Connected {
                server: "Unknown".to_string(),
                ip: String::new(),
            };
        } else {
            self.connection = ConnectionState::Disconnected;
        }
    }

    pub fn load_cached_servers(&mut self) {
        match self.vpn_state.get_servers() {
            Ok(servers) => {
                if !servers.is_empty() {
                    self.servers = servers;
                }
            }
            Err(_) => {}
        }
    }

    pub fn refresh_servers(&mut self) {
        // First load from cache for immediate display
        if let Ok(cached) = self.vpn_state.get_servers() {
            if !cached.is_empty() {
                self.servers = cached;
            }
        }

        // Then refresh from CLI
        match self.vpn_state.refresh_servers() {
            Ok(servers) => {
                self.servers = servers;
                self.show_notification(
                    format!("Refreshed {} servers", self.servers.len()),
                    NotificationType::Success,
                );
            }
            Err(e) => {
                if self.servers.is_empty() {
                    self.show_notification(format!("Failed: {}", e), NotificationType::Error);
                }
            }
        }
    }


    pub fn connect(&mut self) {
        let Some(idx) = self.selected_server else {
            self.show_notification("No server selected".to_string(), NotificationType::Error);
            return;
        };
        let Some(server) = self.servers.get(idx) else {
            self.show_notification("No server selected".to_string(), NotificationType::Error);
            return;
        };
        match self.vpn_state.connect(&server.id) {
            Ok((server_id, ip)) => {
                self.connection = ConnectionState::Connected {
                    server: server_id,
                    ip: ip.unwrap_or_default(),
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
    }

    pub fn connect_random(&mut self) {
        use std::time::SystemTime;
        let servers = self.filtered_servers();
        if servers.is_empty() {
            self.show_notification("No servers available".to_string(), NotificationType::Error);
            return;
        }
        let idx = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as usize)
            % servers.len();
        if let Some(server) = servers.get(idx) {
            let server_id = server.id.clone();
            let _server_name = server.name.clone();
            match self.vpn_state.connect(&server_id) {
                Ok((server_id, ip)) => {
                    let ip_str = ip.unwrap_or_default();
                    self.connection = ConnectionState::Connected {
                        server: server_id.clone(),
                        ip: ip_str.clone(),
                    };
                    self.show_notification(
                        format!("Connected to {} ({})", server_id, ip_str),
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
        }
    }

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

    pub fn filtered_servers(&self) -> Vec<&Server> {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            return self.servers.iter().collect();
        }

        let query_lower = query.to_lowercase();

        self.servers
            .iter()
            .filter(|server| match self.filter {
                ServerFilter::Id => server.id.to_lowercase().contains(&query_lower),
                ServerFilter::Country => {
                    server.country.to_lowercase().contains(&query_lower)
                        || server.id.to_lowercase() == query_lower
                        || self.fuzzy_match(&server.country, &query)
                        || self.fuzzy_match(&server.id, &query)
                }
                ServerFilter::City => {
                    server.city.to_lowercase().contains(&query_lower)
                        || self.fuzzy_match(&server.city, &query)
                }
            })
            .collect()
    }

    fn fuzzy_match(&self, text: &str, query: &str) -> bool {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        if text_lower.starts_with(&query_lower) {
            return true;
        }

        let variants = self.generate_fuzzy_variants(&query_lower);
        variants.iter().any(|v| text_lower.contains(v))
    }

    fn generate_fuzzy_variants(&self, query: &str) -> Vec<String> {
        let mut variants = vec![query.to_string()];

        let no_vowels: String = query
            .chars()
            .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            .collect();
        if !no_vowels.is_empty() && no_vowels != query {
            variants.push(no_vowels);
        }

        let country_expansions = HashMap::from([
            ("jp", "japan"),
            ("us", "united states"),
            ("uk", "united kingdom"),
            ("de", "germany"),
            ("fr", "france"),
            ("au", "australia"),
            ("ca", "canada"),
            ("nl", "netherlands"),
            ("se", "sweden"),
            ("ch", "switzerland"),
            ("kr", "south korea"),
            ("sg", "singapore"),
            ("hk", "hong kong"),
            ("br", "brazil"),
            ("in", "india"),
            ("ru", "russia"),
            ("cn", "china"),
        ]);

        if let Some(expansion) = country_expansions.get(query) {
            variants.push(expansion.to_string());
        }

        variants
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
    }

    pub fn set_filter(&mut self, filter: ServerFilter) {
        self.filter = filter;
    }

    pub fn select_next(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }
        if let Some(idx) = self.selected_server {
            self.selected_server = Some((idx + 1).min(filtered.len() - 1));
        } else {
            self.selected_server = Some(0);
        }
    }

    pub fn select_prev(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }
        if let Some(idx) = self.selected_server {
            self.selected_server = Some(idx.saturating_sub(1));
        } else {
            self.selected_server = Some(0);
        }
    }

    pub fn select_first(&mut self) {
        let filtered = self.filtered_servers();
        if !filtered.is_empty() {
            self.selected_server = Some(0);
        }
    }

    pub fn select_last(&mut self) {
        let filtered = self.filtered_servers();
        if !filtered.is_empty() {
            self.selected_server = Some(filtered.len() - 1);
        }
    }

    pub fn select_page_down(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }
        let page_size = 10;
        if let Some(idx) = self.selected_server {
            self.selected_server = Some((idx + page_size).min(filtered.len() - 1));
        } else {
            self.selected_server = Some(0);
        }
    }

    pub fn select_page_up(&mut self) {
        let filtered = self.filtered_servers();
        if filtered.is_empty() {
            return;
        }
        let page_size = 10;
        if let Some(idx) = self.selected_server {
            self.selected_server = Some(idx.saturating_sub(page_size));
        } else {
            self.selected_server = Some(0);
        }
    }
}
