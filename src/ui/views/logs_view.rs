use crate::state::format_relative_time;
use crate::state::{AppState, NotificationType};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Wrap};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render_logs_view(
    state: &AppState,
    table_state: &mut TableState,
    f: &mut Frame<'_>,
    area: Rect,
    is_focused: bool,
) {
    let theme = state.get_theme();

    let logs = &state.notification_state.notification_log;
    if logs.is_empty() {
        let empty_table = Table::new(
            vec![Row::new(vec![Cell::from("No logs yet")])],
            vec![Constraint::Fill(1)],
        )
        .style(Style::default().fg(theme.warning));
        f.render_widget(empty_table, area);
        return;
    }

    let constraints = if logs.len() > 1 {
        [Constraint::Min(3), Constraint::Length(4)]
    } else {
        [Constraint::Fill(0), Constraint::Length(0)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let table_area = chunks[0];
    let detail_area = chunks[1];

    let max_type_len = 6;
    let max_time_len = logs
        .iter()
        .map(|n| format_relative_time(n.timestamp).len())
        .max()
        .unwrap_or(6);

    let header_cells = vec![
        Cell::from(format!("{:<width$}", "Type", width = max_type_len))
            .style(Style::new().bold().fg(theme.foreground)),
        Cell::from("Message").style(Style::new().bold().fg(theme.foreground)),
        Cell::from(format!("{:<width$}", "Time", width = max_time_len + 1))
            .style(Style::new().bold().fg(theme.foreground)),
    ];
    let header_row = Row::new(header_cells).height(1);

    let widths = vec![
        Constraint::Length(max_type_len as u16),
        Constraint::Fill(1),
        Constraint::Length((max_time_len + 1) as u16),
    ];

    let logs_iter = logs.iter().rev();
    let selected_idx = state.ui_state.logs_selected.unwrap_or(0);

    let rows: Vec<Row> = logs_iter
        .map(|n| {
            let (type_str, color) = match n.notification_type {
                NotificationType::Info => ("[INFO] ", theme.primary),
                NotificationType::Success => ("[OK]   ", theme.success),
                NotificationType::Warning => ("[WARN] ", theme.warning),
                NotificationType::Error => ("[ERR]  ", theme.error),
            };

            let type_cell = format!("{:<width$}", type_str, width = max_type_len);
            let time_str = format_relative_time(n.timestamp);
            let time_cell = format!("{:<width$}", time_str, width = max_time_len + 1);

            Row::new(vec![
                Cell::from(type_cell).style(Style::default().fg(color)),
                Cell::from(n.message.clone()).style(Style::default().fg(theme.foreground)),
                Cell::from(time_cell).style(Style::default().fg(theme.dim)),
            ])
        })
        .collect();

    let selected = selected_idx.min(rows.len().saturating_sub(1));
    table_state.select(Some(selected));

    let table = Table::new(rows, widths)
        .header(header_row)
        .column_spacing(2)
        .row_highlight_style(if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        })
        .highlight_symbol("> ");

    f.render_stateful_widget(table, table_area, table_state);

    if is_focused && logs.len() > 1 {
        let selected_log = logs.iter().rev().nth(selected_idx);
        if let Some(log) = selected_log {
            let detail_block = Block::default()
                .title(Line::from(vec![Span::styled(
                    " Details ",
                    Style::default().fg(theme.secondary),
                )]))
                .borders(Borders::NONE);

            let detail_text = Paragraph::new(log.message.as_str())
                .block(detail_block)
                .style(Style::default().fg(theme.dim))
                .wrap(Wrap { trim: true });

            f.render_widget(detail_text, detail_area);
        }
    }
}
