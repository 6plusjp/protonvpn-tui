# issue026: Countries pane spinner visibility issue

## Status: ✅ FIXED (2026-03-08)

## Summary

The loading spinner (`◐`) in the countries pane only displays when a country is explicitly selected (via Enter/l key), but it should also display when navigating with cursor keys (j/k) without pressing Enter/l.

## Problem

According to issue024 specification, when the user moves the cursor to a new country using j/k keys:
1. The cities should be fetched automatically
2. A spinner should be displayed in the countries pane to indicate loading

However, the spinner only appears when the country is "selected" (i.e., after pressing Enter/l), not when simply moving the cursor.

## Expected Behavior

```
┌─────────────────────────────────────────────────┐
│ Countries         │ Cities                     │
├────────────────────┼───────────────────────────┤
│   Japan           │                            │  ← Cursor on Japan, loading
│ ◐ United States   │                            │  ← Spinner should show!
│   Germany         │                            │
└────────────────────┴───────────────────────────┘
```

The spinner should appear for ANY country that is currently loading cities, regardless of whether it is selected.

---

## Deep Code Analysis

### Render Logic (servers_view.rs:77-94)

```rust
let is_loading_this = state
    .pending_cities_country
    .as_deref()
    .map(|c| c == &server.id)
    .unwrap_or(false);

let cities_str = if is_loading_this {
    "◐".to_string()
} else if server.cities.is_empty() {
    "-".to_string()
} else {
    server.cities.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", ")
};
```

**Key observation**: The spinner logic (`is_loading_this`) is INDEPENDENT of `is_selected`. It only checks if `pending_cities_country == server.id`.

### j/k Navigation Flow (app_state.rs:804-839)

1. User presses j/k → `handle_navigation_down/up()` → `select_next/prev()`
2. `select_next()` → updates `selected_server` → calls `switch_cities_to_selected()`
3. `switch_cities_to_selected()`:
   ```rust
   if self.current_country_code.as_deref() != Some(country_code) {
       self.current_cities.clear();
       self.current_country_code = Some(country_code.clone());
       if let Some(cities) = self.vpn_state.get_cached_cities(country_code) {
           self.current_cities = cities;
       } else {
           self.load_cities_async(country_code);  // Sets pending_cities_country
       }
   }
   ```

4. `load_cities_async()` (app_state.rs:841-855):
   ```rust
   fn load_cities_async(&mut self, country_code: &str) {
       let country_code = country_code.to_string();
       self.pending_cities_country = Some(country_code.clone());
       // ... spawn async task
   }
   ```

### Server Cities Population (client.rs:546-563)

When servers are loaded, each `Server` gets cities from cache:
```rust
fn countries_to_servers(&self, countries: &HashMap<String, String>) -> Vec<Server> {
    let cities_map = /* get from cache */;
    servers.iter().map(|(code, name)| Server {
        id: code.clone(),
        country: name.clone(),
        cities: cities_map.get(code).cloned().unwrap_or_default(),  // May be empty!
    }).collect()
}
```

**Critical**: `server.cities` is populated from cache. If never loaded, it's empty (`Vec::default()`).

---

## CONFIRMED Root Causes (2026-03-08)

> **UPDATE (2026-03-08)**: After discussion with user, Bug #1 is NOT a bug - it's expected behavior.

### Bug #1: j/k to Cached Country - No Spinner At All
**Status**: ❌ NOT A BUG - This is expected behavior.

**Reason**: If cities are already cached, no loading is needed when cursor moves. The spinner should NOT appear.

**Confirmed**: 
- `Enter/l` key → `move_to_cities()` → `fetch_cities()` → always reloads and shows spinner ✅
- `j/k` cursor → cached: no spinner needed (correct)

### Bug #2: Spinner disappears when cursor moves away from loading country

**Root Cause**: `pending_cities_country` only stores ONE country at a time.

When user navigates from Country A (loading) to Country B:
1. `switch_cities_to_selected()` is called for Country B
2. `pending_cities_country` is updated to Country B
3. Country A's spinner condition (`pending_cities_country == "A"`) becomes false
4. Spinner disappears immediately, even though Country A is still loading

**Timeline**:
1. j → Country A (uncached) → `pending_cities_country = "A"` → spinner shows ✅
2. j → Country B → `pending_cities_country = "B"` → Country A spinner disappears ❌
3. Country B's async may start, spinner for B may show

**Debug Evidence** (from user log):
```
Loading cities for AR...  → OK: Loaded 1 cities ✅
Loading cities for AU...  → OK: Loaded 5 cities ✅
Loading cities for BT...  (started)
Loading cities for BY...  (started)
...
Loaded 1 cities           ← Only 1 success notification!
```

Multiple async tasks are spawned, but only the latest one's result is processed. Previous loading countries lose their spinner when cursor moves.

---

## Possible Root Causes (DEPRECATED - See Above)

> The following hypotheses have been confirmed or refuted. See "CONFIRMED Root Causes" above.

### Hypothesis 1: Code is Correct - User Perception Issue
**Status**: ❌ REFUTED - There are actual bugs.

### Hypothesis 2: Cache State Affects Behavior
**Status**: ✅ CONFIRMED - This is Bug #1.

### Hypothesis 3: Initial Load Timing
**Status**: ✅ CONFIRMED - Related to Bug #1.

### Hypothesis 4: Missing Call to switch_cities_to_selected
**Status**: ❌ REFUTED - Function is called correctly.

---

## Summary Table

| Scenario | Cache Status | Spinner While Cursor On | Spinner When Cursor Moves Away | Issue? |
|----------|--------------|------------------------|-------------------------------|--------|
| j/k to cached country | Cached | No (expected) | N/A | ❌ |
| j/k to uncached country | Not Cached | Yes ✅ | **No - disappears** ❌ | ✅ #2 |
| Enter/l select | Any | Yes | Yes (stays until loaded) | ❌ |

**Actual Issue**: When cursor moves away from a loading country, the spinner disappears because `pending_cities_country` only holds one country.

---

## Fix Suggestions

### Fix for Bug #2 (Spinner disappears when cursor moves away)

The spinner shows correctly while cursor is on a loading country. The issue is that when cursor moves to another country, the previous country's spinner disappears.

**Root Cause**: `pending_cities_country` is a single `Option<String>`, only tracking one country.

**Suggested Fixes**:
1. **Track multiple loading countries**: Change `pending_cities_country` to `Option<HashSet<String>>` to track all countries currently loading
2. **Or track pending countries separately**: Add a new field to track which countries have pending loads, independent of current selection
3. **Or simplify (if acceptable)**: Accept that spinner only shows for current cursor position - this is actually consistent with showing spinner only for selected country

---

## Files Involved

| File | Function | Purpose |
|------|----------|---------|
| `src/state/app_state.rs` | `select_next()` (804) | Cursor down - calls switch_cities_to_selected |
| `src/state/app_state.rs` | `select_prev()` (813) | Cursor up - calls switch_cities_to_selected |
| `src/state/app_state.rs` | `switch_cities_to_selected()` (822) | Checks cache, calls load_cities_async |
| `src/state/app_state.rs` | `load_cities_async()` (841) | Sets pending_cities_country |
| `src/ui/views/servers_view.rs` | `render_countries_pane()` (32) | Renders spinner based on pending_cities_country |
| `src/vpn/client.rs` | `countries_to_servers()` (546) | Populates server.cities from cache |

---

## Debug Evidence (2026-03-08)

User provided debug log showing race condition:

```
│1m ago    [INFO] Refreshing servers...                                                                                                                                                                                                                                    │
│1m ago    [OK]   Refreshed 129 servers                                                                                                                                                                                                                                    │
│1m ago    [INFO] Loading cities for AR...                                                                                                                                                                                                                                 │
│1m ago    [OK]   Loaded 1 cities                                                                                                                                                                                                                                          │
│just now  [INFO] Loading cities for AU...                                                                                                                                                                                                                                 │
│just now  [OK]   Loaded 5 cities                                                                                                                                                                                                                                          │
│just now  [INFO] Loading cities for BT...                                                                                                                                                                                                                                 │
│just now  [INFO] Loading cities for BY...                                                                                                                                                                                                                                 │
│just now  [INFO] Loading cities for CA...                                                                                                                                                                                                                                 │
│just now  [INFO] Loading cities for CH...                                                                                                                                                                                                                                 │
│just now  [INFO] Loading cities for CI...                                                                                                                                                                                                                                 │
│just now  [OK]   Loaded 1 cities                                                                                                                                                                                                                                            │
```

**Observation**: Only ONE "OK" (success) notification for multiple "Loading" events. This confirms:
1. Multiple async tasks are spawned (expected)
2. Most complete but are discarded (race condition)
3. Only the last one's result is processed

---

## Acceptance Criteria

- [x] Spinner appears in countries pane when cursor is on a loading country (j/k for uncached countries)
- [x] Spinner appears for Enter/l selection (always)
- [x] Spinner clears when cities are loaded
- [x] **Spinner remains visible even after cursor moves away** - should track all loading countries, not just current

---

## Fix Implemented (2026-03-08)

### Changes Made

1. **`pending_cities`**: Changed from `Option<CitiesReceiver>` to `HashMap<String, CitiesReceiver>` to handle multiple async tasks simultaneously

2. **`pending_cities_country`** (removed): No longer needed - `pending_cities` keys now track loading countries

3. **`sync_connection_state()`**: Updated to process all pending results at once, removing completed countries from the HashMap

4. **`load_cities_async()` and `fetch_cities()`**: Updated to insert receivers into HashMap

5. **Render logic**: Updated to check `pending_cities.contains_key(&server.id)` instead of `pending_cities_country`

### How It Works Now

- When user j/k navigates to uncached countries, each country's async task is tracked in the HashMap
- Spinner remains visible for all loading countries (keys in `pending_cities`)
- When async completes, the country is removed from HashMap and spinner disappears
- Multiple async tasks can complete and all are processed correctly
