use crate::ui::styles::Theme;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};

pub fn centered_block(title: &str, theme: &Theme, focused: bool) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}

pub fn block_with_title(title: String, theme: &Theme, focused: bool) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}
