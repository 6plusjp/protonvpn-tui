//! Application state management

use crate::config::Settings;
use crate::state::ConnectionState;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
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
pub struct AppState {
    pub connection: ConnectionState,
    pub current_view: crate::state::AppView,
    pub search_query: String,
    pub filter: ServerFilter,
    pub sort: ServerSort,
    pub sort_direction: SortDirection,
    pub config: Settings,
    pub servers: Vec<Server>,
    pub selected_server: Option<usize>,
    pub settings_selected: Option<usize>,
    pub vpn_state: VpnState,
    pub notification: Option<Notification>,
    pub notification_log: Vec<Notification>,
    pending_refresh:
        Option<std::sync::mpsc::Receiver<crate::error::AppResult<Vec<crate::vpn::Server>>>>,
    previous_connection: Option<ConnectionState>,
    pending_connect:
        Option<std::sync::mpsc::Receiver<Result<(String, Option<String>), crate::error::AppError>>>,
    pending_disconnect: Option<std::sync::mpsc::Receiver<Result<(), crate::error::AppError>>>,
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
            sort: ServerSort::default(),
            sort_direction: SortDirection::default(),
            config: Settings::default(),
            servers: Vec::new(),
            selected_server: None,
            settings_selected: None,
            vpn_state: VpnState::new(),
            notification: None,
            notification_log: Vec::new(),
            pending_refresh: None,
            previous_connection: None,
            pending_connect: None,
            pending_disconnect: None,
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
            message: message.clone(),
            notification_type,
        });
        self.notification_log.push(Notification {
            message,
            notification_type,
        });
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    pub fn sync_connection_state(&mut self) {
        // Check for pending server refresh result
        if let Some(rx) = &self.pending_refresh {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(servers) => {
                        self.servers = servers;
                        self.show_notification(
                            format!("Refreshed {} servers", self.servers.len()),
                            NotificationType::Success,
                        );
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Refresh failed: {}", e),
                            NotificationType::Error,
                        );
                    }
                }
                self.pending_refresh = None;
            }
        }

        // Only check pending connection result when connecting
        if self.connection.is_connecting() {
            // Try to receive result from background thread
            if let Some(rx) = &self.pending_connect {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok((server, ip)) => {
                            self.show_notification(
                                format!("Connected to {}", &server),
                                NotificationType::Success,
                            );
                            self.connection = ConnectionState::Connected {
                                server,
                                ip: ip.unwrap_or_default(),
                            };
                            self.previous_connection = None;
                            self.pending_connect = None;
                            return;
                        }
                        Err(e) => {
                            if let Some(prev) = self.previous_connection.take() {
                                self.connection = prev;
                            } else {
                                self.connection = ConnectionState::Disconnected;
                            }
                            self.show_notification(
                                format!("Connection failed: {}", e),
                                NotificationType::Error,
                            );
                            self.pending_connect = None;
                            return;
                        }
                    }
                }
            }
        }

        if self.connection.is_disconnecting() {
            if let Some(rx) = &self.pending_disconnect {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(()) => {
                            self.connection = ConnectionState::Disconnected;
                            self.show_notification(
                                "Disconnected".to_string(),
                                NotificationType::Info,
                            );
                            self.previous_connection = None;
                            self.pending_disconnect = None;
                        }
                        Err(e) => {
                            if let Some(prev) = self.previous_connection.take() {
                                self.connection = prev;
                            } else {
                                self.connection = ConnectionState::Disconnected;
                            }
                            self.show_notification(
                                format!("Disconnect failed: {}", e),
                                NotificationType::Error,
                            );
                            self.pending_disconnect = None;
                        }
                    }
                }
            }
        }

        // When already connected, don't keep checking system state
        // This prevents flickering between connected/disconnected
        if self.connection.is_connected() {
            return;
        }

        // When disconnected, check if externally connected
        if self.connection == ConnectionState::Disconnected {
            if self.vpn_state.is_connected() {
                self.connection = ConnectionState::Connected {
                    server: "Unknown".to_string(),
                    ip: String::new(),
                };
            }
            return;
        }
    }

    pub fn refresh_servers(&mut self) {
        if let Ok(cached) = self.vpn_state.get_servers() {
            if !cached.is_empty() {
                self.servers = cached;
            }
        }

        self.show_notification("Refreshing servers...".to_string(), NotificationType::Info);

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_refresh = Some(rx);
        std::thread::spawn(move || {
            let mut vpn_state = VpnState::new();
            let result = vpn_state.refresh_servers();
            let _ = tx.send(result);
        });
    }

    pub fn connect(&mut self) {
        if self.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
            );
            return;
        }
        let Some(idx) = self.selected_server else {
            self.show_notification("No server selected".to_string(), NotificationType::Error);
            return;
        };

        let filtered = self.filtered_servers();
        let server_id = match filtered.get(idx) {
            Some(server) => server.id.clone(),
            None => {
                self.show_notification("No server selected".to_string(), NotificationType::Error);
                return;
            }
        };

        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Connecting;
        self.show_notification("Connecting...".to_string(), NotificationType::Info);

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_connect = Some(rx);
        std::thread::spawn(move || {
            let mut vpn_state = VpnState::new();
            let result = vpn_state.connect(&server_id);
            let _ = tx.send(result);
        });
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
        if self.connection.is_disconnected() {
            return;
        }

        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Disconnecting;
        self.show_notification("Disconnecting...".to_string(), NotificationType::Info);

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending_disconnect = Some(rx);
        std::thread::spawn(move || {
            let mut vpn_state = VpnState::new();
            let result = vpn_state.disconnect();
            let _ = tx.send(result);
        });
    }

    pub fn filtered_servers(&self) -> Vec<Server> {
        let query = self.search_query.to_lowercase();

        let connected_server_id = match &self.connection {
            ConnectionState::Connected { server, .. } => Some(server.clone()),
            _ => None,
        };

        let mut result: Vec<Server> = if query.is_empty() {
            self.servers.clone()
        } else {
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
                .cloned()
                .collect()
        };

        match (self.sort, self.sort_direction) {
            (ServerSort::Id, SortDirection::Asc) => {
                result.sort_by(|a, b| a.id.to_lowercase().cmp(&b.id.to_lowercase()))
            }
            (ServerSort::Id, SortDirection::Desc) => {
                result.sort_by(|a, b| b.id.to_lowercase().cmp(&a.id.to_lowercase()))
            }
            (ServerSort::Country, SortDirection::Asc) => {
                result.sort_by(|a, b| a.country.to_lowercase().cmp(&b.country.to_lowercase()))
            }
            (ServerSort::Country, SortDirection::Desc) => {
                result.sort_by(|a, b| b.country.to_lowercase().cmp(&a.country.to_lowercase()))
            }
        }

        if let Some(ref connected_id) = connected_server_id {
            if let Some(pos) = result
                .iter()
                .position(|s| connected_id.starts_with(&s.id) || s.id.starts_with(connected_id))
            {
                let server = result.remove(pos);
                result.insert(0, server);
            }
        }

        result
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

    pub fn cycle_sort(&mut self) {
        self.sort_direction = self.sort_direction.toggle();
    }

    pub fn cycle_sort_field(&mut self) {
        self.sort = self.sort.next();
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

    pub fn settings_select_next(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        if let Some(idx) = self.settings_selected {
            self.settings_selected = Some((idx + 1).min(count - 1));
        } else {
            self.settings_selected = Some(0);
        }
    }

    pub fn settings_select_prev(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        if let Some(idx) = self.settings_selected {
            self.settings_selected = Some(idx.saturating_sub(1));
        } else {
            self.settings_selected = Some(0);
        }
    }

    pub fn settings_select_first(&mut self, count: usize) {
        if count > 0 {
            self.settings_selected = Some(0);
        }
    }

    pub fn settings_select_last(&mut self, count: usize) {
        if count > 0 {
            self.settings_selected = Some(count - 1);
        }
    }

    pub fn settings_select_page_down(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let page_size = 10;
        if let Some(idx) = self.settings_selected {
            self.settings_selected = Some((idx + page_size).min(count - 1));
        } else {
            self.settings_selected = Some(0);
        }
    }

    pub fn settings_select_page_up(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let page_size = 10;
        if let Some(idx) = self.settings_selected {
            self.settings_selected = Some(idx.saturating_sub(page_size));
        } else {
            self.settings_selected = Some(0);
        }
    }
}
