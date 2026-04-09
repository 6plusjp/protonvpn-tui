use crate::state::{AppState, AppView, Pane};
use crate::ui::styles::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

fn hint_bracket_open(theme: &Theme) -> Span<'static> {
    Span::styled("[", Style::default().fg(theme.dim))
}

fn hint_bracket_close(theme: &Theme) -> Span<'static> {
    Span::styled("]", Style::default().fg(theme.dim))
}

fn hint_key<'a>(theme: &Theme, key: &'a str) -> Span<'a> {
    Span::styled(key, Style::default().fg(theme.warning))
}

fn hint_action<'a>(theme: &Theme, action: &'a str) -> Span<'a> {
    Span::styled(action, Style::default().fg(theme.foreground))
}

fn hint_spacer() -> Span<'static> {
    Span::styled("  ", Style::default())
}

fn hint<'a>(theme: &Theme, key: &'a str, action: &'a str) -> Vec<Span<'a>> {
    vec![
        hint_bracket_open(theme),
        hint_key(theme, key),
        hint_bracket_close(theme),
        hint_action(theme, action),
    ]
}

fn hint_with_spacer<'a>(theme: &Theme, key: &'a str, action: &'a str) -> Vec<Span<'a>> {
    let mut spans = vec![hint_spacer()];
    spans.extend(hint(theme, key, action));
    spans
}

pub fn render_footer(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.theme();
    let action_spans = get_footer_action_hints(state);
    let mut text = Line::from(action_spans);
    text.spans.push(Span::raw("  "));
    text.spans.extend(hint(&theme, "Tab", "switch view"));
    text.spans.extend(hint_with_spacer(&theme, "?", "help"));
    text.spans.extend(hint_with_spacer(&theme, "q", "quit"));

    f.render_widget(Paragraph::new(text), area);
}

fn get_footer_action_hints(state: &AppState) -> Vec<Span<'_>> {
    let theme = state.theme();
    let is_disconnected = state.connection_manager.connection.is_disconnected();

    match (state.ui_state.current_view, state.ui_state.pane_focus) {
        (AppView::Servers, Pane::Countries) => {
            let mut hints = vec![];
            hints.extend(hint(&theme, "j/k", "navigate"));
            hints.extend(hint_with_spacer(&theme, "l", "cities"));
            hints.extend(hint_with_spacer(&theme, "a", "sort"));
            hints.extend(hint_with_spacer(&theme, "c", "connect"));
            if !is_disconnected {
                hints.extend(hint_with_spacer(&theme, "d", "disconnect"));
            }
            hints.extend(hint_with_spacer(&theme, "r", "refresh"));
            hints.extend(hint_with_spacer(&theme, "f", "fastest"));
            hints.extend(hint_with_spacer(&theme, "p", "p2p"));
            hints.extend(hint_with_spacer(&theme, "t", "tor"));
            hints.extend(hint_with_spacer(&theme, "s", "securecore"));
            hints
        }
        (AppView::Servers, Pane::Cities) => {
            let mut hints = vec![];
            hints.extend(hint(&theme, "j/k", "navigate"));
            hints.extend(hint_with_spacer(&theme, "h", "countries"));
            hints.extend(hint_with_spacer(&theme, "a", "sort"));
            hints.extend(hint_with_spacer(&theme, "c", "connect"));
            if !is_disconnected {
                hints.extend(hint_with_spacer(&theme, "d", "disconnect"));
            }
            hints.extend(hint_with_spacer(&theme, "r", "refresh"));
            hints.extend(hint_with_spacer(&theme, "f", "fastest"));
            hints.extend(hint_with_spacer(&theme, "p", "p2p"));
            hints.extend(hint_with_spacer(&theme, "t", "tor"));
            hints.extend(hint_with_spacer(&theme, "s", "secure core"));
            hints
        }
        (AppView::Tools, Pane::Settings) => {
            let mut hints = vec![];
            hints.extend(hint(&theme, "j/k", "navigate"));
            hints.extend(hint_with_spacer(&theme, "l", "logs"));
            hints.extend(hint_with_spacer(&theme, "Enter", "toggle expand"));
            hints.extend(get_global_connect_hints(state, is_disconnected));
            hints
        }
        (AppView::Tools, Pane::Logs) => {
            let mut hints = vec![];
            hints.extend(hint(&theme, "j/k", "navigate"));
            hints.extend(hint_with_spacer(&theme, "h", "settings"));
            hints.extend(get_global_connect_hints(state, is_disconnected));
            hints
        }
        (AppView::Help, _) => {
            let mut hints = vec![
                hint_bracket_open(&theme),
                hint_key(&theme, "Esc"),
                Span::styled("/", Style::default().fg(theme.dim)),
                hint_key(&theme, "?"),
                hint_bracket_close(&theme),
                hint_action(&theme, "return"),
            ];
            hints.extend(get_global_connect_hints(state, is_disconnected));
            hints
        }
        _ => vec![],
    }
}

fn get_global_connect_hints(state: &AppState, is_disconnected: bool) -> Vec<Span<'_>> {
    let theme = state.theme();
    let mut hints = vec![];
    if !is_disconnected {
        hints.extend(hint_with_spacer(&theme, "d", "disconnect"));
    }
    hints.extend(hint_with_spacer(&theme, "f", "fastest"));
    hints.extend(hint_with_spacer(&theme, "p", "p2p"));
    hints.extend(hint_with_spacer(&theme, "t", "tor"));
    hints.extend(hint_with_spacer(&theme, "s", "securecore"));
    hints.extend(hint_with_spacer(&theme, "r", "random"));
    hints
}
