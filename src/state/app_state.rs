//! Application state management

use crate::config::ProtonSettings;
use crate::constants::state::MAX_NOTIFICATION_LOG;
use crate::constants::state::PAGE_SIZE;
use crate::constants::ui::{MAX_VISIBLE_NOTIFICATIONS, NOTIFICATION_TIMER_DEFAULT};
use crate::state::async_tasks::{create_channel, AsyncResult, AsyncTaskManager};
use crate::state::log_persistence;
use crate::state::ConnectionState;
use crate::state::Pane;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
use crate::ui::styles::Theme;
use crate::vpn::Server;
use crate::vpn::VpnState;
use chrono::{DateTime, Utc};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

/// Input mode for text input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Filter input mode (/)
    Filter,
    /// DNS input mode (for custom DNS)
    DnsInput,
}

pub type ConnectResult = (String, Option<String>);
pub type ConnectReceiver = mpsc::Receiver<AsyncResult<ConnectResult>>;
pub type ServerReceiver = mpsc::Receiver<AsyncResult<Vec<Server>>>;
pub type DisconnectReceiver = mpsc::Receiver<AsyncResult<()>>;
pub type CitiesReceiver = mpsc::Receiver<AsyncResult<Vec<crate::vpn::City>>>;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Error,
}

/// Notification popup (legacy - used for notification_log)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: DateTime<Utc>,
}

/// Toast notification with individual timer for stacked display
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timer: u8,
}

/// Main application state
///
/// Fields are organized into logical groups for better maintainability:
/// - Connection & Async: VPN connection state and background task management
/// - Server Data: Server list and filtering cache
/// - UI State: View selection, filters, and sorting
/// - Notification: User notifications and history
/// - Config: Application settings cache
pub struct AppState {
    // === Connection & Async (深い結合) ===
    pub connection: ConnectionState,
    pub vpn_state: Arc<VpnState>,
    previous_connection: Option<ConnectionState>,
    async_manager: AsyncTaskManager,
    #[allow(clippy::type_complexity)]
    pending_refresh: Option<ServerReceiver>,
    pending_connect: Option<ConnectReceiver>,
    pending_disconnect: Option<DisconnectReceiver>,
    pending_cities: Option<CitiesReceiver>,
    pending_cities_country: Option<String>,
    pending_connect_city: Option<ConnectReceiver>,

    // === Server Data ===
    pub(crate) servers: Vec<Server>,
    filtered_servers_cache: Mutex<Option<(Vec<Server>, u64)>>,
    filtered_servers_version: u64,
    pub(crate) current_cities: Vec<crate::vpn::City>,
    pub(crate) current_country_code: Option<String>,

    // === UI State ( views から直接アクセス ) ===
    pub current_view: crate::state::AppView,
    pub selected_server: Option<usize>,
    pub selected_city: Option<usize>,
    pub pane_focus: Pane,
    pub settings_selected: Option<usize>,
    pub search_query: String,
    pub filter: ServerFilter,
    pub sort: ServerSort,
    pub sort_direction: SortDirection,
    pub is_dark_theme: bool,
    pub input_mode: InputMode,
    pub dns_input: String,

    // === Notification ===
    pub notifications: Vec<ToastNotification>,
    pub notification_log: Vec<Notification>,

    // === Config (独立してロード可能) ===
    proton_settings_cache: Option<ProtonSettings>,
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
            current_view: crate::state::AppView::Servers,
            search_query: String::new(),
            filter: ServerFilter::default(),
            sort: ServerSort::default(),
            sort_direction: SortDirection::default(),
            is_dark_theme: true,
            servers: Vec::new(),
            current_cities: Vec::new(),
            current_country_code: None,
            selected_server: Some(0),
            selected_city: Some(0),
            pane_focus: Pane::Countries,
            settings_selected: Some(0),
            input_mode: InputMode::Normal,
            dns_input: String::new(),
            vpn_state: Arc::new(VpnState::new()),
            notifications: Vec::new(),
            notification_log: log_persistence::load_notification_log(),
            async_manager: AsyncTaskManager::new(),
            pending_refresh: None,
            previous_connection: None,
            pending_connect: None,
            pending_disconnect: None,
            pending_cities: None,
            pending_cities_country: None,
            pending_connect_city: None,
            proton_settings_cache: ProtonSettings::load(),
            filtered_servers_cache: Mutex::new(None),
            filtered_servers_version: 0,
        }
    }

    /// Get current theme based on dark/light mode
    pub fn get_theme(&self) -> Theme {
        if self.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    // === Getters for tight coupling reduction ===

    /// Get current connection state
    pub fn get_connection(&self) -> &ConnectionState {
        &self.connection
    }

    /// Get current view
    pub fn get_current_view(&self) -> crate::state::AppView {
        self.current_view
    }

    /// Get current pane focus
    pub fn get_pane_focus(&self) -> Pane {
        self.pane_focus
    }

    /// Get selected server index
    pub fn get_selected_server(&self) -> Option<usize> {
        self.selected_server
    }

    /// Get selected city index
    pub fn get_selected_city(&self) -> Option<usize> {
        self.selected_city
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.invalidate_filtered_cache();
    }

    pub fn set_servers(&mut self, servers: Vec<Server>) {
        self.servers = servers;
        self.invalidate_filtered_cache();

        self.load_cities_for_selected_server();
    }

    fn load_cities_for_selected_server(&mut self) {
        if let Some(idx) = self.selected_server {
            if let Some(server) = self.filtered_servers().get(idx) {
                if self.current_country_code.as_deref() != Some(&server.id) {
                    self.fetch_cities(&server.id);
                }
            }
        }
    }

    pub fn get_proton_settings(&self) -> Option<&ProtonSettings> {
        self.proton_settings_cache.as_ref()
    }

    pub fn get_proton_protocol(&self) -> Option<String> {
        self.proton_settings_cache
            .as_ref()
            .and_then(|ps| ps.protocol.clone())
    }

    pub fn switch_view(&mut self) {
        self.current_view = self.current_view.next();
    }

    pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notifications.push(ToastNotification {
            message: message.clone(),
            notification_type,
            timer: NOTIFICATION_TIMER_DEFAULT,
        });

        if self.notifications.len() > MAX_VISIBLE_NOTIFICATIONS {
            self.notifications.remove(0);
        }

        self.notification_log.push(Notification {
            message,
            notification_type,
            timestamp: Utc::now(),
        });
        if self.notification_log.len() > MAX_NOTIFICATION_LOG {
            self.notification_log.remove(0);
        }

        log_persistence::save_notification_log(&self.notification_log);
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn tick_notifications(&mut self) {
        for notification in &mut self.notifications {
            if notification.timer > 0 {
                notification.timer -= 1;
            }
        }
        self.notifications.retain(|n| n.timer > 0);
    }

    /// Sync connection state with background tasks.
    /// Returns true if any notification was shown during sync.
    /// Sync connection state with system (call periodically)
    pub fn sync_connection_state(&mut self) -> bool {
        let mut notification_shown = false;
        // Check for pending server refresh result
        if let Some(rx) = self.pending_refresh.as_mut() {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(servers) => {
                        self.set_servers(servers);
                        tracing::info!("Server list refreshed: {} servers", self.servers.len());
                        self.show_notification(
                            format!("Refreshed {} servers", self.servers.len()),
                            NotificationType::Success,
                        );
                        notification_shown = true;
                    }
                    Err(e) => {
                        tracing::warn!("Server list refresh failed: {}", e);
                        self.show_notification(
                            format!("Refresh failed: {}", e),
                            NotificationType::Error,
                        );
                        notification_shown = true;
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
                            tracing::info!("Successfully connected to server: {}", server);
                            self.connection = ConnectionState::Connected {
                                server,
                                ip: ip.unwrap_or_default(),
                            };
                            self.previous_connection = None;
                            self.pending_connect = None;
                            return true;
                        }
                        Err(e) => {
                            if let Some(prev) = self.previous_connection.take() {
                                self.connection = prev;
                            } else {
                                self.connection = ConnectionState::Disconnected;
                            }
                            tracing::warn!("Connection failed: {}", e);
                            self.show_notification(
                                format!("Connection failed: {}", e),
                                NotificationType::Error,
                            );
                            self.pending_connect = None;
                            return true;
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
                            tracing::info!("Successfully disconnected from VPN");
                            self.show_notification(
                                "Disconnected".to_string(),
                                NotificationType::Info,
                            );
                            notification_shown = true;
                            self.previous_connection = None;
                            self.pending_disconnect = None;
                        }
                        Err(e) => {
                            if let Some(prev) = self.previous_connection.take() {
                                self.connection = prev;
                            } else {
                                self.connection = ConnectionState::Disconnected;
                            }
                            tracing::warn!("Disconnect failed: {}", e);
                            self.show_notification(
                                format!("Disconnect failed: {}", e),
                                NotificationType::Error,
                            );
                            notification_shown = true;
                            self.pending_disconnect = None;
                        }
                    }
                }
            }
        }

        // Check for pending cities fetch result
        if let Some(rx) = self.pending_cities.as_mut() {
            if let Ok(result) = rx.try_recv() {
                let pending_country = self.pending_cities_country.clone();

                match result {
                    Ok(cities) => {
                        if let Some(pending) = pending_country {
                            if self.current_country_code.as_deref() == Some(&pending) {
                                self.current_cities = cities.clone();
                            }
                        }
                        self.show_notification(
                            format!("Loaded {} cities", cities.len()),
                            NotificationType::Success,
                        );
                        notification_shown = true;
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Failed to load cities: {}", e),
                            NotificationType::Error,
                        );
                        notification_shown = true;
                    }
                }
                self.pending_cities = None;
                self.pending_cities_country = None;
                self.invalidate_filtered_cache();
            }
        }

        // Check for pending connect city result
        if let Some(rx) = self.pending_connect_city.as_mut() {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok((server, ip)) => {
                        self.show_notification(
                            format!("Connected to {}", &server),
                            NotificationType::Success,
                        );
                        notification_shown = true;
                        tracing::info!(
                            "Successfully connected to server (connect_city): {}",
                            server
                        );
                        self.connection = ConnectionState::Connected {
                            server,
                            ip: ip.unwrap_or_default(),
                        };
                        self.previous_connection = None;
                        self.pending_connect_city = None;
                    }
                    Err(e) => {
                        if let Some(prev) = self.previous_connection.take() {
                            self.connection = prev;
                        } else {
                            self.connection = ConnectionState::Disconnected;
                        }
                        tracing::warn!("Connection failed (connect_city): {}", e);
                        self.show_notification(
                            format!("Connection failed: {}", e),
                            NotificationType::Error,
                        );
                        notification_shown = true;
                        self.pending_connect_city = None;
                    }
                }
            }
        }

        // When already connected, don't keep checking system state
        // This prevents flickering between connected/disconnected
        if self.connection.is_connected() {
            return notification_shown;
        }

        // When disconnected, check if externally connected
        if self.connection == ConnectionState::Disconnected && self.vpn_state.is_connected() {
            self.connection = ConnectionState::Connected {
                server: "Unknown".to_string(),
                ip: String::new(),
            };
        }

        notification_shown
    }

    pub fn refresh_servers(&mut self) {
        tracing::info!("Refreshing server list");
        let cached = self.vpn_state.get_servers();
        if !cached.is_empty() {
            self.set_servers(cached);
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

        tracing::info!("Connecting to server: {}", server_id);
        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Connecting;
        self.show_notification("Connecting...".to_string(), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.pending_connect = Some(rx);
        self.async_manager
            .spawn_connect(self.vpn_state.clone(), server_id, tx);
    }

    pub fn connect_random(&mut self) {
        tracing::info!("Connecting to random server");
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

        tracing::info!("Disconnecting from VPN");
        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Disconnecting;
        self.show_notification("Disconnecting...".to_string(), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.pending_disconnect = Some(rx);
        self.async_manager
            .spawn_disconnect(self.vpn_state.clone(), tx);
    }

    pub fn fetch_cities(&mut self, country_code: &str) {
        let country_code = country_code.to_string();

        if let Ok(cities) = self.vpn_state.list_cities_with_features(&country_code) {
            self.current_cities = cities.clone();
            self.current_country_code = Some(country_code.clone());
        }

        self.pending_cities_country = Some(country_code.clone());

        self.invalidate_filtered_cache();

        self.show_notification(
            format!("Loading cities for {}...", country_code),
            NotificationType::Info,
        );

        let (tx, rx) = create_channel();
        self.pending_cities = Some(rx);
        self.async_manager
            .spawn_cities(self.vpn_state.clone(), country_code, tx);
    }

    pub fn set_cities(&mut self, cities: Vec<crate::vpn::City>, country_code: String) {
        self.current_cities = cities;
        self.current_country_code = Some(country_code);
    }

    pub fn connect_city(&mut self, city: &str) {
        if self.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
            );
            return;
        }

        let city = city.to_string();
        self.previous_connection = Some(self.connection.clone());
        self.connection = ConnectionState::Connecting;
        self.show_notification(format!("Connecting to {}...", city), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.pending_connect_city = Some(rx);
        self.async_manager
            .spawn_connect_city(self.vpn_state.clone(), city, tx);
    }

    /// Get filtered and sorted server list
    pub fn filtered_servers(&self) -> Vec<Server> {
        let version = self.filtered_servers_version;
        let cached = match self.filtered_servers_cache.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to lock filtered_servers_cache: {}", e);
                return self.compute_filtered_servers();
            }
        };
        if let Some((ref cached_result, cached_version)) = *cached {
            if cached_version == version {
                return cached_result.clone();
            }
        }
        drop(cached);

        let result = self.compute_filtered_servers();
        let mut cache = match self.filtered_servers_cache.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to lock filtered_servers_cache for write: {}", e);
                return result;
            }
        };
        *cache = Some((result.clone(), version));
        result
    }

    pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
        let query = self.search_query.to_lowercase();

        let connected_server_id = match &self.connection {
            ConnectionState::Connected { server, .. } => Some(server.clone()),
            _ => None,
        };

        // Always get servers from VPN state cache (includes cities)
        let servers = self.vpn_state.get_servers();
        let mut result: Vec<Server> = if query.is_empty() {
            servers.clone()
        } else {
            servers
                .iter()
                .filter(|server| {
                    server.id.to_lowercase().contains(&query)
                        || server.country.to_lowercase().contains(&query)
                        || server
                            .cities
                            .iter()
                            .any(|c| c.name.to_lowercase().contains(&query))
                        || self.fuzzy_match(&servers, &server.country, &query)
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
            if !connected_id.is_empty() {
                if let Some(pos) = result
                    .iter()
                    .position(|s| connected_id.starts_with(&s.id) || s.id.starts_with(connected_id))
                {
                    let server = result.remove(pos);
                    result.insert(0, server);
                }
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
        self.selected_server.move_next(self.get_selection_bounds());
    }

    pub fn select_prev(&mut self) {
        self.selected_server.move_prev(self.get_selection_bounds());
    }

    pub fn select_first(&mut self) {
        self.selected_server.move_first(self.get_selection_bounds());
    }

    pub fn select_last(&mut self) {
        self.selected_server.move_last(self.get_selection_bounds());
    }

    pub fn select_page_down(&mut self) {
        self.selected_server
            .move_page_down(self.get_selection_bounds());
    }

    pub fn select_page_up(&mut self) {
        self.selected_server
            .move_page_up(self.get_selection_bounds());
    }

    pub fn move_to_cities(&mut self) {
        if let Some(idx) = self.selected_server {
            let servers = self.filtered_servers();
            if let Some(server) = servers.get(idx) {
                if self.current_country_code.as_deref() != Some(&server.id) {
                    self.fetch_cities(&server.id);
                    self.current_country_code = Some(server.id.clone());
                }
            }
        }
        self.pane_focus = Pane::Cities;
    }

    pub fn move_to_countries(&mut self) {
        self.pane_focus = Pane::Countries;
    }

    pub fn city_select_next(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_next(bounds);
        }
    }

    pub fn city_select_prev(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_prev(bounds);
        }
    }

    pub fn city_select_first(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_first(bounds);
        }
    }

    pub fn city_select_last(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_last(bounds);
        }
    }

    pub fn city_select_page_down(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_page_down(bounds);
        }
    }

    pub fn city_select_page_up(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.selected_city.move_page_up(bounds);
        }
    }

    fn get_selection_bounds(&self) -> usize {
        if self.current_view == crate::state::AppView::Servers && self.pane_focus == Pane::Cities {
            self.current_cities.len()
        } else {
            self.filtered_servers().len()
        }
    }

    pub fn settings_select_next(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_next(count);
    }

    pub fn settings_select_prev(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_prev(count);
    }

    pub fn settings_select_first(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_first(count);
    }

    pub fn settings_select_last(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_last(count);
    }

    pub fn settings_select_page_down(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_page_down(count);
    }

    pub fn settings_select_page_up(&mut self) {
        let count = self
            .get_proton_settings()
            .map(|ps| ps.settings_count())
            .unwrap_or(0)
            .max(7)
            + 1;
        self.settings_selected.move_page_up(count);
    }

    pub fn toggle_settings(&mut self, index: usize) {
        let ps = self.get_proton_settings();

        // For DNS (index 2), Enter always shows input prompt
        if index == 2 {
            self.input_mode = InputMode::DnsInput;
            self.dns_input = String::new();
            self.show_notification(
                "Enter DNS IPs (e.g., 1.1.1.1,9.9.9.9)".to_string(),
                NotificationType::Info,
            );
            return;
        }

        let result = match index {
            0 => {
                let current = ps.and_then(|p| p.killswitch);
                self.vpn_state.toggle_killswitch(current)
            }
            1 => {
                let current = ps.and_then(|p| p.ipv6);
                self.vpn_state.toggle_ipv6(current)
            }
            2 => {
                unreachable!("DNS handled before match")
            }
            3 => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.netshield);
                let next = 0;
                self.vpn_state.set_netshield(current, next)
            }
            4 => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.moderate_nat);
                self.vpn_state.toggle_moderate_nat(current)
            }
            5 => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.vpn_accelerator);
                self.vpn_state.toggle_vpn_accelerator(current)
            }
            6 => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.port_forwarding);
                self.vpn_state.toggle_port_forwarding(current)
            }
            7 => {
                self.is_dark_theme = !self.is_dark_theme;
                tracing::info!(
                    "Theme changed to {}",
                    if self.is_dark_theme { "Dark" } else { "Light" }
                );
                self.show_notification(
                    format!(
                        "Theme changed to {}",
                        if self.is_dark_theme { "Dark" } else { "Light" }
                    ),
                    NotificationType::Info,
                );
                return;
            }
            _ => {
                self.show_notification(
                    "Invalid setting selection".to_string(),
                    NotificationType::Error,
                );
                return;
            }
        };

        match result {
            Ok(msg) => {
                tracing::info!("Setting updated: {}", msg);
                self.show_notification(
                    format!("Setting updated: {}", msg),
                    NotificationType::Success,
                );
                self.proton_settings_cache = None;
            }
            Err(e) => {
                tracing::warn!("Failed to update setting: {}", e);
                self.show_notification(
                    format!("Failed to update setting: {}", e),
                    NotificationType::Error,
                );
            }
        }
    }

    pub fn toggle_settings_off(&mut self, index: usize) {
        let ps = self.get_proton_settings();

        // For DNS (index 2), Space turns off directly
        if index == 2 {
            let dns_enabled = ps.map(|p| p.custom_dns.enabled).unwrap_or(false);
            if dns_enabled {
                match self.vpn_state.disable_custom_dns() {
                    Ok(msg) => {
                        self.show_notification(
                            format!("DNS disabled: {}", msg),
                            NotificationType::Success,
                        );
                        self.proton_settings_cache = None;
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Failed to disable DNS: {}", e),
                            NotificationType::Error,
                        );
                    }
                }
            } else {
                self.show_notification("DNS is already off".to_string(), NotificationType::Info);
            }
            return;
        }

        // For other settings, use regular toggle
        self.toggle_settings(index);
    }

    pub fn apply_dns_setting(&mut self, dns_ips: &str) {
        let ips: Vec<&str> = dns_ips
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if ips.is_empty() {
            self.show_notification("No DNS IPs provided".to_string(), NotificationType::Error);
            return;
        }

        let dns_list = ips.join(",");
        let result = self.vpn_state.set_custom_dns(&dns_list);

        match result {
            Ok(msg) => {
                self.show_notification(format!("DNS updated: {}", msg), NotificationType::Success);
                self.proton_settings_cache = None;
            }
            Err(e) => {
                self.show_notification(
                    format!("Failed to update DNS: {}", e),
                    NotificationType::Error,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::log_persistence;
    use crate::vpn::City;

    fn setup() {
        log_persistence::set_test_mode(true);
    }

    fn make_servers() -> Vec<Server> {
        vec![
            Server {
                id: "JP".to_string(),
                country: "Japan".to_string(),
                cities: vec![
                    City::new("Tokyo".to_string()),
                    City::new("Osaka".to_string()),
                ],
            },
            Server {
                id: "US".to_string(),
                country: "United States".to_string(),
                cities: vec![City::new("New York".to_string())],
            },
            Server {
                id: "DE".to_string(),
                country: "Germany".to_string(),
                cities: vec![City::new("Berlin".to_string())],
            },
            Server {
                id: "GB".to_string(),
                country: "United Kingdom".to_string(),
                cities: vec![City::new("London".to_string())],
            },
            Server {
                id: "FR".to_string(),
                country: "France".to_string(),
                cities: vec![City::new("Paris".to_string())],
            },
        ]
    }

    #[test]
    fn test_filtered_servers_empty_query() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = String::new();

        let result = state.filtered_servers();

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filtered_servers_by_id() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "jp".to_string();
        state.filter = ServerFilter::Id;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "JP");
    }

    #[test]
    fn test_filtered_servers_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "japan".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_by_country_exact_match() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "JP".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "JP");
    }

    #[test]
    fn test_filtered_servers_by_city() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "tokyo".to_string();
        state.filter = ServerFilter::City;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert!(result[0].cities.iter().any(|c| c.name == "Tokyo"));
    }

    #[test]
    fn test_filtered_servers_case_insensitive() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "JAPAN".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_id() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
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
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
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
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
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
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
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
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));
        state.search_query = "u".to_string();
        state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_starts_with() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnState::with_test_servers(make_servers()));

        let result = state.compute_filtered_servers();
        assert!(!result.is_empty());
    }
}

#[cfg(test)]
mod notification_tests {
    use super::{AppState, NotificationType};
    use crate::state::log_persistence;

    fn setup() {
        log_persistence::set_test_mode(true);
    }

    #[test]
    fn test_show_notification_adds_to_vector() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Success);

        assert_eq!(state.notifications.len(), 2);
    }

    #[test]
    fn test_tick_notifications_removes_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);

        for _ in 0..30 {
            state.tick_notifications();
        }

        assert!(state.notifications.is_empty());
    }

    #[test]
    fn test_tick_notifications_preserves_non_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Info);

        state.tick_notifications();

        assert_eq!(state.notifications.len(), 2);
        assert!(state.notifications.iter().all(|n| n.timer < 30));
    }

    #[test]
    fn test_max_notifications_enforced() {
        setup();
        let mut state = AppState::new();
        for i in 0..5 {
            state.show_notification(format!("Msg {}", i), NotificationType::Info);
        }

        assert_eq!(state.notifications.len(), 3);
        assert!(state.notifications.iter().any(|n| n.message == "Msg 2"));
        assert!(state.notifications.iter().any(|n| n.message == "Msg 3"));
        assert!(state.notifications.iter().any(|n| n.message == "Msg 4"));
    }

    #[test]
    fn test_notification_log_preserves_all() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Msg 1".to_string(), NotificationType::Info);
        state.show_notification("Msg 2".to_string(), NotificationType::Error);

        assert_eq!(state.notification_log.len(), 2);
    }

    #[test]
    fn test_clear_notifications() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Error);

        state.clear_notifications();

        assert!(state.notifications.is_empty());
        assert_eq!(state.notification_log.len(), 2);
    }
}
