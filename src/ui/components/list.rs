use crate::ui::styles::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::ListItem,
};

pub fn styled_list_item(
    text: &str,
    is_selected: bool,
    is_focused: bool,
    theme: &Theme,
) -> ListItem<'static> {
    let prefix = if is_selected { "> " } else { "  " };
    let style = if is_selected {
        if is_focused {
            Style::default()
                .fg(theme.foreground)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.foreground)
        }
    } else {
        Style::default().fg(theme.foreground)
    };

    let text_owned = text.to_string();
    ListItem::new(Line::from(vec![
        Span::raw(prefix),
        Span::styled(text_owned, style),
    ]))
}

pub fn connected_list_item(text: &str, theme: &Theme) -> ListItem<'static> {
    let prefix = "* ";
    let style = Style::default()
        .fg(theme.connected)
        .add_modifier(Modifier::BOLD);

    let text_owned = text.to_string();
    ListItem::new(Line::from(vec![
        Span::raw(prefix),
        Span::styled(text_owned, style),
    ]))
}
