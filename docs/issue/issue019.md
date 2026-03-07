# issue019: Bug - Footer inconsistency across views

## Summary

The footer displays inconsistently across different views, with some views missing action hints or having formatting issues.

## Problem

Currently, `render_footer()` in `src/ui/app.rs` provides different footer hints based on `(AppView, Pane)` combination, but:

1. **Some views are missing from the match** - need to verify all `AppView` variants are covered
2. **Filter mode doesn't show footer** - when in filter input mode, no keyboard hints are displayed
3. **Inconsistent formatting** - some views end with `| ` separator, others don't

## Current Coverage Analysis

| View | Pane | Has Footer? |
|------|------|------------|
| Servers | Countries | ✓ Complete |
| Servers | Cities | ✓ Complete |
| Settings | - | ✓ Complete |
| Logs | - | ✓ (minimal: scroll only) |
| Help | - | ✓ (static text) |

## Additional Issues to Check

1. **Filter mode footer**: When user presses `/` to filter, what footer is shown? Should show:
   - `[Enter]` - execute filter
   - `[Esc]` - cancel filter

2. **Empty states**: When cities list is empty, does footer still show "cities" hint?

3. **Connection state**: When connected/disconnecting, are footer hints appropriate?

## Solution

1. Audit all `AppView` variants and ensure complete coverage
2. Add filter mode footer handling
3. Standardize separator format (trailing `| ` vs no separator)
4. Add conditional hints based on connection state

---

## Acceptance Criteria

- [ ] All views have appropriate, consistent footer hints
- [ ] Filter mode shows proper input hints
- [ ] Footer formatting is consistent (separator usage)
- [ ] Connection state affects available actions (e.g., "c" connect vs "d" disconnect)
