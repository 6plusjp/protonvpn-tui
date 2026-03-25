# issue061: Architecture - ServerCache misplaced in ui_state.rs

## Summary

`ServerCache` in `src/state/ui_state.rs` is **not UI state** - it's a caching mechanism for filtered server lists.

## Problem

```rust
// src/state/ui_state.rs
pub struct ServerCache {
    cache: RwLock<Option<(Vec<Server>, u64)>>,
    version: u64,
}
```

It caches the results of server filtering operations, which is a **domain/data concern**, not a UI concern.

## Misplaced Classification

| Current Location | Proper Location |
|-----------------|-----------------|
| `state/ui_state.rs` | `vpn/` or `server_data.rs` |

## Files Affected

- `src/state/ui_state.rs` - contains misplaced `ServerCache`

## Recommendation

Move `ServerCache` to a more appropriate location:

**Option 1**: Move to `vpn/cache.rs` alongside other caching logic
**Option 2**: Move to `state/server_data.rs` (after cleaning up the unused `ServerDataState`)

Also rename to `FilteredServerCache` to avoid collision with `vpn::ServerCache`.

## Severity

🟢 LOW - Misclassification, not a bug

## Labels

`architecture` `refactoring`
