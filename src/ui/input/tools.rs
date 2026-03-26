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
            input.state.ui_state.settings_last_key_g = false;

            let selected_idx = input.state.ui_state.settings_selected.unwrap_or(0);
            if let Some(key) = SettingKey::from_index(selected_idx) {
                let current_index = match key {
                    SettingKey::Theme => input.state.ui_state.theme_mode.index(),
                    SettingKey::Footer => {
                        if input.state.ui_state.show_footer {
                            0
                        } else {
                            1
                        }
                    }
                    _ => {
                        let ps = input.state.config_state.proton_settings_cache.as_ref();
                        get_setting_index(key, ps)
                    }
                };
                input.state.ui_state.settings_option_selected = current_index;
            } else {
                input.state.ui_state.settings_option_selected = 0;
            }
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
                    let new_mode = ThemeMode::from_index(option_idx).unwrap_or(ThemeMode::System);
                    input.state.save_theme(new_mode);
                    input.state.ui_state.preview_theme_mode = None;
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
            input.state.ui_state.preview_theme_mode = None;
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

                    if key == SettingKey::Theme {
                        let option_idx = input.state.ui_state.settings_option_selected;
                        let preview_mode =
                            ThemeMode::from_index(option_idx).unwrap_or(ThemeMode::System);
                        input.state.ui_state.preview_theme_mode = Some(preview_mode);
                    }
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

            if let Some(idx) = input.state.ui_state.settings_selected {
                if let Some(key) = SettingKey::from_index(idx) {
                    if key == SettingKey::Theme {
                        let option_idx = input.state.ui_state.settings_option_selected;
                        let preview_mode =
                            ThemeMode::from_index(option_idx).unwrap_or(ThemeMode::System);
                        input.state.ui_state.preview_theme_mode = Some(preview_mode);
                    }
                }
            }
            None
        }
        (true, KeyCode::Char('n')) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(idx) = input.state.ui_state.settings_selected {
                if let Some(key) = SettingKey::from_index(idx) {
                    let opt_count = key.selectable_option_count();
                    input.state.ui_state.settings_option_selected =
                        (input.state.ui_state.settings_option_selected + 1)
                            .min(opt_count.saturating_sub(1));

                    if key == SettingKey::Theme {
                        let option_idx = input.state.ui_state.settings_option_selected;
                        let preview_mode =
                            ThemeMode::from_index(option_idx).unwrap_or(ThemeMode::System);
                        input.state.ui_state.preview_theme_mode = Some(preview_mode);
                    }
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

            if let Some(idx) = input.state.ui_state.settings_selected {
                if let Some(key) = SettingKey::from_index(idx) {
                    if key == SettingKey::Theme {
                        let option_idx = input.state.ui_state.settings_option_selected;
                        let preview_mode =
                            ThemeMode::from_index(option_idx).unwrap_or(ThemeMode::System);
                        input.state.ui_state.preview_theme_mode = Some(preview_mode);
                    }
                }
            }
            None
        }

        _ => None,
    }
}

fn get_setting_index(
    key: SettingKey,
    proton_settings: Option<&crate::config::ProtonSettings>,
) -> usize {
    let ps = match proton_settings {
        Some(p) => p,
        None => return 0,
    };

    match key {
        SettingKey::Killswitch => match ps.killswitch {
            Some(0) => 0,
            Some(1) => 1,
            _ => 0,
        },
        SettingKey::Ipv6 => {
            if ps.ipv6 == Some(true) {
                1
            } else {
                0
            }
        }
        SettingKey::Dns => {
            if ps.custom_dns.enabled {
                1
            } else {
                0
            }
        }
        SettingKey::NetShield => match ps.features.as_ref().and_then(|f| f.netshield) {
            Some(0) => 0,
            Some(1) => 1,
            Some(2) => 2,
            _ => 0,
        },
        SettingKey::ModerateNat => ps
            .features
            .as_ref()
            .and_then(|f| f.moderate_nat)
            .map(|v| if v { 1 } else { 0 })
            .unwrap_or(0),
        SettingKey::VpnAccelerator => ps
            .features
            .as_ref()
            .and_then(|f| f.vpn_accelerator)
            .map(|v| if v { 1 } else { 0 })
            .unwrap_or(0),
        SettingKey::PortForwarding => ps
            .features
            .as_ref()
            .and_then(|f| f.port_forwarding)
            .map(|v| if v { 1 } else { 0 })
            .unwrap_or(0),
        SettingKey::AnonymousCrashReports => ps
            .anonymous_crash_reports
            .map(|v| if v { 1 } else { 0 })
            .unwrap_or(0),
        SettingKey::Theme => 0,
        SettingKey::Footer => 0,
    }
}
