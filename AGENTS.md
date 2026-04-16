# AGENTS.md

**Project**: protonvpn-tui  
**Language**: Rust  
**TUI Framework**: crossterm + ratatui  
**Architecture**: Single binary CLI application

---

## Project overview

ProtonVPN TUI is a terminal UI application for Proton VPN. It wraps the `protonvpn` commands and provides an interactive TUI for connecting to VPN servers, managing connections, and viewing connection statistics.

---

## Structure

```
protonvpn-tui/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── error.rs          # Error types (AppError, VpnError)
│   ├── constants.rs      # Application constants
│   ├── paths.rs          # Path utilities (config, cache, logs)
│   ├── vpn/              # VPN backend (protonvpn wrapper)
│   │   ├── mod.rs
│   │   ├── async_tasks.rs
│   │   ├── cache.rs
│   │   ├── client.rs
│   │   └── types.rs
│   ├── ui/               # TUI components
│   │   ├── mod.rs
│   │   ├── app.rs        # Main TUI application (event loop + orchestrator)
│   │   ├── render.rs     # View rendering logic
│   │   ├── styles.rs     # Theme and styling
│   │   ├── keymap.rs     # Key binding definitions
│   │   ├── input/        # Key handling (extracted from app.rs)
│   │   │   ├── mod.rs
│   │   │   ├── app_action.rs
│   │   │   ├── input_state.rs
│   │   │   ├── handler.rs
│   │   │   ├── common.rs
│   │   │   ├── servers.rs
│   │   │   ├── tools.rs
│   │   │   ├── help.rs
│   │   │   └── filter.rs
│   │   ├── renderers/    # Rendering functions (extracted from app.rs)
│   │   │   ├── mod.rs
│   │   │   ├── header.rs
│   │   │   ├── footer.rs
│   │   │   ├── notification.rs
│   │   │   └── input.rs
│   │   ├── components/   # Reusable widgets
│   │   │   ├── mod.rs
│   │   │   ├── block.rs
│   │   │   ├── list.rs
│   │   │   └── pane_table.rs
│   │   └── views/        # Full views
│   │       ├── help_view.rs
│   │       ├── logs_view.rs
│   │       ├── servers_view.rs
│   │       ├── settings_view.rs
│   │       └── tools_view.rs
│   ├── state/            # Application state
│   │   ├── mod.rs
│   │   ├── app_state.rs  # Main application state
│   │   ├── app_state_impl.rs # VPN operations
│   │   ├── app_view.rs   # View enum
│   │   ├── config_state.rs
│   │   ├── connection_manager.rs # Async management
│   │   ├── connection_state.rs
│   │   ├── event_handler.rs # Async event processing
│   │   ├── log_persistence.rs
│   │   ├── navigation.rs # Selection/pane navigation
│   │   ├── notifications.rs
│   │   ├── server_cache.rs
│   │   ├── server_filter.rs
│   │   ├── server_ops.rs # Server filtering/sorting
│   │   ├── server_sort.rs
│   │   ├── settings_ops.rs # Settings management
│   │   └── ui_state.rs
│   └── config/           # Configuration
│       ├── mod.rs
│       ├── settings.rs
│       └── user_config.rs
├── docs/                 # Documentation (source of truth)
│   ├── issue/            # Issue tracking (issue001.md, issue002.md, ...)
│   ├── reference/        # Reference docs (user-managed)
│   └── policy/           # Project policies
├── tests/                # Integration tests
├── AGENTS.md             # AI agent context
└── Cargo.toml
```

---

## Required Context

Load these policy documents before working on this project:

- [@docs/policy/policy.md](docs/policy/policy.md) — Project policy overview
- [@docs/policy/commit-message-rule.md](docs/policy/commit-message-rule.md) — Commit message format
- [@docs/policy/naming-conventions.md](docs/policy/naming-conventions.md) — Naming conventions
- [@docs/policy/reference-convention.md](docs/policy/reference-convention.md) — Document reference syntax
- [@docs/policy/coding-standards.md](docs/policy/coding-standards.md) — Rust coding standards
- [@docs/policy/rust-maintainability.md](docs/policy/rust-maintainability.md) — Rust maintainability guidelines

---

## Build and test commands

```bash
# Build
cargo build              # Debug build
cargo build --release   # Release build (LTO enabled)

# Run
cargo run               # Debug run
cargo run --release     # Release run

# Check
cargo check             # Type check only
cargo clippy            # Linting

# Test
cargo test             # Run tests

# Format
cargo fmt              # Format code
cargo fmt --check     # Check formatting

# Clean
cargo clean            # Clean build artifacts
```

**Environment variables for debugging**:

```bash
RUST_LOG=debug cargo run    # Enable debug logging
RUST_BACKTRACE=1 cargo run  # Enable backtrace on panic
```

---

## Code style guidelines

Follow `docs/policy/coding-standards.md` for all coding standards.

---

## Module visibility conventions

Public API types are re-exported from module roots. Implementation modules stay private.

| Module | Public | Private |
|--------|--------|---------|
| `state/` | `AppState`, `ConnectionState`, `AppView`, etc. (via `pub use`) | `app_state_impl`, `navigation`, `server_ops`, `settings_ops`, `event_handler` |
| `ui/` | `app`, `views`, `Theme`, `ThemeMode`, `KeyMap`, `KeyArrow`, `KeyMatcher` | `components`, `input`, `keymap`, `render`, `renderers`, `styles` |

When importing from `state` or `ui`, use the re-exported path:

```rust
// Correct
use crate::ui::{Theme, ThemeMode, KeyMap};
use crate::state::{AppState, ConnectionState};

// Wrong — private module access
use crate::ui::styles::Theme;
use crate::state::app_state_impl::AppState;
```

---

## Testing instructions

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

**Note**: This project uses `tempfile` and `assert_cmd` for integration testing.

---

## Security considerations

- **Credential handling**: VPN credentials are handled by `protonvpn` directly
- **Process execution**: VPN commands are executed via `std::process::Command`
- **No secret storage**: This app does not store any VPN credentials
- **Network**: All VPN traffic goes through Proton's encrypted tunnels

---

## Don't / Do

### Don't

- ❌ Leave code in broken state after failures
- ❌ Commit without explicit request
- ❌ Speculate about unread code
- ❌ Use `unwrap()` on `Option` or `Result` without justification
- ❌ Skip `cargo check` before submitting changes
- ❌ Use getter/setter patterns (e.g., `get_field()`, `set_field()`) — use `pub` fields or methods directly

### Do

- ✅ Run `cargo check` after every change
- ✅ Add tracing/logging for debugging
- ✅ Test with `cargo test` before marking complete
- ✅ Keep changes focused and minimal
- ✅ Ask for clarification when requirements are ambiguous
- ✅ Create issue document in `docs/issue/` before starting implementation

---

## References

- [protonvpn](https://github.com/protonvpn/proton-vpn-cli) - Official CLI
- [ratatui](https://ratatui.rs/) - TUI library
- [crossterm](https://docs.rs/crossterm/latest/crossterm/) - Terminal library
