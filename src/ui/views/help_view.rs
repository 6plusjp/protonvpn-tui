use crate::config::{KeyBinding, UserConfig};
use crate::ui::components::centered_block;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::AppState;

const KEY_WIDTH: usize = 12;

fn format_keybinding(key: &KeyBinding) -> String {
    let mut s = String::new();
    if key.modifiers == crate::config::KeyModifier::Control {
        s.push_str("Ctrl+");
    }
    s.push(key.code);
    s
}

pub fn render_help_view(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.get_theme();
    let block = centered_block("Help", &theme, true);
    let bindings = &state.key_bindings;

    let categories: Vec<(&str, Vec<(String, String)>)> = vec![
        (
            "Connection",
            vec![
                (
                    format_keybinding(&bindings.connect),
                    "Connect to selected server".to_string(),
                ),
                (
                    format_keybinding(&bindings.random_connect),
                    "Random connect".to_string(),
                ),
                (
                    format_keybinding(&bindings.connect_fastest),
                    "Connect to fastest server".to_string(),
                ),
                (
                    format_keybinding(&bindings.connect_p2p),
                    "Connect to P2P server".to_string(),
                ),
                (
                    format_keybinding(&bindings.connect_tor),
                    "Connect to Tor server".to_string(),
                ),
                (
                    format_keybinding(&bindings.securecore),
                    "Connect to SecureCore server".to_string(),
                ),
                (
                    format_keybinding(&bindings.disconnect),
                    "Disconnect from VPN".to_string(),
                ),
                (
                    format_keybinding(&bindings.refresh),
                    "Refresh server list".to_string(),
                ),
            ],
        ),
        (
            "Navigation",
            vec![
                (
                    format!(
                        "{} / {}",
                        bindings.navigation_up.code, bindings.navigation_down.code
                    ),
                    "Navigate up / down".to_string(),
                ),
                (
                    "↑ / ↓".to_string(),
                    "Navigate up / down (alternative)".to_string(),
                ),
                (
                    format_keybinding(&bindings.go_first),
                    "Go to top (press twice)".to_string(),
                ),
                (
                    format_keybinding(&bindings.go_last),
                    "Go to bottom".to_string(),
                ),
                (
                    format_keybinding(&bindings.page_down),
                    "Page down".to_string(),
                ),
                (format_keybinding(&bindings.page_up), "Page up".to_string()),
                (
                    format_keybinding(&bindings.pane_next),
                    "Move to right pane".to_string(),
                ),
                (
                    format_keybinding(&bindings.pane_prev),
                    "Move to left pane".to_string(),
                ),
            ],
        ),
        (
            "Sorting",
            vec![
                (
                    format_keybinding(&bindings.sort_by_code),
                    "Sort by code".to_string(),
                ),
                (
                    format_keybinding(&bindings.sort_by_country),
                    "Sort by country".to_string(),
                ),
                ("← / →".to_string(), "Toggle sort direction".to_string()),
            ],
        ),
        (
            "View",
            vec![
                ("Tab".to_string(), "Switch view".to_string()),
                ("?".to_string(), "Show this help".to_string()),
                ("Esc".to_string(), "Return to previous view".to_string()),
                ("/".to_string(), "Open filter".to_string()),
                ("q".to_string(), "Quit application".to_string()),
            ],
        ),
    ];

    let mut help_text = Vec::new();

    let normal_style = ratatui::style::Style::default().fg(theme.foreground);

    for (i, (category_name, keybinds)) in categories.iter().enumerate() {
        if i > 0 {
            help_text.push(Line::from(""));
        }

        help_text.push(Line::from(vec![Span::styled(
            format!("[{}]", category_name),
            normal_style,
        )]));
        help_text.push(Line::from(""));

        for (key, action) in keybinds {
            let padded_key = format!("{:<width$}", key, width = KEY_WIDTH);
            let key_style = Style::default().fg(theme.warning);
            help_text.push(Line::from(vec![
                Span::styled("  ", normal_style),
                Span::styled(padded_key, key_style),
                Span::styled("  ", normal_style),
                Span::styled(action.as_str(), normal_style),
            ]));
        }
    }

    help_text.push(Line::from(""));
    help_text.push(Line::from(vec![Span::styled(
        "Key bindings can be customized in:",
        normal_style,
    )]));
    help_text.push(Line::from(vec![Span::styled(
        UserConfig::config_display_path(),
        normal_style,
    )]));

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    f.render_widget(block, area);
    f.render_widget(Paragraph::new(help_text), inner_area);
}
