# issue002: Servers view - Country list with city details and connection

## Summary

Implement a Servers view that displays available countries and allows users to navigate to city-level details for connection.

## Problem

Users need an intuitive way to browse VPN servers by country and city, and connect to specific locations. The current implementation lacks a dedicated servers browsing interface.

## Solution

### Screen 1: Country List (Servers view)

Display a list of available countries with the following columns:

| ID  | Country       | Cities       |
| --- | ------------- | ------------ |
| JP  | Japan         | Tokyo, Osaka |
| US  | United States | New York     |

### Screen 2: City Details (on country selection)

When a country is selected, navigate to a detail view that shows:

- **Cached data display**: Show cities and features from cache while loading fresh data
- **Background command**: Execute `protonvpn cities --country <selected country>` to fetch fresh data
- **Table format**:

| City  | Features    |
| ----- | ----------- |
| Tokyo | P2P, Secure |
| Osaka | P2P         |

### Connection

- Use `j/k` keys for navigation (up/down)
- Press 'c' or Enter on a city to connect via `protonvpn connect --city <selected city>`

---

## Technical Notes

### Data Flow

1. **On startup / refresh_servers**:
   - Load cached countries list
   - Display cached cities for each country (if available)
   - **DO NOT** run `protonvpn cities --country <ID>` for all countries simultaneously (too many)

2. **On country selection (Enter key)**:
   - Immediately show cached cities (if available)
   - Spawn async task to fetch fresh data via `protonvpn cities --country <selected country>`
   - Show notification when background task completes (like other async functions)
   - Update display when new data arrives

3. **On city selection ('c' or Enter key)**:
   - Execute `protonvpn connect --city <selected city>`
   - Handle connection result (success/error notification)

### Navigation Flow

```
Startup -> Servers view -> Navigate to country -> Enter (not 'c') -> Cities view -> Navigate to city -> 'c' or Enter -> Connect
```

### AppView Enum

```rust
// src/state/app_view.rs
pub enum AppView {
    #[default]
    Servers,
    Stats,
    Settings,
    Help,
    Cities,  // new - city list for selected country
}
```

### Required Code Changes

| Component                  | Status     | Changes                                                            |
| -------------------------- | ---------- | ------------------------------------------------------------------ |
| `src/state/app_view.rs`    | ✅ Done    | `Servers` and `Cities` views exist                                 |
| `src/state/async_tasks.rs` | 🔲 Pending | Add `spawn_cities` task                                           |
| `src/state/app_state.rs`   | 🔲 Pending | Add `pending_cities` field, notification handling                |
| `src/vpn/types.rs`         | 🔲 Pending | Add `City` struct with `features` field                            |
| `src/vpn/cache.rs`         | 🔲 Pending | Update cache to store city features                               |
| `src/vpn/client.rs`        | 🔲 Pending | Add `connect_city()` method                                        |
| `src/ui/views/servers_view.rs` | ✅ Done | Implemented                                                       |
| `src/ui/views/cities_view.rs` | 🔲 Partial | Exists but uses hardcoded data                                   |

### VPN Client Commands

```rust
// src/vpn/client.rs

// List cities - already implemented
pub fn list_cities(&mut self, country_code: &str) -> AppResult<Vec<String>> {
    if let Some(cities) = self.cache.cities.get(country_code) {
        return Ok(cities.clone());
    }
    let output = Command::new(&self.cli_path)
        .args(["cities", "--country", country_code])
        .output()
        ...
}

// TODO: Add connect_city method
pub fn connect_city(&mut self, city: &str) -> AppResult<(String, Option<String>)> {
    let output = Command::new(&self.cli_path)
        .args(["connect", "--city", city])
        .output()
        ...
}
```

### Async Task Pattern (Follow Existing)

```rust
// src/state/async_tasks.rs

#[derive(Debug)]
pub enum AsyncOperation {
    RefreshComplete(Result<Vec<Server>, AppError>),
    ConnectComplete(Result<(String, Option<String>), AppError>),
    DisconnectComplete(Result<(), AppError>),
    CitiesComplete(Result<Vec<City>, AppError>),  // new
}

// New task
pub fn spawn_cities(
    &self,
    vpn_state: VpnState,
    country_code: String,
    sender: mpsc::Sender<AsyncResult<Vec<City>>>,
) {
    std::thread::spawn(move || {
        let mut state = vpn_state;
        let result = state.list_cities_with_features(&country_code);
        let _ = sender.send(result);
    });
}
```

### Cache Strategy

- Countries list: Cache in `ServerCache`, invalidate on session start
- Cities per country: Cache with country code as key, TTL of 5 minutes
- Features per city: Parse from `protonvpn cities --country <country>` output
- On refresh_servers: Show cached cities only, fetch fresh on country selection

### Notification Pattern (Follow Existing)

```rust
// src/state/app_state.rs

// In sync_connection_state():
if let Some(rx) = self.pending_cities.as_mut() {
    if let Ok(result) = rx.try_recv() {
        match result {
            Ok(cities) => {
                self.show_notification(
                    format!("Loaded {} cities", cities.len()),
                    NotificationType::Success,
                );
            }
            Err(e) => {
                self.show_notification(
                    format!("Failed to load cities: {}", e),
                    NotificationType::Error,
                );
            }
        }
        self.pending_cities = None;
    }
}
```

### Server Types Changes

```rust
// src/vpn/types.rs

pub struct Server {
    pub id: String,
    pub country: String,
    pub cities: Vec<City>,  // Changed from Vec<String>
}

pub struct City {
    pub name: String,
    pub features: Vec<String>,  // P2P, Secure, etc.
}
```

### Command Reference

```bash
# List available countries
protonvpn countries

# Get cities for a specific country
protonvpn cities --country JP

# Connect to a specific city
protonvpn connect --city Tokyo
```

---

## Acceptance Criteria

- [x] AppView: Connect renamed to Servers, Cities view added
- [x] Countries list displays ID, Country name, and city count
- [ ] Cached cities display on startup/refresh_servers (no blocking fetch)
- [ ] Selecting a country (Enter key) navigates to city detail view
- [ ] Cached city data displays immediately while fetching fresh data
- [ ] `protonvpn cities --country <selected country>` runs in background without blocking UI
- [ ] Notification shown when background cities fetch completes
- [x] j/k navigation works in both country list and city detail views
- [ ] 'c' or Enter key on city triggers `protonvpn connect --city <selected city>`
- [ ] Connection result shows notification (success/error)
- [ ] Back navigation (Escape) returns to previous view

---

## Tests

Regression tests are located in `tests/issue002_test.rs`.

```bash
cargo test --test issue002_test
```

Current test coverage:
- AppView enum (Servers, Cities navigation)
- Server struct with cities
- VpnClient and VpnState methods existence
