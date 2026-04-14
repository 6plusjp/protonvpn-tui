//! UI components

use crate::ui::Theme;
use ratatui::style::{Modifier, Style};

pub mod block;
pub mod pane_table;

pub use block::block;
pub use pane_table::{CitiesTable, CountriesTable};

pub fn highlight_style(theme: &Theme, is_focused: bool) -> Style {
    if is_focused {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(theme.background)
            .bg(theme.accent)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}
