use crate::state::AppState;
use ratatui::{
    layout::{Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_filter_input(
    state: &AppState,
    f: &mut Frame<'_>,
    area: Rect,
    filter_mode: bool,
    filter_input: &str,
    has_filter_active: bool,
) {
    let theme = state.theme();
    let prompt = "search: ";
    let placeholder = "Esc to cancel...";

    let input_text = if filter_mode {
        filter_input
    } else if has_filter_active {
        state.ui_state.search_query.as_str()
    } else {
        ""
    };

    let inner_area = area.inner(Margin::new(1, 0));
    let cursor = prompt.len() + input_text.len();

    let line = if input_text.is_empty() {
        Line::from(vec![
            Span::styled(prompt, Style::default().fg(theme.primary)),
            Span::styled(placeholder, Style::default().fg(theme.dim)),
        ])
    } else {
        let text_style = if filter_mode {
            theme.foreground
        } else {
            theme.success
        };
        Line::from(vec![
            Span::styled(prompt, Style::default().fg(theme.primary)),
            Span::styled(input_text, Style::default().fg(text_style)),
        ])
    };

    let paragraph = Paragraph::new(line);

    f.render_widget(paragraph, inner_area);

    if filter_mode && inner_area.width > cursor as u16 {
        f.set_cursor_position((inner_area.x + cursor as u16, area.y));
    }
}

pub fn render_dns_input(state: &AppState, f: &mut Frame<'_>, area: Rect) {
    let theme = state.theme();
    let prompt = "DNS IPs: ";
    let placeholder = "comma-separated IPs(eg. 1.1.1.1,9.9.9.9)...";
    let dns_input = &state.ui_state.dns_input;

    let inner_area = area.inner(Margin::new(2, 0));
    let cursor = prompt.len() + dns_input.len();

    let line = if dns_input.is_empty() {
        Line::from(vec![
            Span::styled(prompt, Style::default().fg(theme.primary)),
            Span::styled(placeholder, Style::default().fg(theme.dim)),
        ])
    } else {
        Line::from(vec![
            Span::styled(prompt, Style::default().fg(theme.primary)),
            Span::styled(dns_input.as_str(), Style::default().fg(theme.foreground)),
        ])
    };

    let paragraph = Paragraph::new(line);

    f.render_widget(paragraph, inner_area);

    if inner_area.width > cursor as u16 {
        f.set_cursor_position((inner_area.x + cursor as u16, area.y));
    }
}
