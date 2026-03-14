//! TUI styles and themes

use ratatui::style::Color;

/// Theme mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Terminal,
    CatppuccinMocha,
    CatppuccinLatte,
    Dracula,
    Nord,
    Gruvbox,
    TokyoNight,
}

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
    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::System => Self::system(),
            ThemeMode::Terminal => Self::terminal(),
            ThemeMode::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeMode::CatppuccinLatte => Self::catppuccin_latte(),
            ThemeMode::Dracula => Self::dracula(),
            ThemeMode::Nord => Self::nord(),
            ThemeMode::Gruvbox => Self::gruvbox(),
            ThemeMode::TokyoNight => Self::tokyo_night(),
        }
    }

    pub fn dark() -> Self {
        Self::from_mode(ThemeMode::System)
    }

    pub fn light() -> Self {
        Self::from_mode(ThemeMode::CatppuccinLatte)
    }

    pub fn system() -> Self {
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

    pub fn terminal() -> Self {
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

    pub fn catppuccin_mocha() -> Self {
        Self {
            background: Color::Rgb(0x1e, 0x1e, 0x2e),
            foreground: Color::Rgb(0xcd, 0xd6, 0xf4),
            primary: Color::Rgb(0x89, 0xb4, 0xfa),
            secondary: Color::Rgb(0x94, 0xe2, 0xd5),
            accent: Color::Rgb(0xf5, 0xc2, 0xe7),
            error: Color::Rgb(0xf3, 0x8b, 0xa8),
            success: Color::Rgb(0xa6, 0xe3, 0xa1),
            warning: Color::Rgb(0xf9, 0xe2, 0xaf),
            block_border: Color::Rgb(0x89, 0xb4, 0xfa),
            selection: Color::Rgb(0x45, 0x52, 0x6b),
            connected: Color::Rgb(0xa6, 0xe3, 0xa1),
            key_hint: Color::Rgb(0xf9, 0xe2, 0xaf),
        }
    }

    pub fn catppuccin_latte() -> Self {
        Self {
            background: Color::Rgb(0xef, 0xf1, 0xf5),
            foreground: Color::Rgb(0x4c, 0x4f, 0x69),
            primary: Color::Rgb(0x04, 0x0a, 0x9f),
            secondary: Color::Rgb(0x20, 0x9f, 0xa3),
            accent: Color::Rgb(0xea, 0x9c, 0xcb),
            error: Color::Rgb(0xd2, 0x0f, 0x1f),
            success: Color::Rgb(0x40, 0xa0, 0x2e),
            warning: Color::Rgb(0xdf, 0x8e, 0x1d),
            block_border: Color::Rgb(0x04, 0x0a, 0x9f),
            selection: Color::Rgb(0xcc, 0xd0, 0xe0),
            connected: Color::Rgb(0x40, 0xa0, 0x2e),
            key_hint: Color::Rgb(0xdf, 0x8e, 0x1d),
        }
    }

    pub fn dracula() -> Self {
        Self {
            background: Color::Rgb(0x28, 0x2a, 0x36),
            foreground: Color::Rgb(0xf8, 0xf8, 0xf2),
            primary: Color::Rgb(0xbd, 0x93, 0xf9),
            secondary: Color::Rgb(0x50, 0xfa, 0x7b),
            accent: Color::Rgb(0xff, 0x79, 0xc6),
            error: Color::Rgb(0xff, 0x55, 0x55),
            success: Color::Rgb(0x50, 0xfa, 0x7b),
            warning: Color::Rgb(0xf1, 0xfa, 0x8c),
            block_border: Color::Rgb(0xbd, 0x93, 0xf9),
            selection: Color::Rgb(0x44, 0x49, 0x5a),
            connected: Color::Rgb(0x50, 0xfa, 0x7b),
            key_hint: Color::Rgb(0xf1, 0xfa, 0x8c),
        }
    }

    pub fn nord() -> Self {
        Self {
            background: Color::Rgb(0x2e, 0x34, 0x40),
            foreground: Color::Rgb(0xec, 0xef, 0xf4),
            primary: Color::Rgb(0x88, 0xc0, 0xd0),
            secondary: Color::Rgb(0x81, 0xa1, 0xc1),
            accent: Color::Rgb(0xb4, 0x8e, 0xad),
            error: Color::Rgb(0xbf, 0x61, 0x6a),
            success: Color::Rgb(0xa3, 0xbe, 0x8c),
            warning: Color::Rgb(0xeb, 0xcb, 0x8b),
            block_border: Color::Rgb(0x88, 0xc0, 0xd0),
            selection: Color::Rgb(0x43, 0x4c, 0x5e),
            connected: Color::Rgb(0xa3, 0xbe, 0x8c),
            key_hint: Color::Rgb(0xeb, 0xcb, 0x8b),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            background: Color::Rgb(0x28, 0x28, 0x28),
            foreground: Color::Rgb(0xeb, 0xdb, 0xb2),
            primary: Color::Rgb(0xfe, 0x80, 0x19),
            secondary: Color::Rgb(0x8e, 0xc0, 0x7c),
            accent: Color::Rgb(0xd3, 0x86, 0x9b),
            error: Color::Rgb(0xfb, 0x49, 0x34),
            success: Color::Rgb(0xb8, 0xbb, 0x26),
            warning: Color::Rgb(0xfa, 0xbd, 0x2f),
            block_border: Color::Rgb(0xfe, 0x80, 0x19),
            selection: Color::Rgb(0x50, 0x49, 0x3e),
            connected: Color::Rgb(0xb8, 0xbb, 0x26),
            key_hint: Color::Rgb(0xfa, 0xbd, 0x2f),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            background: Color::Rgb(0x1a, 0x1b, 0x26),
            foreground: Color::Rgb(0xc0, 0xca, 0xf5),
            primary: Color::Rgb(0x7a, 0xa2, 0xf7),
            secondary: Color::Rgb(0x7d, 0xcf, 0xff),
            accent: Color::Rgb(0xbb, 0x9a, 0xf7),
            error: Color::Rgb(0xf7, 0x76, 0x8e),
            success: Color::Rgb(0x9e, 0xce, 0x6a),
            warning: Color::Rgb(0xe0, 0xaf, 0x68),
            block_border: Color::Rgb(0x7a, 0xa2, 0xf7),
            selection: Color::Rgb(0x36, 0x3d, 0x52),
            connected: Color::Rgb(0x9e, 0xce, 0x6a),
            key_hint: Color::Rgb(0xe0, 0xaf, 0x68),
        }
    }
}
