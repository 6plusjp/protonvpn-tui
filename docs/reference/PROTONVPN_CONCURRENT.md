# ProtonVPN CLI Concurrent Execution

## Overview

Analysis of whether multiple `protonvpn` commands can run concurrently, based on official CLI source code.

**Verified against**: `proton-vpn-cli` v0.1.x source code (GitHub: ProtonVPN/proton-vpn-cli)

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

### Official CLI Architecture (from source code)

Each `protonvpn` command invocation creates a new `Controller` instance:

```python
# location_discovery.py:76
controller = await Controller.create(params=ctx.obj, click_ctx=ctx)
```

Both `countries list` and `connect` internally call `get_updated_server_list()`:

```python
# controller.py - get_all_countries() (used by countries list)
async def get_all_countries(self) -> List[Country]:
    server_list = await self.get_updated_server_list()
    return server_list.group_by_country()

# controller.py - find_logical_server() (used by connect)
async def find_logical_server(...):
    server_list = await self.get_updated_server_list()
    # ... find server
```

This means:
- Both commands fetch/update server list from Proton API
- Each CLI invocation is a separate process (no shared state at process level)
- API requests may be duplicated but don't interfere

### Commands Calling `get_updated_server_list()`

All commands that internally call `get_updated_server_list()`:

| Command | Call Path | Source File |
|---------|-----------|-------------|
| `protonvpn countries list` | `get_all_countries()` → `get_updated_server_list()` | `location_discovery.py:83` |
| `protonvpn cities list <CC>` | `get_all_countries()` → `get_updated_server_list()` | `location_discovery.py:146` |
| `protonvpn connect` | `find_logical_server()` → `get_updated_server_list()` | `server.py:108` |
| `protonvpn status` (when connected) | Direct call to `get_updated_server_list()` | `server.py:241` |

**Flow diagram**:

```
┌─────────────────────┐
│ countries list      │──→ get_all_countries() ──┐
└─────────────────────┘                            │
┌─────────────────────┐                            ├──→ get_updated_server_list()
│ cities list <CC>   │──→ get_all_countries() ────┤
└─────────────────────┘                            │
┌─────────────────────┐                            │
│ connect             │──→ find_logical_server() ─┤
└─────────────────────┘                            │
┌─────────────────────┐                            │
│ status (connected)  │───────────────────────────┘
└─────────────────────┘
```

**Key insight**: Every command that fetches server information calls `get_updated_server_list()`, so running multiple such commands simultaneously will result in duplicate API requests (but no interference since each CLI invocation is a separate process).

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

impl Default for AsyncTaskManager {
    fn default() -> Self {
        Self::new_with_workers(4)  // Default trait: 4 worker threads
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
- `proton/vpn/cli/commands/location_discovery.py` - countries/cities commands implementation
- `proton/vpn/cli/core/controller.py` - core logic with `get_updated_server_list()`
- `src/vpn/async_tasks.rs` - Thread pool implementation
- `src/vpn/client.rs` - CLI execution layer

---

## Revision History

- 2026-04-06: Added full list of commands calling `get_updated_server_list()` - verified all server-fetching commands call this method internally
- 2026-04-06: Updated with official source code analysis - verified both `countries list` and `connect` call `get_updated_server_list()`, each CLI invocation creates new Controller instance
- 2026-04-02: Updated command syntax, added thread pool documentation, added config commands
- 2026-03-07: Initial analysis based on code review
