# issue038 - Filter Performance Optimization & Sort State Display

## Summary

Two UX improvements for the servers view table:
1. **Filter performance**: Optimize filter operations that cause lag on every keystroke
2. **Sort indicator**: Display current sort direction (↑/↓) in table headers

---

## Issue 1: Filter Performance

### Current Behavior

Every keystroke in filter mode triggers full recomputation:

```
src/state/app_state.rs:compute_filtered_servers()
```

**Bottlenecks identified:**

| Issue | Location | Impact |
|-------|----------|--------|
| Repeated `.to_lowercase()` | Lines 761-766, 781-790 | O(n) string allocations per keystroke |
| Fuzzy matching on every filter | Lines 767, 808-817 | O(n) variants generated per server |
| No debouncing | `handle_filter_input()` in app.rs | Recomputes on every single keystroke |
| Cloning on every filter | Lines 755, 775, 740 | Memory allocation overhead |

### Expected Behavior

Filter should feel responsive even with 1000+ servers. No visible lag on typing.

### Suggested Fix

1. **Query length-based filter optimization** (user proposal):
   - 1-2 chars → search by `Code` only (fast, exact match)
   - 3+ chars → search by `Country` + fuzzy matching
   - Skip: City search (expensive)

   Fuzzy matchingはCountry検索でのみ使用 (japan → ja/jp/ap/pa/an/jap/apa/pan/jpn でHIT)

2. **Add debouncing** (100-200ms) to filter input - biggest win

3. **Replace fuzzy matching with pre-computed N-gram HashSet**:
   - ライブラリ不要。`HashSet<String>`だけで実装可能
   - サーバー読み込み時に各国のN-gramsを事前計算 (1回だけ)
   - 検索時はO(query_len)のHashSet lookupのみ

   ```rust
   // サーバー読み込み時 (src/vpn/types.rs の Server に追加)
   #[derive(Clone)]
   pub struct Server {
       // ... existing fields ...
       pub country_ngrams: HashSet<String>,  // 追加
   }

   fn generate_ngrams(text: &str) -> HashSet<String> {
       let lower = text.to_lowercase();
       let mut grams = HashSet::new();
       // "japan" -> "ja", "jp", "ap", "pa", "an", "jap", "apa", "pan", "jpn"...
       for len in 2..=4 {
           for window in lower.as_bytes().windows(len) {
               grams.insert(String::from_utf8_lossy(window).to_string());
           }
       }
       grams
   }

   // 検索時 (O(query_len) lookup)
   fn matches_fuzzy(query: &str, country_grams: &HashSet<String>) -> bool {
       for len in 2..=query.len().min(4) {
           for window in query.as_bytes().windows(len) {
               if country_grams.contains(&String::from_utf8_lossy(window).to_string()) {
                   return true;
               }
           }
       }
       false
   }
   ```

   | 手法 | 検索時計算量 | メモリ |
   |------|-------------|--------|
   | 現在 (variant生成) | O(n × query_len) | 0 |
   | N-gram HashSet | O(query_len) | ~100KB/1000 servers |

4. **Pre-compute lowercase versions** on server load

#### Code location for optimization

In `src/state/app_state.rs:compute_filtered_servers()`:

```rust
let query_len = query.len();

let matches = match (self.ui_state.filter, query_len) {
    // 1-2 chars: Code only (fast path)
    (_, 1..=2) => matches_code,
    // 3+ chars: Country only  
    (ServerFilter::Code, 3..) => matches_code,
    (ServerFilter::Country, 3..) => matches_country,
    (ServerFilter::City, 3..) => matches_city,
};
```

---

## Issue 2: Sort State in Table Header

### Current Behavior

Sort direction exists in state (`ServerSort`, `SortDirection` in `src/state/server_sort.rs`) but is NOT displayed in the table header.

- `SortDirection::Asc` → label is `"↑"`
- `SortDirection::Desc` → label is `"↓"`

These labels are defined but unused.

### Expected Behavior

Table headers should show current sort state:

```
Code ↑    Country    Cities     (ascending)
Code ↓    Country    Cities     (descending)
```

### Files to Modify

| File | Change |
|------|--------|
| `src/ui/components/pane_table.rs` | Modify `header_row()` to accept sort state |
| `src/ui/views/servers_view.rs` | Pass current sort state to header |
| `src/state/server_sort.rs` | Already has `label()` method returning `↑`/`↓` |

### Implementation Notes

- `SortDirection` already has `label()` method: `"↑"` / `"↓"`
- Need to pass sort column + direction to `PaneTable::header_row()`
- Example output: `"Code ↑"` or `"Country ↓"`

---

## Related Files

- `src/state/app_state.rs` - Filter logic (lines 744-806)
- `src/state/server_sort.rs` - Sort state types
- `src/state/ui_state.rs` - SearchQuery, ServerCache
- `src/ui/components/pane_table.rs` - Table header rendering
- `src/ui/views/servers_view.rs` - Servers view rendering
- `src/ui/app.rs` - Filter input handling (lines 656-704)

---

## Priority

- **Filter performance**: High (affects usability on every use)
- **Sort indicator**: Medium (nice-to-have, low effort)
