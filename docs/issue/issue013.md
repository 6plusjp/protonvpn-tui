# issue013: Yazi-style Split-pane Navigation

## Summary

Implement Yazi-inspired split-pane navigation where Servers list and Cities panel are always side-by-side.

## Yazi-style Behavior

| Key | Action |
|-----|--------|
| Default | Split-pane view (Servers left, Cities panel right - empty initially) |
| Enter or l | Load cities in right panel, stay in split-pane |
| h or Backspace | Close cities panel, return focus to Servers |
| Esc | Close cities panel |
| j/k | Navigate within focused pane |
| Tab or w | Switch focus between Servers and Cities panes |

## Current Behavior

1. User selects a server in Servers view
2. User presses Enter
3. Full view switches to Cities view
4. User presses Esc to return to Servers
5. Cursor resets to top (see issue011)

## Desired Behavior (Yazi-style)

1. Split-pane view is **always** visible (Servers left, Cities right)
2. When server selected + Enter → Cities load in right panel (split-pane maintained)
3. User can navigate between Servers (left) and Cities (right)
4. Press h or Backspace or Esc → Close cities panel, return focus to Servers
5. Servers selection is preserved when returning from Cities

## UI Mockup

```
┌─────────────────────────────────────────────────────────┐
│  │  Servers          │  Cities for JP                    │
├──┴───────────────────┼───────────────────────────────────┤
│> │  JP #1            │  > Tokyo #1                       │
│  │    JP #2          │    Osaka #2                       │
│  │    US #1          │    Yokohama #3                   │
│  │    DE #1          │                                   │
│  │                   │                                   │
│  │  [j/k] navigate   │  [j/k] navigate  [h/Backspace] close │
└──┴───────────────────┴───────────────────────────────────┘
  ^ focus indicator (>)
```

## Implementation Approach

### Chosen: Option B - Extend Servers View

- Modify `AppView::Servers` to support split-pane mode
- Remove or integrate `AppView::Cities` (unified into Servers)
- Add `split_pane_mode: SplitPaneState` to track split-pane state
- Independent selection state managed via existing `selected_server`

### Key Bindings

| Key | Action |
|-----|--------|
| Enter or l | Open cities panel, switch focus to Cities |
| Backspace or h | Close cities panel, return focus to Servers |
| j/k | Navigate within focused pane |
| c | Connect to selected item (server or city depending on focus) |

### Layout

- Default ratio: 60% Servers / 40% Cities
- User configurable (future enhancement)

### Footer

Dynamic key hints based on state:

| State | Footer Keys |
|-------|-------------|
| Panel closed | `[j/k] navigate [l/Enter] cities [c] connect [d] disconnect [r] refresh` |
| Panel open + Servers focus | `[j/k] navigate [h/Backspace] close [c] connect` |
| Panel open + Cities focus | `[j/k] navigate [h/Backspace] back [c/Enter] connect` |

Global keys (always visible): `[?] help [Tab] switch view [q] quit`

### State Management

- `split_pane_open: bool` - Whether cities panel is open
- `pane_focus: Pane` - Which pane has focus (Servers or Cities)
- Use existing `selected_server` for both panes:
  - When focus is on Servers: index into filtered_servers
  - When focus is on Cities: index into current_cities

## Required Changes

1. **`src/state/app_view.rs`**:
   - Add `Pane` enum (Servers, Cities)

2. **`src/state/app_state.rs`**:
   - Add `split_pane_open: bool` field
   - Add `pane_focus: Pane` field

3. **`src/ui/views/servers_view.rs`**:
   - Modify to support split-pane rendering
   - Add optional cities panel on right side

4. **`src/ui/views/mod.rs`**:
   - (No changes needed if reusing servers_view)

5. **`src/ui/app.rs`**:
   - Update Enter/l handling: open cities panel, switch focus to Cities
   - Update Backspace/h handling: close cities panel, focus to Servers
   - Update j/k handling: navigate based on pane_focus
   - Update render_footer: dynamic key hints based on split_pane_open and pane_focus
   - Remove or update AppView::Cities handling (unified)

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
