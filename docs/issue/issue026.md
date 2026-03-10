# issue026: Feature - Pane Entity for Table Headers

## Summary

Create unified `PaneTable` entities for the two panes in Servers view, then apply the pattern to all other table-based views.

## Problem

Currently, table headers are handled inconsistently across different views. Each view may have its own way of defining and rendering column headers, making it difficult to:

- Maintain consistent header styling
- Reuse header logic across views
- Easily add/remove columns

---

## Servers View - Two Panes

The Servers view has a horizontal split with 2 panes:

### 1. Countries Pane (Left - 60%)

**Current display:**

```
ID   Country      Cities
JPN  Japan        Tokyo, Osaka, Kyoto
USA  United States New York, Los Angeles, San Francisco
...
```

**Proposed Table Entity:**

```rust
struct CountriesTable {
    title: "Countries",
    columns: [
        Column { name: "ID", width: 4, align: Left },
        Column { name: "Country", width: dynamic, align: Left },
        Column { name: "Cities", width: dynamic, align: Left },
    ],
}
```

### 2. Cities Pane (Right - 40%)

**Current display:**

```
Tokyo             TOR, Standard
Osaka             #105, Secure Core
...
```

**Proposed Table Entity:**

```rust
struct CitiesTable {
    title: "{Country Code} - Cities",
    columns: [
        Column { name: "City", width: 15, align: Left },
        Column { name: "Server", width: dynamic, align: Left },
    ],
}
```

---

## Scope

### Phase 1: Servers View (This Issue)

- [ ] Create `CountriesTable` entity
- [ ] Create `CitiesTable` entity
- [ ] Implement header rendering with column definitions

### Phase 2: Other Views (Future Issues)

- Stats view table
- Logs view table
- Settings view (if applicable)

---

## Data Structure (Refined)

```rust
use ratatui::style::Style;

/// Column alignment
#[derive(Clone, Copy)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

/// Column definition
pub struct Column {
    pub name: &'static str,
    pub width: u16,
    pub align: ColumnAlign,
}

/// Table pane with header
pub struct PaneTable {
    pub title: String,
    pub columns: Vec<Column>,
    pub header_style: Style,
    pub row_style: Style,
}

impl PaneTable {
    /// Render header row
    pub fn header(&self) -> String { ... }
}
```

---

## Acceptance Criteria

### Servers View

- [ ] `CountriesTable` struct defined with ID, Country, Cities columns
- [ ] `CitiesTable` struct defined with City, Server columns
- [ ] Headers render correctly with proper alignment
- [ ] Column widths are configurable
- [ ] Focused pane indicator works (">" prefix)

### General

- [ ] Pattern can be reused in Stats, Logs views
- [ ] Consistent styling across all table headers
- [ ] Easy to add new columns without changing multiple files

---

## Vision - Before & After Example

### Current (Before)

```
┌─────────────────────────────────────────────┬────────────────────────────┐
│ > Countries                                 │   Cities                   │
├─────────────────────────────────────────────┼────────────────────────────┤
│ JP  Japan          Tokyo, Osaka, Kyoto      │   (no country selected)    │
│ US  United States  New York, Los Angeles    │                            │
│ DE  Germany        Frankfurt, Berlin        │                            │
│ GB  United Kingdom London, Manchester       │                            │
└─────────────────────────────────────────────┴────────────────────────────┘
```

**Problems:**

- Column headers are hardcoded in rendering logic
- Can't easily change column order or add new columns
- No consistent styling API
- Width calculations are inline and duplicated

### After (With PaneTable Entity)

```
┌─────────────────────────────────────────────┼────────────────────────────┐
│ > Countries                          [▼]    │  Cities              [▼]   │
├─────────────────────────────────────────────┼────────────────────────────┤
│ ID   Country      Cities                    │  City       Features       │
├─────────────────────────────────────────────┼────────────────────────────┤
│ JP   Japan        Tokyo, Osaka, Kyoto       │  Tokyo      TOR, Standard  │
│ US   United States New York, Los Angeles    │  Osaka      P2P, Secure    │
│ DE   Germany      Frankfurt, Berlin         │                            │
└─────────────────────────────────────────────┴────────────────────────────┘
```

**Benefits:**

- Headers defined as data (`PaneTable` struct)
- Easy to add/remove columns
- Consistent styling across all panes
- Reusable in Stats, Logs, Settings views
- Column widths are configurable and consistent

---

## Future Expansion - Other Views

Once `PaneTable` is established, applying to other views becomes trivial:

### Stats View Example

```rust
struct StatsTable {
    title: "Connection Statistics",
    columns: [
        Column { name: "Metric", width: 15, align: Left },
        Column { name: "Value", width: 20, align: Left },
        Column { name: "Status", width: 10, align: Center },
    ],
}
```

### Logs View Example

```rust
struct LogsTable {
    title: "Logs",
    columns: [
        Column { name: "Time", width: 8, align: Left },
        Column { name: "Event", width: 20, align: Left },
        Column { name: "Details", width: dynamic, align: Left },
    ],
}
```

---

## Notes

- Related to: issue022 (Logs view)
- The `Pane` enum already exists in `src/state/app_view.rs`
- Consider integration with existing styling system (`Theme`)
