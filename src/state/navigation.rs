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
