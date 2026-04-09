# ProtonVPN TUI

A terminal UI application for Proton VPN, built with Rust.

![Platform](https://img.shields.io/badge/platform-Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Rust](https://img.shields.io/badge/rust-2021-orange)

## Overview

ProtonVPN TUI provides an interactive terminal interface for Proton VPN. It wraps the `protonvpn` CLI commands and offers a user-friendly TUI for:

- Browsing and connecting to VPN servers
- Managing VPN settings
- Viewing connection logs

> **Note**: This is an **unofficial** third-party application. The underlying `protonvpn` CLI can sometimes be unstable.

## Requirements

- **Linux** (protonvpn-cli is Linux-only)
- **ProtonVPN CLI** ([install](https://github.com/ProtonVPN/proton-vpn-cli/)) installed and configured
  - Must run `protonvpn signin` manually before using this TUI
  - Must run `protonvpn signout` via CLI when needed (not supported in TUI)
- Rust 2021 edition or later

## Installation

```bash
# Clone the repository
git clone https://github.com/6plusjp/protonvpn-tui
cd protonvpn-tui

# Build
cargo build --release

# Run
cargo run --release
```

## Uninstallation

```bash
# Remove the built binary
rm -rf ~/.local/bin/protonvpn-tui

# Optionally remove configuration and cache
rm -rf ~/.config/protonvpn-tui
rm -f /tmp/protonvpn-tui.log
```

## Features

### Server List (Servers View)

- Browse servers by country → city hierarchy
- Fuzzy search (type "japan" → matches "JP")
- Server features display (P2P, Secure Core, Tor)

### Connection Types

- Connect to fastest server
- Connect to specific country or city
- Connect to random server
- Connect to P2P-optimized server
- Connect to Tor server
- Connect to Secure Core server

### Settings (Tools View)

- Kill Switch toggle
- IPv6 protection
- Moderate NAT
- VPN Accelerator
- Port Forwarding
- NetShield (malware/ads/trackers blocking)
- Custom DNS servers

### Logs

- Connection history
- Notification events

### Themes

Multiple themes available (toggle in Settings view):

- System (terminal colors)
- Catppuccin Mocha (dark)
- Catppuccin Latte (light)
- Dracula
- Nord
- Gruvbox
- Tokyo Night

**Theme Preview**: When navigating theme options, the UI updates in real-time to preview the selected theme. Press Enter to confirm, or Esc to cancel.

## Keybindings

### Navigation

| Key           | Action                             |
| ------------- | ---------------------------------- |
| `j` / `k`     | Navigate up/down                   |
| `↑` / `↓`     | Navigate up/down (arrow keys)      |
| `Ctrl+n` / `Ctrl+p` | Navigate down/up (vim-style) |
| `g`           | Jump to top                        |
| `G`           | Jump to bottom                     |
| `Ctrl+d`      | Page down                          |
| `Ctrl+u`      | Page up                            |
| `l`           | Next pane (countries → cities)     |
| `h`           | Previous pane (cities → countries) |
| `Tab`         | Switch view (Servers ↔ Tools)     |
| `/`           | Search/filter                      |
| `Esc`         | Clear filter / Cancel              |

### Connection

| Key     | Action                         |
| ------- | ------------------------------ |
| `Enter` | Connect to selected server     |
| `c`     | Connect (when server selected) |
| `d`     | Disconnect                     |
| `r`     | Refresh server list            |
| `x`     | Connect to random server       |
| `f`     | Connect to fastest server      |
| `p`     | Connect to P2P server          |
| `t`     | Connect to Tor server          |
| `s`     | Connect to Secure Core server  |

### Sorting

| Key | Action                    |
| --- | ------------------------- |
| `1` | Sort by server ID        |
| `2` | Sort by country          |
| `a` | Toggle sort direction    |

### Other

| Key | Action |
| --- | ------ |
| `?` | Help   |
| `q` | Quit   |

### Session Time

> **Note**: Session time is not sourced from `protonvpn status` — the CLI does not expose an uptime field in current versions (v0.1.8+).

Instead, session time is tracked internally by the TUI:
- On successful connection, the app records a timestamp (`connected_at`)
- Session time = current time - `connected_at`
- If you reconnect to the **same** server, the session is preserved
- If you connect to a **different** server, a new session begins
- Session persists across app restarts (stored in cache)

## Configuration

Settings are stored in `~/.config/protonvpn-tui/config.toml`.

Logs are written to `/tmp/protonvpn-tui.log` by default, or the path specified by `PROTONVPN_TUI_LOG`.

## Architecture

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

## CLI Commands Used

This TUI wraps the following `protonvpn` commands:

| Command                                  | Purpose                       |
| ---------------------------------------- | ----------------------------- |
| `protonvpn countries`                    | List available countries      |
| `protonvpn cities --country <CC>`        | List cities for a country     |
| `protonvpn connect`                      | Connect to fastest server     |
| `protonvpn connect --country <CC>`       | Connect to a country          |
| `protonvpn connect --city <city>`        | Connect to a city             |
| `protonvpn connect --random`             | Connect to random server      |
| `protonvpn connect --p2p`                | Connect to P2P server         |
| `protonvpn connect --tor`                | Connect to Tor server         |
| `protonvpn connect --securecore`         | Connect to Secure Core server |
| `protonvpn disconnect`                   | Disconnect from VPN           |
| `protonvpn config set <setting> <value>` | Change settings               |

## License

MIT License - see [LICENSE](LICENSE) for details.
