# issue030 - Startup Performance and UX Improvements

## Status: Partially Implemented (2026-03-11)

### Implemented

- [x] Async initialization (non-blocking startup)
- [x] Loading indicator on startup
- [x] `is_initialized` flag to track initialization state
- [x] Error propagation to UI (errors shown via notification)

### Not Implemented

- [x] ~~Error propagation to UI~~ - Implemented via async event notifications
- [x] ~~Dirty flag optimization~~ Hash computation replaced with ratatui's automatic repaint detection
- [x] ~~Progress indicator (percentage)~~ - Basic loading text implemented; percentage not feasible

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

### 3. Hash Computation Overhead (Low Priority) → Use ratatui Auto-Repaint

**Status**: Not implemented - currently using manual hash-based change detection

**Current Implementation**:
```rust
// src/ui/app.rs - 毎フレーム実行
let current_hash = self.compute_render_hash();
if current_hash != self.last_render_hash {
    terminal.draw(|f| self.render(f))?;
    self.last_render_hash = current_hash;
}
```

**Proposed Solution**: Use ratatui's automatic repaint detection

Instead of manually computing hash, leverage ratatui's built-in frame comparison:
- Remove `compute_render_hash()` function
- Remove `last_render_hash` field from App
- Let ratatui handle frame diffing automatically

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
| Medium | Error propagation to UI | Low | ✅ Done |
| Medium | Loading indicator on startup | Low | ✅ Done |
| Medium | Remove redundant hardcoded countries list | Low | Pending |
| Low | Hash computation → ratatui auto-repaint | Low | Pending |

---

## UX Redundancy: Loading Indicator vs Hardcoded Countries

### Problem

There are two mutually exclusive approaches for startup UX:

1. **Loading Indicator**: Show "Loading..." → wait for data → display countries
   - Pros: Always shows fresh data
   - Cons: User sees nothing initially

2. **Hardcoded Countries**: Show countries immediately from `FALLBACK_COUNTRIES`
   - Pros: Instant display
   - Cons: Data may be stale/outdated

**Current State**: Both are implemented (redundant)
- Loading indicator exists (`render_loading()`)
- Hardcoded list exists (`FALLBACK_COUNTRIES` in cache.rs)

### Decision

Remove `FALLBACK_COUNTRIES` and rely on loading indicator only:
- Simpler code (remove ~140 lines of hardcoded data)
- Consistent UX (always load fresh data)
- Works better with async initialization

### Implementation

1. Remove `FALLBACK_COUNTRIES` from `src/vpn/cache.rs`
2. Remove usage in `src/vpn/client.rs:refresh_countries()`
3. Test: startup shows loading → then shows actual countries

## Implementation Impact

### Using ratatui Auto-Repaint

**Benefits:**
- Removes ~30 lines of hash computation code
- Simpler render loop (no manual state tracking)
- Ratatui handles frame diffing efficiently

**Risks / Considerations:**
- Behavior change: must test that UI still updates correctly when state changes
- Some edge cases (e.g., animations) may need explicit `frame.request_repaint()`
- Requires ratatui version that supports this feature

**Migration Steps:**
1. Remove `last_render_hash: u64` field from `App` struct
2. Remove `compute_render_hash()` method
3. Simplify render loop to always draw (or use ratatui's built-in comparison)
4. Test all views: servers, settings, logs, help

## Files Changed

- `src/state/app_state.rs`: Added `is_initialized` flag, updated sync logic
- `src/ui/app.rs`: Added `render_loading()` function, updated render loop

## References

- Related issue: issue029 (similar timing issue)
- ratatui async patterns
