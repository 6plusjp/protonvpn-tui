# issue030 - Startup Performance and UX Improvements

## Status: Partially Implemented (2026-03-12)

### Implemented

- [x] Async initialization (non-blocking startup)
- [x] Loading indicator as popup overlay (with TUI background)
- [x] `is_initialized` flag to track initialization state
- [x] Error propagation to UI (errors shown via notification)
- [x] Remove FALLBACK_COUNTRIES (error on CLI failure instead of fallback)

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

**Solution**: Added `render_loading()` function that shows popup overlay during initialization, while keeping the TUI background visible.

**Before** (full screen replacement):
```
┌─────────────────────────────────┐
│        ProtonVPN TUI            │
│                                 │
│      Loading servers...         │
│                                 │
└─────────────────────────────────┘
```

**After** (popup overlay with TUI background):
```
┌─────────────────────────────────────────┐
│  ● Disconnected              ProtonVPN │
├─────────────────────────────────────────┤
│  🔍 Search servers...                   │
├─────────────────────────────────────────┤
│                                         │
│    ┌─ Loading ─────────┐                │
│    │ ProtonVPN TUI    │                │
│    │                   │                │
│    │ Loading servers...│                │
│    └───────────────────┘                │
│                                         │
├─────────────────────────────────────────┤
│  ↑↓ navigate  c:connect  r:refresh      │
└─────────────────────────────────────────┘
```

**Implementation Details**:
- Removed early return in `render()` method
- Render main TUI first, then overlay loading popup
- Uses `Block::bordered()` with warning color border
- Uses `Clear` widget to clear popup background

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| High | Async initialization with loading screen | Medium | ✅ Done |
| Medium | Error propagation to UI | Low | ✅ Done |
| Medium | Loading popup overlay with TUI background | Low | ✅ Done |
| Medium | Remove redundant hardcoded countries list | Low | ✅ Done |
| Low | Hash computation → ratatui auto-repaint | Low | Pending |

---

## FALLBACK_COUNTRIES Removal (Completed)

### Decision

Removed `FALLBACK_COUNTRIES` and rely on loading indicator:
- Simpler code (remove ~130 lines of hardcoded data)
- Consistent UX (always load fresh data)
- Works better with async initialization
- CLI failure now shows error notification to user

### Implementation

1. Removed `FALLBACK_COUNTRIES` from `src/vpn/cache.rs`
2. Removed `use_fallback_countries()` from `src/vpn/client.rs`
3. Changed `refresh_countries()` to return `AppError` on CLI failure
4. CLI failure now displays error notification via async event system

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
- `src/ui/app.rs`: Added `render_loading()` function, updated render loop for popup overlay
- `src/vpn/cache.rs`: Removed `FALLBACK_COUNTRIES` constant (~130 lines)
- `src/vpn/client.rs`: Changed `refresh_countries()` to return error instead of fallback

## References

- Related issue: issue029 (similar timing issue)
- ratatui async patterns
