# issue013: Yazi-style Split-pane Navigation

## Summary

Implement Yazi-inspired split-pane navigation where Servers list and Cities panel are always side-by-side.

## Behavior

| Key | Action |
|-----|--------|
| Default | Split-pane view (Countries left, Cities right - empty initially) |
| Enter or l | Move focus from Countries to Cities |
| h or Backspace | Move focus from Cities back to Countries |
| Esc | Clear filter |
| j/k | Navigate within focused pane |
| Tab | Cycle views: Servers → Settings → Logs → Servers (position preserved) |

## UI Mockup

```
┌─────────────────────────────────────────────────────────┐
│  ProtonVPN TUI                                    │
├─────────────────────────────────────────────────────────┤
│  > Countries          │  Cities for JP                   │
│  ├────────────────────┼─────────────────────────────────┤
│  │> JP #1             │    Tokyo #1                     │
│  │   JP #2            │    Osaka #2                     │
│  │   US #1            │    Yokohama #3                  │
│  │   DE #1            │                                  │
│  │                    │                                  │
└────────────────────────┴─────────────────────────────────┘
  ^ Countries: > only    ^ Cities: normal highlight
```

## Detailed Specification

### Layout
- Default ratio: 60% Countries / 40% Cities
- Left panel: Countries (shows server list, "Countries" as title)
- Right panel: Cities for selected country

### Focus Management
- **Countries (left panel)**: Shows `>` indicator for selected item
- **Cities (right panel)**: Normal selection highlight (no `>`)

### State Persistence
- When switching views (Tab), return to exact same position when coming back
- Countries selection is preserved when moving focus to Cities and back

### Key Bindings

| Key | Action |
|-----|--------|
| Enter or l | Move focus from Countries to Cities |
| h or Backspace | Move focus from Cities to Countries |
| j/k | Navigate within focused pane |
| c | Connect (to server in Countries, or city in Cities) |
| gg/G | Go to top/bottom of focused pane |
| Ctrl+d/u | Page down/up in focused pane |

### Footer

| Focus | Footer Keys |
|-------|-------------|
| Countries | `[j/k] navigate [l/Enter] cities [c] connect [d] disconnect [r] refresh [s] sort [f] field [h] back` |
| Cities | `[j/k] navigate [c/Enter] connect [h/Backspace] countries` |

Global keys (always visible): `[?] help [Tab] switch view [q] quit`

### Initial State
- On first launch: Both panels empty
- After servers loaded: First country selected, its cities shown in right panel

## Implementation Approach

### Chosen: Option B - Extend Servers View

- Modify `AppView::Servers` to always show split-pane
- Remove `split_pane_open` concept (always open in Servers view)
- Use `pane_focus: Pane` to track focus (Countries or Cities)

### Required Changes

1. **`src/state/app_view.rs`**:
   - Add `Pane` enum (Countries, Cities)

2. **`src/state/app_state.rs`**:
   - Add `pane_focus: Pane` field
   - Add `selected_city: Option<usize>` for independent city selection

3. **`src/ui/views/servers_view.rs`**:
   - Always render split-pane (no single-pane mode)
   - Countries: `>` highlight only
   - Cities: normal selection highlight

4. **`src/ui/app.rs`**:
   - Enter/l: switch focus to Cities
   - Backspace/h: switch focus to Countries
   - Tab: cycle views, preserve position
   - Update footer for new bindings

## Related Issues

- issue011: Cursor resets when returning from Cities (RESOLVED)
- issue012: Notification timing bug (separate)

## Tags

- feature
- ui
- navigation
- split-pane
- ux
- yazi-style
