# issue036 - Move async_tasks.rs to vpn/ module

## Summary

Move `async_tasks.rs` from `src/state/` to `src/vpn/` to improve module organization.

## Background

From issue027 (S3):

> Move `async_tasks.rs` to `vpn/`
> 
> **Status**: Skipped - Requires file move
> **Reason**: Simple file move but requires updating imports across codebase.

## Current State

```
src/
├── state/
│   ├── async_tasks.rs    # VPN async operations
│   └── app_state.rs
└── vpn/
    ├── client.rs
    ├── cache.rs
    └── types.rs
```

## Proposed Change

```
src/
├── state/
│   └── app_state.rs
└── vpn/
    ├── async_tasks.rs    # Moved here
    ├── client.rs
    ├── cache.rs
    └── types.rs
```

## Files to Update

- Move: `src/state/async_tasks.rs` → `src/vpn/async_tasks.rs`
- Update imports in:
  - `src/state/app_state.rs`
  - `src/ui/app.rs`
  - `src/vpn/mod.rs` (add module export)

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| Low | Move async_tasks.rs to vpn/ | Low | Pending |

## Notes

- Pure refactor, no functional changes
- Improves logical grouping (VPN operations in vpn/ module)
