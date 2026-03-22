use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app_action::AppAction;
use super::common::handle_common_navigation;
use super::InputState;
use crate::config::SettingKey;
use crate::state::{InputMode, Pane};
use crate::ui::styles::ThemeMode;

pub fn handle_tools_key(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    let pane = input.state.ui_state.pane_focus;

    match key_event.code {
        KeyCode::Char('h') => {
            input.state.ui_state.pane_focus = Pane::Settings;
            None
        }
        KeyCode::Char('l') => {
            input.state.ui_state.pane_focus = Pane::Logs;
            None
        }
        _ => match pane {
            Pane::Settings => handle_settings_pane_key(input, key_event),
            Pane::Logs => {
                if handle_common_navigation(input, key_event) {
                    return None;
                }
                None
            }
            _ => None,
        },
    }
}

fn handle_settings_pane_key(input: &mut InputState, key_event: KeyEvent) -> Option<AppAction> {
    let expanded = input.state.ui_state.settings_expanded;

    match (expanded, key_event.code) {
        (false, KeyCode::Enter) => {
            input.state.ui_state.settings_expanded = true;
            input.state.ui_state.settings_option_selected = 0;
            input.state.ui_state.settings_last_key_g = false;
            None
        }
        (false, KeyCode::Char('c')) => {
            if let Some(idx) = input.state.ui_state.selected_city {
                let city_name = input.state.current_cities.get(idx).map(|c| c.name.clone());
                if let Some(name) = city_name {
                    input.state.connect_city(&name);
                }
            } else {
                input.state.connect();
            }
            None
        }
        (false, KeyCode::Char('j') | KeyCode::Down | KeyCode::Char('n'))
            if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            input.state.ui_state.settings_last_key_g = false;
            input.state.settings_select_next();
            None
        }
        (false, KeyCode::Char('k') | KeyCode::Up | KeyCode::Char('p'))
            if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            input.state.ui_state.settings_last_key_g = false;
            input.state.settings_select_prev();
            None
        }
        (false, KeyCode::Char('g')) => {
            if input.state.ui_state.settings_last_key_g {
                input.state.ui_state.settings_last_key_g = false;
                input.state.settings_select_first();
            } else {
                input.state.ui_state.settings_last_key_g = true;
            }
            None
        }
        (false, KeyCode::Char('G')) => {
            input.state.ui_state.settings_last_key_g = false;
            input.state.settings_select_last();
            None
        }
        (false, _) => {
            if handle_common_navigation(input, key_event) {
                input.state.ui_state.settings_last_key_g = false;
                None
            } else {
                None
            }
        }

        (true, KeyCode::Enter) => {
            if let Some(idx) = input.state.ui_state.settings_selected {
                let key = match SettingKey::from_index(idx) {
                    Some(k) => k,
                    None => {
                        input.state.ui_state.settings_expanded = false;
                        return None;
                    }
                };

                if key == SettingKey::Dns {
                    input.state.ui_state.settings_expanded = false;
                    input.state.ui_state.input_mode = InputMode::DnsInput;
                    input.state.ui_state.dns_input = String::new();
                    input.state.show_notification(
                        "Enter DNS IPs (e.g., 1.1.1.1,9.9.9.9)".to_string(),
                        crate::state::NotificationType::Info,
                        None,
                    );
                    return None;
                }

                if key == SettingKey::Theme {
                    input.state.ui_state.settings_expanded = false;
                    let option_idx = input.state.ui_state.settings_option_selected;
                    let new_mode = match option_idx {
                        0 => ThemeMode::System,
                        1 => ThemeMode::CatppuccinMocha,
                        2 => ThemeMode::CatppuccinLatte,
                        3 => ThemeMode::Dracula,
                        4 => ThemeMode::Nord,
                        5 => ThemeMode::Gruvbox,
                        6 => ThemeMode::TokyoNight,
                        _ => ThemeMode::System,
                    };
                    input.state.save_theme(new_mode);
                    let theme_name = match new_mode {
                        ThemeMode::System => "System",
                        ThemeMode::CatppuccinMocha => "Catppuccin Mocha",
                        ThemeMode::CatppuccinLatte => "Catppuccin Latte",
                        ThemeMode::Dracula => "Dracula",
                        ThemeMode::Nord => "Nord",
                        ThemeMode::Gruvbox => "Gruvbox",
                        ThemeMode::TokyoNight => "Tokyo Night",
                    };
                    input.state.show_notification(
                        format!("Theme changed to {}", theme_name),
                        crate::state::NotificationType::Info,
                        None,
                    );
                    return None;
                }

                if key == SettingKey::Footer {
                    input.state.ui_state.settings_expanded = false;
                    let option_idx = input.state.ui_state.settings_option_selected;
                    let new_show_footer = option_idx == 0;
                    input.state.save_footer(new_show_footer);
                    let status = if new_show_footer { "on" } else { "off" };
                    input.state.show_notification(
                        format!("Footer set to {}", status),
                        crate::state::NotificationType::Info,
                        None,
                    );
                    return None;
                }

                let option_idx = input.state.ui_state.settings_option_selected;
                if let Some((config_key, value)) = key.get_selectable_option_command(option_idx) {
                    input.state.spawn_config_set(config_key, value);
                    input.state.show_notification(
                        "Applying setting...".to_string(),
                        crate::state::NotificationType::Info,
                        None,
                    );
                }
            }
            input.state.ui_state.settings_expanded = false;
            None
        }
        (true, KeyCode::Esc) => {
            input.state.ui_state.settings_expanded = false;
            None
        }
        (true, KeyCode::Char('j') | KeyCode::Down)
            if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            if let Some(idx) = input.state.ui_state.settings_selected {
                if let Some(key) = SettingKey::from_index(idx) {
                    let opt_count = key.selectable_option_count();
                    input.state.ui_state.settings_option_selected =
                        (input.state.ui_state.settings_option_selected + 1)
                            .min(opt_count.saturating_sub(1));
                }
            }
            None
        }
        (true, KeyCode::Char('k') | KeyCode::Up)
            if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            input.state.ui_state.settings_option_selected = input
                .state
                .ui_state
                .settings_option_selected
                .saturating_sub(1);
            None
        }
        (true, KeyCode::Char('n')) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(idx) = input.state.ui_state.settings_selected {
                if let Some(key) = SettingKey::from_index(idx) {
                    let opt_count = key.selectable_option_count();
                    input.state.ui_state.settings_option_selected =
                        (input.state.ui_state.settings_option_selected + 1)
                            .min(opt_count.saturating_sub(1));
                }
            }
            None
        }
        (true, KeyCode::Char('p')) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            input.state.ui_state.settings_option_selected = input
                .state
                .ui_state
                .settings_option_selected
                .saturating_sub(1);
            None
        }

        _ => None,
    }
}
