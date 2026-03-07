# issue024: Feature - Lazy country/city loading on cursor movement

## Summary

Load cities when user navigates to a country via cursor movement (j/k), not just on Enter/l.

## Problem

Currently, cities are only loaded when user presses `Enter` or `l` on a country. This means:
1. User must explicitly select country to see cities
2. Cities can't be preloaded for faster navigation

## Solution

Load cities automatically when cursor moves to a new country (j/k navigation):

### Behavior

```
┌─────────────────────────────────────────────┐
│ Countries         │ Cities                 │
├────────────────────┼───────────────────────┤
│ > United States    │ [cities load here]    │
│   Japan            │                       │
│   Germany          │                       │
└────────────────────┴───────────────────────┘
       ↑ cursor on US
```

When cursor moves from US to JP:
1. If cities for JP are empty → trigger `load_cities("JP")`
2. Show loading indicator
3. Update cities pane when ready

### Conditions for Loading

| Condition | Action |
|-----------|--------|
| Cursor moves to new country | Check if cities empty |
| Cities empty | Load cities for that country |
| Cities already loaded | Use cached (no reload) |
| Already loading | Don't trigger duplicate load |

### Implementation

In cursor movement handler:

```rust
fn move_cursor_down(&mut self) {
    let old_idx = self.selected_server;
    // ... move cursor ...
    let new_idx = self.selected_server;
    
    // If country changed, load cities if needed
    if old_idx != new_idx {
        self.load_cities_for_selected_server();
    }
}
```

### UI Feedback

- Show spinner/loading indicator in cities pane while loading
- Brief notification: "Loading cities for Japan..."

---

## Related: Fix issue018 (load_cities race condition)

This feature will expose the race condition more frequently. Ensure issue018 is fixed first.

---

## Acceptance Criteria

- [ ] Moving cursor to new country triggers city load (if empty)
- [ ] Cached cities are not reloaded
- [ ] Loading indicator shows during fetch
- [ ] No duplicate loads for same country
- [ ] Works with both j/k navigation
- [ ] Race condition fixed (issue018)
