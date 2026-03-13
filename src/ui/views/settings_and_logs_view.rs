use crate::state::{AppState, Pane};
use crate::ui::views::{logs_view, settings_view};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

pub fn render_settings_and_logs_view(
    state: &mut AppState,
    settings_list_state: &mut ratatui::widgets::ListState,
    logs_list_state: &mut ratatui::widgets::TableState,
    f: &mut Frame<'_>,
    area: Rect,
) {
    let theme = state.get_theme();
    let is_settings_focused = state.ui_state.pane_focus == Pane::Settings;
    let is_logs_focused = state.ui_state.pane_focus == Pane::Logs;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let settings_area = chunks[0];
    let logs_area = chunks[1];

    let settings_title = if is_settings_focused {
        " > Settings "
    } else {
        "   Settings "
    };

    let logs_title = if is_logs_focused {
        " > Logs "
    } else {
        "   Logs "
    };

    let settings_block = Block::default()
        .title(settings_title)
        .borders(Borders::ALL)
        .border_style(if is_settings_focused {
            Style::default().fg(theme.selection)
        } else {
            Style::default().fg(theme.secondary)
        });

    let logs_block = Block::default()
        .title(logs_title)
        .borders(Borders::ALL)
        .border_style(if is_logs_focused {
            Style::default().fg(theme.selection)
        } else {
            Style::default().fg(theme.secondary)
        });

    let settings_inner = settings_block.inner(settings_area);
    let logs_inner = logs_block.inner(logs_area);

    f.render_widget(settings_block, settings_area);
    f.render_widget(logs_block, logs_area);

    settings_view::render_settings_view(
        state,
        settings_list_state,
        f,
        settings_inner,
        is_settings_focused,
    );
    logs_view::render_logs_view(state, logs_list_state, f, logs_inner, is_logs_focused);
}
