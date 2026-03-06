use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::state::{AppState, NotificationType};

pub fn render_logs_view(
    state: &AppState,
    list_state: &mut ListState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let block = Block::default().title(" Logs ").borders(Borders::ALL);

    let items: Vec<ListItem> = state
        .notification_log
        .iter()
        .map(|n| {
            let (prefix, color) = match n.notification_type {
                NotificationType::Info => ("[INFO] ", Color::Cyan),
                NotificationType::Success => ("[OK]   ", Color::Green),
                NotificationType::Error => ("[ERR]  ", Color::Red),
            };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::raw(&n.message),
            ]);
            ListItem::new(line)
        })
        .collect();

    if items.is_empty() {
        let empty_list = List::new(vec![ListItem::new("No logs yet")])
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty_list, area);
        return;
    }

    let selected = list_state.selected().unwrap_or(0).min(items.len() - 1);
    list_state.select(Some(selected));

    let list = List::new(items).block(block);

    f.render_stateful_widget(list, area, list_state);
}
