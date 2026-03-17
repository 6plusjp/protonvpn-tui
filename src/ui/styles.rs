//! TUI styles and themes

use ratatui::style::Color;

/// Theme mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
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
    pub selection: Color,
    pub inactive: Color,
    pub scrollbar: Color,
    pub disabled: Color,
    pub muted: Color,
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
        // Terminal theme - uses terminal's ANSI 16-color palette
        // These colors reference the terminal's customizable color palette (colors 0-15)
        // Users can customize these colors in their terminal settings
        Self {
            // background: Reset - use terminal's default background
            background: Color::Reset,
            // foreground: Reset - use terminal's default text color
            foreground: Color::Reset,
            // primary: Cyan (#00ffff) - main accent (ANSI cyan)
            primary: Color::Cyan,
            // secondary: Blue (#0000ff) - secondary accent (ANSI blue)
            secondary: Color::Blue,
            // accent: Magenta (#ff00ff) - highlight (ANSI magenta)
            accent: Color::Magenta,
            // error: Red (#ff0000) - error indicator (ANSI red)
            error: Color::Red,
            // success: Green (#008000) - success indicator (ANSI green)
            success: Color::Green,
            // warning: Yellow (#ffff00) - warning indicator (ANSI yellow)
            warning: Color::Yellow,
            // selection: DarkGray - selection background (ANSI dark gray)
            selection: Color::DarkGray,
            // inactive: DarkGray - inactive state (ANSI dark gray)
            inactive: Color::DarkGray,
            // scrollbar: DarkGray - scrollbar (ANSI dark gray)
            scrollbar: Color::DarkGray,
            // disabled: DarkGray - disabled state (ANSI dark gray)
            disabled: Color::DarkGray,
            // muted: Gray - muted text (ANSI gray)
            muted: Color::Gray,
        }
    }

    pub fn catppuccin_mocha() -> Self {
        // Catppuccin Mocha: Dark theme with warm pastel colors
        Self {
            // background: Base (#1e1e2e) - dark blue-gray
            background: Color::Rgb(0x1e, 0x1e, 0x2e),
            // foreground: Text (#cdd6f4) - soft lavender white
            foreground: Color::Rgb(0xcd, 0xd6, 0xf4),
            // primary: Blue (#89b4fa) - sky blue
            primary: Color::Rgb(0x89, 0xb4, 0xfa),
            // secondary: Teal (#94e2d5) - mint green
            secondary: Color::Rgb(0x94, 0xe2, 0xd5),
            // accent: Pink (#f5c2e7) - soft pink
            accent: Color::Rgb(0xf5, 0xc2, 0xe7),
            // error: Red (#f38ba8) - coral red
            error: Color::Rgb(0xf3, 0x8b, 0xa8),
            // success: Green (#a6e3a1) - pastel green
            success: Color::Rgb(0xa6, 0xe3, 0xa1),
            // warning: Yellow (#f9e2af) - soft yellow
            warning: Color::Rgb(0xf9, 0xe2, 0xaf),
            // selection: Surface1 (#455274) - darker blue-gray
            selection: Color::Rgb(0x45, 0x52, 0x6b),
            // inactive: Overlay0 (#6c7086) - muted gray
            inactive: Color::Rgb(0x6c, 0x70, 0x86),
            // scrollbar: Surface1 (#455274) - darker blue-gray
            scrollbar: Color::Rgb(0x45, 0x52, 0x6b),
            // disabled: Overlay0 (#6c7086) - muted gray
            disabled: Color::Rgb(0x6c, 0x70, 0x86),
            // muted: Overlay0 (#6c7086) - muted gray
            muted: Color::Rgb(0x6c, 0x70, 0x86),
        }
    }

    pub fn catppuccin_latte() -> Self {
        // Catppuccin Latte: Light theme with warm pastel colors
        Self {
            // background: Base (#eff1f5) - light cream
            background: Color::Rgb(0xef, 0xf1, 0xf5),
            // foreground: Text (#4c4f69) - dark gray
            foreground: Color::Rgb(0x4c, 0x4f, 0x69),
            // primary: Blue (#040a9f) - deep blue
            primary: Color::Rgb(0x04, 0x0a, 0x9f),
            // secondary: Teal (#209fa3) - teal
            secondary: Color::Rgb(0x20, 0x9f, 0xa3),
            // accent: Pink (#ea9ccb) - rose pink
            accent: Color::Rgb(0xea, 0x9c, 0xcb),
            // error: Red (#d20f1f) - bright red
            error: Color::Rgb(0xd2, 0x0f, 0x1f),
            // success: Green (#40a02e) - green
            success: Color::Rgb(0x40, 0xa0, 0x2e),
            // warning: Yellow (#df8e1d) - orange-yellow
            warning: Color::Rgb(0xdf, 0x8e, 0x1d),
            // selection: Surface1 (#ccd0e0) - light gray
            selection: Color::Rgb(0xcc, 0xd0, 0xe0),
            // inactive: Overlay0 (#9ca0b0) - gray
            inactive: Color::Rgb(0x9c, 0xa0, 0xb0),
            // scrollbar: Surface1 (#ccd0e0) - light gray
            scrollbar: Color::Rgb(0xcc, 0xd0, 0xe0),
            // disabled: Overlay0 (#9ca0b0) - gray
            disabled: Color::Rgb(0x9c, 0xa0, 0xb0),
            // muted: Overlay0 (#9ca0b0) - gray
            muted: Color::Rgb(0x9c, 0xa0, 0xb0),
        }
    }

    pub fn dracula() -> Self {
        // Dracula: Dark theme with purple/pink accents
        Self {
            // background: Dark (#282a36) - dark purple-gray
            background: Color::Rgb(0x28, 0x2a, 0x36),
            // foreground: Light (#f8f8f2) - off-white
            foreground: Color::Rgb(0xf8, 0xf8, 0xf2),
            // primary: Purple (#bd93f9) - lavender purple
            primary: Color::Rgb(0xbd, 0x93, 0xf9),
            // secondary: Green (#50fa7b) - bright green
            secondary: Color::Rgb(0x50, 0xfa, 0x7b),
            // accent: Pink (#ff79c6) - hot pink
            accent: Color::Rgb(0xff, 0x79, 0xc6),
            // error: Red (#ff5555) - bright red
            error: Color::Rgb(0xff, 0x55, 0x55),
            // success: Green (#50fa7b) - bright green
            success: Color::Rgb(0x50, 0xfa, 0x7b),
            // warning: Yellow (#f1fa8c) - light yellow
            warning: Color::Rgb(0xf1, 0xfa, 0x8c),
            // selection: Selection (#44495a) - dark gray
            selection: Color::Rgb(0x44, 0x49, 0x5a),
            // inactive: Gray (#6272a8) - muted purple-gray
            inactive: Color::Rgb(0x62, 0x72, 0x88),
            // scrollbar: Selection (#44495a) - dark gray
            scrollbar: Color::Rgb(0x44, 0x49, 0x5a),
            // disabled: Gray (#6272a8) - muted purple-gray
            disabled: Color::Rgb(0x62, 0x72, 0x88),
            // muted: Gray (#6272a8) - muted purple-gray
            muted: Color::Rgb(0x62, 0x72, 0x88),
        }
    }

    pub fn nord() -> Self {
        // Nord: Dark theme with arctic blue/north accent colors
        Self {
            // background: Polar Night (#2e3440) - dark blue-gray
            background: Color::Rgb(0x2e, 0x34, 0x40),
            // foreground: Snow Storm (#eceff4) - off-white
            foreground: Color::Rgb(0xec, 0xef, 0xf4),
            // primary: Frost (#88c0d0) - icy cyan
            primary: Color::Rgb(0x88, 0xc0, 0xd0),
            // secondary: Frost (#81a1c1) - muted blue
            secondary: Color::Rgb(0x81, 0xa1, 0xc1),
            // accent: Aurora (#b48ead) - muted purple
            accent: Color::Rgb(0xb4, 0x8e, 0xad),
            // error: Aurora (#bf616a) - soft red
            error: Color::Rgb(0xbf, 0x61, 0x6a),
            // success: Aurora (#a3be8c) - soft green
            success: Color::Rgb(0xa3, 0xbe, 0x8c),
            // warning: Aurora (#ebcb8b) - soft yellow
            warning: Color::Rgb(0xeb, 0xcb, 0x8b),
            // selection: Polar Night (#434c5e) - darker blue-gray
            selection: Color::Rgb(0x43, 0x4c, 0x5e),
            // inactive: Polar Night (#4c566a) - gray-blue
            inactive: Color::Rgb(0x4c, 0x56, 0x6a),
            // scrollbar: Polar Night (#434c5e) - darker blue-gray
            scrollbar: Color::Rgb(0x43, 0x4c, 0x5e),
            // disabled: Polar Night (#4c566a) - gray-blue
            disabled: Color::Rgb(0x4c, 0x56, 0x6a),
            // muted: Polar Night (#4c566a) - gray-blue
            muted: Color::Rgb(0x4c, 0x56, 0x6a),
        }
    }

    pub fn gruvbox() -> Self {
        // Gruvbox: Dark theme with retro warm colors
        Self {
            // background: Dark0 (#282828) - dark brown
            background: Color::Rgb(0x28, 0x28, 0x28),
            // foreground: Light0 (#ebdbb2) - warm beige
            foreground: Color::Rgb(0xeb, 0xdb, 0xb2),
            // primary: Blue (#83a598) - soft blue
            primary: Color::Rgb(0x83, 0xa5, 0x98),
            // secondary: Purple (#d3869b) - muted purple
            secondary: Color::Rgb(0xd3, 0x86, 0x9b),
            // accent: Orange (#fe8019) - bright orange
            accent: Color::Rgb(0xfe, 0x80, 0x19),
            // error: Red (#fb4934) - bright red
            error: Color::Rgb(0xfb, 0x49, 0x34),
            // success: Green (#b8bb26) - yellow-green
            success: Color::Rgb(0xb8, 0xbb, 0x26),
            // warning: Yellow (#fabd2f) - bright yellow
            warning: Color::Rgb(0xfa, 0xbd, 0x2f),
            // selection: Dark4 (#504945) - medium brown
            selection: Color::Rgb(0x50, 0x49, 0x3e),
            // inactive: Dark3 (#665c54) - muted brown
            inactive: Color::Rgb(0x66, 0x5d, 0x4e),
            // scrollbar: Dark4 (#504945) - medium brown
            scrollbar: Color::Rgb(0x50, 0x49, 0x3e),
            // disabled: Dark3 (#665c54) - muted brown
            disabled: Color::Rgb(0x66, 0x5d, 0x4e),
            // muted: Dark3 (#665c54) - muted brown
            muted: Color::Rgb(0x66, 0x5d, 0x4e),
        }
    }

    pub fn tokyo_night() -> Self {
        // Tokyo Night: Dark theme with night sky colors
        Self {
            // background: Night (#1a1b26) - deep dark blue
            background: Color::Rgb(0x1a, 0x1b, 0x26),
            // foreground: Day (#c0caf5) - soft lavender white
            foreground: Color::Rgb(0xc0, 0xca, 0xf5),
            // primary: Blue (#7aa2f7) - bright blue
            primary: Color::Rgb(0x7a, 0xa2, 0xf7),
            // secondary: Cyan (#7dcfdd) - light cyan
            secondary: Color::Rgb(0x7d, 0xcf, 0xff),
            // accent: Purple (#bb9af7) - soft purple
            accent: Color::Rgb(0xbb, 0x9a, 0xf7),
            // error: Red (#f7768e) - soft red
            error: Color::Rgb(0xf7, 0x76, 0x8e),
            // success: Green (#9ece6a) - green
            success: Color::Rgb(0x9e, 0xce, 0x6a),
            // warning: Yellow (#e0af68) - amber
            warning: Color::Rgb(0xe0, 0xaf, 0x68),
            // selection: Black (#363d52) - dark blue-gray
            selection: Color::Rgb(0x36, 0x3d, 0x52),
            // inactive: Comment (#565f89) - muted blue-gray
            inactive: Color::Rgb(0x56, 0x5f, 0x75),
            // scrollbar: Black (#363d52) - dark blue-gray
            scrollbar: Color::Rgb(0x36, 0x3d, 0x52),
            // disabled: Comment (#565f89) - muted blue-gray
            disabled: Color::Rgb(0x56, 0x5f, 0x75),
            // muted: Comment (#565f89) - muted blue-gray
            muted: Color::Rgb(0x56, 0x5f, 0x75),
        }
    }
}
