# Issue 043: Add Multiple Theme Options

**Date**: 2026-03-13  
**Status**: Open  
**Priority**: Medium  
**Category**: Feature / UI

---

## Problem Statement

Currently, the application only supports two themes: `dark` and `light`. This is limiting for users who prefer popular color palettes commonly found in terminal applications and developer tools.

## Background

### Current Implementation

The theme system is defined in `src/ui/styles.rs`:

```rust
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub block_border: Color,
    pub selection: Color,
    pub connected: Color,
    pub key_hint: Color,
}
```

- **Location**: `src/ui/styles.rs` (Theme struct + `dark()` / `light()` constructors)
- **State**: `src/state/ui_state.rs` stores `is_dark_theme: bool`
- **Access**: `src/state/app_state.rs` has `get_theme()` method

### Adding New Themes

To add a new theme, the following changes are needed:

1. Add a new constructor method in `src/ui/styles.rs` (e.g., `pub fn nord() -> Self`)
2. Extend `UiState` to use a `ThemeMode` enum instead of `is_dark_theme: bool`
3. Update `AppState.get_theme()` to match on the enum
4. Add toggle logic in `toggle_settings()` in `app_state.rs`

---

## Proposed Themes

### 1. Catppuccin (Recommended)

Popular soothing pastel theme with 4 flavors:

- **Mocha** (dark) - Original, cozy feeling
- **Frappé** - Muted aesthetic
- **Macchiato** - Medium contrast
- **Latte** (light) - Light variant

Key colors (Mocha):
- Background: `#1e1e2e`
- Foreground: `#cdd6f4`
- Primary: `#89b4fa` (Blue)
- Secondary: `#94e2d5` (Teal)
- Accent: `#f5c2e7` (Pink)
- Error: `#f38ba8`
- Success: `#a6e3a1`
- Warning: `#f9e2af`

**Source**: https://catppuccin.com/palette/

### 2. Dracula

The most famous developer theme:

- Background: `#282a36`
- Foreground: `#f8f8f2`
- Primary: `#bd93f9` (Purple)
- Secondary: `#50fa7b` (Green)
- Accent: `#ff79c6` (Pink)
- Error: `#ff5555`
- Success: `#50fa7b`
- Warning: `#f1fa8c`

**Source**: https://draculatheme.com/

### 3. Nord

Arctic, north-bluish color palette:

- Background: `#2e3440`
- Foreground: `#eceff4`
- Primary: `#88c0d0` (Cyan)
- Secondary: `#81a1c1` (Blue)
- Accent: `#b48ead` (Purple)
- Error: `#bf616a`
- Success: `#a3be8c`
- Warning: `#ebcb8b`

**Source**: https://www.nordtheme.com/

### 4. Gruvbox

Retro warm colors:

- Background: `#282828`
- Foreground: `#ebdbb2`
- Primary: `#fe8019` (Orange)
- Secondary: `#8ec07c` (Green)
- Accent: `#d3869b` (Purple)
- Error: `#fb4934`
- Success: `#b8bb26`
- Warning: `#fabd2f`

**Source**: https://github.com/morhetz/gruvbox

### 5. Tokyo Night

Japanese night sky aesthetic:

- Background: `#1a1b26`
- Foreground: `#c0caf5`
- Primary: `#7aa2f7` (Blue)
- Secondary: `#7dcfff` (Cyan)
- Accent: `#bb9af7` (Purple)
- Error: `#f7768e`
- Success: `#9ece6a`
- Warning: `#e0af68`

**Source**: https://github.com/enkia/tokyo-night-vscode-theme

---

## New Proposal: Terminal Theme (Use Terminal's ANSI Colors)

### Rationale

Instead of forcing users to choose between dark/light or custom color schemes, the application should respect the terminal's configured ANSI color palette. This provides:

1. **Seamless integration** - The TUI blends naturally with the user's terminal setup
2. **Accessibility** - Users with custom terminal color schemes (e.g., for accessibility) get proper support
3. **Simplicity** - No configuration needed for most users
4. **Familiar feel** - Uses the same colors the user sees in their terminal

### Background

Terminals have 16 standard ANSI colors that users can customize in their terminal settings:
- Black, Red, Green, Yellow, Blue, Magenta, Cyan, White (standard)
- Bright Black, Bright Red, Bright Green, Bright Yellow, Bright Blue, Bright Magenta, Bright Cyan, Bright White (bright)

Most terminals (iTerm2, Windows Terminal, Alacritty, etc.) allow users to change these colors.

### Proposed Change

1. **Add `Terminal` theme** that uses the terminal's configured ANSI colors:
   - Use `Color::Index(n)` in ratatui to reference ANSI colors (0-15)
   - The terminal's actual colors are used, not hardcoded values
   - Only semantic colors (error, success, warning) should be explicitly mapped

2. **Deprecate/Remove `dark` and `light` themes**:
   - These are redundant with the new named palettes
   - Users who want the old behavior can use Catppuccin (dark) or Latte (light)
   - Legacy code can be removed after migration period

### Implementation

```rust
/// Uses terminal's configured ANSI colors (16-color palette)
pub fn terminal() -> Self {
    Self {
        background: Color::Index(0),   // Terminal's "Black" (user-configured)
        foreground: Color::Index(7),   // Terminal's "White" (user-configured)
        primary: Color::Index(6),      // Terminal's "Cyan"
        secondary: Color::Index(4),    // Terminal's "Blue"
        accent: Color::Index(5),       // Terminal's "Magenta"
        error: Color::Index(1),        // Terminal's "Red"
        success: Color::Index(2),      // Terminal's "Green"
        warning: Color::Index(3),       // Terminal's "Yellow"
        block_border: Color::Index(6),  // Terminal's "Cyan"
        selection: Color::Index(14),   // Terminal's "Bright Cyan"
        connected: Color::Index(2),    // Terminal's "Green"
        key_hint: Color::Index(3),     // Terminal's "Yellow"
    }
}
```

**Note**: `Color::Index(n)` uses the terminal's ANSI color palette (0-15). The actual colors displayed depend on the user's terminal configuration. This is different from `Color::Reset` which would inherit nothing.

### ANSI Color Index Reference

| Index | Standard | Bright  |
|-------|----------|---------|
| 0     | Black    | —        |
| 1     | Red      | —        |
| 2     | Green    | —        |
| 3     | Yellow   | —        |
| 4     | Blue     | —        |
| 5     | Magenta  | —        |
| 6     | Cyan     | —        |
| 7     | White    | —        |
| 8     | —        | Bright Black   |
| 9     | —        | Bright Red     |
| 10    | —        | Bright Green   |
| 11    | —        | Bright Yellow  |
| 12    | —        | Bright Blue    |
| 13    | —        | Bright Magenta |
| 14    | —        | Bright Cyan    |
| 15    | —        | Bright White   |

### Updated Theme Enum

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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
```

---

## Implementation Notes

### Theme Enum

Replace `is_dark_theme: bool` with:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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
```

### Theme Selector UI

Consider adding a theme selector in settings view that allows users to cycle through available themes.

### Considerations

- All named colors should map to `ratatui::style::Color::Rgb(r, g, b)` for precise color matching
- Some terminals may not support full 24-bit color - consider fallback options
- User preference should be persisted in config

---

## Related

- Current themes: `src/ui/styles.rs`
- Theme state: `src/state/ui_state.rs`
- Theme access: `src/state/app_state.rs`
