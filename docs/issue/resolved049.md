# issue049-filterbox-occupies-space-when-inactive

## Summary

The filter box occupies valuable screen space even when inactive. It should only be displayed when filtering is active, and be a minimal 1-line input that replaces the footer.

## Status

**[Implemented]**

## Implementation

### Changes Made

| File | Change |
|------|--------|
| `src/ui/app.rs:886-932` | Modified `render()` layout - filter replaces footer when active |
| `src/ui/app.rs:941-980` | Rewrote `render_filter_input()` - 1-line, no border, prompt + placeholder |
| `src/ui/app.rs:983-1000` | Rewrote `render_dns_input()` - same style as filter |
| `src/ui/app.rs:7,64,118,121,127` | Added `SetCursorStyle::SteadyBar` for cursor shape |

### Layout Logic

```rust
// Filter/DNS input replaces footer when active
let show_filter = self.filter_mode || has_filter_active;
let is_dns_input = self.state.ui_state.input_mode == InputMode::DnsInput;
let show_footer = self.state.ui_state.show_footer && !show_filter && !is_dns_input;
```

- Filter active: Header + Main + Filter line (1行)
- DNS input: Header + Main + DNS input line (1行)
- Normal: Header + Main + Footer (1行)

### Filter Input Rendering

```rust
// Prompt + placeholder with different colors
let prompt = "find: ";
let placeholder = "type to filter...";

let line = if input_text.is_empty() {
    Line::from(vec![
        Span::styled(prompt, Style::default().fg(theme.primary)),
        Span::styled(placeholder, Style::default().fg(theme.muted)),
    ])
} else {
    // show input text
};
```

### DNS Input Rendering

```rust
let prompt = "DNS IPs: ";
let placeholder = "comma-separated IPs...";
```

### Cursor

- Uses `SteadyBar` cursor shape
- Set at TUI start/exit
- Position: after prompt (with padding 2)

### Placeholder States

| State | Display |
|-------|---------|
| Filter empty | `find: type to filter...` |
| Filter typing | `find: {input}` |
| DNS empty | `DNS IPs: comma-separated IPs...` |
| DNS typing | `DNS IPs: {input}` |

## Notes

- Filter and footer are mutually exclusive (save vertical space)
- 1-line input at footer position (minimal, vim-like)
- Prompt in primary color, placeholder in muted color
- Padding 2 on left for visual breathing room
- DNS input follows same pattern as filter
