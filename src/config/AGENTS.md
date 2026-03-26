# Config Module Architecture

## Overview

The `config/` module handles user configuration and settings management.

## Module Structure

```
src/config/
├── mod.rs          # Module root - re-exports public APIs
├── settings.rs     # Setting keys and helpers
└── user_config.rs  # User configuration struct and persistence
```

## Configuration Persistence

User config is stored in `~/.config/protonvpn-tui/config.toml`:

```rust
// Load config
UserConfig::load()

// Save config
user_config.save()
```

### 3. Settings vs Config

- **Settings**: VPN settings managed by `protonvpn` CLI (killswitch, DNS, NetShield, etc.)
- **UserConfig**: App-specific preferences (theme, keybindings, footer)

## Public API (from mod.rs)

```rust
pub use settings::{SettingKey, ProtonSettings};
pub use user_config::UserConfig;
```

## Important: Read-Only File

**`~/.config/Proton/VPN/settings.json` is READ-ONLY for this TUI.**

- This file is owned and managed by the `protonvpn` CLI
- **NEVER** write to this file directly from the TUI
- All setting changes **MUST** go through `protonvpn config set` commands (see `src/vpn/client.rs`)
- `ProtonSettings::load()` is the only valid access pattern — read-only deserialization
- If you need to modify VPN settings, use `VpnClient::set_config()`, `set_custom_dns()`, `disable_custom_dns()`, etc.

**Why**: Direct file writes could corrupt state, conflict with CLI operations, or break during concurrent access.
