//! Navigation and selection methods for AppState

use crate::config::SettingKey;
use crate::state::app_state::Navigatable;
use crate::state::{AppView, Pane};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionTarget {
    Server,
    City,
    Settings,
    Logs,
}

#[derive(Clone, Copy)]
enum NavAction {
    Next,
    Prev,
    First,
    Last,
    PageDown,
    PageUp,
}

impl crate::state::AppState {
    /// Returns true if the selection changed.
    fn navigate_inner(selection: &mut Option<usize>, bounds: usize, action: NavAction) -> bool {
        if bounds == 0 {
            return false;
        }

        let old_idx = *selection;
        match action {
            NavAction::Next => selection.move_next(bounds),
            NavAction::Prev => selection.move_prev(bounds),
            NavAction::First => selection.move_first(bounds),
            NavAction::Last => selection.move_last(bounds),
            NavAction::PageDown => selection.move_page_down(bounds),
            NavAction::PageUp => selection.move_page_up(bounds),
        }

        old_idx != *selection
    }

    fn navigate_to(&mut self, target: SelectionTarget, action: NavAction) -> bool {
        match target {
            SelectionTarget::Server => {
                let bounds = self.selection_bounds();
                if Self::navigate_inner(&mut self.ui_state.selected_server, bounds, action) {
                    self.switch_cities_to_selected();
                    return true;
                }
            }
            SelectionTarget::City => {
                let bounds = self.current_cities.len();
                return Self::navigate_inner(&mut self.ui_state.selected_city, bounds, action);
            }
            SelectionTarget::Settings => {
                let count = SettingKey::ALL.len();
                if Self::navigate_inner(&mut self.ui_state.settings_selected, count, action) {
                    self.ui_state.preview_theme_mode = None;
                    return true;
                }
            }
            SelectionTarget::Logs => {
                let bounds = self.notification_state.notification_log.len();
                return Self::navigate_inner(&mut self.ui_state.logs_selected, bounds, action);
            }
        }
        false
    }

    // Generic navigation methods with target parameter
    pub fn navigate_next(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::Next);
    }

    pub fn navigate_prev(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::Prev);
    }

    pub fn navigate_first(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::First);
    }

    pub fn navigate_last(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::Last);
    }

    pub fn navigate_page_down(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::PageDown);
    }

    pub fn navigate_page_up(&mut self, target: SelectionTarget) {
        self.navigate_to(target, NavAction::PageUp);
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
    use crate::state::AppState;
    use crate::test_helpers::test_helpers::{self, make_servers};
    use crate::vpn::{City, VpnClient};

    use test_helpers::setup;

    #[test]
    fn test_select_next_at_last_stays() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(4);
        state.navigate_next(SelectionTarget::Server);
        assert_eq!(state.ui_state.selected_server, Some(4));
    }

    #[test]
    fn test_select_prev_at_first_stays() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(0);
        state.navigate_prev(SelectionTarget::Server);
        assert_eq!(state.ui_state.selected_server, Some(0));
    }

    #[test]
    fn test_select_first_last() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.selected_server = Some(2);
        state.navigate_first(SelectionTarget::Server);
        assert_eq!(state.ui_state.selected_server, Some(0));
        state.navigate_last(SelectionTarget::Server);
        assert_eq!(state.ui_state.selected_server, Some(4));
    }

    #[test]
    fn test_city_select_methods_with_empty_cities() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.current_cities.clear();
        state.ui_state.selected_city = None;
        state.navigate_next(SelectionTarget::City);
        state.navigate_prev(SelectionTarget::City);
        state.navigate_first(SelectionTarget::City);
        state.navigate_last(SelectionTarget::City);
        state.navigate_page_down(SelectionTarget::City);
        state.navigate_page_up(SelectionTarget::City);
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
        state.navigate_next(SelectionTarget::City);
        assert_eq!(state.ui_state.selected_city, Some(1));
        state.navigate_next(SelectionTarget::City);
        assert_eq!(state.ui_state.selected_city, Some(1));
        state.navigate_prev(SelectionTarget::City);
        assert_eq!(state.ui_state.selected_city, Some(0));
        state.navigate_first(SelectionTarget::City);
        assert_eq!(state.ui_state.selected_city, Some(0));
        state.navigate_last(SelectionTarget::City);
        assert_eq!(state.ui_state.selected_city, Some(1));
    }

    #[test]
    fn test_settings_select_methods() {
        setup();
        let mut state = AppState::new();
        let count = SettingKey::ALL.len();
        state.ui_state.settings_selected = Some(0);
        state.navigate_next(SelectionTarget::Settings);
        assert_eq!(state.ui_state.settings_selected, Some(1));
        state.navigate_prev(SelectionTarget::Settings);
        assert_eq!(state.ui_state.settings_selected, Some(0));
        state.navigate_first(SelectionTarget::Settings);
        assert_eq!(state.ui_state.settings_selected, Some(0));
        state.navigate_last(SelectionTarget::Settings);
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

    #[test]
    fn test_navigate_inner_returns_false_on_zero_bounds() {
        let mut selection: Option<usize> = Some(0);
        let changed = AppState::navigate_inner(&mut selection, 0, NavAction::Next);
        assert!(!changed);
        assert_eq!(selection, Some(0));
    }

    #[test]
    fn test_navigate_inner_all_actions() {
        let mut selection: Option<usize> = Some(2);

        AppState::navigate_inner(&mut selection, 5, NavAction::First);
        assert_eq!(selection, Some(0));

        AppState::navigate_inner(&mut selection, 5, NavAction::Last);
        assert_eq!(selection, Some(4));

        AppState::navigate_inner(&mut selection, 5, NavAction::Prev);
        assert_eq!(selection, Some(3));

        AppState::navigate_inner(&mut selection, 5, NavAction::Next);
        assert_eq!(selection, Some(4));

        let changed = AppState::navigate_inner(&mut selection, 5, NavAction::Next);
        assert!(!changed);
        assert_eq!(selection, Some(4));
    }
}
