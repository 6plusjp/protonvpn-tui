# issue022: Feature - Enhanced Logs View

## Summary

Enhanced Logs view with scroll support, selection highlight, and more detailed messages.

## Implemented Features (2026-03-09)

### 1. Logs scrollable
- j/k or arrow keys to navigate
- Ctrl+d/u for page up/down
- g/G for go to first/last

### 2. Selection highlight
- Selected item shows with `>` indicator and background color

### 3. Sort by newest first
- Logs display with newest entries at the top (reverse chronological order)

### 4. Detailed log messages
- `"Connecting..."` → `"Connecting to Japan..."`
- `"Disconnected"` → `"Disconnected from Japan..."`

---

## Not Implemented (Postponed)

- Settings + Logs tabs integration
- Full log content display (popup/expand)
- Copy to clipboard (removed due to terminal limitations)
- Log persistence (issue018)

---

## Acceptance Criteria

- [x] Logs are scrollable with j/k keys
- [x] Selected log is highlighted
- [x] Newest logs appear at top
- [x] Log messages are detailed (include server/city name)
- [ ] Settings and Logs accessible via tabs in unified view
- [ ] Long log messages show full content on selection
- [ ] Logs persist across restarts (issue018)

---

## Commits

| Feature | Commit |
|---------|--------|
| Detailed log messages | `afcff59` |
| Logs scrolling | `c002c2c` |
| Selection indicator | `cda6875` |
| Highlight style | `cda6875` |
| Newest first | `0e670e3` |
