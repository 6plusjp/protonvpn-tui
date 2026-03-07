# issue023: Feature - Filter state machine (proper ESC handling)

## Summary

Implement proper state machine for filter input: Normal → `/` → FilterInput → ESC → FilterActive → ESC → Normal.

## Current Behavior (to verify)

When user presses `/`:
1. Filter input appears
2. User types query
3. User presses ESC

**Current issue**: After ESC, state transitions may be unclear:
- Does ESC cancel filter input and return to Normal?
- Or does it apply the filter and go to FilterActive?

## Desired State Machine

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

### States

| State | Description | UI |
|-------|-------------|-----|
| **Normal** | Regular list view | Full server list |
| **FilterInput** | User typing filter | Input box at bottom |
| **FilterActive** | Filter applied | Filtered list + indicator |

### Transitions

| From | Key | To | Action |
|------|-----|-----|--------|
| Normal | `/` | FilterInput | Show input box |
| FilterInput | `Enter` | FilterActive | Apply filter |
| FilterInput | `Esc` | Normal | Discard input, clear filter |
| FilterActive | `Esc` | Normal | Clear filter, show all |
| FilterActive | `/` | FilterInput | Show input with current filter |

## Implementation Notes

### State Enum

```rust
pub enum FilterMode {
    Inactive,      // Normal view
    Input,        // User typing
    Active,       // Filter applied
}
```

### Current Issues to Fix

1. **ESC in FilterInput**: Should discard input and return to Normal
2. **ESC in FilterActive**: Should clear filter and return to Normal
3. **Navigation during filter**: j/k should navigate filtered list
4. **Filter indicator**: Show when filter is active (e.g., "Filter: abc [ESC to clear]")

### Edge Cases

- Empty filter input + Enter: Should show all (same as ESC)
- Filter with no results: Show "No results" message
- Switching views clears filter

---

## Acceptance Criteria

- [ ] `/` enters FilterInput mode
- [ ] ESC in FilterInput discards input, returns to Normal
- [ ] Enter in FilterInput applies filter, enters FilterActive
- [ ] ESC in FilterActive clears filter, returns to Normal
- [ ] Visual indicator shows current filter when active
- [ ] j/k navigation works correctly in all filter states
- [ ] No state leaks between view switches
