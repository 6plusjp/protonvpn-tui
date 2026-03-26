# issue082: Refactor - Consolidate AsyncEvent and Job enums

## Summary

**RESOLVED** - Refactored async_tasks.rs to reduce duplication using `execute_and_notify` helper.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`async_tasks.rs` had a 120-line `execute_job` match block with duplicated patterns for each job type.

## Solution

Created an `execute_and_notify` helper function that handles the common pattern of executing an operation and sending success/failure events. Each job variant now uses this helper.

### Changes

| File | Change |
|------|--------|
| `src/vpn/async_tasks.rs` | Added `execute_and_notify` helper; refactored `execute_job` |

### Code Reduction

- **Before**: ~396 lines
- **After**: ~350 lines
- **Reduction**: ~46 lines

## Acceptance Criteria

- [x] `execute_and_notify` helper implemented
- [x] All job variants use the helper
- [x] Async operations work correctly
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
