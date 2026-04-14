use crate::ui::styles::Theme;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
};

pub fn block(
    title: impl Into<String>,
    theme: &Theme,
    focused: bool,
    centered: bool,
) -> Block<'static> {
    let border_color = if focused { theme.primary } else { theme.dim };
    let title = if centered {
        format!(" {} ", title.into())
    } else {
        title.into()
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(border_color))
}
