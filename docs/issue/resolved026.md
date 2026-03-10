# issue026: Feature - Pane Entity for Table Headers ✅ IMPLEMENTED

## Summary

Create unified `PaneTable` entities for the two panes in Servers view, then apply the pattern to all other table-based views.

## Status: DONE ✅

Implemented:
- `ColumnAlign`, `Column`, `PaneTable` structs in `src/ui/components/pane_table.rs`
- `CountriesTable::table()`, `CitiesTable::table()` helpers
- `format_row_with_widths()` for dynamic column alignment
- Used in `servers_view.rs`
- **Table widget migration**: Changed from List to Table for fixed header support
- `header_row()` and `column_widths()` methods for Table widget integration

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

- [x] `CountriesTable` struct defined with ID, Country, Cities columns
- [x] `CitiesTable` struct defined with City, Features columns  
- [x] Headers render correctly with proper alignment (via block title)
- [x] Column widths are configurable
- [x] Focused pane indicator works (">" prefix)
- [x] **Table widget (ratatui)** - ✅ IMPLEMENTED using Table with header()
- [x] **Fixed Header in List** - Not needed - Table has built-in header

### Fixed Header Implementation (ratatui)

ratatuiのWidget別の固定ヘッダー対応:

| Widget | 組み込み固定ヘッダー | 実装方法 |
|--------|---------------------|----------|
| **Table** | ✅ あり | `.header(Row::new(...))` |
| **List** | ❌ なし | `Layout::vertical` で領域分割 |

#### Table の場合（推奨）

```rust
use ratatui::widgets::{Table, Row, Cell, Block};

let table = Table::new(rows, widths)
    .header(
        Row::new(vec!["ID", "Country", "Cities"])
            .style(Style::new().bold())
            .bottom_margin(1),  // ヘッダーとデータの間の余白
    )
    .block(Block::new().title("Countries"));
```

#### List の場合（手動レイアウト分割が必要）

```rust
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, List, Paragraph},
};

// 領域を分割: ヘッダー(固定) + リスト(残り)
let [header_area, list_area] = Layout::vertical([
    Constraint::Length(1),  // ヘッダー: 1行
    Constraint::Fill(1),   // リスト: 残り全部
])
.areas(area);

// ヘッダーを先にレンダリング（スクロール해도 固定）
let header = Paragraph::new("ID   Country      Cities")
    .block(Block::bordered().title("Countries"));
frame.render_widget(header, header_area);

// リストはヘッダーの下の領域に表示
let list = List::new(items).block(block);
frame.render_stateful_widget(list, list_area, list_state);
```

**参考:**
- [ratatui Table docs](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html#method.header)
- [GitHub Discussion #1924](https://github.com/ratatui/ratatui/discussions/1924)

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
