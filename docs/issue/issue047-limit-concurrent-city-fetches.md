# issue047-limit-concurrent-city-fetches

## Summary

Limit concurrent city fetches to prevent queue buildup when user rapidly navigates country list.

## Status

**[Proposed]** - Pending implementation

### Preliminary Decision

| Parameter | Current | Proposed |
|-----------|---------|----------|
| Thread pool workers | 4 | 10 |
| Max pending city fetches | ∞ (unlimited) | 5 |

**Rationale:**
- 10 workers: Improves overall VPN operations (connect, disconnect, refresh)
- 5 pending: Reasonable limit - users unlikely to scroll through 5+ countries faster than fetch completes

## Context

### Current Implementation

```
User navigates countries (j/k)
        │
        ▼
switch_cities_to_selected()  [app_state.rs:997]
        │
        ├── selected_server changed?
        │         │
        │         ▼
        │   current_country_code changed?
        │         │
        │         Yes ▼
        │         │
        ▼         ▼
fetch_cities(country_code)  [app_state.rs:1011]
        │
        ├── cached?
        │   Yes → return (no async)
        │
        ▼
submit Job::Cities to ThreadPool  [async_tasks.rs]
        │
        ▼
pending_cities.insert(country_code, rx)  [connection.rs:82]
```

### Key Components

| Component | Location | Details |
|-----------|----------|---------|
| **ThreadPool** | `async_tasks.rs:78-128` | 4 workers, `VecDeque<Job>` queue, `Condvar` for signaling |
| **AsyncTaskManager** | `async_tasks.rs:225-352` | `new_with_workers(4)` by default |
| **Job::Cities** | `async_tasks.rs:44-48` | `country_code: String`, returns `Vec<City>` |
| **Trigger** | `app_state.rs:997` `switch_cities_to_selected()` | Called on every selection change |
| **Fetch** | `app_state.rs:1011` `fetch_cities()` | Submits to pool, stores receiver |
| **Pending storage** | `connection.rs:82` `pending_cities: HashMap<String, CitiesReceiver>` | Tracks in-flight requests |
| **Result polling** | `app_state.rs:480-511` | Iterates all pending, processes first available |

### Current Behavior

```rust
// app_state.rs:997-1008
fn switch_cities_to_selected(&mut self) {
    if let Some(idx) = self.ui_state.selected_server {
        if let Some(server) = self.filtered_servers().get(idx) {
            let country_code = &server.code;
            if self.current_country_code.as_deref() != Some(country_code) {
                self.current_cities.clear();
                self.current_country_code = Some(country_code.to_string());
                self.fetch_cities(country_code);  // ← Called on EVERY change
            }
        }
    }
}

// app_state.rs:1011-1033
fn fetch_cities(&mut self, country_code: &str) {
    let country_code = country_code.to_string();

    // Check cache first
    if let Some(cities) = self.vpn_state.cached_cities(&country_code) {
        self.current_cities = cities;
        return;
    }

    // Submit job to thread pool - NO CHECK if already pending!
    let (tx, rx) = create_channel();
    self.connection_manager.pending_cities.insert(country_code.clone(), rx);
    self.connection_manager.async_manager.spawn_cities(
        self.vpn_state.clone(),
        country_code,
        tx,
    );
}
```

### Problem

**Actually implemented (partial deduplication):**
- Same country: `HashMap::insert` overwrites previous entry, only 1 pending per country
- Stale results: Line 512 checks `current_country_code == country_code` before applying

**Remaining issue:**
- Different countries: Rapid navigation JP→DE→FR queues 3 separate requests
- No limit on number of different countries fetched concurrently
- All requests compete for 4 worker threads

**Edge case concern:**
If limit is reached, `fetch_cities()` will skip. But:
- `l` key (Countries→Cities): Needs to always fetch or cities won't display
- `r` key (reload cities): Should always fetch
- Normal navigation (j/k): Can skip if limit reached

### Expected Behavior

- Limit concurrent city fetches (max 5)
- Navigation (j/k): Skip if limit reached (acceptable)
- Explicit request (l key, r key): Always fetch
- Same country: Already deduplicated (works)

## Analysis

### Why ThreadPool Alone Isn't Enough

- ThreadPool limits **concurrent execution** to 4 workers
- Does NOT limit **pending requests** in queue (`VecDeque<Job>`)
- Jobs queue up faster than workers can process during rapid navigation

### Potential Solutions

**Current State**: Same-country deduplication already works.

**Option A: Limit Concurrent + Force Flag (Recommended)**
- Limit concurrent city fetches to N countries (e.g., 5)
- Add `force` parameter to `fetch_cities()`:
  - `force=true`: Always fetch (l key, r key)
  - `force=false`: Skip if limit reached (j/k navigation)
- This ensures user can always access cities when explicitly requested

**Option B: Debounce Selection**
- Debounce `switch_cities_to_selected()` calls
- Only fetch after user stops navigating (e.g., 150-200ms delay)
- UX benefit: user sees final selection, not intermediate loads

## Requirements

1. Prevent unbounded queue growth of city fetch requests
2. Avoid fetching same country multiple times during rapid navigation
3. Consider: Skip if already pending (simplest) OR debounce new requests
4. Thread pool should remain general-purpose

## Implementation Hints

### 1. Thread Pool: Increase workers to 10

```rust
// src/state/connection.rs or where AsyncTaskManager is created
async_manager: AsyncTaskManager::new_with_workers(10),  // was 4
```

### 2. Limit pending city fetches to 5

```rust
// In app_state.rs or connection.rs
const MAX_PENDING_CITY_FETCHES: usize = 5;

fn fetch_cities(&mut self, country_code: &str, force: bool) {
    let country_code = country_code.to_string();

    // Check cache
    if let Some(cities) = self.vpn_state.cached_cities(&country_code) {
        self.current_cities = cities;
        return;
    }

    // Check if already pending (same-country dedup already exists)
    if self.connection_manager.pending_cities.contains_key(&country_code) {
        return;
    }

    // Limit concurrent different-country fetches (unless forced)
    if !force && self.connection_manager.pending_cities.len() >= MAX_PENDING_CITY_FETCHES {
        return;  // Too many pending, skip
    }

    // ... rest of existing code
}
```

**force = true calls:**
- `move_to_cities()` (l key): `fetch_cities(&server.code, true)`
- `reload_cities()` (r key): `fetch_cities(&country_code, true)`

**force = false calls:**
- `switch_cities_to_selected()` (j/k navigation): `fetch_cities(country_code, false)`

```rust
// Option B: Debounce - requires timer/debounce state
// In ui_state or app_state:
pub debounce_timer: Option<Instant>;

// In switch_cities_to_selected():
self.debounce_timer = Some(Instant::now());
// Then in check_pending_async_events or a timer tick, process after delay
```

## Notes

- Same-country deduplication already works via `HashMap::insert`
- Thread pool: 10 workers (was 4) - benefits all VPN operations
- Pending limit: 5 countries (was unlimited)
- force flag ensures explicit requests (l key, r key) always work

## References

- `spawn_cities()` in `async_tasks.rs:279`
- `fetch_cities()` in `app_state.rs:1011`
- `switch_cities_to_selected()` in `app_state.rs:997`
- `pending_cities` in `connection.rs:82`
- Result polling in `app_state.rs:480-511`
