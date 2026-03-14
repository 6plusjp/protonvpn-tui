# Issue 043: Add Multiple Theme Options

**Date**: 2026-03-13  
**Status**: Resolved  
**Priority**: Medium  
**Category**: Feature / UI

---

## Summary

Added multiple theme support with 8 themes: System, Terminal, Catppuccin (Mocha/Latte), Dracula, Nord, Gruvbox, Tokyo Night. Also added new color fields for more granular theme control.

---

## Theme Colors

### Theme struct

```rust
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,      // Titles, active elements
    pub secondary: Color,     // Secondary info, protocols
    pub accent: Color,       // Special elements
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub block_border: Color, // Block borders (focused)
    pub selection: Color,     // Selected row background
    pub connected: Color,    // Connected servers
    pub key_hint: Color,     // Keyboard shortcuts (bright)
    pub inactive: Color,      // Dimmed/inactive text
    pub scrollbar: Color,
    pub disabled: Color,      // Disabled state text
    pub muted: Color,        // Secondary info (server numbers, timestamps)
}
```

### ThemeMode Enum

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

## Implemented Changes

| Status | Item |
|--------|------|
| ✅ | Add ThemeMode enum with 8 themes |
| ✅ | Add theme constructors (system, terminal, catppuccin_mocha, catppuccin_latte, dracula, nord, gruvbox, tokyo_night) |
| ✅ | Replace is_dark_theme bool with theme_mode ThemeMode |
| ✅ | Update theme toggle to cycle through all themes |
| ✅ | Add theme selector in settings view |
| ✅ | Upgrade ratatui from 0.26 to 0.30 |
| ✅ | Fix deprecated API calls |
| ✅ | Add new color fields (inactive, scrollbar, disabled, muted) |
| ✅ | Apply theme colors to all UI components |

---

## Known Limitations

- **Terminal theme**: Uses same colors as System theme. Ratatui 0.30 does not support `Color::Index(n)` for terminal ANSI colors.

---

## Files Changed

- `src/ui/styles.rs` - Theme struct and constructors
- `src/state/ui_state.rs` - theme_mode field
- `src/state/app_state.rs` - get_theme() method
- `src/config/settings.rs` - Theme selectable options
- `src/ui/views/settings_view.rs` - Theme selector UI
- `src/ui/app.rs` - Footer hints, theme toggle
- `src/ui/components/block.rs` - centered_block with focus parameter
- `src/ui/components/pane_table.rs` - Header styling
- `src/ui/views/*.rs` - Various view updates

---

## Related

- Theme definitions: `src/ui/styles.rs`
- Theme state: `src/state/ui_state.rs`
- Theme access: `src/state/app_state.rs`
