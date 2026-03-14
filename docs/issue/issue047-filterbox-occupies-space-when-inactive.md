# issue047-filterbox-occupies-space-when-inactive

## Summary

The filter box occupies valuable screen space even when inactive. It should only be displayed when filtering is active.

## Context

### Current Behavior

In `src/ui/app.rs`, the rendering logic always reserves 3 rows for the filter box:

```rust
// Line 857-865: Always allocates 3 rows for filter
let chunks = Layout::default()
    .constraints([
        Constraint::Length(3),  // Header
        Constraint::Length(3),  // ← Always reserved for filter
        Constraint::Min(0),    // Main view
        Constraint::Length(1), // Footer
    ])
    .split(f.size());
```

Even when:
- `filter_mode == false` (not actively typing in filter)
- `has_filter_active == false` (search query is empty)

It still shows a placeholder "[press / to search]" and wastes 3 rows.

### Expected Behavior

The filter box should only appear when filtering is active:
- When user presses `/` to enter filter mode → show
- When there's an active search query → show
- Otherwise → hide (main view expands)

## Requirements

### 1. Conditional Layout for Filter Box

Modify `render()` in `src/ui/app.rs`:

```rust
let show_filter = self.filter_mode || has_filter_active;

let constraints = if show_filter {
    vec![
        Constraint::Length(3), // Header
        Constraint::Length(3), // Filter (when active)
        Constraint::Min(0),    // Main view
        Constraint::Length(1), // Footer
    ]
} else {
    vec![
        Constraint::Length(3), // Header
        Constraint::Min(0),     // Main view (expanded)
        Constraint::Length(1), // Footer
    ]
};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints)
    .split(f.size());
```

### 2. Conditional Render

Since chunk indices shift:

```rust
if show_filter {
    self.render_filter_input(f, chunks[1], has_filter_active);
    self.render_main(f, chunks[2]);
    self.render_footer(f, chunks[3]);
} else {
    self.render_main(f, chunks[1]);
    self.render_footer(f, chunks[2]);
}
```

### 3. Dynamic Footer Hint (NEW)

When filter mode is active (`filter_mode == true`), change the footer hint from the default view hints to filter-specific hints:

**Normal state:**
```
[j/k] navigate  [Enter] connect  [c] country  [d] disconnect
```

**Filter mode active:**
```
[Esc] cancel  [Enter] apply filter
```

This gives users immediate feedback that they're in filter mode and how to exit.

## Files to Modify

| File | Changes |
|------|---------|
| `src/ui/app.rs` | Modify `render()` to conditionally allocate filter box, adjust render calls, update footer hints for filter mode |

## Notes

- Existing filter logic (`filter_mode`, `filter_input`, `has_filter_active`) can be reused as-is
- This issue can be combined with issue046 if footer hints are being updated there
- UX: Starting with filter hidden, then showing when user presses `/` feels natural
