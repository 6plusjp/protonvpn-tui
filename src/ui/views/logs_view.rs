use crate::state::format_relative_time;
use crate::state::{AppState, NotificationType};
use crate::ui::components::centered_block;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
    Frame,
};

pub fn render_logs_view(
    state: &AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = state.get_theme();
    let block = centered_block("Logs", &theme);

    let items: Vec<ListItem> = state
        .notification_log
        .iter()
        .rev()
        .enumerate()
        .map(|(idx, n)| {
            let selected = state.logs_selected.unwrap_or(0);
            let is_selected = idx == selected;
            let (prefix, color) = match n.notification_type {
                NotificationType::Info => ("  [INFO] ", theme.primary),
                NotificationType::Success => ("  [OK]   ", theme.success),
                NotificationType::Error => ("  [ERR]  ", theme.error),
            };
            let relative_time = format_relative_time(n.timestamp);
            let prefix_str = if is_selected { "> " } else { "  " };
            let line = Line::from(vec![
                Span::raw(prefix_str),
                Span::raw(format!("{:<6}", relative_time)),
                Span::styled(prefix, Style::default().fg(color)),
                Span::raw(&n.message),
            ]);
            let item = ListItem::new(line);
            if is_selected {
                item.style(Style::default().fg(theme.foreground).bg(theme.selection))
            } else {
                item
            }
        })
        .collect();

    if items.is_empty() {
        let empty_list = List::new(vec![ListItem::new("No logs yet")])
            .block(block)
            .style(Style::default().fg(theme.secondary));
        f.render_widget(empty_list, area);
        return;
    }

    let selected = state
        .logs_selected
        .map(|idx| idx.min(items.len() - 1))
        .unwrap_or(0)
        .min(items.len().saturating_sub(1));
    list_state.select(Some(selected));

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(theme.foreground));

    f.render_stateful_widget(list, area, list_state);
}
