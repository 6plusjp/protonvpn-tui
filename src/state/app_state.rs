//! Application state management

use crate::config::{ProtonSettings, SettingKey};
use crate::constants::state::PAGE_SIZE;
use crate::state::AsyncEvent;
use crate::state::AsyncResult;
use crate::state::ConfigState;
use crate::state::ConnectionManager;
use crate::state::ConnectionState;
use crate::state::InputMode;
use crate::state::NotificationState;
use crate::state::NotificationType;
use crate::state::Pane;
use crate::state::ServerCache;
use crate::state::ServerDataState;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
use crate::state::UiState;
use crate::ui::styles::Theme;
use crate::vpn::async_tasks::create_channel;
use crate::vpn::ConnectResult;
use crate::vpn::Server;
use crate::vpn::VpnClient;
use std::sync::mpsc;
use std::sync::Arc;

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

/// Main application state
///
/// Fields are organized into logical groups for better maintainability:
/// - Connection & Async: VPN connection state and background task management
/// - Server Data: Server list and filtering cache
/// - UI State: View selection, filters, and sorting
/// - Notification: User notifications and history
/// - Config: Application settings cache
pub struct AppState {
    // === VPN State (needed by connection methods) ===
    pub vpn_state: Arc<VpnClient>,

    // === Connection Manager ===
    pub connection_manager: ConnectionManager,

    // === Server Data ===
    pub(crate) servers: Vec<Server>,
    server_cache: ServerCache,
    #[allow(dead_code)]
    is_initialized: bool,
    pub current_cities: Vec<crate::vpn::City>,
    pub current_country_code: Option<String>,

    // === Server Data  ===
    pub server_data: ServerDataState,

    // === UI State  ===
    pub ui_state: UiState,

    // === Notification  ===
    pub notification_state: NotificationState,

    // === Config  ===
    pub config_state: ConfigState,
    pub proton_settings_cache: Option<ProtonSettings>,
    pub key_bindings: crate::config::KeyBindings,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let vpn_state = Arc::new(VpnClient::new());
        let servers = vpn_state.servers();

        Self {
            vpn_state,
            connection_manager: ConnectionManager::new(),
            ui_state: UiState::new(),
            server_data: ServerDataState::new(),
            servers,
            current_cities: Vec::new(),
            current_country_code: None,
            notification_state: NotificationState::new(),
            config_state: ConfigState::new(),
            proton_settings_cache: ProtonSettings::load(),
            server_cache: ServerCache::new(),
            is_initialized: false,
            key_bindings: crate::config::KeyBindings::default(),
        }
    }

    /// Get current theme based on dark/light mode
    pub fn get_theme(&self) -> Theme {
        if self.ui_state.is_dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    // === Getters for tight coupling reduction ===

    /// Wait for async events with timeout (event-driven)
    /// Returns true if any events were processed
    pub fn wait_for_async_events(&mut self, timeout: std::time::Duration) -> bool {
        let events = self.connection_manager.async_notifier.wait_timeout(timeout);
        if events.is_empty() {
            return false;
        }
        self.process_async_events()
    }

    // === Search query ===

    pub fn set_search_query(&mut self, query: String) {
        self.ui_state.search_query.set(query);
        self.invalidate_filtered_cache();

        let filtered_len = self.filtered_servers().len();
        if let Some(idx) = self.ui_state.selected_server {
            if idx >= filtered_len {
                self.ui_state.selected_server = Some(0);
            }
        }

        self.switch_cities_to_selected();
    }

    pub fn set_servers(&mut self, servers: Vec<Server>) {
        self.servers = servers;
        self.server_data.servers = self.servers.clone();
        self.invalidate_filtered_cache();

        if let Some(idx) = self.ui_state.selected_server {
            if let Some(server) = self.filtered_servers().get(idx) {
                self.current_country_code = Some(server.code.clone());
                self.fetch_cities(&server.code);
            }
        }
    }

    pub fn switch_view(&mut self) {
        let new_view = self.ui_state.current_view.next();
        self.ui_state.previous_view = self.ui_state.current_view;
        self.ui_state.current_view = new_view;
        self.ui_state.pane_focus = new_view.default_pane();
    }

    pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
        self.notification_state.show(message, notification_type);
    }

    /// Process async events notified via Condvar (event-driven)
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
                    self.server_data.is_initialized = true;
                    if self.vpn_state.is_cli_unavailable() {
                        self.show_notification(
                            "ProtonVPN CLI unavailable. VPN functionality disabled.".to_string(),
                            NotificationType::Error,
                        );
                    } else {
                        self.show_notification(
                            format!("Refreshed {} servers", self.servers.len()),
                            NotificationType::Success,
                        );
                    }
                    notification_shown = true;
                    self.connection_manager.pending_refresh.remove(&());
                }
                AsyncEvent::ServersRefreshFailed(e) => {
                    tracing::warn!("Server list refresh failed: {}", e);
                    self.server_data.is_initialized = true;
                    self.show_notification(
                        format!("Refresh failed: {}", e),
                        NotificationType::Error,
                    );
                    notification_shown = true;
                    self.connection_manager.pending_refresh.remove(&());
                }
                AsyncEvent::Connected(server, ip, city, country) => {
                    self.show_notification(
                        format!("Connected to {}", &server),
                        NotificationType::Success,
                    );
                    tracing::info!("Successfully connected to server: {}", server);
                    self.connection_manager.connection = ConnectionState::Connected {
                        server,
                        ip: ip.unwrap_or_default(),
                        city,
                        country,
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
                        format!("Connection failed: {}", e),
                        NotificationType::Error,
                    );
                    self.connection_manager.pending_connect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::Disconnected => {
                    self.show_notification("Disconnected".to_string(), NotificationType::Info);
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
                    );
                    if let Some(prev) = self.connection_manager.previous_connection.take() {
                        self.connection_manager.connection = prev;
                    }
                    self.connection_manager.pending_disconnect.remove(&());
                    notification_shown = true;
                }
                AsyncEvent::CitiesLoaded(_, _) => {
                    // Handled by pending_cities try_recv in check_pending_async_events
                }
                AsyncEvent::ConnectCityResult(server, ip, city, country) => {
                    self.show_notification(
                        format!("Connected to {}", server),
                        NotificationType::Success,
                    );
                    self.connection_manager.connection = ConnectionState::Connected {
                        server,
                        ip: ip.unwrap_or_default(),
                        city,
                        country,
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
                        format!("Connection failed: {}", e),
                        NotificationType::Error,
                    );
                    self.connection_manager.pending_connect_city.remove(&());
                    notification_shown = true;
                }
            }
        }
        notification_shown
    }

    /// Check pending async events and update connection state.
    /// Returns true if any notification was shown during check.
    pub fn check_pending_async_events(&mut self) -> bool {
        let mut notification_shown = false;
        // Check for notified async events (event-driven)
        notification_shown |= self.process_async_events();
        // Check for pending server refresh result
        if let Some(rx) = self.connection_manager.pending_refresh.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(servers) => {
                        self.set_servers(servers);
                        tracing::info!("Server list refreshed: {} servers", self.servers.len());
                        self.server_data.is_initialized = true;

                        if self.vpn_state.is_cli_unavailable() {
                            self.show_notification(
                                "ProtonVPN CLI unavailable. VPN functionality disabled."
                                    .to_string(),
                                NotificationType::Error,
                            );
                        } else {
                            self.show_notification(
                                format!("Refreshed {} servers", self.servers.len()),
                                NotificationType::Success,
                            );
                        }
                        notification_shown = true;
                    }
                    Err(e) => {
                        tracing::warn!("Server list refresh failed: {}", e);
                        self.server_data.is_initialized = true;
                        self.show_notification(
                            format!("Refresh failed: {}", e),
                            NotificationType::Error,
                        );
                        notification_shown = true;
                    }
                }
                self.connection_manager.pending_refresh.remove(&());
            }
        }

        // Only check pending connection result when connecting
        if self.connection_manager.connection.is_connecting() {
            // Try to receive result from background thread
            if let Some(rx) = self.connection_manager.pending_connect.get_mut(&()) {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(conn_result) => {
                            self.show_notification(
                                format!("Connected to {}", &conn_result.server_id),
                                NotificationType::Success,
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
                                format!("Connection failed: {}", e),
                                NotificationType::Error,
                            );
                            self.connection_manager.pending_connect.remove(&());
                            return true;
                        }
                    }
                }
            }
        }

        if self.connection_manager.connection.is_disconnecting() {
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
                            self.show_notification(msg, NotificationType::Info);
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
                                format!("Disconnect failed: {}", e),
                                NotificationType::Error,
                            );
                            notification_shown = true;
                            self.connection_manager.pending_disconnect.remove(&());
                        }
                    }
                }
            }
        }

        // Check for pending cities fetch results
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
            self.connection_manager.pending_cities.remove(&country_code);
        }

        if !self.connection_manager.pending_cities.is_empty() {
            self.invalidate_filtered_cache();
        }

        // Check for pending connect city result
        if let Some(rx) = self.connection_manager.pending_connect_city.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(conn_result) => {
                        self.show_notification(
                            format!("Connected to {}", &conn_result.server_id),
                            NotificationType::Success,
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
                            format!("Connection failed: {}", e),
                            NotificationType::Error,
                        );
                        notification_shown = true;
                        self.connection_manager.pending_connect_city.remove(&());
                    }
                }
            }
        }

        // Check for pending config_set result
        if let Some(rx) = self.connection_manager.pending_config_set.get_mut(&()) {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(msg) => {
                        self.show_notification(
                            format!("Setting updated: {}", msg),
                            NotificationType::Success,
                        );
                        self.clear_settings_cache();
                    }
                    Err(e) => {
                        tracing::warn!("Config set failed: {}", e);
                        self.show_notification(
                            format!("Failed to update setting: {}", e),
                            NotificationType::Error,
                        );
                    }
                }
                self.connection_manager.pending_config_set.remove(&());
            }
        }

        // When already connected, don't keep checking system state
        // This prevents flickering between connected/disconnected
        if self.connection_manager.connection.is_connected() {
            return notification_shown;
        }

        // When disconnected, check if externally connected
        if self.connection_manager.connection == ConnectionState::Disconnected
            && self.vpn_state.is_connected()
        {
            let server = self
                .vpn_state
                .get_connected_server_name()
                .unwrap_or_else(|| "Unknown".to_string());
            self.connection_manager.connection = ConnectionState::Connected {
                server,
                ip: String::new(),
                city: None,
                country: None,
            };
        }

        notification_shown
    }

    pub fn refresh_servers(&mut self) {
        tracing::info!("Refreshing server list");
        let cached = self.vpn_state.servers();
        if !cached.is_empty() {
            self.set_servers(cached);
        }

        self.show_notification("Refreshing servers...".to_string(), NotificationType::Info);

        let (tx, rx) = create_channel();
        self.connection_manager.pending_refresh.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_refresh_servers(self.vpn_state.clone(), tx);
    }

    pub fn connect(&mut self) {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
            );
            return;
        }
        let Some(idx) = self.ui_state.selected_server else {
            self.show_notification("No server selected".to_string(), NotificationType::Error);
            return;
        };

        let filtered = self.filtered_servers();
        let server = match filtered.get(idx) {
            Some(server) => server,
            None => {
                self.show_notification("No server selected".to_string(), NotificationType::Error);
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
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect(self.vpn_state.clone(), server_id, tx);
    }

    pub fn connect_random(&mut self) {
        tracing::info!("Connecting to random server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to random server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_random(self.vpn_state.clone(), tx);
    }

    pub fn connect_fastest(&mut self) {
        tracing::info!("Connecting to fastest server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to fastest server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_fastest(self.vpn_state.clone(), tx);
    }

    pub fn connect_p2p(&mut self) {
        tracing::info!("Connecting to P2P server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to P2P server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_p2p(self.vpn_state.clone(), tx);
    }

    pub fn connect_tor(&mut self) {
        tracing::info!("Connecting to Tor server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to Tor server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_tor(self.vpn_state.clone(), tx);
    }

    pub fn connect_securecore(&mut self) {
        tracing::info!("Connecting to SecureCore server");
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(
            "Connecting to SecureCore server...".to_string(),
            NotificationType::Info,
        );

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_securecore(self.vpn_state.clone(), tx);
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
        );

        let (tx, rx) = create_channel();
        self.connection_manager.pending_disconnect.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_disconnect(self.vpn_state.clone(), tx);
    }

    pub fn connect_city(&mut self, city: &str) {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification(
                "Still connecting, please wait...".to_string(),
                NotificationType::Info,
            );
            return;
        }

        let city = city.to_string();
        self.connection_manager.previous_connection =
            Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(format!("Connecting to {}...", city), NotificationType::Info);

        let (tx, rx): (
            mpsc::Sender<AsyncResult<ConnectResult>>,
            mpsc::Receiver<AsyncResult<ConnectResult>>,
        ) = create_channel();
        self.connection_manager.pending_connect_city.insert((), rx);
        self.connection_manager
            .async_manager
            .spawn_connect_city(self.vpn_state.clone(), city, tx);
    }

    pub fn spawn_config_set(&mut self, key: String, value: String) {
        let (tx, rx) = create_channel();
        self.connection_manager.pending_config_set.insert((), rx);
        self.connection_manager.async_manager.spawn_config_set(
            self.vpn_state.clone(),
            key,
            value,
            tx,
        );
    }

    /// Get filtered and sorted server list
    pub fn filtered_servers(&self) -> Vec<Server> {
        if let Some(cached) = self.server_cache.get_cached() {
            return cached;
        }

        let result = self.compute_filtered_servers();
        self.server_cache.set_cached(result.clone());
        result
    }

    pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
        let query = &self.ui_state.search_query.query_lower;

        let connected_server_id = match &self.connection_manager.connection {
            ConnectionState::Connected { server, .. } => Some(server.clone()),
            _ => None,
        };

        // Always get servers from VPN state cache (includes cities)
        let servers = self.vpn_state.servers();
        let mut result: Vec<Server> = if query.is_empty() {
            servers.clone()
        } else {
            servers
                .iter()
                .filter(|server| {
                    let q = query.as_str();
                    let matches_code = server.code.to_lowercase().contains(q);
                    let matches_country = server.country.to_lowercase().contains(q);
                    let matches_city = server
                        .cities
                        .iter()
                        .any(|c| c.name.to_lowercase().contains(q));
                    let matches_fuzzy = self.fuzzy_match(&servers, &server.country, q);

                    match self.ui_state.filter {
                        ServerFilter::Code => matches_code || matches_fuzzy,
                        ServerFilter::Country => matches_country || matches_fuzzy,
                        ServerFilter::City => matches_city || matches_fuzzy,
                    }
                })
                .cloned()
                .collect()
        };

        match (self.ui_state.sort, self.ui_state.sort_direction) {
            (ServerSort::Code, SortDirection::Asc) => {
                result.sort_by(|a, b| a.code.to_lowercase().cmp(&b.code.to_lowercase()))
            }
            (ServerSort::Code, SortDirection::Desc) => {
                result.sort_by(|a, b| b.code.to_lowercase().cmp(&a.code.to_lowercase()))
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
                if let Some(pos) = result.iter().position(|s| {
                    connected_id.starts_with(&s.code) || s.code.starts_with(connected_id)
                }) {
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
            if server.code.to_lowercase() == query {
                variants.push(server.country.to_lowercase());
                break;
            }
        }

        variants
    }

    fn invalidate_filtered_cache(&mut self) {
        self.server_cache.invalidate();
    }

    pub fn cycle_filter(&mut self) {
        self.ui_state.filter = self.ui_state.filter.next();
        self.invalidate_filtered_cache();
    }

    pub fn cycle_sort(&mut self) {
        self.ui_state.sort_direction = self.ui_state.sort_direction.toggle();
        self.invalidate_filtered_cache();
    }

    pub fn cycle_sort_field(&mut self) {
        self.ui_state.sort = self.ui_state.sort.next();
        self.invalidate_filtered_cache();
    }

    pub fn set_sort_by_code(&mut self) {
        self.ui_state.sort = ServerSort::Code;
        self.invalidate_filtered_cache();
    }

    pub fn set_sort_by_country(&mut self) {
        self.ui_state.sort = ServerSort::Country;
        self.invalidate_filtered_cache();
    }

    pub fn toggle_sort_direction(&mut self) {
        self.ui_state.sort_direction = self.ui_state.sort_direction.toggle();
        self.invalidate_filtered_cache();
    }

    pub fn set_filter(&mut self, filter: ServerFilter) {
        self.ui_state.filter = filter;
        self.invalidate_filtered_cache();
    }

    pub fn select_next(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_next(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_prev(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_prev(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    fn switch_cities_to_selected(&mut self) {
        if let Some(idx) = self.ui_state.selected_server {
            if let Some(server) = self.filtered_servers().get(idx) {
                let country_code = &server.code;

                if self.current_country_code.as_deref() != Some(country_code) {
                    self.current_cities.clear();
                    self.current_country_code = Some(country_code.to_string());
                    self.fetch_cities(country_code);
                }
            }
        }
    }

    fn fetch_cities(&mut self, country_code: &str) {
        let country_code = country_code.to_string();

        if let Some(cities) = self.vpn_state.cached_cities(&country_code) {
            self.current_cities = cities;
            return;
        }

        self.show_notification(
            format!("Loading cities for {}...", country_code),
            NotificationType::Info,
        );

        let (tx, rx) = create_channel();
        self.connection_manager
            .pending_cities
            .insert(country_code.clone(), rx);
        self.connection_manager.async_manager.spawn_cities(
            self.vpn_state.clone(),
            country_code,
            tx,
        );
    }

    pub fn reload_cities(&mut self) {
        if let Some(country_code) = self.current_country_code.clone() {
            self.current_cities.clear();

            if let Err(e) = self.vpn_state.clear_cities_cache(&country_code) {
                tracing::warn!("Failed to clear cities cache: {}", e);
            }

            self.show_notification(
                format!("Loading cities for {}...", country_code),
                NotificationType::Info,
            );

            let (tx, rx) = create_channel();
            self.connection_manager
                .pending_cities
                .insert(country_code.clone(), rx);
            self.connection_manager.async_manager.spawn_cities(
                self.vpn_state.clone(),
                country_code,
                tx,
            );
        }
    }

    pub fn select_first(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_first(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_last(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_last(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_page_down(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_page_down(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_page_up(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_page_up(self.get_selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn move_to_cities(&mut self) {
        if let Some(idx) = self.ui_state.selected_server {
            let servers = self.filtered_servers();
            if let Some(server) = servers.get(idx) {
                self.current_cities.clear();
                self.current_country_code = Some(server.code.clone());
                self.fetch_cities(&server.code);
            }
        }
        self.ui_state.pane_focus = Pane::Cities;
    }

    pub fn move_to_countries(&mut self) {
        self.ui_state.pane_focus = Pane::Countries;
    }

    pub fn city_select_next(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_next(bounds);
        }
    }

    pub fn city_select_prev(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_prev(bounds);
        }
    }

    pub fn city_select_first(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_first(bounds);
        }
    }

    pub fn city_select_last(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_last(bounds);
        }
    }

    pub fn city_select_page_down(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_page_down(bounds);
        }
    }

    pub fn city_select_page_up(&mut self) {
        let bounds = self.current_cities.len();
        if bounds > 0 {
            self.ui_state.selected_city.move_page_up(bounds);
        }
    }

    fn get_selection_bounds(&self) -> usize {
        if self.ui_state.current_view == crate::state::AppView::Servers
            && self.ui_state.pane_focus == Pane::Cities
        {
            self.current_cities.len()
        } else {
            self.filtered_servers().len()
        }
    }

    pub fn settings_select_next(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_next(count);
    }

    pub fn settings_select_prev(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_prev(count);
    }

    pub fn settings_select_first(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_first(count);
    }

    pub fn settings_select_last(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_last(count);
    }

    pub fn settings_select_page_down(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_page_down(count);
    }

    pub fn settings_select_page_up(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_page_up(count);
    }

    pub fn logs_select_next(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_next(bounds);
        }
    }

    pub fn logs_select_prev(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_prev(bounds);
        }
    }

    pub fn logs_select_first(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_first(bounds);
        }
    }

    pub fn logs_select_last(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_last(bounds);
        }
    }

    pub fn logs_select_page_down(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_page_down(bounds);
        }
    }

    pub fn logs_select_page_up(&mut self) {
        let bounds = self.notification_state.notification_log.len();
        if bounds > 0 {
            self.ui_state.logs_selected.move_page_up(bounds);
        }
    }

    pub fn toggle_settings(&mut self, index: usize) {
        let key = match SettingKey::from_index(index) {
            Some(k) => k,
            None => {
                self.show_notification(
                    "Invalid setting selection".to_string(),
                    NotificationType::Error,
                );
                return;
            }
        };

        if key == SettingKey::Dns {
            self.ui_state.input_mode = InputMode::DnsInput;
            self.ui_state.dns_input = String::new();
            self.show_notification(
                "Enter DNS IPs (e.g., 1.1.1.1,9.9.9.9)".to_string(),
                NotificationType::Info,
            );
            return;
        }

        if key == SettingKey::Theme {
            self.ui_state.is_dark_theme = !self.ui_state.is_dark_theme;
            tracing::info!(
                "Theme changed to {}",
                if self.ui_state.is_dark_theme {
                    "Dark"
                } else {
                    "Light"
                }
            );
            self.show_notification(
                format!(
                    "Theme changed to {}",
                    if self.ui_state.is_dark_theme {
                        "Dark"
                    } else {
                        "Light"
                    }
                ),
                NotificationType::Info,
            );
            return;
        }

        let ps = self.config_state.proton_settings_cache.as_ref();
        let result = match key {
            SettingKey::Killswitch => {
                let current = ps.and_then(|p| p.killswitch);
                self.vpn_state.toggle_killswitch(current)
            }
            SettingKey::Ipv6 => {
                let current = ps.and_then(|p| p.ipv6);
                self.vpn_state.toggle_ipv6(current)
            }
            SettingKey::Dns => unreachable!(),
            SettingKey::NetShield => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.netshield);
                let next = 0;
                self.vpn_state.set_netshield(current, next)
            }
            SettingKey::ModerateNat => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.moderate_nat);
                self.vpn_state.toggle_moderate_nat(current)
            }
            SettingKey::VpnAccelerator => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.vpn_accelerator);
                self.vpn_state.toggle_vpn_accelerator(current)
            }
            SettingKey::PortForwarding => {
                let current = ps
                    .and_then(|p| p.features.as_ref())
                    .and_then(|f| f.port_forwarding);
                self.vpn_state.toggle_port_forwarding(current)
            }
            SettingKey::Theme => unreachable!(),
        };

        match result {
            Ok(msg) => {
                tracing::info!("Setting updated: {}", msg);
                self.show_notification(
                    format!("Setting updated: {}", msg),
                    NotificationType::Success,
                );
                self.config_state.proton_settings_cache = None;
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
        let key = match SettingKey::from_index(index) {
            Some(k) => k,
            None => return,
        };

        if key == SettingKey::Dns {
            let ps = self.config_state.proton_settings_cache.as_ref();
            let dns_enabled = ps.map(|p| p.custom_dns.enabled).unwrap_or(false);
            if dns_enabled {
                match self.vpn_state.disable_custom_dns() {
                    Ok(msg) => {
                        self.show_notification(
                            format!("DNS disabled: {}", msg),
                            NotificationType::Success,
                        );
                        self.config_state.proton_settings_cache = None;
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

        for ip in &ips {
            if !is_valid_ip(ip) {
                self.show_notification(
                    format!("Invalid IP address: {}", ip),
                    NotificationType::Error,
                );
                return;
            }
        }

        let dns_list = ips.join(",");
        let result = self.vpn_state.set_custom_dns(&dns_list);

        match result {
            Ok(msg) => {
                self.show_notification(format!("DNS updated: {}", msg), NotificationType::Success);
                self.config_state.proton_settings_cache = None;
            }
            Err(e) => {
                self.show_notification(
                    format!("Failed to update DNS: {}", e),
                    NotificationType::Error,
                );
            }
        }
    }

    pub fn apply_setting(&mut self, key: &str, value: &str) -> Result<String, String> {
        self.vpn_state
            .set_config(key, value)
            .map_err(|e| e.to_string())
    }

    pub fn clear_settings_cache(&mut self) {
        self.config_state.clear_cache();
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
                code: "JP".to_string(),
                country: "Japan".to_string(),
                cities: vec![
                    City::new("Tokyo".to_string()),
                    City::new("Osaka".to_string()),
                ],
            },
            Server {
                code: "US".to_string(),
                country: "United States".to_string(),
                cities: vec![City::new("New York".to_string())],
            },
            Server {
                code: "DE".to_string(),
                country: "Germany".to_string(),
                cities: vec![City::new("Berlin".to_string())],
            },
            Server {
                code: "GB".to_string(),
                country: "United Kingdom".to_string(),
                cities: vec![City::new("London".to_string())],
            },
            Server {
                code: "FR".to_string(),
                country: "France".to_string(),
                cities: vec![City::new("Paris".to_string())],
            },
        ]
    }

    #[test]
    fn test_filtered_servers_empty_query() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());

        let result = state.filtered_servers();

        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_filtered_servers_by_id() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("jp".to_string());
        state.ui_state.filter = ServerFilter::Code;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "JP");
    }

    #[test]
    fn test_filtered_servers_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("japan".to_string());
        state.ui_state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_by_country_exact_match() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("JP".to_string());
        state.ui_state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "JP");
    }

    #[test]
    fn test_filtered_servers_by_city() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("tokyo".to_string());
        state.ui_state.filter = ServerFilter::City;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert!(result[0].cities.iter().any(|c| c.name == "Tokyo"));
    }

    #[test]
    fn test_filtered_servers_case_insensitive() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("JAPAN".to_string());
        state.ui_state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_id() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = ServerSort::Code;
        state.ui_state.sort_direction = SortDirection::Asc;

        let result = state.filtered_servers();

        assert_eq!(result[0].code, "DE");
        assert_eq!(result[4].code, "US");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_id() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = ServerSort::Code;
        state.ui_state.sort_direction = SortDirection::Desc;

        let result = state.filtered_servers();

        assert_eq!(result[0].code, "US");
        assert_eq!(result[4].code, "DE");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = ServerSort::Country;
        state.ui_state.sort_direction = SortDirection::Asc;

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "France");
        assert_eq!(result[4].country, "United States");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = ServerSort::Country;
        state.ui_state.sort_direction = SortDirection::Desc;

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "United States");
        assert_eq!(result[4].country, "France");
    }

    #[test]
    fn test_filtered_servers_multiple_matches() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("u".to_string());
        state.ui_state.filter = ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_fuzzy_match_starts_with() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));

        let result = state.compute_filtered_servers();
        assert!(!result.is_empty());
    }
}

#[cfg(test)]
mod notification_tests {
    use super::{AppState, NotificationType};
    use crate::constants::ui::NOTIFICATION_TIMER_DEFAULT;
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

        assert_eq!(state.notification_state.notifications.len(), 2);
    }

    #[test]
    fn test_tick_notifications_removes_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);

        for _ in 0..NOTIFICATION_TIMER_DEFAULT {
            state.notification_state.tick();
        }

        assert!(state.notification_state.notifications.is_empty());
    }

    #[test]
    fn test_tick_notifications_preserves_non_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Info);

        state.notification_state.tick();

        assert_eq!(state.notification_state.notifications.len(), 2);
        assert!(state
            .notification_state
            .notifications
            .iter()
            .all(|n| n.timer < NOTIFICATION_TIMER_DEFAULT));
    }

    #[test]
    fn test_max_notifications_enforced() {
        setup();
        let mut state = AppState::new();
        for i in 0..5 {
            state.show_notification(format!("Msg {}", i), NotificationType::Info);
        }

        assert_eq!(state.notification_state.notifications.len(), 3);
        assert!(state
            .notification_state
            .notifications
            .iter()
            .any(|n| n.message == "Msg 2"));
        assert!(state
            .notification_state
            .notifications
            .iter()
            .any(|n| n.message == "Msg 3"));
        assert!(state
            .notification_state
            .notifications
            .iter()
            .any(|n| n.message == "Msg 4"));
    }

    #[test]
    fn test_notification_log_preserves_all() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Msg 1".to_string(), NotificationType::Info);
        state.show_notification("Msg 2".to_string(), NotificationType::Error);

        assert_eq!(state.notification_state.notification_log.len(), 2);
    }

    #[test]
    fn test_clear_notifications() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info);
        state.show_notification("Test 2".to_string(), NotificationType::Error);

        state.notification_state.notifications.clear();

        assert!(state.notification_state.notifications.is_empty());
        assert_eq!(state.notification_state.notification_log.len(), 2);
    }
}

fn is_valid_ip(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts
        .iter()
        .all(|p| p.chars().all(|c| c.is_ascii_digit()) && p.parse::<u8>().is_ok())
}
