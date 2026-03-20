# UI Module Architecture

## Overview

The `ui/` module contains all TUI (Terminal User Interface) components using ratatui.

## Module Structure

```
src/ui/
├── mod.rs          # Module root - re-exports public APIs
├── app.rs          # Main TUI application (App struct, run loop)
├── render.rs       # Render helpers
├── styles.rs       # Theme definitions and colors
├── components/     # Reusable widgets
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

### 3. Views

Each view is a separate module that renders a full screen:

- **ServersView**: Server list with country/city hierarchy
- **ToolsView**: VPN settings toggles
- **SettingsView**: App settings (theme, footer)
- **LogsView**: Connection history
- **HelpView**: Keybinding reference

## Public API (from mod.rs)

```rust
pub use app::App;
pub use styles::Theme;
```
