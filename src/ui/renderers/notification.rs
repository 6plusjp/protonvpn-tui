use crate::constants::ui::{NOTIFICATION_MSG_MAX_LEN, POPUP_WIDTH_MAX, POPUP_WIDTH_MIN};
use crate::state::{AppState, NotificationType};
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn render_notification_popup(state: &AppState, f: &mut Frame<'_>) {
    let theme = state.theme();
    let terminal = f.area();

    let notifications: Vec<_> = state
        .notification_state
        .notifications
        .iter()
        .rev()
        .collect();

    for (i, notification) in notifications.iter().enumerate() {
        let position_from_bottom = i;
        let popup_height = 3u16;

        let y = terminal
            .height
            .saturating_sub(3 + (position_from_bottom as u16 * popup_height));
        if y < 1 {
            break;
        }

        let (fg_color, title): (ratatui::style::Color, Option<&str>) =
            match notification.notification_type {
                NotificationType::Info => (theme.primary, None),
                NotificationType::Success => (theme.success, None),
                NotificationType::Warning => (theme.warning, None),
                NotificationType::Error => (theme.error, None),
            };

        let raw_msg = &notification.message;
        let msg_single_line = raw_msg.replace('\n', " ");
        let max_len = NOTIFICATION_MSG_MAX_LEN;
        let message = if msg_single_line.len() > max_len {
            msg_single_line[..max_len - 3].to_string()
        } else {
            msg_single_line
        };

        let popup_width = (message.len() + 4).clamp(POPUP_WIDTH_MIN, POPUP_WIDTH_MAX) as u16;

        let x = terminal.width.saturating_sub(popup_width + 1);
        let area = Rect::new(x, y, popup_width, popup_height);

        let mut lines = Vec::new();
        if let Some(title) = title {
            lines.push(Line::from(title).centered());
        }
        lines.push(Line::from(message.as_str()).centered());

        let block = Block::bordered()
            .border_style(Style::default().fg(fg_color))
            .style(Style::default().fg(theme.foreground).bg(theme.background));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .style(Style::default().fg(theme.foreground))
            .alignment(ratatui::layout::Alignment::Center);

        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}
