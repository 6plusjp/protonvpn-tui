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
│   ├── vpn/              # VPN backend (protonvpn wrapper)
│   │   ├── client.rs
│   │   ├── types.rs
│   │   └── state.rs
│   ├── ui/               # TUI components
│   │   ├── components/  # Reusable widgets
│   │   ├── views/       # Full views (connect, stats, settings)
│   │   └── styles.rs    # Theme and styling
│   ├── state/            # Application state
│   │   ├── app_state.rs
│   │   └── async_tasks.rs
│   └── config/           # Configuration
│       └── settings.rs
├── RUST_DESIGN_RULES.md  # Coding standards
├── ROADMAP.md            # Project phases
└── Cargo.toml
```

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

Follow `RUST_DESIGN_RULES.md` for all coding standards.

### Key conventions

- **Error handling**: Use `thiserror` for custom errors, `anyhow` for application errors
- **Logging**: Use `tracing` for structured logging
- **Serialization**: Use `serde` with derive macros
- **UI**: Use `ratatui` for TUI components

### Dependencies (Cargo.toml)

```toml
crossterm = "0.27"
ratatui = { version = "0.26", default-features = false, features = ["crossterm"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
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

- ❌ Use `as any`, `@ts-ignore`, or suppress type errors
- ❌ Leave code in broken state after failures
- ❌ Commit without explicit request
- ❌ Speculate about unread code
- ❌ Use `unwrap()` on `Option` or `Result` without justification
- ❌ Skip `cargo check` before submitting changes

### Do

- ✅ Run `cargo check` after every change
- ✅ Follow `RUST_DESIGN_RULES.md` strictly
- ✅ Add tracing/logging for debugging
- ✅ Test with `cargo test` before marking complete
- ✅ Keep changes focused and minimal
- ✅ Ask for clarification when requirements are ambiguous

---

## Agent Roles

### Project Lead (Default)

Analyze user intent and delegate to appropriate specialists.

**Workflow**:

1. Read ROADMAP.md → RUST_DESIGN_RULES.md → AGENTS.md
2. Determine which specialist to invoke
3. Break down complex tasks into smaller pieces
4. Coordinate between specialists

### Specialist Reference

| Trigger                              | Specialist   |
| ------------------------------------ | ------------ |
| Implementing features, refactoring   | Rust Core    |
| UI components, views, keybindings    | TUI/UI       |
| VPN connections, server listing      | VPN Backend  |
| Settings, configuration, persistence | Config       |
| Runtime panics, logic errors         | Debug Helper |
| Compiler errors, type mismatches     | Lint Hunter  |
| Design decisions, architecture       | Architect    |

### Key files to reference

| File                     | Purpose         |
| ------------------------ | --------------- |
| `src/vpn/client.rs`      | VPN CLI wrapper |
| `src/vpn/types.rs`       | VPN data types  |
| `src/ui/components/`     | UI widgets      |
| `src/ui/views/`          | Full views      |
| `src/state/app_state.rs` | App state       |
| `src/config/settings.rs` | Configuration   |

---

## References

- [protonvpn](https://github.com/protonvpn/proton-vpn-cli) - Official CLI
- [ratatui](https://ratatui.rs/) - TUI library
- [crossterm](https://docs.rs/crossterm/latest/crossterm/) - Terminal library
- [thiserror](https://docs.rs/thiserror/latest/thiserror/) - Error handling
- [tracing](https://docs.rs/tracing/latest/tracing/) - Structured logging
