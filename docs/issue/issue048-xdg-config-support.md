# issue048 - XDG Config File Support

## Summary

Implement XDG Base Directory Specification support for user configuration files (`~/.config/protonvpn-tui/config.toml`).

## Motivation

Currently, the application has no persistent user configuration. Theme, footer visibility, and key bindings are hardcoded or managed only at runtime. Users need a way to persist these settings across sessions.

## Background

### XDG Base Directory Specification

- **Config location**: `$XDG_CONFIG_HOME/protonvpn-tui/` (defaults to `~/.config/protonvpn-tui/`)
- **Config file**: `config.toml`
- **Fallback**: If `$XDG_CONFIG_HOME` is not set, use `~/.config/` as default

### Existing Patterns in Codebase

The codebase already uses `dirs` crate for reading Proton VPN CLI settings (read-only):

```rust
// src/config/settings.rs (line 177-182)
let config_path = dirs::config_dir()?
    .join("Proton")
    .join("VPN")
    .join("settings.json");
```

> **Note**: `settings.json` is ProtonVPN CLI's config file, not TUI's. The TUI reads it but cannot modify it.

Dependencies already available:
- `dirs = "=5.0"` - for getting config directories
- `toml = "=0.8"` - for serialization/deserialization
- `serde = { version = "=1.0", features = ["derive"] }`

### Config File Location (Priority Order)

1. `$XDG_CONFIG_HOME/protonvpn-tui/config.toml` (if `$XDG_CONFIG_HOME` is set)
2. `~/.config/protonvpn-tui/config.toml` (fallback)

### Current State of Configurable Settings

| Setting | Location | Type | Default |
|---------|----------|------|---------|
| Theme | `src/ui/styles.rs` - `ThemeMode` enum | Runtime state | `ThemeMode::System` |
| Footer | `src/state/ui_state.rs` - `show_footer` | Runtime state | `true` |
| Key Bindings | `src/config/settings.rs` - `KeyBindings` struct | Hardcoded defaults | See `KeyBindings::default()` |

## Proposal

### Config File Structure

```toml
# ~/.config/protonvpn-tui/config.toml

[ui]
theme = "System"  # System, Terminal, CatppuccinMocha, CatppuccinLatte, Dracula, Nord, Gruvbox, TokyoNight
footer = true     # Show/hide footer

[keybindings]
navigation_down = { code = "j", modifiers = [] }
navigation_up = { code = "k", modifiers = [] }
page_down = { code = "d", modifiers = ["Control"] }
page_up = { code = "u", modifiers = ["Control"] }
go_first = { code = "g", modifiers = [] }
go_last = { code = "G", modifiers = ["Shift"] }
connect = { code = "c", modifiers = [] }
disconnect = { code = "d", modifiers = [] }
refresh = { code = "r", modifiers = [] }
random_connect = { code = "x", modifiers = [] }
pane_next = { code = "l", modifiers = [] }
pane_prev = { code = "h", modifiers = [] }
sort_by_code = { code = "1", modifiers = [] }
sort_by_country = { code = "2", modifiers = [] }
connect_fastest = { code = "f", modifiers = [] }
connect_p2p = { code = "p", modifiers = [] }
connect_tor = { code = "t", modifiers = [] }
securecore = { code = "s", modifiers = [] }
```

### Implementation Tasks

1. **Create config struct** (`src/config/user_config.rs`)
   - `UserConfig` struct with `ui` and `keybindings` sections
   - `UiConfig` for theme and footer
   - `KeyBindingConfig` matching existing `KeyBinding` format

2. **Implement config loading**
   - First try `$XDG_CONFIG_HOME/protonvpn-tui/config.toml`
   - Fallback to `~/.config/protonvpn-tui/config.toml`
   - Create app-specific directory if not exists
   - Load `config.toml` with fallback to defaults
   - Handle missing file gracefully (use defaults)

3. **Integrate with app initialization**
   - Load config at startup in `main.rs`
   - Pass config to `AppState` initialization

4. **Create default config file** (optional)
   - On first run, create config with defaults
   - Or just use hardcoded defaults

### Key Files to Modify

| File | Change |
|------|--------|
| `src/config/mod.rs` | Add `user_config` module |
| `src/config/user_config.rs` | NEW - Config structs and loading |
| `src/state/ui_state.rs` | Accept theme/footer in constructor |
| `src/state/app_state.rs` | Accept keybindings in constructor |
| `src/main.rs` | Load config at startup |

### Alternative Approaches

1. **Use `directories` crate** instead of `dirs` - more features but requires adding new dependency
2. **Use `xdg` crate** - specialized for XDG but might be overkill
3. **Use `directories` crate** - cross-platform, comprehensive

Recommendation: Use existing `dirs` crate for minimal dependency addition.

### Error Handling

- If XDG_CONFIG_HOME is set: use `$XDG_CONFIG_HOME/protonvpn-tui/`
- If XDG_CONFIG_HOME is NOT set: use `~/.config/protonvpn-tui/`
- If config directory doesn't exist: create it
- If config file doesn't exist: use defaults
- If config file is invalid: log warning, use defaults
- Don't crash on config errors

### Code Pattern for Fallback

```rust
use std::path::PathBuf;

fn get_config_path() -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME first
    if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("protonvpn-tui").join("config.toml");
        if path.exists() {
            return Some(path);
        }
    }
    
    // Fallback to ~/.config/
    dirs::config_dir().map(|p| p.join("protonvpn-tui").join("config.toml"))
}
```

## References

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [dirs crate docs](https://docs.rs/dirs/latest/dirs/fn.config_dir.html)
- Existing config loading: `src/config/settings.rs:176-191`

## Status

- [ ] Investigated
- [ ] Planned
- [ ] In Progress
- [ ] Resolved
