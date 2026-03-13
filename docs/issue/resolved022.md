# issue022: Feature - Enhanced Logs View

## Summary

Enhanced Logs view with scroll support, selection highlight, and more detailed messages.

## Implemented Features

### 1. Logs scrollable
- j/k or arrow keys to navigate
- Ctrl+d/u for page up/down
- g/G for go to first/last

### 2. Selection highlight
- Focused pane: selected item shows with `>` indicator and background color
- Unfocused pane: no highlight (same as servers view)

### 3. Sort by newest first
- Logs display with newest entries at the top (reverse chronological order)

### 4. Detailed log messages
- `"Connecting..."` → `"Connecting to Japan..."`
- `"Disconnected"` → `"Disconnected from Japan..."`

### 5. Settings + Logs Split View

- **Left pane**: Settings list
- **Right pane**: Logs table  
- **Navigation**: h/l keys to move between panes
- **Tab key**: Switch to next top-level view (Servers ↔ SettingsAndLogs)
- **Focus indicator**: `>` prefix in title + colored border

```
┌──────────────────────┬──────────────────────┐
│> Settings            │  Logs                │
├──────────────────────┼──────────────────────┤
│ > VPN Protocol   ▼  │   Type     Message  │
│   Kill Switch    [ ] │   [OK]   Connected  │
│   DNS             ▼  │   [INFO]  Scanning  │
├──────────────────────┼──────────────────────┤
│                      │ Details:             │
│                      │ Full message here... │
└──────────────────────┴──────────────────────┘
```

**Navigation keys**:
- `Tab`: Switch between Servers and SettingsAndLogs views
- `h` / `l`: Switch between Settings and Logs panes
- `j` / `k`: Navigate up/down in focused pane
- `Enter`: Toggle/input setting value (Settings pane)
- `Space`: Toggle setting off (Settings pane)
- `g` / `G`: Go to first/last item

---

## Acceptance Criteria

- [x] Logs are scrollable with j/k keys
- [x] Selected log is highlighted (focused pane only)
- [x] Newest logs appear at top
- [x] Log messages are detailed (include server/city name)
- [x] Selected log shows full content in detail pane (focused pane only)
- [x] Logs persist across restarts (implemented in log_persistence.rs)
- [x] Settings and Logs views merged into split pane (left/right)
- [x] h/l keys switch between Settings and Logs panes
- [x] Tab switches between Servers and SettingsAndLogs views
- [x] Focus indicator shows in title (`>` prefix) and border color

---

## Commits

| Feature | Commit |
|---------|--------|
| Detailed log messages | `afcff59` |
| Logs scrolling | `c002c2c` |
| Selection indicator | `cda6875` |
| Highlight style | `cda6875` |
| Newest first | `0e670e3` |
| Settings + Logs split view | (this PR) |
