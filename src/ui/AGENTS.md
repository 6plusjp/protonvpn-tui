# UI Module Architecture

## Overview

The `ui/` module contains all TUI (Terminal User Interface) components using ratatui.

## Module Structure

```
src/ui/
├── mod.rs          # Module root
├── app.rs          # Main TUI application (TuiApp struct, run loop)
├── render.rs       # Render helpers
├── styles.rs       # Theme definitions and colors
├── keymap.rs       # Key binding definitions
├── input/          # Key handling (extracted from app.rs)
│   ├── mod.rs
│   ├── app_action.rs
│   ├── input_state.rs
│   ├── handler.rs
│   ├── common.rs
│   ├── servers.rs
│   ├── tools.rs
│   ├── help.rs
│   └── filter.rs
├── renderers/      # Rendering functions (extracted from app.rs)
│   ├── mod.rs
│   ├── header.rs
│   ├── footer.rs
│   ├── notification.rs
│   └── input.rs
├── components/     # Reusable widgets
│   ├── mod.rs
│   ├── block.rs
│   ├── list.rs
│   └── pane_table.rs
└── views/          # Full screen views
    ├── help_view.rs
    ├── logs_view.rs
    ├── servers_view.rs
    ├── settings_view.rs
    └── tools_view.rs
```

## Theme

Themes are defined in `styles.rs` using the `Theme` struct:

```rust
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    // ... more fields
}
```

## Views

Each view is a separate module that renders a full screen:

- **ServersView**: Server list with country/city hierarchy
- **ToolsView**: VPN settings toggles
- **SettingsView**: App settings (theme, footer)
- **LogsView**: Connection history
- **HelpView**: Keybinding reference

## Keymap

Key bindings are defined in `keymap.rs` and dispatched through `input/handler.rs`.

## Renderers

Rendering is split into focused modules under `renderers/`:

| Module | Responsibility |
|--------|----------------|
| `header.rs` | Top bar (connection status, server info) |
| `footer.rs` | Bottom bar (key hints) |
| `notification.rs` | Toast notifications |
| `input.rs` | Search/filter input overlay |
