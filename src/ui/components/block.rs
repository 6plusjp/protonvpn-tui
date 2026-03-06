use crate::ui::styles::Theme;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};

pub fn centered_block(title: &str, theme: &Theme) -> Block<'static> {
    Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.block_border))
}

pub fn block_with_title(title: &str, theme: &Theme) -> Block<'static> {
    centered_block(title, theme)
}
