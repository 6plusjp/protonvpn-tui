use crate::state::format_relative_time;
use crate::state::{AppState, NotificationType};
use crate::ui::components::centered_block;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    widgets::{Cell, Row, Table, TableState},
    Frame,
};

pub fn render_logs_view(
    state: &AppState,
    table_state: &mut TableState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = state.get_theme();
    let block = centered_block("Logs", &theme);

    let logs = &state.notification_state.notification_log;
    if logs.is_empty() {
        let empty_table = Table::new(
            vec![Row::new(vec![Cell::from("No logs yet")])],
            vec![ratatui::layout::Constraint::Fill(1)],
        )
        .block(block)
        .style(Style::default().fg(theme.secondary));
        f.render_widget(empty_table, area);
        return;
    }

    let max_type_len = 6;
    let max_time_len = logs
        .iter()
        .map(|n| format_relative_time(n.timestamp).len())
        .max()
        .unwrap_or(6);

    let header_cells = vec![
        Cell::from(format!("{:<width$}", "Type", width = max_type_len)).style(Style::new().bold()),
        Cell::from("Message").style(Style::new().bold()),
        Cell::from(format!("{:<width$}", "Time", width = max_time_len + 1))
            .style(Style::new().bold()),
    ];
    let header_row = Row::new(header_cells).height(1);

    let widths = vec![
        ratatui::layout::Constraint::Length(max_type_len as u16),
        ratatui::layout::Constraint::Fill(1),
        ratatui::layout::Constraint::Length((max_time_len + 1) as u16),
    ];

    let logs_iter = logs.iter().rev();
    let selected_idx = state.ui_state.logs_selected.unwrap_or(0);

    let rows: Vec<Row> = logs_iter
        .enumerate()
        .map(|(idx, n)| {
            let is_selected = idx == selected_idx;

            let (type_str, color) = match n.notification_type {
                NotificationType::Info => ("[INFO] ", theme.primary),
                NotificationType::Success => ("[OK]   ", theme.success),
                NotificationType::Warning => ("[WARN] ", theme.warning),
                NotificationType::Error => ("[ERR]  ", theme.error),
            };

            let type_cell = format!("{:<width$}", type_str, width = max_type_len);
            let time_str = format_relative_time(n.timestamp);
            let time_cell = format!("{:<width$}", time_str, width = max_time_len + 1);

            let style = if is_selected {
                Style::default()
                    .fg(theme.foreground)
                    .bg(theme.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground)
            };

            Row::new(vec![
                Cell::from(type_cell).style(Style::default().fg(color)),
                Cell::from(n.message.clone()),
                Cell::from(time_cell),
            ])
            .style(style)
        })
        .collect();

    let selected = selected_idx.min(rows.len().saturating_sub(1));
    table_state.select(Some(selected));

    let table = Table::new(rows, widths)
        .header(header_row)
        .block(block)
        .column_spacing(2)
        .highlight_style(
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(table, area, table_state);
}
