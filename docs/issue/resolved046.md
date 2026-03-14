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

## Implementation

### 1. Footer Hints for Tools View

Added hints to `get_footer_action_hints()` in `src/ui/app.rs`:

**Settings pane hints:**
- `[j/k]` navigate (via common navigation)
- `[gg]` first - go to first item
- `[G]` last - go to last item
- `[C-d]` page down
- `[C-u]` page up
- `[l]` switch to Logs pane
- `[Enter]` toggle expand
- `[c]` connect (if disconnected)

**Logs pane hints:**
- `[j/k]` navigate
- `[h]` switch to Settings pane

### 2. Footer Visibility Toggle

Added UI-only setting to toggle footer:

**Implementation locations:**
- `src/config/settings.rs`: Added `SettingKey::Footer` variant with options `["on", "off"]`
- `src/state/ui_state.rs`: Added `show_footer: bool` field with default `true`
- Added `toggle_footer()` method
- Added `settings_last_key_g: bool` for gg key sequence

**Render logic:**
- Check `state.ui_state.show_footer` before calling `render_footer()`
- When false, remove `Constraint::Length(1)` for footer from layout

## Files Modified

| File | Changes |
|------|---------|
| `src/ui/app.rs` | Added hints for Tools panes, conditionally render footer, gg/G key handling, common navigation |
| `src/config/settings.rs` | Added `SettingKey::Footer` |
| `src/state/ui_state.rs` | Added `show_footer` and `settings_last_key_g` fields, `toggle_footer()` method |
| `src/ui/views/settings_view.rs` | Added Footer handling in display |
| `src/state/app_state.rs` | Added Footer to match arm (unreachable) |

## Notes

- Theme toggle (`SettingKey::Theme`) already exists as UI-only setting - used as reference
- Footer hints follow existing style pattern: `Span::styled("key", Style::default().fg(theme.key_hint))`
- Default footer visibility: ON (backward compatible)
- Navigation uses common key bindings (Ctrl+d, Ctrl+u, etc.) via `handle_common_navigation()`
- `gg` key sequence implemented with `settings_last_key_g` flag