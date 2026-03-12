# issue038 - City Loading and Filtering Issues

## Summary

Three related issues regarding city loading behavior, duplication, and error handling:

1. **City loading doesn't trigger properly when filtering** - should load on country selection
2. **City duplication** - no deduplication in the pipeline
3. **Silent failure** - no user notification when cities fail to load from cache

---

## Issue 1: Filter does not trigger city loading

### Description

When using the filter (search) to find countries, cities for the highlighted country are not loaded. Currently, cities are loaded when:
- Navigating between countries (j/k keys) via `switch_cities_to_selected()`
- Moving to cities pane (l or Enter key) via `move_to_cities()`

The problem is that simply applying a filter does not trigger city loading, even though a country is highlighted and should have its cities loaded.

### Current Behavior

- Filter is applied, a country is highlighted
- But cities are not loaded until user presses j/k, l, or Enter
- User expects cities to load automatically when a country is highlighted

### Expected Behavior

When filter is applied and a country is highlighted (selected), cities should load automatically.

### Root Cause

In `set_search_query()`, when filter changes, cities are not loaded. Additionally, there's a bug where `selected_server` index might be out of bounds after filtering.

### Fix Applied

Modified `set_search_query()` in `src/state/app_state.rs`:
1. Reset `selected_server` to valid index if out of bounds
2. Call `switch_cities_to_selected()` to load cities for the highlighted country

```rust
pub fn set_search_query(&mut self, query: String) {
    self.ui_state.search_query.set(query);
    self.invalidate_filtered_cache();

    let filtered_len = self.filtered_servers().len();
    if let Some(idx) = self.ui_state.selected_server {
        if idx >= filtered_len {
            self.ui_state.selected_server = Some(0);
        }
    }

    self.switch_cities_to_selected();
}
```

### Status: FIXED

---

## Issue 2: City Duplication

### Description

Cities appear duplicated in the list. This is because the `protonvpn` CLI may return duplicate city entries, and there is no deduplication logic anywhere in the pipeline.

### Current Behavior

No deduplication exists in:
1. **Parser** (`src/vpn/types.rs:110-173`) - `parse_cities_with_features()` pushes every parsed city without checking for duplicates
2. **Cache** (`src/vpn/client.rs:338`) - Inserts directly without deduplication
3. **App State** (`src/state/app_state.rs:458`) - Direct assignment without deduplication

### Root Cause Location

- `src/vpn/types.rs:169` - Parser pushes cities without dedup check
- `src/vpn/client.rs:315-344` - `list_cities_with_features()` inserts to cache without dedup

### Suggested Fix

Add deduplication in `parse_cities_with_features()` function in `src/vpn/types.rs`:

```rust
// After parsing, before returning:
cities.sort_by_key(|c| c.name.clone());
cities.dedup_by_key(|c| c.name.clone());
```

Or use HashSet during parsing to preserve order while removing duplicates.

### Status: PENDING

---

## Issue 3: Silent Failure on loaded_cities Error

### Description

When `loaded_cities` fails to load (e.g., corrupted cache file, read permission issues), no error notification is shown to the user. The app silently falls back to an empty cache.

### Current Behavior

In `src/vpn/client.rs:41-53`:
```rust
let cache = match ServerCache::load(cache_path.clone()) {
    Ok(c) => c,
    Err(e) => {
        tracing::warn!("Failed to load server cache: {}, using empty cache", e);
        ServerCache::default()  // Silent fallback - no user notification
    }
};
```

The error is only logged as a warning, not surfaced to the user.

### Expected Behavior

User should see a notification when cache fails to load, informing them that data will be fetched fresh from CLI.

### Root Cause Location

- `src/vpn/client.rs:41-53` - Cache loading failure only logs warning

### Suggested Fix

Return error from cache loading or add user notification in the app initialization flow. Since the app has a notification system (`show_notification`), this could be called during startup if cache loading fails.

### Status: PENDING

---

## Related Files

| File | Lines | Purpose |
|------|-------|---------|
| `src/state/app_state.rs` | 177-187, 848-867 | City loading triggers |
| `src/vpn/types.rs` | 110-173 | City parsing (no dedup) |
| `src/vpn/client.rs` | 41-53, 303-344 | Cache loading and city fetching |
| `src/vpn/cache.rs` | 33-40 | Cache load/save |

---

## Priority

1. **High** - Issue 1: City loading behavior is a core UX issue (FIXED)
2. **Medium** - Issue 2: Duplication affects usability (PENDING)
3. **Low** - Issue 3: Silent failure is a minor UX issue (app still works) (PENDING)
