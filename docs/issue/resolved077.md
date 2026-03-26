# issue077: Refactor - Navigation method duplication

## Summary

**RESOLVED** - Refactored navigation.rs to eliminate duplicated selection methods using a generic helper.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`navigation.rs` had 24 methods with identical patterns duplicated 4 times (server, city, settings, logs selection).

## Solution

Extracted a generic `navigate_inner` helper function that handles all navigation actions (Next, Prev, First, Last, PageDown, PageUp). Each selection group now uses this helper instead of duplicating the logic.

### Changes

| File | Change |
|------|--------|
| `src/state/navigation.rs` | Added `NavAction` enum and `navigate_inner` helper; refactored all 24 methods |

### Code Reduction

- **Before**: ~412 lines
- **After**: ~350 lines
- **Reduction**: ~60 lines

## Acceptance Criteria

- [x] Generic `navigate_inner` helper implemented
- [x] All 24 navigation methods use the helper
- [x] Existing tests pass without modification
- [x] Behavior is identical to current implementation

## Verification

```bash
cargo test navigation
# test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 67 filtered out
```
