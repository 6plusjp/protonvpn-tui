use crate::state::{AppState, AppView, Pane};
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_footer(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.theme();

    let action_spans = get_footer_action_hints(state);

    let mut text = Line::from(action_spans);

    let is_help_view = state.ui_state.current_view == AppView::Help;

    if is_help_view {
        text.spans.push(Span::raw("  "));
        text.spans.extend(vec![
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("Tab", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("switch view", Style::default().fg(theme.foreground)),
            Span::styled("  ", Style::default().fg(theme.dim)),
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("q", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("quit", Style::default().fg(theme.foreground)),
        ]);
    } else {
        text.spans.push(Span::raw("  "));
        text.spans.extend(vec![
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("Tab", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("switch view", Style::default().fg(theme.foreground)),
            Span::styled("  ", Style::default().fg(theme.dim)),
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("?", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("help", Style::default().fg(theme.foreground)),
            Span::styled("  ", Style::default().fg(theme.dim)),
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("q", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("quit", Style::default().fg(theme.foreground)),
        ]);
    }

    f.render_widget(Paragraph::new(text), area);
}

fn get_footer_action_hints(state: &AppState) -> Vec<Span<'_>> {
    let theme = state.theme();
    let is_disconnected = state.connection_manager.connection.is_disconnected();

    match (state.ui_state.current_view, state.ui_state.pane_focus) {
        (AppView::Servers, Pane::Countries) => {
            let mut hints = vec![
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("j/k", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("navigate", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("l", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("cities", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("c", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("connect", Style::default().fg(theme.foreground)),
            ];
            if !is_disconnected {
                hints.extend([
                    Span::styled("  ", Style::default().fg(theme.dim)),
                    Span::styled("[", Style::default().fg(theme.dim)),
                    Span::styled("d", Style::default().fg(theme.warning)),
                    Span::styled("]", Style::default().fg(theme.dim)),
                    Span::styled("disconnect", Style::default().fg(theme.foreground)),
                ]);
            }
            hints.extend([
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("r", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("refresh", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("f", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("fastest", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("p", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("p2p", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("t", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("tor", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("s", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("securecore", Style::default().fg(theme.foreground)),
            ]);
            hints
        }
        (AppView::Servers, Pane::Cities) => {
            let mut hints = vec![
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("j/k", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("navigate", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("h", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("countries", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("c", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("connect", Style::default().fg(theme.foreground)),
            ];
            if !is_disconnected {
                hints.extend([
                    Span::styled("  ", Style::default().fg(theme.dim)),
                    Span::styled("[", Style::default().fg(theme.dim)),
                    Span::styled("d", Style::default().fg(theme.warning)),
                    Span::styled("]", Style::default().fg(theme.dim)),
                    Span::styled("disconnect", Style::default().fg(theme.foreground)),
                ]);
            }
            hints.extend([
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("r", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("refresh", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("f", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("fastest", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("p", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("p2p", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("t", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("tor", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("s", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("secure core", Style::default().fg(theme.foreground)),
            ]);
            hints
        }
        (AppView::Tools, Pane::Settings) => {
            let mut hints = vec![
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("j/k", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("navigate", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("l", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("logs", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("Enter", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("toggle expand", Style::default().fg(theme.foreground)),
            ];
            hints.extend(get_global_connect_hints(state, is_disconnected));
            hints
        }
        (AppView::Tools, Pane::Logs) => {
            let mut hints = vec![
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("j/k", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("navigate", Style::default().fg(theme.foreground)),
                Span::styled("  ", Style::default().fg(theme.dim)),
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("h", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("settings", Style::default().fg(theme.foreground)),
            ];
            hints.extend(get_global_connect_hints(state, is_disconnected));
            hints
        }
        (AppView::Help, _) => {
            let mut hints = vec![
                Span::styled("[", Style::default().fg(theme.dim)),
                Span::styled("Esc", Style::default().fg(theme.warning)),
                Span::styled("/", Style::default().fg(theme.dim)),
                Span::styled("?", Style::default().fg(theme.warning)),
                Span::styled("]", Style::default().fg(theme.dim)),
                Span::styled("return", Style::default().fg(theme.foreground)),
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
        hints.extend([
            Span::styled("  ", Style::default().fg(theme.dim)),
            Span::styled("[", Style::default().fg(theme.dim)),
            Span::styled("d", Style::default().fg(theme.warning)),
            Span::styled("]", Style::default().fg(theme.dim)),
            Span::styled("disconnect", Style::default().fg(theme.foreground)),
        ]);
    }
    hints.extend([
        Span::styled("  ", Style::default().fg(theme.dim)),
        Span::styled("[", Style::default().fg(theme.dim)),
        Span::styled("f", Style::default().fg(theme.warning)),
        Span::styled("]", Style::default().fg(theme.dim)),
        Span::styled("fastest", Style::default().fg(theme.foreground)),
        Span::styled("  ", Style::default().fg(theme.dim)),
        Span::styled("[", Style::default().fg(theme.dim)),
        Span::styled("p", Style::default().fg(theme.warning)),
        Span::styled("]", Style::default().fg(theme.dim)),
        Span::styled("p2p", Style::default().fg(theme.foreground)),
        Span::styled("  ", Style::default().fg(theme.dim)),
        Span::styled("[", Style::default().fg(theme.dim)),
        Span::styled("t", Style::default().fg(theme.warning)),
        Span::styled("]", Style::default().fg(theme.dim)),
        Span::styled("tor", Style::default().fg(theme.foreground)),
        Span::styled("  ", Style::default().fg(theme.dim)),
        Span::styled("[", Style::default().fg(theme.dim)),
        Span::styled("s", Style::default().fg(theme.warning)),
        Span::styled("]", Style::default().fg(theme.dim)),
        Span::styled("securecore", Style::default().fg(theme.foreground)),
        Span::styled("  ", Style::default().fg(theme.dim)),
        Span::styled("[", Style::default().fg(theme.dim)),
        Span::styled("r", Style::default().fg(theme.warning)),
        Span::styled("]", Style::default().fg(theme.dim)),
        Span::styled("random", Style::default().fg(theme.foreground)),
    ]);
    hints
}
