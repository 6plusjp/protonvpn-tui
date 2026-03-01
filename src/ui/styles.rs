//! TUI styles and themes

use crossterm::style::Color;

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
        }
    }
}
