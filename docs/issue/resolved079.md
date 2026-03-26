# issue079: Refactor - Footer span styling repetition

## Summary

**RESOLVED** - Refactored footer.rs to eliminate repeated Span::styled patterns using helper functions.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

`renderers/footer.rs` had 40+ repeated `Span::styled()` patterns for keyboard hints.

## Solution

Extracted helper functions for creating styled spans: `hint_bracket_open`, `hint_bracket_close`, `hint_key`, `hint_action`, `hint_spacer`, `hint`, and `hint_with_spacer`.

### Changes

| File | Change |
|------|--------|
| `src/ui/renderers/footer.rs` | Added 7 helper functions; refactored all hint rendering |

### Code Reduction

- **Before**: ~261 lines
- **After**: ~143 lines
- **Reduction**: ~118 lines

## Acceptance Criteria

- [x] Helper functions implemented
- [x] All hints use the helpers
- [x] Visual appearance unchanged
- [x] Tests pass

## Verification

```bash
cargo test
# test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
