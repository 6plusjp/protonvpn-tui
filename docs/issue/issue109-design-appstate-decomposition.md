# issue034: Design - AppState Decomposition

## Summary

Design and evaluate breaking down the monolithic 12-field `AppState` into composed sub-state structs.

## Problem

`src/state/app_state.rs` contains a struct with 12+ fields:

```rust
pub struct AppState {
    pub vpn_state: Arc<VpnClient>,
    pub connection_manager: ConnectionManager,
    pub servers: Vec<Server>,
    pub current_cities: Vec<City>,
    pub favorites: Vec<ServerId>,
    pub ui_state: UiState,
    pub notification_state: NotificationState,
    pub config_state: ConfigState,
    // ... more
}
```

This violates Single Responsibility Principle and makes testing difficult.

## Design Options

### Option A: Keep as-is (Minimal Change)
- Keep `AppState` monolithic
- Add clear documentation
- Add tests for each field accessor

### Option B: Group into Sub-states
```rust
pub struct AppState {
    pub vpn: VpnState,           // vpn_state + connection_manager
    pub data: DataState,         // servers + cities + favorites
    pub ui: UiState,          // current view, notifications
    pub config: ConfigState,   // user preferences
}
```

### Option C: Builder Pattern
```rust
pub struct AppState {
    inner: AppStateInner,   // Private
}

impl AppState {
    pub fn vpn(&self) -> &VpnClient { ... }
    pub fn servers(&self) -> &[Server] { ... }
}
```

## Investigation Required

- [ ] Evaluate Option B/C impact on all callers
- [ ] Check for field-level locking/mutex patterns that would break
- [ ] Consider impact on `Arc<AppState>` pattern used in async tasks

## Notes

This is a **design issue**, not a bug. Recommend starting with:
1. Document current usage patterns
2. Model sub-state boundaries
3. Prototype one sub-state extraction as proof-of-concept

### Files to Investigate

| File | Usage |
|------|-------|
| `src/ui/app.rs` | AppState access patterns |
| `src/state/connection_manager.rs` | Async state access |
| `src/state/*` | All field accesses |