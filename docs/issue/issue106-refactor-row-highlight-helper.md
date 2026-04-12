# issue031: Refactor - Row Highlight Helper Extraction

## Summary

Extract duplicated `row_highlight_style` logic across 3 view files into a shared helper in `components/mod.rs`.

## Problem

Identical style pattern in 3 locations:

### servers_view.rs (lines 160-167, 286-293)
```rust
.row_highlight_style(if is_focused {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default()
})
```

### logs_view.rs (lines 102-109)
```rust
.row_highlight_style(if is_focused {
    Style::default()
        .fg(theme.background)
        .bg(theme.accent)
        .add_modifier(Modifier::BOLD)
} else {
    Style::default()
})
```

### tools_view.rs (similar pattern)

## Solution

Add helper to `src/ui/components/mod.rs`:

```rust
pub fn row_highlight_style(theme: &Theme, focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(theme.background)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    }
}
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/components/mod.rs` | Add `row_highlight_style()` helper |
| `src/ui/views/servers_view.rs` | Use helper |
| `src/ui/views/logs_view.rs` | Use helper |
| `src/ui/views/tools_view.rs` | Use helper |

### Acceptance Criteria

- [ ] Helper function added
- [ ] 3 view files refactored (or update planned)
- [ ] `cargo test` passes
- [ ] Estimated ~15 lines reduced