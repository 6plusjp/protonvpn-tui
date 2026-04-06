use crate::config::{KeyBindings, UiConfig, UserConfig};
use crate::constants::state::PAGE_SIZE;
use crate::state::FilteredServerCache;
use crate::state::NotificationState;
use crate::state::NotificationType;
use crate::state::UiState;
use crate::ui::{KeyMap, Theme, ThemeMode};
use crate::vpn::Server;
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
    pub vpn_state: Arc<crate::vpn::VpnClient>,

    // === Connection Manager ===
    pub connection_manager: crate::state::ConnectionManager,

    // === Server Data ===
    pub(crate) servers: Vec<Server>,
    pub(crate) server_cache: FilteredServerCache,
    pub is_initialized: bool,
    pub current_cities: Vec<crate::vpn::City>,
    pub current_country_code: Option<String>,

    // === UI State  ===
    pub ui_state: UiState,

    // === Notification  ===
    pub notification_state: NotificationState,

    // === Config  ===
    pub config_state: crate::state::ConfigState,
    pub key_bindings: KeyBindings,
    pub user_config: UserConfig,

    // === KeyMap (centralized keybindings) ===
    pub keymap: KeyMap,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::from_config(
            &KeyBindings::default(),
            &UiConfig::default(),
            UserConfig::load(),
        )
    }

    pub fn from_config(
        key_bindings: &KeyBindings,
        ui_config: &UiConfig,
        user_config: UserConfig,
    ) -> Self {
        let vpn_state = Arc::new(crate::vpn::VpnClient::new());
        let servers = vpn_state.cached_servers();
        let ui_state = UiState::from_config(&ui_config.theme, ui_config.footer);

        let mut state = Self {
            vpn_state,
            connection_manager: crate::state::ConnectionManager::new(),
            ui_state,
            servers,
            current_cities: Vec::new(),
            current_country_code: None,
            notification_state: NotificationState::new(),
            config_state: crate::state::ConfigState::new(),
            server_cache: FilteredServerCache::new(),
            is_initialized: false,
            key_bindings: key_bindings.clone(),
            user_config,
            keymap: KeyMap::default(),
        };

        state.ui_state.favorite_countries =
            state.user_config.ui.favorites.iter().cloned().collect();

        state
    }

    pub fn theme(&self) -> Theme {
        let mode = self
            .ui_state
            .preview_theme_mode
            .unwrap_or(self.ui_state.theme_mode);
        Theme::from_mode(mode)
    }

    pub fn save_theme(&mut self, theme_mode: ThemeMode) {
        self.ui_state.theme_mode = theme_mode;
        let theme_str = match theme_mode {
            ThemeMode::System => "System",
            ThemeMode::CatppuccinMocha => "CatppuccinMocha",
            ThemeMode::CatppuccinLatte => "CatppuccinLatte",
            ThemeMode::Dracula => "Dracula",
            ThemeMode::Nord => "Nord",
            ThemeMode::Gruvbox => "Gruvbox",
            ThemeMode::TokyoNight => "TokyoNight",
        };
        self.user_config.ui.theme = theme_str.to_string();
        self.user_config.mark_ui_field_modified("theme");
        self.user_config.save();
    }

    pub fn save_footer(&mut self, show_footer: bool) {
        self.ui_state.show_footer = show_footer;
        self.user_config.ui.footer = show_footer;
        self.user_config.mark_ui_field_modified("footer");
        self.user_config.save();
    }

    pub fn save_favorites(&mut self) {
        self.user_config.ui.favorites = self.ui_state.favorite_countries.iter().cloned().collect();
        self.user_config.mark_ui_field_modified("favorites");
        self.user_config.save();
    }

    pub fn reload_user_config(&mut self) {
        self.user_config = UserConfig::load();
        self.ui_state.theme_mode = theme_mode_from_str(&self.user_config.ui.theme);
        self.ui_state.show_footer = self.user_config.ui.footer;
        self.ui_state.favorite_countries = self.user_config.ui.favorites.iter().cloned().collect();
    }

    pub fn switch_view(&mut self) {
        let new_view = self.ui_state.current_view.next();
        self.ui_state.previous_view = self.ui_state.current_view;
        self.ui_state.current_view = new_view;
        self.ui_state.pane_focus = new_view.default_pane();
    }

    pub fn show_notification(
        &mut self,
        message: String,
        notification_type: NotificationType,
        operation_key: Option<String>,
    ) {
        self.notification_state
            .show(message, notification_type, operation_key);
    }

    pub fn set_search_query(&mut self, query: String) {
        self.ui_state.search_query.set(query);
        self.server_cache.invalidate();

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
        self.server_cache.invalidate();

        if let Some(idx) = self.ui_state.selected_server {
            if let Some(server) = self.filtered_servers().get(idx) {
                self.current_country_code = Some(server.code.clone());
                self.fetch_cities(&server.code, true);
            }
        }
    }
}

fn theme_mode_from_str(s: &str) -> ThemeMode {
    match s {
        "System" => ThemeMode::System,
        "CatppuccinMocha" => ThemeMode::CatppuccinMocha,
        "CatppuccinLatte" => ThemeMode::CatppuccinLatte,
        "Dracula" => ThemeMode::Dracula,
        "Nord" => ThemeMode::Nord,
        "Gruvbox" => ThemeMode::Gruvbox,
        "TokyoNight" => ThemeMode::TokyoNight,
        _ => ThemeMode::System,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_helpers::{self, make_servers};
    use crate::vpn::VpnClient;

    use test_helpers::setup;

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
        state.ui_state.filter = crate::state::ServerFilter::Code;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "JP");
    }

    #[test]
    fn test_filtered_servers_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("japan".to_string());
        state.ui_state.filter = crate::state::ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_by_country_exact_match() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("JP".to_string());
        state.ui_state.filter = crate::state::ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].code, "JP");
    }

    #[test]
    fn test_filtered_servers_by_city() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("tokyo".to_string());
        state.ui_state.filter = crate::state::ServerFilter::City;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert!(result[0].cities.iter().any(|c| c.name == "Tokyo"));
    }

    #[test]
    fn test_filtered_servers_case_insensitive() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set("JAPAN".to_string());
        state.ui_state.filter = crate::state::ServerFilter::Country;

        let result = state.filtered_servers();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].country, "Japan");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_id() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = crate::state::ServerSort::Code;
        state.ui_state.sort_direction = crate::state::SortDirection::Asc;
        state.ui_state.favorite_countries.clear();
        state.server_cache.invalidate();

        let result = state.filtered_servers();

        assert_eq!(result[0].code, "DE");
        assert_eq!(result[4].code, "US");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_id() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = crate::state::ServerSort::Code;
        state.ui_state.sort_direction = crate::state::SortDirection::Desc;
        state.ui_state.favorite_countries.clear();
        state.server_cache.invalidate();

        let result = state.filtered_servers();

        assert_eq!(result[0].code, "US");
        assert_eq!(result[4].code, "DE");
    }

    #[test]
    fn test_filtered_servers_sort_asc_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = crate::state::ServerSort::Country;
        state.ui_state.sort_direction = crate::state::SortDirection::Asc;
        state.ui_state.favorite_countries.clear();
        state.server_cache.invalidate();

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "France");
        assert_eq!(result[4].country, "United States");
    }

    #[test]
    fn test_filtered_servers_sort_desc_by_country() {
        let mut state = AppState::new();
        state.vpn_state = Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.search_query.set(String::new());
        state.ui_state.sort = crate::state::ServerSort::Country;
        state.ui_state.sort_direction = crate::state::SortDirection::Desc;
        state.ui_state.favorite_countries.clear();
        state.server_cache.invalidate();

        let result = state.filtered_servers();

        assert_eq!(result[0].country, "United States");
        assert_eq!(result[4].country, "France");
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
        state.show_notification("Test 1".to_string(), NotificationType::Info, None);
        state.show_notification("Test 2".to_string(), NotificationType::Success, None);

        assert_eq!(state.notification_state.notifications.len(), 2);
    }

    #[test]
    fn test_tick_notifications_removes_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info, None);

        for _ in 0..NOTIFICATION_TIMER_DEFAULT {
            state.notification_state.tick();
        }

        assert!(state.notification_state.notifications.is_empty());
    }

    #[test]
    fn test_tick_notifications_preserves_non_expired() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test 1".to_string(), NotificationType::Info, None);
        state.show_notification("Test 2".to_string(), NotificationType::Info, None);

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
            state.show_notification(format!("Msg {}", i), NotificationType::Info, None);
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
        state.show_notification("Msg 1".to_string(), NotificationType::Info, None);
        state.show_notification("Msg 2".to_string(), NotificationType::Error, None);

        assert_eq!(state.notification_state.notification_log.len(), 2);
    }

    #[test]
    fn test_clear_notifications() {
        setup();
        let mut state = AppState::new();
        state.show_notification("Test".to_string(), NotificationType::Info, None);
        state.show_notification("Test 2".to_string(), NotificationType::Error, None);

        state.notification_state.notifications.clear();

        assert!(state.notification_state.notifications.is_empty());
        assert_eq!(state.notification_state.notification_log.len(), 2);
    }
}
