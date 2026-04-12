# issue029: Refactor - Row Styling Helper Extraction

## Summary

Extract duplicated row styling logic in `servers_view.rs` into reusable helper functions.

## Problem

Identical style tuple creation pattern appears twice in `servers_view.rs` (lines 118-141 and 257-266):

```rust
let (row_style, status_style, code_style, country_style, cities_style) =
    if is_selected && is_focused {
        let s = Style::default().fg(theme.background).bg(theme.accent).add_modifier(Modifier::BOLD);
        (s, s, s, s, s)
    } else if is_selected && is_connected {
        let s = Style::default().fg(theme.success).add_modifier(Modifier::BOLD);
        (s, s, s, s, s)
    } else if is_selected {
        let s = Style::default().fg(theme.foreground);
        (s, s, s, s, s)
    } else if is_connected {
        let s = Style::default().fg(theme.success).add_modifier(Modifier::BOLD);
        (s, s, s, s, s)
    } else {
        let n = Style::default().fg(theme.foreground);
        (n, n, n, n, n)
    };
```

## Solution

Extract to helper functions in `servers_view.rs`:

```rust
/// Compute all styles for a server row based on selection state
fn compute_server_row_styles(
    is_selected: bool,
    is_focused: bool,
    is_connected: bool,
    theme: &Theme,
) -> (Style, Style, Style, Style, Style) {
    // ... logic extracted
}

/// Convenience for computing a single highlight style
fn compute_highlight_style(
    is_selected: bool,
    is_focused: bool,
    theme: &Theme,
) -> Style { ... }
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/views/servers_view.rs` | Extract to helper functions |

### Acceptance Criteria

- [ ] Duplicated code eliminated (~25 lines reduced)
- [ ] Function logic preserved
- [ ] `cargo test` passes
- [ ] Clear, documented helpers for future modification