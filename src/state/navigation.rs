//! Navigation and selection methods for AppState

use crate::config::SettingKey;
use crate::state::app_state::Navigatable;
use crate::state::{AppView, Pane};

/// Main application state - navigation and selection methods
impl crate::state::AppState {
    // === Server selection ===

    pub fn select_next(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_next(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_prev(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_prev(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_first(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_first(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_last(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_last(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_page_down(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_page_down(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub fn select_page_up(&mut self) {
        let old_idx = self.ui_state.selected_server;
        self.ui_state
            .selected_server
            .move_page_up(self.selection_bounds());

        if old_idx != self.ui_state.selected_server {
            self.switch_cities_to_selected();
        }
    }

    pub(crate) fn switch_cities_to_selected(&mut self) {
        if let Some(idx) = self.ui_state.selected_server {
            if let Some(server) = self.filtered_servers().get(idx) {
                let country_code = &server.code;

                if self.current_country_code.as_deref() != Some(country_code) {
                    self.current_cities.clear();
                    self.current_country_code = Some(country_code.to_string());
                    self.fetch_cities(country_code, false);
                }
            }
        }
    }

    // === City selection ===

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

    // === Settings selection ===

    pub fn settings_select_next(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_next(count);
        self.ui_state.preview_theme_mode = None;
    }

    pub fn settings_select_prev(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_prev(count);
        self.ui_state.preview_theme_mode = None;
    }

    pub fn settings_select_first(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_first(count);
        self.ui_state.preview_theme_mode = None;
    }

    pub fn settings_select_last(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_last(count);
        self.ui_state.preview_theme_mode = None;
    }

    pub fn settings_select_page_down(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_page_down(count);
        self.ui_state.preview_theme_mode = None;
    }

    pub fn settings_select_page_up(&mut self) {
        let count = SettingKey::ALL.len();
        self.ui_state.settings_selected.move_page_up(count);
        self.ui_state.preview_theme_mode = None;
    }

    // === Logs selection ===

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

    // === Pane navigation ===

    pub fn move_to_cities(&mut self) {
        if let Some(idx) = self.ui_state.selected_server {
            let servers = self.filtered_servers();
            if let Some(server) = servers.get(idx) {
                self.current_cities.clear();
                self.current_country_code = Some(server.code.clone());
                self.fetch_cities(&server.code, true);
            }
        }
        self.ui_state.pane_focus = Pane::Cities;
    }

    pub fn move_to_countries(&mut self) {
        self.ui_state.pane_focus = Pane::Countries;
    }

    // === Helpers ===

    fn selection_bounds(&self) -> usize {
        if self.ui_state.current_view == AppView::Servers
            && self.ui_state.pane_focus == Pane::Cities
        {
            self.current_cities.len()
        } else {
            self.filtered_servers().len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::log_persistence;
    use crate::state::AppState;
    use crate::vpn::{City, Server, VpnClient};

    fn setup() {
        log_persistence::set_test_mode(true);
    }

    fn make_servers() -> Vec<Server> {
        vec![
            Server {
                code: "JP".to_string(),
                code_lower: "jp".to_string(),
                country: "Japan".to_string(),
                country_lower: "japan".to_string(),
                cities: vec![
                    City::new("Tokyo".to_string()),
                    City::new("Osaka".to_string()),
                ],
            },
            Server {
                code: "US".to_string(),
                code_lower: "us".to_string(),
                country: "United States".to_string(),
                country_lower: "united states".to_string(),
                cities: vec![City::new("New York".to_string())],
            },
            Server {
                code: "DE".to_string(),
                code_lower: "de".to_string(),
                country: "Germany".to_string(),
                country_lower: "germany".to_string(),
                cities: vec![City::new("Berlin".to_string())],
            },
            Server {
                code: "GB".to_string(),
                code_lower: "gb".to_string(),
                country: "United Kingdom".to_string(),
                country_lower: "united kingdom".to_string(),
                cities: vec![City::new("London".to_string())],
            },
            Server {
                code: "FR".to_string(),
                code_lower: "fr".to_string(),
                country: "France".to_string(),
                country_lower: "france".to_string(),
                cities: vec![City::new("Paris".to_string())],
            },
        ]
    }

    #[test]
    fn test_select_next_at_last_stays() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(4);
        state.select_next();
        assert_eq!(state.ui_state.selected_server, Some(4));
    }

    #[test]
    fn test_select_prev_at_first_stays() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(0);
        state.select_prev();
        assert_eq!(state.ui_state.selected_server, Some(0));
    }

    #[test]
    fn test_select_first_last() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(2);
        state.select_first();
        assert_eq!(state.ui_state.selected_server, Some(0));
        state.select_last();
        assert_eq!(state.ui_state.selected_server, Some(4));
    }

    #[test]
    fn test_city_select_methods_with_empty_cities() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.current_cities.clear();
        state.ui_state.selected_city = None;
        state.city_select_next();
        state.city_select_prev();
        state.city_select_first();
        state.city_select_last();
        state.city_select_page_down();
        state.city_select_page_up();
        assert_eq!(state.ui_state.selected_city, None);
    }

    #[test]
    fn test_city_select_methods_with_cities() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.current_cities = vec![
            City::new("Tokyo".to_string()),
            City::new("Osaka".to_string()),
        ];
        state.ui_state.selected_city = Some(0);
        state.city_select_next();
        assert_eq!(state.ui_state.selected_city, Some(1));
        state.city_select_next();
        assert_eq!(state.ui_state.selected_city, Some(1));
        state.city_select_prev();
        assert_eq!(state.ui_state.selected_city, Some(0));
        state.city_select_first();
        assert_eq!(state.ui_state.selected_city, Some(0));
        state.city_select_last();
        assert_eq!(state.ui_state.selected_city, Some(1));
    }

    #[test]
    fn test_settings_select_methods() {
        setup();
        let mut state = AppState::new();
        let count = SettingKey::ALL.len();
        state.ui_state.settings_selected = Some(0);
        state.settings_select_next();
        assert_eq!(state.ui_state.settings_selected, Some(1));
        state.settings_select_prev();
        assert_eq!(state.ui_state.settings_selected, Some(0));
        state.settings_select_first();
        assert_eq!(state.ui_state.settings_selected, Some(0));
        state.settings_select_last();
        assert_eq!(state.ui_state.settings_selected, Some(count - 1));
    }

    #[test]
    fn test_move_to_cities_updates_pane_focus() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.sort = crate::state::ServerSort::Code;
        state.ui_state.sort_direction = crate::state::SortDirection::Asc;
        state.server_cache.invalidate();
        state.ui_state.selected_server = Some(3);
        state.ui_state.pane_focus = Pane::Countries;
        state.move_to_cities();
        assert_eq!(state.ui_state.pane_focus, Pane::Cities);
        assert_eq!(state.current_country_code, Some("JP".to_string()));
    }

    #[test]
    fn test_move_to_countries_updates_pane_focus() {
        setup();
        let mut state = AppState::new();
        state.ui_state.pane_focus = Pane::Cities;
        state.move_to_countries();
        assert_eq!(state.ui_state.pane_focus, Pane::Countries);
    }
}
