# issue048-xdg-config-support

## Summary

Implement XDG Base Directory Specification support for user configuration files (`~/.config/protonvpn-tui/config.toml`).

## Status

**[Implemented]** - Complete

## Implementation

### Changes Made

| File | Change |
|------|--------|
| `src/config/user_config.rs` | NEW - Config structs and loading logic |
| `src/config/mod.rs` | Added `user_config` module |
| `src/state/ui_state.rs` | Added `from_config(theme, footer)` constructor |
| `src/state/app_state.rs` | Added `user_config` field, `save_theme()`, `save_footer()` methods |
| `src/ui/app.rs` | Changed to use `save_theme()` and `save_footer()` |
| `src/main.rs` | Load config at startup |

### Config File Location

1. `$XDG_CONFIG_HOME/protonvpn-tui/config.toml` (if `$XDG_CONFIG_HOME` is set)
2. `~/.config/protonvpn-tui/config.toml` (fallback)

### Behavior

1. **No config file**: Uses hardcoded defaults (theme = System, footer = true, default keybindings)
2. **On setting change**: Saves the changed setting to config file
   - If footer=false → saves `footer = false`
   - If footer=true (explicitly set) → saves `footer = true` (preserves user choice)
3. **All defaults**: If no settings differ from defaults, config file is left unchanged

### Example Config Files

**After changing footer to false:**
```toml
[ui]
footer = false
```

**After changing theme to Nord:**
```toml
[ui]
theme = "Nord"
```

**After changing navigation keys:**
```toml
[keybindings]
navigation_down = { code = "n", modifiers = [] }
navigation_up = { code = "p", modifiers = [] }
```

### Features

- **Lazy creation**: Config file is only created when user changes a setting
- **Minimal persistence**: Only saves values different from defaults
- **Auto-cleanup**: If all settings revert to defaults, config file is deleted
- **Fallback**: If config file is invalid or missing, uses hardcoded defaults

## References

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- `src/config/user_config.rs` - Config structs and loading
- `src/state/app_state.rs` - Config persistence methods
