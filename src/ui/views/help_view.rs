use crate::config::{KeyBinding, KeyModifier, UserConfig};
use crate::ui::components::block;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyCategory {
    Customizable,
    Fixed,
}

fn format_keybinding(key: &KeyBinding) -> String {
    let mut s = String::new();
    if key.modifiers == KeyModifier::Control {
        s.push_str("Ctrl+");
    } else if key.modifiers == KeyModifier::Shift {
        s.push_str("Shift+");
    }
    if key.code == 'g' && key.modifiers == KeyModifier::None {
        s.push_str("gg");
    } else {
        s.push(key.code);
    }
    s
}

pub fn render_help_view(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.theme();
    let block = block("Help", &theme, true, true);
    let bindings = &state.key_bindings;

    #[allow(clippy::type_complexity)]
    let categories: Vec<(&str, Vec<(&str, String, KeyCategory)>)> = vec![
        (
            "Connection",
            vec![
                (
                    "connect",
                    "Connect to selected server".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "random_connect",
                    "Random connect".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "connect_fastest",
                    "Connect to fastest server".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "connect_p2p",
                    "Connect to P2P server".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "connect_tor",
                    "Connect to Tor server".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "securecore",
                    "Connect to SecureCore server".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "disconnect",
                    "Disconnect from VPN".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "refresh",
                    "Refresh server list".to_string(),
                    KeyCategory::Customizable,
                ),
            ],
        ),
        (
            "Navigation",
            vec![
                (
                    "navigation",
                    "Navigate up / down".to_string(),
                    KeyCategory::Fixed,
                ),
                (
                    "arrow",
                    "Navigate up / down (alternative)".to_string(),
                    KeyCategory::Fixed,
                ),
                (
                    "ctrl_np",
                    "Navigate up / down (vim-style)".to_string(),
                    KeyCategory::Fixed,
                ),
                (
                    "go_first",
                    "Go to top (press twice)".to_string(),
                    KeyCategory::Fixed,
                ),
                ("go_last", "Go to bottom".to_string(), KeyCategory::Fixed),
                ("page_down", "Page down".to_string(), KeyCategory::Fixed),
                ("page_up", "Page up".to_string(), KeyCategory::Fixed),
                (
                    "pane_next",
                    "Move to right pane".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "pane_prev",
                    "Move to left pane".to_string(),
                    KeyCategory::Customizable,
                ),
            ],
        ),
        (
            "Sorting",
            vec![
                (
                    "sort_by_code",
                    "Sort by code".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "sort_by_country",
                    "Sort by country".to_string(),
                    KeyCategory::Customizable,
                ),
                (
                    "sort_direction",
                    "Toggle sort direction".to_string(),
                    KeyCategory::Fixed,
                ),
            ],
        ),
        (
            "View",
            vec![
                ("tab", "Switch view".to_string(), KeyCategory::Fixed),
                ("help", "Show this help".to_string(), KeyCategory::Fixed),
                (
                    "esc",
                    "Return to previous view".to_string(),
                    KeyCategory::Fixed,
                ),
                ("slash", "Open filter".to_string(), KeyCategory::Fixed),
                ("quit", "Quit application".to_string(), KeyCategory::Fixed),
                ("ctrl_c", "Quit application".to_string(), KeyCategory::Fixed),
            ],
        ),
    ];

    let mut help_text = Vec::new();

    let normal_style = Style::default().fg(theme.foreground);
    let customizable_style = Style::default().fg(theme.secondary);
    let fixed_style = Style::default().fg(theme.warning);

    for (i, (category_name, keybinds)) in categories.iter().enumerate() {
        if i > 0 {
            help_text.push(Line::from(""));
        }

        help_text.push(Line::from(vec![Span::styled(
            format!("[{}]", category_name),
            normal_style,
        )]));
        help_text.push(Line::from(""));

        for (action, key_display, category) in keybinds {
            let is_navigation_pair = matches!(*action, "navigation" | "arrow" | "ctrl_np");

            let key_string = if is_navigation_pair {
                match *action {
                    "navigation" => format!(
                        "{} / {}",
                        bindings.navigation_up.code, bindings.navigation_down.code
                    ),
                    "arrow" => "↑ / ↓".to_string(),
                    "ctrl_np" => "Ctrl+n / Ctrl+p".to_string(),
                    _ => key_display.to_string(),
                }
            } else if *action == "go_first" {
                format_keybinding(&bindings.go_first)
            } else if *action == "go_last" {
                format_keybinding(&bindings.go_last)
            } else if *action == "page_down" {
                format_keybinding(&bindings.page_down)
            } else if *action == "page_up" {
                format_keybinding(&bindings.page_up)
            } else if *action == "pane_next" {
                format_keybinding(&bindings.pane_next)
            } else if *action == "pane_prev" {
                format_keybinding(&bindings.pane_prev)
            } else if *action == "sort_by_code" {
                format_keybinding(&bindings.sort_by_code)
            } else if *action == "sort_by_country" {
                format_keybinding(&bindings.sort_by_country)
            } else if *action == "sort_direction" {
                format_keybinding(&bindings.sort_direction)
            } else if *action == "connect" {
                format_keybinding(&bindings.connect)
            } else if *action == "random_connect" {
                format_keybinding(&bindings.random_connect)
            } else if *action == "connect_fastest" {
                format_keybinding(&bindings.connect_fastest)
            } else if *action == "connect_p2p" {
                format_keybinding(&bindings.connect_p2p)
            } else if *action == "connect_tor" {
                format_keybinding(&bindings.connect_tor)
            } else if *action == "securecore" {
                format_keybinding(&bindings.securecore)
            } else if *action == "disconnect" {
                format_keybinding(&bindings.disconnect)
            } else if *action == "refresh" {
                format_keybinding(&bindings.refresh)
            } else if *action == "tab" {
                "Tab".to_string()
            } else if *action == "help" {
                "?".to_string()
            } else if *action == "esc" {
                "Esc".to_string()
            } else if *action == "slash" {
                "/".to_string()
            } else if *action == "quit" {
                "q".to_string()
            } else if *action == "ctrl_c" {
                "Ctrl+c".to_string()
            } else {
                key_display.to_string()
            };

            let key_style = if *category == KeyCategory::Customizable {
                customizable_style
            } else {
                fixed_style
            };

            let sep_style = Style::default().fg(theme.dim);

            const KEY_ALIGN_WIDTH: usize = 18;

            let key_with_sep: Vec<Span> = if is_navigation_pair {
                let padded = format!("{:<width$}", key_string, width = KEY_ALIGN_WIDTH);
                let padded_parts: Vec<&str> = padded.split(" / ").collect();
                vec![
                    Span::styled(padded_parts.first().unwrap_or(&"").to_string(), key_style),
                    Span::styled(" / ", sep_style),
                    Span::styled(padded_parts.get(1).unwrap_or(&"").to_string(), key_style),
                ]
            } else {
                let padded = format!("{:<width$}", key_string, width = KEY_ALIGN_WIDTH);
                vec![Span::styled(padded, key_style)]
            };

            help_text.push(Line::from({
                let mut line = vec![Span::styled("  ", normal_style)];
                line.extend(key_with_sep);
                line.push(Span::styled("  ", normal_style));
                line.push(Span::styled(key_display, normal_style));
                line
            }));
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
