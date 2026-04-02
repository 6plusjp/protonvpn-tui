# ProtonVPN CLI Concurrent Execution

## Overview

Analysis of whether multiple `protonvpn` commands can run concurrently.

**Note**: This is based on code analysis, not official Proton documentation.

---

## Concurrent Execution Matrix

| Command A | Command B | Can Run Concurrently? |
|-----------|-----------|---------------------|
| `protonvpn countries list` | `protonvpn cities list <CC>` | ✅ Yes |
| `protonvpn countries list` | `protonvpn status` | ✅ Yes |
| `protonvpn cities list JP` | `protonvpn cities list US` | ✅ Yes |
| `protonvpn connect` | `protonvpn connect` | ❌ No |
| `protonvpn connect --random` | `protonvpn connect` | ❌ No |
| `protonvpn connect` | `protonvpn disconnect` | ❌ No |
| `protonvpn connect` | `protonvpn countries list` | ✅ Yes |
| `protonvpn connect` | `protonvpn cities list <CC>` | ✅ Yes |
| `protonvpn connect` | `protonvpn config set <key> <val>` | ✅ Yes (config is independent) |

---

## Analysis

### Read-Only Commands (Can Run in Parallel)

- `protonvpn countries list`
- `protonvpn cities list <CC>`
- `protonvpn info`
- `protonvpn status`

These commands only:
1. Create a new `Controller` instance
2. Make HTTP API calls to fetch server information
3. Do NOT interact with NetworkManager

Since each invocation creates a fresh API session, multiple read-only commands can run concurrently.

### Connection Commands (Cannot Run in Parallel)

- `protonvpn connect`
- `protonvpn connect --random`
- `protonvpn connect --country <CC>`
- `protonvpn connect --city <city>`
- `protonvpn connect --p2p`
- `protonvpn connect --tor`
- `protonvpn connect --securecore`
- `protonvpn disconnect`

These commands interact with **NetworkManager** to manage VPN connections. NetworkManager only allows one active VPN connection at a time, so these commands must be serialized.

### Config Commands

- `protonvpn config set <key> <value>`

Config commands modify local settings and can run concurrently with connection commands, but should not be run in parallel with each other to avoid race conditions.

### Cross-Compatibility

- Read-only commands (`countries`, `cities`, `status`, `info`) can run while a VPN is connected
- Config commands can run while a VPN is connected
- Connection commands must be serialized (only one at a time)

---

## Implementation Notes

### Current protonvpn-tui Implementation

The current implementation uses a **thread pool** (`AsyncTaskManager`) for VPN operations:

```rust
// src/vpn/async_tasks.rs
pub struct AsyncTaskManager {
    pool: Arc<ThreadPool>,
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self::new_with_workers(10)  // Default: 10 worker threads
    }
}
```

The thread pool uses a job queue with `Condvar` for efficient blocking:

| Component | Purpose |
|-----------|---------|
| `ThreadPool` | Fixed-size thread pool for VPN operations |
| `Job` enum | Different VPN operations (Connect, Disconnect, Refresh, etc.) |
| `AsyncNotifier` | Event-driven notification system for async results |
| `AsyncEvent` enum | Result types from async operations |

**Job Types**:
- `RefreshServers` - Fetch country/city lists
- `Connect` / `ConnectRandom` / `ConnectCity` - Connection operations
- `Disconnect` - Disconnection
- `Cities` - Fetch cities for a country
- `ConfigSet` - Modify VPN settings
- `ConnectFastest` / `ConnectP2P` / `ConnectTor` / `ConnectSecureCore` - Special connection types

### Concurrency Control

The thread pool allows multiple jobs to be queued and executed in parallel, but the underlying `protonvpn` CLI enforces serialization for connection commands. The TUI handles this by:

1. Queueing jobs to the thread pool
2. Each thread executes `protonvpn` CLI via `std::process::Command`
3. NetworkManager serializes VPN connection operations
4. Results are sent via `AsyncNotifier` channels

### Previous Implementation (Historical)

Earlier versions used sequential async tasks with `pending_*` fields:
- `pending_refresh` - server list refresh
- `pending_cities` - cities fetch
- `pending_connect` - VPN connection
- `pending_disconnect` - VPN disconnection
- `pending_connect_city` - city connection

This was replaced with the thread pool approach for better performance and parallelism.

---

## Sources

- `proton-vpn-cli` repository: https://github.com/ProtonVPN/proton-vpn-cli
- `location_discovery.py` - countries/cities commands
- `controller.py` - core logic
- `src/vpn/async_tasks.rs` - Thread pool implementation
- `src/vpn/client.rs` - CLI execution layer

---

## Revision History

- 2026-04-02: Updated command syntax, added thread pool documentation, added config commands
- 2026-03-07: Initial analysis based on code review
