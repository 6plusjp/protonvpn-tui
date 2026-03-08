# issue023: Feature - Filter state machine (proper ESC handling)

## Summary

Implement proper state machine for filter input: Normal → `/` → FilterInput → ESC → FilterActive → ESC → Normal.

## Status: RESOLVED

Implemented in PR #xxx.

---

## Problem

When user presses `/`:
1. Filter input appears
2. User types query
3. User presses ESC

**Issue**: After ESC, state transitions were unclear - ESC did nothing when filter was already applied.

---

## Solution

### State Machine

```
┌──────────┐   '/'    ┌───────────────┐  input   ┌──────────────┐
│  Normal  │─────────→│ FilterInput   │─────────→│ FilterActive│
└──────────┘          └───────────────┘          └──────────────┘
     ↑                      │                         │
     │                      │ ESC                     │ ESC
     │                      ↓                         │
     │               ┌───────────────┐                │
     └───────────────│    Normal     │←───────────────┘
              (discard) └───────────────┘ (clear filter)
```

### Implementation

1. **Filter box always visible** - Shown between header and main view
2. **Visual states**:
   - Inactive: `filter: [press / to search]` (gray)
   - Input (editing): `filter: {text}` (primary color)
   - Active (applied): `filter: {text}` (green)

3. **Key handling**:
   - `/` - Enter filter input mode
   - Enter - Apply filter
   - Esc - Clear filter and return to Normal (both from Input and Active states)

### Files Changed

- `src/ui/app.rs`:
  - `render()` - Always show filter box between header and main
  - `render_filter_input()` - Visual states for filter
  - `handle_common_keys()` - Handle Esc to clear filter

---

## Acceptance Criteria

- [x] `/` enters FilterInput mode
- [x] ESC in FilterInput discards input, returns to Normal
- [x] Enter in FilterInput applies filter, enters FilterActive
- [x] ESC in FilterActive clears filter, returns to Normal
- [x] Visual indicator shows current filter when active
- [x] j/k navigation works correctly in all filter states
- [x] No state leaks between view switches

---

## Notes

- Filter box is always visible for better UX
- Filter query is preserved when pressing `/` during Active state
- Color coding helps identify current state
