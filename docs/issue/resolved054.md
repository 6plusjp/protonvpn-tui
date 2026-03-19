# Issue 054: ServerCache Name Collision

## Status

**[Resolved]**

## Created: 2026-03-19

## Summary

There are **two different `ServerCache` structs** in the codebase with the same name but completely different purposes.

---

## Problem

| Location | Purpose |
|----------|---------|
| `vpn/cache.rs` | Disk-persisted cache of countries and cities |
| `state/ui_state.rs` | In-memory filtered server cache with versioning for invalidation |

---

## Resolution

Renamed `ServerCache` in `state/ui_state.rs` to `FilteredServerCache`.

## Changes Made

| File | Change |
|------|--------|
| `src/state/ui_state.rs` | Renamed `ServerCache` → `FilteredServerCache` |
| `src/state/app_state.rs` | Updated import and usage to `FilteredServerCache` |

### Diff

```diff
// src/state/ui_state.rs
-pub struct ServerCache {
+pub struct FilteredServerCache {

-impl ServerCache {
+impl FilteredServerCache {

-impl Default for ServerCache {
+impl Default for FilteredServerCache {

// src/state/app_state.rs
-use crate::state::ServerCache;
+use crate::state::FilteredServerCache;

-server_cache: ServerCache,
+server_cache: FilteredServerCache,

-server_cache: ServerCache::new(),
+server_cache: FilteredServerCache::new(),
```

## Verification

- `cargo check` ✓
- `cargo test` ✓ (163 tests passed, 2 ignored)
