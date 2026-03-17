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
