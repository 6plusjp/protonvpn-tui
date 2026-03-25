# issue058: Naming - connection.rs naming confusion with connection_state.rs

## Summary

Two files with similar names serve completely different purposes.

## Problem

| File | Contains | Purpose |
|------|----------|---------|
| `src/state/connection.rs` | `ConnectionManager`, `AsyncEvent`, `AsyncNotifier` | Manages async operations and pending tasks |
| `src/state/connection_state.rs` | `ConnectionState` enum | VPN connection state values (Disconnected, Connecting, Connected, etc.) |

The similar names cause confusion about their respective responsibilities.

## Files Affected

- `src/state/connection.rs`

## Recommendation

Rename `connection.rs` to `connection_manager.rs` to clarify its purpose:

```bash
mv src/state/connection.rs src/state/connection_manager.rs
```

Then update:

- `src/state/mod.rs` - update module declaration
- All imports referencing `state::connection`

## Resolution

### 2026-03-19

Renamed `src/state/connection.rs` to `src/state/connection_manager.rs` and updated:

- `src/state/mod.rs` - updated module declaration
- `src/state/AGENTS.md` - updated documentation

All imports via `crate::state::*` continue to work due to re-export in `mod.rs`.

## Severity

🟡 MEDIUM - Naming confusion, not a bug

## Labels

`naming` `architecture`
