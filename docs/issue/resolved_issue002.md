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
- Press Enter on a country to navigate to city detail view
- Press 'c' or Enter on a city to connect via `protonvpn connect --city <selected city>`
- Press Escape to return to Servers view from Cities view

### Keyboard Controls

#### Servers View
| Key       | Action                        |
| --------- | ----------------------------- |
| j / Down  | Move down                     |
| k / Up    | Move up                       |
| g         | Go to top (first press)       |
| G         | Go to bottom (last item)      |
| Enter     | Navigate to Cities view       |
| c         | Connect to selected country   |
| r         | Refresh servers               |
| d         | Disconnect                    |
| x         | Connect to random server      |
| /         | Filter servers                |
| s         | Cycle sort                    |
| f         | Cycle sort field              |
| ?         | Help view                     |
| Tab       | Switch view                   |
| q         | Quit                          |

#### Cities View
| Key           | Action                        |
| ------------- | ----------------------------- |
| j / Down      | Move down                     |
| k / Up        | Move up                       |
| g             | Go to top (first press)       |
| G             | Go to bottom (last item)      |
| c / Enter     | Connect to selected city      |
| Escape        | Return to Servers view        |

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

| Component                      | Status     | Changes                                                            |
| ------------------------------ | ---------- | ------------------------------------------------------------------ |
| `src/state/app_view.rs`        | ✅ Done    | `Servers` and `Cities` views exist                                 |
| `src/state/async_tasks.rs`     | ✅ Done    | Added `spawn_cities` and `spawn_connect_city` tasks               |
| `src/state/app_state.rs`       | ✅ Done    | Added `pending_cities`, `pending_connect_city`, `current_cities`, notification handling |
| `src/vpn/types.rs`             | ✅ Done    | Added `City` struct with `features` field                          |
| `src/vpn/cache.rs`             | ✅ Done    | Updated cache to store `Vec<City>`                                |
| `src/vpn/client.rs`            | ✅ Done    | Added `connect_city()`, `list_cities_with_features()` methods     |
| `src/vpn/state.rs`             | ✅ Done    | Added `connect_city()`, `list_cities_with_features()` methods     |
| `src/ui/views/servers_view.rs` | ✅ Done    | Implemented                                                        |
| `src/ui/views/cities_view.rs`  | ✅ Done    | Updated to use real data                                          |
| `src/ui/app.rs`                | ✅ Done    | Added keyboard handling for navigation, connection                 |

### VPN Client Commands

```rust
// src/vpn/client.rs

// List cities - returns city names only (backward compatibility)
pub fn list_cities(&mut self, country_code: &str) -> AppResult<Vec<String>> {
    let cities = self.list_cities_with_features(country_code)?;
    Ok(cities.into_iter().map(|c| c.name).collect())
}

// List cities with features
pub fn list_cities_with_features(&mut self, country_code: &str) -> AppResult<Vec<City>> {
    if let Some(cities) = self.cache.cities.get(country_code) {
        return Ok(cities.clone());
    }
    let output = Command::new(&self.cli_path)
        .args(["cities", "--country", country_code])
        .output()
        ...
    // Parse and cache results
}

// Connect to a specific city
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

pub struct City {
    pub name: String,
    pub features: Vec<String>,  // P2P, Secure, etc.
}

impl City {
    pub fn new(name: String) -> Self {
        Self { name, features: Vec::new() }
    }
    
    pub fn with_features(name: String, features: Vec<String>) -> Self {
        Self { name, features }
    }
}

pub struct Server {
    pub id: String,          // Country code (e.g., "JP", "US")
    pub country: String,     // Full country name
    pub cities: Vec<City>,   // City information with features
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
- [x] Cached cities display on startup/refresh_servers (no blocking fetch)
- [x] Selecting a country (Enter key) navigates to city detail view
- [x] Cached city data displays immediately while fetching fresh data
- [x] `protonvpn cities --country <selected country>` runs in background without blocking UI
- [x] Notification shown when background cities fetch completes
- [x] j/k navigation works in both country list and city detail views
- [x] 'c' key on city triggers `protonvpn connect --city <selected city>`
- [x] Enter key on city triggers `protonvpn connect --city <selected city>`
- [x] Connection result shows notification (success/error)
- [x] Back navigation (Escape) returns to previous view

---

## Implementation Details

### New Methods Added

#### src/vpn/client.rs
- `connect_city(&mut self, city: &str) -> AppResult<(String, Option<String>)>` - Connect to a specific city
- `list_cities_with_features(&mut self, country_code: &str) -> AppResult<Vec<City>>` - List cities with features
- `parse_cities_with_features(&self, output: &str) -> Vec<City>` - Parse CLI output

#### src/vpn/state.rs
- `connect_city(&mut self, city: &str) -> AppResult<(String, Option<String>)>`
- `list_cities_with_features(&mut self, country_code: &str) -> AppResult<Vec<City>>`

#### src/state/app_state.rs
- `fetch_cities(&mut self, country_code: &str)` - Load cached cities and spawn async fetch
- `set_cities(&mut self, cities: Vec<City>, country_code: String)` - Update current cities
- `connect_city(&mut self, city: &str)` - Initiate city connection

### AppState Fields Added
- `current_cities: Vec<City>` - Currently displayed cities in Cities view
- `current_country_code: Option<String>` - Country code for current Cities view
- `pending_cities: Option<CitiesReceiver>` - Async receiver for cities fetch
- `pending_connect_city: Option<ConnectReceiver>` - Async receiver for city connection

## Tests

Regression tests are located in `tests/server_test.rs`.

```bash
cargo test --test issue002_test
```

Current test coverage:
- AppView enum (Servers, Cities navigation)
- Server struct with cities
- VpnClient and VpnState methods existence
