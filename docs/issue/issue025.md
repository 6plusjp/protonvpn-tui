# issue025: Feature - Globe visualization (world map integration)

## Summary

Add a globe/world map visualization that shows server locations geographically.

## Problem

Currently, server selection is list-based only. Users can't visualize server locations geographically.

## Solution

Add a new view (accessible via `g` key or new tab) that shows:

1. **World map** with server country markers
2. **Server status** indicators (online/offline/load)
3. **Interactive selection** - click/move to select country

### Implementation Approaches

| Approach | Description | Complexity |
|----------|-------------|------------|
| **A) ASCII art globe** | Simple text-based map | Low |
| **B) Unicode/Box drawing** | Better visual with box characters | Medium |
| **C) Terminal graphics** | Use ratatui's canvas or alacritty-termion | High |
| **D) External tool** | Call `glances` or `glow` | Medium |

### Recommendation: Option A/B (ASCII/Unicode)

Keep it simple and self-contained:

```
        ┌──────────────────────────────────────────────┐
        │              Server Map                      │
        │                                              │
        │    US ●━━━━━━● JP                           │
        │       ╲        ╲                            │
        │        ╲        ╲                           │
        │    DE ●──────────● SG                       │
        │           ╲                                     │
        │            ● NL                                │
        │                                              │
        │    Selected: United States (New York)         │
        │    Load: 45% | Latency: 120ms                │
        └──────────────────────────────────────────────┘
```

### Key Features

1. **Map rendering**: Show continents with server markers
2. **Color coding**: Green (low load), Yellow (medium), Red (high)
3. **Selection sync**: Selecting on map updates server list
4. **Server list sync**: Selecting in list highlights on map
5. **Quick connect**: Press `c` on map marker to connect

### Technical Implementation

```rust
// src/ui/views/globe_view.rs

pub fn render_globe_view(state: &AppState, f: &mut Frame, area: Rect) {
    // Draw world map with ASCII art
    // Overlay server markers based on coordinates
    // Handle selection
}
```

### Map Data

Need coordinates for countries. Can hardcode or fetch:

```rust
pub const COUNTRY_COORDS: &[(&str, (u16, u16))] = &[
    ("US", (30, 20)),
    ("JP", (80, 30)),
    ("DE", (45, 18)),
    ("GB", (42, 15)),
    ("SG", (70, 50)),
    // ...
];
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `h/j/k/l` | Move selection on map |
| `c` | Connect to selected server |
| `Enter` | Show city list for selected country |
| `Tab` | Switch to server list view |
| `1-9` | Quick connect to server #N |

## Dependencies

No new dependencies required if using ASCII/box drawing approach.

---

## Acceptance Criteria

- [ ] Globe view accessible via keyboard shortcut
- [ ] World map displays with server country markers
- [ ] Markers show server load via color
- [ ] Map selection syncs with server list
- [ ] Server list selection highlights on map
- [ ] Quick connect works from map
- [ ] Responsive to terminal size
