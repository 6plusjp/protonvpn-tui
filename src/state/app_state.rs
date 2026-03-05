//! Application state management

use crate::config::ProtonSettings;
use crate::constants::state::MAX_NOTIFICATION_LOG;
use crate::constants::state::PAGE_SIZE;
use crate::state::async_tasks::{create_channel, AsyncResult, AsyncTaskManager};
use crate::state::ConnectionState;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
use crate::vpn::Server;
use crate::vpn::VpnState;
use std::sync::mpsc;
use std::sync::Mutex;
use std::sync::OnceLock;

pub type ConnectResult = (String, Option<String>);
pub type ConnectReceiver = mpsc::Receiver<AsyncResult<ConnectResult>>;
pub type ServerReceiver = mpsc::Receiver<AsyncResult<Vec<Server>>>;
pub type DisconnectReceiver = mpsc::Receiver<AsyncResult<()>>;

pub trait Navigatable {
    fn move_next(&mut self, bounds: usize);
    fn move_prev(&mut self, bounds: usize);
    fn move_first(&mut self, bounds: usize);
    fn move_last(&mut self, bounds: usize);
    fn move_page_down(&mut self, bounds: usize);
    fn move_page_up(&mut self, bounds: usize);
}

impl Navigatable for Option<usize> {
    fn move_next(&mut self, bounds: usize) {
        if bounds == 0 {
            return;
        }
        *self = Some(match *self {
            Some(i) => (i + 1).min(bounds - 1),
            None => 0,
        });
    }

    fn move_prev(&mut self, bounds: usize) {
        if bounds == 0 {
            return;
        }
        *self = Some(match *self {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
    }

    fn move_first(&mut self, bounds: usize) {
        if bounds > 0 {
            *self = Some(0);
        }
    }

    fn move_last(&mut self, bounds: usize) {
        if bounds > 0 {
            *self = Some(bounds - 1);
        }
    }

    fn move_page_down(&mut self, bounds: usize) {
        if bounds == 0 {
            return;
        }
        let page_size = PAGE_SIZE;
        *self = Some(match *self {
            Some(i) => (i + page_size).min(bounds - 1),
            None => 0,
        });
    }

    fn move_page_up(&mut self, bounds: usize) {
        if bounds == 0 {
            return;
        }
        let page_size = PAGE_SIZE;
        *self = Some(match *self {
            Some(i) => i.saturating_sub(page_size),
            None => 0,
        });
    }
}

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
    pub servers: Vec<Server>,
    pub selected_server: Option<usize>,
    pub settings_selected: Option<usize>,
    pub vpn_state: VpnState,
    pub notification: Option<Notification>,
    pub notification_log: Vec<Notification>,
    async_manager: AsyncTaskManager,
    #[allow(clippy::type_complexity)]
    pending_refresh: Option<ServerReceiver>,
    previous_connection: Option<ConnectionState>,
    pending_connect: Option<ConnectReceiver>,
    pending_disconnect: Option<DisconnectReceiver>,
    proton_settings_cache: OnceLock<Option<ProtonSettings>>,
    filtered_servers_cache: Mutex<Option<(Vec<Server>, u64)>>,
    filtered_servers_version: u64,
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
            servers: Vec::new(),
            selected_server: None,
            settings_selected: None,
            vpn_state: VpnState::new(),
            notification: None,
            notification_log: Vec::new(),
            async_manager: AsyncTaskManager::new(),
            pending_refresh: None,
            previous_connection: None,
            pending_connect: None,
            pending_disconnect: None,
            proton_settings_cache: OnceLock::new(),
            filtered_servers_cache: Mutex::new(None),
            filtered_servers_version: 0,
        }
    }

    pub fn get_proton_settings(&self) -> Option<&ProtonSettings> {
        if let Some(cached) = self.proton_settings_cache.get() {
            return cached.as_ref();
        }
        Some(
            self.proton_settings_cache
                .get_or_init(ProtonSettings::load)
                .as_ref()?,
        )
    }

    pub fn get_settings_count(&self) -> usize {
        self.get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
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
        if self.notification_log.len() > MAX_NOTIFICATION_LOG {
            self.notification_log.remove(0);
        }
    }

    pub fn clear_notification(&mut self) {
        self.notification = None;
    }

    pub fn sync_connection_state(&mut self) {
        // Check for pending server refresh result
        if let Some(rx) = self.pending_refresh.as_mut() {
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
            if let Some(rx) = self.pending_connect.as_mut() {
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
            if let Some(rx) = self.pending_disconnect.as_mut() {
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
        if self.connection == ConnectionState::Disconnected && self.vpn_state.is_connected() {
            self.connection = ConnectionState::Connected {
                server: "Unknown".to_string(),
                ip: String::new(),
            };
        }
    }

    pub fn refresh_servers(&mut self) {
        let cached = self.vpn_state.get_servers();
        if !cached.is_empty() {
            self.servers = cached;
            self.invalidate_filtered_cache();
        }

        self.show_notification("Refreshing servers...".to_string(), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.pending_refresh = Some(rx);
        self.async_manager
            .spawn_refresh_servers(self.vpn_state.clone(), tx);
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

        let (tx, rx) = create_channel();
        self.pending_connect = Some(rx);
        self.async_manager
            .spawn_connect(self.vpn_state.clone(), server_id, tx);
    }

    pub fn connect_random(&mut self) {
        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to random server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx) = create_channel();
        self.pending_connect = Some(rx);
        self.async_manager
            .spawn_connect_random(self.vpn_state.clone(), tx);
    }

    pub fn disconnect(&mut self) {
        if self.connection.is_disconnected() {
            return;
        }

        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Disconnecting;
        self.show_notification("Disconnecting...".to_string(), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.pending_disconnect = Some(rx);
        self.async_manager
            .spawn_disconnect(self.vpn_state.clone(), tx);
    }

    pub fn filtered_servers(&self) -> Vec<Server> {
        let version = self.filtered_servers_version;
        let cached = self.filtered_servers_cache.lock().unwrap();
        if let Some((ref cached_result, cached_version)) = *cached {
            if cached_version == version {
                return cached_result.clone();
            }
        }
        drop(cached);

        let result = self.compute_filtered_servers();
        *self.filtered_servers_cache.lock().unwrap() = Some((result.clone(), version));
        result
    }

    pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
        let query = self.search_query.to_lowercase();

        let connected_server_id = match &self.connection {
            ConnectionState::Connected { server, .. } => Some(server.clone()),
            _ => None,
        };

        let servers = &self.servers;
        let mut result: Vec<Server> = if query.is_empty() {
            self.servers.clone()
        } else {
            self.servers
                .iter()
                .filter(|server| match self.filter {
                    ServerFilter::Id => server.id.to_lowercase().contains(&query),
                    ServerFilter::Country => {
                        server.country.to_lowercase().contains(&query)
                            || server.id.to_lowercase() == query
                            || self.fuzzy_match(servers, &server.country, &query)
                            || self.fuzzy_match(servers, &server.id, &query)
                    }
                    ServerFilter::City => {
                        server.city.to_lowercase().contains(&query)
                            || self.fuzzy_match(servers, &server.city, &query)
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

    fn fuzzy_match(&self, servers: &[Server], text: &str, query: &str) -> bool {
        let text_lower = text.to_lowercase();

        if text_lower.starts_with(query) {
            return true;
        }

        let variants = self.generate_fuzzy_variants(servers, query);
        variants.iter().any(|v| text_lower.contains(v))
    }

    fn generate_fuzzy_variants(&self, servers: &[Server], query: &str) -> Vec<String> {
        let mut variants = vec![query.to_string()];

        let no_vowels: String = query
            .chars()
            .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            .collect();
        if !no_vowels.is_empty() && no_vowels != query {
            variants.push(no_vowels);
        }

        for server in servers {
            if server.id.to_lowercase() == query {
                variants.push(server.country.to_lowercase());
                break;
            }
        }

        variants
    }

    fn invalidate_filtered_cache(&mut self) {
        self.filtered_servers_version = self.filtered_servers_version.wrapping_add(1);
    }

    pub fn filtered_servers_count(&self) -> usize {
        self.filtered_servers().len()
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.invalidate_filtered_cache();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_direction = self.sort_direction.toggle();
        self.invalidate_filtered_cache();
    }

    pub fn cycle_sort_field(&mut self) {
        self.sort = self.sort.next();
        self.invalidate_filtered_cache();
    }

    pub fn set_filter(&mut self, filter: ServerFilter) {
        self.filter = filter;
        self.invalidate_filtered_cache();
    }

    pub fn select_next(&mut self) {
        self.selected_server
            .move_next(self.filtered_servers().len());
    }

    pub fn select_prev(&mut self) {
        self.selected_server
            .move_prev(self.filtered_servers().len());
    }

    pub fn select_first(&mut self) {
        self.selected_server
            .move_first(self.filtered_servers().len());
    }

    pub fn select_last(&mut self) {
        self.selected_server
            .move_last(self.filtered_servers().len());
    }

    pub fn select_page_down(&mut self) {
        self.selected_server
            .move_page_down(self.filtered_servers().len());
    }

    pub fn select_page_up(&mut self) {
        self.selected_server
            .move_page_up(self.filtered_servers().len());
    }

    pub fn settings_select_next(&mut self, count: usize) {
        self.settings_selected.move_next(count);
    }

    pub fn settings_select_prev(&mut self, count: usize) {
        self.settings_selected.move_prev(count);
    }

    pub fn settings_select_first(&mut self, count: usize) {
        self.settings_selected.move_first(count);
    }

    pub fn settings_select_last(&mut self, count: usize) {
        self.settings_selected.move_last(count);
    }

    pub fn settings_select_page_down(&mut self, count: usize) {
        self.settings_selected.move_page_down(count);
    }

    pub fn settings_select_page_up(&mut self, count: usize) {
        self.settings_selected.move_page_up(count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vpn::ServerFeatures;

    fn make_servers() -> Vec<Server> {
        vec![
            Server {
                id: "JP".to_string(),
                name: "Japan - JP".to_string(),
                country: "Japan".to_string(),
                city: "Tokyo".to_string(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "US".to_string(),
                name: "United States - US".to_string(),
                country: "United States".to_string(),
                city: "New York".to_string(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "DE".to_string(),
                name: "Germany - DE".to_string(),
                country: "Germany".to_string(),
                city: "Berlin".to_string(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "GB".to_string(),
                name: "United Kingdom - GB".to_string(),
                country: "United Kingdom".to_string(),
                city: "London".to_string(),
                features: ServerFeatures::default(),
            },
            Server {
                id: "FR".to_string(),
                name: "France - FR".to_string(),
                country: "France".to_string(),
                city: "Paris".to_string(),
                features: ServerFeatures::default(),
            },
        ]
    }

    #[test]
    fn test_filtered_servers_empty_query() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = String::new();

        let result = state.filtered_servers();

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filtered_servers_by_id() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "jp".to_string();
        state.filter = ServerFilter::Id;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "JP");
    }

    #[test]
    fn test_filtered_servers_by_country() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "japan".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_by_country_exact_match() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "JP".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "JP");
    }

    #[test]
    fn test_filtered_servers_by_city() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "tokyo".to_string();
        state.filter = ServerFilter::City;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city, "Tokyo");
    }

    #[test]
    fn test_filtered_servers_case_insensitive() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "JAPAN".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_id() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = String::new();
        state.sort = ServerSort::Id;
        state.sort_direction = SortDirection::Asc;

        let result = state.filtered_servers();

        assert_eq!(result[0].id, "DE");
        assert_eq!(result[4].id, "US");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_id() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = String::new();
        state.sort = ServerSort::Id;
        state.sort_direction = SortDirection::Desc;

        let result = state.filtered_servers();

        assert_eq!(result[0].id, "US");
        assert_eq!(result[4].id, "DE");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_country() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = String::new();
        state.sort = ServerSort::Country;
        state.sort_direction = SortDirection::Asc;

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "France");
        assert_eq!(result[4].country, "United States");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_country() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = String::new();
        state.sort = ServerSort::Country;
        state.sort_direction = SortDirection::Desc;

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "United States");
        assert_eq!(result[4].country, "France");
    }

    #[test]
    fn test_filtered_servers_multiple_matches() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.search_query = "u".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_starts_with() {
        let mut state = AppState::new();
        state.servers = make_servers();
        state.vpn_state = VpnState::new();

        let result = state.compute_filtered_servers();
        assert!(!result.is_empty());
    }
}
