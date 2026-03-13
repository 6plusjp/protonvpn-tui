# issue046-footer-hints-and-toggle

## Summary

Add footer hints to Tools view (Settings/Logs) and implement footer visibility toggle in Settings.

## Context

Currently, the footer displays context-aware key hints in the Servers view but shows nothing in the Tools view (Settings/Logs). Additionally, there is no way to toggle footer visibility.

### Current Behavior

- **Servers + Countries pane**: Full key hints (j/k, l, c, d, r, f, p, t, s)
- **Servers + Cities pane**: Partial hints (j/k, c, d, h)
- **Tools + Settings pane**: Empty (no hints)
- **Tools + Logs pane**: Empty (no hints)
- **Footer visibility**: Always on, no toggle

### Expected Behavior

- **Tools + Settings pane**: Navigation and toggle hints
- **Tools + Logs pane**: Navigation hints
- **Footer visibility**: Toggleable via Settings

## Requirements

### 1. Footer Hints for Tools View

Add hints to `get_footer_action_hints()` in `src/ui/app.rs`:

**Settings pane hints:**
- `[j/k]` navigate
- `[Enter]` toggle expand
- `[t]` toggle setting on/off
- `[c]` connect (if disconnected)

**Logs pane hints:**
- `[j/k]` navigate

Use `theme.key_hint` for all key styling (already implemented pattern).

### 2. Footer Visibility Toggle

Add UI-only setting (not VPN config) to toggle footer:

**Implementation locations:**
- `src/config/settings.rs`: Add `SettingKey::Footer` variant
- `src/state/ui_state.rs`: Add `show_footer: bool` field with default `true`
- Add `toggle_footer()` method
- Add `Footer` to `SettingKey::ALL` array with options `["on", "off"]`

**Render logic:**
- Check `state.ui_state.show_footer` before calling `render_footer()`
- When false, reduce layout constraints (remove `Constraint::Length(1)` for footer)

## Files to Modify

| File | Changes |
|------|---------|
| `src/ui/app.rs` | Add hints for Tools panes, conditionally render footer |
| `src/config/settings.rs` | Add `SettingKey::Footer` |
| `src/state/ui_state.rs` | Add `show_footer` field and `toggle_footer()` |

## Notes

- Theme toggle (`SettingKey::Theme`) already exists as UI-only setting - use as reference
- Footer hints should follow existing style pattern: `Span::styled("key", Style::default().fg(theme.key_hint))`
- Default footer visibility: ON (backward compatible)
