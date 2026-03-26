# issue084: Refactor - Unify block creation across views

## Summary

**RESOLVED** - Added `block_with_title()` helper and updated tools_view.rs to use it.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`tools_view.rs` created blocks manually while `servers_view.rs` used `centered_block()`.

## Solution

Added `block_with_title(title: String, theme: &Theme, focused: bool)` to `components/block.rs` and updated `tools_view.rs` to use it.

### Changes

| File | Change |
|------|--------|
| `src/ui/components/block.rs` | Added `block_with_title()` function |
| `src/ui/components/mod.rs` | Updated exports |
| `src/ui/views/tools_view.rs` | Replaced manual block creation with `block_with_title()` |

### Code Reduction

- **tools_view.rs**: ~20 lines removed

## Acceptance Criteria

- [x] `block_with_title()` helper implemented
- [x] `tools_view.rs` uses component functions
- [x] Visual appearance unchanged
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
