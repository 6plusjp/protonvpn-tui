# issue072: Documentation - README and AGENTS.md out of sync with codebase

## Summary

README.md and AGENTS.md files contain stale references that don't match the actual codebase structure. Must be fixed before GitHub publication.

## Problem

Pre-publication review revealed multiple documentation drift issues:

### 1. README.md

- **Line 31**: Clone URL is placeholder `https://github.com/yourusername/protonvpn-tui` — should be `https://github.com/6plusjp/protonvpn-tui`
- **Architecture section** (lines 153-174): Outdated directory tree missing `input/`, `renderers/`, `keymap.rs`

### 2. Root AGENTS.md

| AGENTS.md Lists | Actual Status |
|-----------------|---------------|
| `src/commands/` (placeholder) | Deleted (resolved056.md) |
| `src/state/connection.rs` | Renamed → `connection_manager.rs` (resolved058.md) |
| `src/state/server_data.rs` | Deleted (resolved055.md) |
| (not listed) | `src/state/app_state_impl.rs` added |
| (not listed) | `src/state/event_handler.rs` added |
| (not listed) | `src/state/navigation.rs` added |
| (not listed) | `src/state/server_ops.rs` added |
| (not listed) | `src/state/settings_ops.rs` added |

### 3. src/ui/AGENTS.md

- References `App` struct (actual: `TuiApp`)
- Missing `input/`, `renderers/`, `keymap.rs` modules
- Public API example shows `pub use app::App` — doesn't match actual exports

## Solution

### README.md

1. Fix clone URL to `https://github.com/6plusjp/protonvpn-tui`
2. Update Architecture section to match actual `src/` structure:

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── error.rs          # Error types (AppError, VpnError)
├── constants.rs      # Application constants
├── paths.rs          # Path utilities (config, cache, logs)
├── vpn/              # VPN backend (protonvpn CLI wrapper)
│   ├── client.rs     # CLI execution
│   ├── cache.rs      # Server cache
│   ├── types.rs      # Data types
│   └── async_tasks.rs # Async task management
├── ui/               # TUI components
│   ├── app.rs        # Main TUI application (TuiApp)
│   ├── render.rs     # Render helpers
│   ├── styles.rs     # Theme definitions
│   ├── keymap.rs     # Key bindings
│   ├── input/        # Key handling
│   ├── renderers/    # Rendering functions (header, footer, notification, input)
│   ├── components/   # Reusable widgets (block, list, pane_table)
│   └── views/        # Full views (servers, tools, settings, logs, help)
├── state/            # Application state
│   ├── app_state.rs      # Main state container
│   ├── app_state_impl.rs # VPN operations
│   ├── connection_manager.rs # Async management
│   ├── connection_state.rs
│   ├── navigation.rs     # Selection/pane navigation
│   ├── server_ops.rs     # Server filtering/sorting
│   ├── settings_ops.rs   # Settings management
│   ├── event_handler.rs  # Async event processing
│   ├── notifications.rs  # Toast + log notifications
│   ├── ui_state.rs       # UI state (selection, scroll)
│   ├── app_view.rs       # View enum
│   ├── server_filter.rs  # Filter logic
│   ├── server_sort.rs    # Sort logic
│   ├── config_state.rs   # Config state
│   └── log_persistence.rs # Log persistence
└── config/           # User configuration
    ├── settings.rs
    └── user_config.rs
```

### Root AGENTS.md

1. Remove `src/commands/` (deleted)
2. Remove `src/state/connection.rs` → `connection_manager.rs`
3. Remove `src/state/server_data.rs` (deleted)
4. Add new state modules: `app_state_impl.rs`, `event_handler.rs`, `navigation.rs`, `server_ops.rs`, `settings_ops.rs`
5. Add `src/ui/input/`, `src/ui/renderers/`, `src/ui/keymap.rs` to ui section

### src/ui/AGENTS.md

Option A: Update to match current structure (like root AGENTS.md ui section)

Option B: Deprecate/remove if redundant with root AGENTS.md

## Acceptance Criteria

- [ ] README.md clone URL points to actual repository
- [ ] README.md Architecture section matches actual `src/` structure
- [ ] Root AGENTS.md structure matches actual files (no stale refs, all new files listed)
- [ ] src/ui/AGENTS.md matches current ui module structure or is removed
- [ ] `cargo check` passes (no code changes expected)

## References

- docs/issue/resolved056.md — commands/ deletion
- docs/issue/resolved058-connection-naming-confusion.md — connection.rs → connection_manager.rs
- docs/issue/resolved055.md — server_data.rs deletion
