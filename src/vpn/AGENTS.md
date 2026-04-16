# VPN Module Architecture

## Overview

The `vpn/` module provides a Rust wrapper around the ProtonVPN CLI. It is designed with clear separation of concerns.

## Required Context

Load these policy documents before working on this module:

- [@docs/policy/policy.md](docs/policy/policy.md) — Project policy overview
- [@docs/policy/coding-standards.md](docs/policy/coding-standards.md) — Rust coding standards
- [@docs/policy/reference-convention.md](docs/policy/reference-convention.md) — Document reference syntax

## Module Structure

```
src/vpn/
├── mod.rs          # Module root - re-exports public APIs
├── async_tasks.rs  # Async task management (thread pool for VPN operations)
├── client.rs       # CLI execution layer
├── cache.rs        # Server cache management
└── types.rs        # Data types + parsing functions
```

## Design Principles

### 1. Single Responsibility

Each file has one clear purpose:

| File | Responsibility |
|------|----------------|
| `async_tasks.rs` | Thread pool for async VPN operations (connect, disconnect, refresh) |
| `client.rs` | Execute `protonvpn` CLI commands, manage thread-safe cache access |
| `cache.rs` | Server data structure, caching logic, persistence |
| `types.rs` | Data types (Server, City, etc.) + CLI output parsing |

### 2. No Redundant Layers

**Do NOT create wrapper types** that simply delegate to another type. Use the underlying type directly.

**Bad**:
```rust
// DON'T DO THIS
pub struct VpnState { client: VpnClient }
impl VpnState {
    pub fn connect_country(&self, server: &str) -> AppResult<...> {
        self.client.connect_country(server)?; // Pure delegation - unnecessary
    }
}
```

**Good**:
```rust
// Use VpnClient directly
let client = VpnClient::new();
client.connect_country("JP")?;
```

### 3. Cache is Thread-Safe

The `ServerCache` lives inside `VpnClient` wrapped in `Mutex`:

```rust
// client.rs
pub struct VpnClient {
    cache: Mutex<ServerCache>,  // Thread-safe access
    cache_path: PathBuf,
}
```

Cache operations use helper methods:

```rust
fn with_cache<F, T>(&self, f: F) -> AppResult<T>
where F: FnOnce(&mut ServerCache) -> T

fn save_cache(&self) -> AppResult<()>
```

## When Adding New Functions

### Parse CLI output? → Add to `types.rs`

```rust
// src/vpn/types.rs

/// Parse protonvpn CLI output for X
pub fn parse_xxx(output: &str) -> Result<Type, ParseError> {
    // ...
}
```

### Cache data/operations? → Add to `cache.rs`

```rust
// src/vpn/cache.rs

impl ServerCache {
    pub fn new_method(&mut self, args: Type) {
        // Mutates self (cache state)
    }
}
```

### Execute CLI command? → Add to `client.rs`

```rust
// src/vpn/client.rs

pub fn new_cli_command(&self, args: &str) -> AppResult<Output> {
    // Execute protonvpn CLI
    // Update cache if needed
}
```

### Async VPN operations? → Use `async_tasks.rs`

The `AsyncTaskManager` provides a thread pool for non-blocking VPN operations:

```rust
// src/vpn/async_tasks.rs

let manager = AsyncTaskManager::new();
manager.spawn_connect(vpn_client, server_id, sender);
```

## Public API (from mod.rs)

```rust
// Re-exported from client.rs
pub use client::{VpnClient, ServerCache};
pub use types::{City, Server, ServerFeatures};
pub use cache::countries_to_servers;

// Re-exported from async_tasks.rs
pub use async_tasks::{AsyncTaskManager};
```

## Testing

- **Parsing tests**: In `vpn::types::tests` (in types.rs)
- **Cache tests**: In `vpn::cache::tests` (in cache.rs)
- **Integration tests**: In `tests/` directory

## Anti-Patterns

### Don't add delegation wrappers

```rust
// DON'T DO THIS
mod wrapper {
    use super::VpnClient;
    pub struct Wrapper { client: VpnClient }
    impl Wrapper {
        pub fn connect_country(&self, s: &str) -> AppResult<...> {
            self.client.connect_country(s)? // Unnecessary indirection
        }
    }
}
```

### Don't put Mutex in cache.rs

The `Mutex` belongs in `client.rs` because:
1. It's tied to the lifetime of `VpnClient`
2. It provides thread-safe access to cache operations
3. `cache.rs` should only contain data/logic, not synchronization

### Don't duplicate parsing logic

If you need to parse CLI output, extend the existing functions in `types.rs` rather than creating new ones.
