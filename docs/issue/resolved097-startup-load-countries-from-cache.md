# issue097-startup-load-countries-from-cache.md

## Title

Load countries from cache on startup (instead of always executing CLI)

## Status

**Resolved** - Implemented in PR #XXX

## Problem

Currently, `refresh_servers()` is called on startup, which executes `protonvpn countries list` CLI:

```rust
// src/ui/app.rs:63
state.refresh_servers();
```

`ServerCache` is already loaded from disk (`~/.cache/protonvpn-tui/server_cache.toml`) during `VpnClient::new()`:

```rust
// src/vpn/client.rs:57-63
let cache = match ServerCache::load(cache_path.clone()) {
    Ok(c) => c,
    Err(e) => {
        tracing::warn!("Failed to load server cache: {}", e);
        ServerCache::default()
    }
};
```

### Root Cause

1. **Startup flow**: `TuiApp::new()` → `refresh_servers()` → **always executes CLI**
2. **Unnecessary wait**: CLI is executed even when valid cache exists
3. **Perceived slowness**: Users wait for CLI completion before seeing any servers

**Key point**: Whether cache is stale (older than 24h) cannot be determined until CLI executes. We should prioritize cache on startup only, and always execute CLI for all other cases.

## Solution

### Implementation

Added new method `servers_from_cache()` to `VpnClient`:

```rust
// src/vpn/client.rs:592-607
pub fn servers_from_cache(&self) -> AppResult<Vec<Server>> {
    let countries = self.with_cache(|c| c.countries.clone()).unwrap_or_default();
    let cities = self.with_cache(|c| c.cities.clone()).unwrap_or_default();

    if countries.is_empty() {
        return self.refresh_servers();
    }

    Ok(countries_to_servers(&countries, &cities))
}
```

Updated startup in `app.rs`:

```rust
// src/ui/app.rs:63-67
match state.vpn_state.servers_from_cache() {
    Ok(cached) if !cached.is_empty() => state.set_servers(cached),
    _ => state.refresh_servers(),
}
```

### Files Modified

| File | Change |
|------|--------|
| `src/vpn/client.rs` | Added `servers_from_cache()` method |
| `src/ui/app.rs` | Use `servers_from_cache()` instead of `refresh_servers()` |

## Behavior

| Scenario | Action |
|----------|--------|
| Cache has data | Load from cache immediately (no CLI) |
| Cache is empty | Execute CLI to get fresh data |
| r key pressed | Always execute CLI (unchanged) |

## Verification

- `cargo check`: Passed
- `cargo clippy`: Passed
- `cargo test`: 125 tests passed