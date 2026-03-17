# issue051-agents-md-contradictions

## Summary

AGENTS.md specifies certain conventions that the current implementation violates:
1. **Getter/Setter Patterns**: AGENTS.md prohibits getter/setter patterns, but 31 methods across 11 files use `get_*`/`set_*` prefixes
2. **Structure Documentation**: AGENTS.md doesn't document several actual source files (`error.rs`, `constants.rs`, `app.rs`, etc.)

## Status

**[Open]**

## Problem 1: Getter/Setter Violations

### AGENTS.md Rule
> Don't use getter/setter patterns (e.g., `get_field()`, `set_field()`) — use `pub` fields or methods directly

### Violations Found

| File | Methods |
|------|---------|
| `src/state/app_state.rs` | `get_theme()`, `set_search_query()`, `set_servers()`, `set_sort_by_*()`, `set_filter()` |
| `src/vpn/client.rs` | `get_servers()`, `get_connected_server_info()` |
| `src/state/ui_state.rs` | `set_view()`, `set_cached()` |
| `src/state/connection.rs` | `set_connection()` |
| `src/vpn/cache.rs` | `set_connected()`, `set_disconnected()`, `set_cli_unavailable()` |
| `src/config/settings.rs` | `get_selectable_option_command()` |
| `src/ui/app.rs` | `get_theme()`, `get_footer_action_hints()`, `get_global_connect_hints()` |
| `src/ui/views/settings_view.rs` | `get_setting_value()`, `get_setting_label()` |
| `src/config/user_config.rs` | `get_config_dir()`, `get_config_path()` |
| `src/state/log_persistence.rs` | `get_log_file_path()`, `set_test_mode()` |

**Total: 31 violations across 11 files**

### Analysis

The getter/setter pattern is common in TUI applications for:
- Encapsulating state mutations with validation
- Providing read-only or controlled write access
- Computed properties

## Problem 2: Structure Documentation Gaps

### AGENTS.md Lists

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── vpn/              # VPN backend
├── ui/               # TUI components
│   └── styles.rs    # Theme and styling
├── state/            # Application state
│   └── app_state.rs
└── config/          # Configuration
    └── settings.rs
```

### Actual Structure

| Missing from AGENTS.md | Notes |
|-----------------------|-------|
| `src/error.rs` | Error types (`AppError`, `VpnError`) |
| `src/constants.rs` | Application constants |
| `src/ui/app.rs` | Main TUI application (68KB, core file) |
| `src/ui/render.rs` | Render logic |
| `src/ui/mod.rs` | UI module |
| `src/config/user_config.rs` | User config (vs settings.rs) |
| `src/state/*.rs` | 12 state files (only app_state.rs documented) |
| `src/commands/mod.rs` | Empty placeholder |

### Extra Files Not in AGENTS.md

- `src/vpn/AGENTS.md` - Embedded documentation
- `src/state/AGENTS.md` - Embedded documentation
- `docs/specifications/` - Listed but doesn't exist

## Options

1. **Update AGENTS.md** to document current patterns (low effort)
2. **Refactor code** to remove getter/setters (high effort)
3. **Accept** as project-specific exception (document in AGENTS.md)

## Related

- [@docs/policy/coding-standards.md](docs/policy/coding-standards.md)
