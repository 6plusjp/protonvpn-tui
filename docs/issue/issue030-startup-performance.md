# issue030 - Startup Performance and UX Improvements

## Status: Partially Implemented (2026-03-11)

### Implemented

- [x] Async initialization (non-blocking startup)
- [x] Loading indicator on startup
- [x] `is_initialized` flag to track initialization state

### Not Implemented

- [ ] Error propagation to UI (errors shown via notification, but not during init)
- [ ] Dirty flag optimization (hash computation still used)
- [ ] Progress indicator (percentage) in loading view

---

## Summary

The application has startup performance issues and missing error handling during initialization. These issues cause the app to freeze and provide poor user feedback.

## Problems Identified

### 1. Blocking Initialization (High Priority) ✅ RESOLVED

**Location**: `src/ui/app.rs:41-43`

**Before**:
```rust
let mut state = AppState::new();
state.sync_connection_state();  // Blocks - executed protonvpn command
state.refresh_servers();         // Blocks - waited for result
```

**After** (non-blocking):
```rust
let mut state = AppState::new();
state.refresh_servers();  // Spawns background task, returns immediately
```

**Changes**:
- Removed blocking `sync_connection_state()` from startup
- Use cached servers only at startup
- Refresh servers in background via existing async infrastructure

### 2. Error Handling Ignored (Medium Priority)

**Status**: Partially addressed - errors are shown via notifications after initialization completes

### 3. Hash Computation Overhead (Low Priority)

**Status**: Not implemented - hash computation still used

### 4. No Loading Indicator (Medium Priority) ✅ RESOLVED

**Solution**: Added `render_loading()` function that shows centered "Loading servers..." message during initialization

```
┌─────────────────────────────────┐
│        ProtonVPN TUI            │
│                                 │
│      Loading servers...         │
│                                 │
└─────────────────────────────────┘
```

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| High | Async initialization with loading screen | Medium | ✅ Done |
| Medium | Error propagation to UI | Low | Partial |
| Medium | Loading indicator on startup | Low | ✅ Done |
| Low | Dirty flag optimization | Low | Pending |

## Files Changed

- `src/state/app_state.rs`: Added `is_initialized` flag, updated sync logic
- `src/ui/app.rs`: Added `render_loading()` function, updated render loop

## References

- Related issue: issue029 (similar timing issue)
- ratatui async patterns
