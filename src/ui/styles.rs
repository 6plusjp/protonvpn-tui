//! TUI styles and themes

use ratatui::style::Color;

/// Application color scheme
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    // New fields for unified styling
    pub block_border: Color,
    pub selection: Color,
    pub connected: Color,
    pub key_hint: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            background: Color::Black,
            foreground: Color::White,
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            block_border: Color::Cyan,
            selection: Color::Cyan,
            connected: Color::Green,
            key_hint: Color::Yellow,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::White,
            foreground: Color::Black,
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            block_border: Color::Blue,
            selection: Color::Blue,
            connected: Color::Green,
            key_hint: Color::Yellow,
        }
    }
}
