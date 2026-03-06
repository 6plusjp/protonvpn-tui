# issue004: Cache Inconsistency - Async Operations Don't Sync Back to Main State

## Summary

Cache state becomes inconsistent after async operations because:
1. `refresh_servers` clears `connected_server`/`ip` in a cloned cache that never syncs back
2. `connect` causes countries to disappear due to stale in-memory cache

---

## Symptom

- After running `refresh_servers`, connected server info (`connected_server`, `ip`) disappears
- After `connect`, countries list becomes empty or stale
- The disk cache is correct, but in-memory state becomes inconsistent

---

## Root Cause

### Async Cache Isolation

The async task system works on cloned `VpnState` objects that never synchronize back to the main state.

#### Flow Analysis:

**1. Refresh Servers Flow:**
```
app_state.vpn_state (main)  ─┐
                              │ clone
                              ▼
async thread's vpn_state ──► modify cache ──► save to disk
                              │
                              ▼ (only Vec<Server> returned, NOT cache)
app_state.servers (UI) ◄──────┘
```

- `spawn_refresh_servers(self.vpn_state.clone(), tx)` passes a **clone**
- Async thread modifies its **own copy** of cache and saves to disk
- Only the result (`Vec<Server>`) is sent back, not the updated cache
- **Main `app_state.vpn_state.cache` is never updated**

**2. Connect Flow:**
```
connect() called
  │
  ▼
spawn_connect(self.vpn_state.clone(), ...)  // Clone passed to async
  │
  ▼
async thread: client.connect() ──► set_connected() on CLONED cache ──► save to disk
  │
  ▼
Main state: connection = Connected{server, ip}  // UI updated, but cache not!
  │
  ▼
Later: get_countries() ──► self.cache.countries (STALE!) ──► wrong data
```

### Why "refresh_servers clears connected_server/ip":

Looking at the code:
- `refresh_countries()` at `client.rs:274-292` does:
  ```rust
  self.cache.countries = countries.clone();
  self.cache.last_updated = Some(Utc::now());
  self.save_cache()?;
  ```
- This runs on the **cloned** cache in the async thread
- The **original** `app_state.vpn_state.cache` retains whatever it had
- On disk: countries are refreshed (correct)
- In memory: stale, but may appear to work if countries weren't empty before

### Why "connect causes countries to disappear":

In `client.rs:257-261`:
```rust
pub fn get_countries(&mut self) -> AppResult<HashMap<String, String>> {
    if self.cache.countries.is_empty() {
        return self.refresh_countries();  // Only refreshes if empty!
    }
    Ok(self.cache.countries.clone())
}
```

After refresh:
- Disk cache has new countries (correct)
- `app_state.vpn_state.cache.countries` is stale (not updated)
- If stale cache is **not empty** → returns stale data
- If stale cache **was cleared** → triggers refresh (works by accident)

---

## Code References

### Key Files:

1. **`src/vpn/cache.rs`**
   - `ServerCache` struct definition (lines 14-26)
   - Contains: `countries`, `cities`, `connected_server`, `connected_ip`, `connected_at`

2. **`src/vpn/client.rs`**
   - `VpnClient` struct (line 23-31) - holds cache
   - `refresh_countries()` (lines 274-292) - updates cache
   - `get_countries()` (lines 257-261) - returns stale cache

3. **`src/vpn/state.rs`**
   - `VpnState` struct (lines 6-9)
   - `VpnClient` wrapped here

4. **`src/state/async_tasks.rs:30-40`**
   - `spawn_refresh_servers` clones vpn_state, operates on separate cache

5. **`src/state/app_state.rs`**
   - `refresh_servers()` (lines 392-403) - spawns async with clone
   - Async result handling (lines 233-250) - updates only `self.servers`

---

## Fix Required: Use Arc<VpnState>

### Overview

Wrap `VpnState` with `Arc` so all threads share the same instance. This is the correct architectural solution for shared state - the entire VPN state (including cache) is shared, not just the cache.

### Why This Approach?

**Advantages:**
- Entire VPN state is shared, not just cache - more architecturally correct
- Future additions to VpnState/VpnClient are automatically thread-safe
- Clear ownership model - one state, many references
- No need for manual Clone impl on VpnClient

**Disadvantages:**
- More changes required (VpnClient, VpnState, app_state all affected)
- All methods must use `&self` instead of `&mut self`
- Every call site needs `.clone()` (cheap with Arc, but still)

### Implementation Steps

**Step 1: Update `src/vpn/client.rs`**

Add Mutex import and wrap cache access:

```rust
use std::sync::Mutex;

pub struct VpnClient {
    cli_path: String,
    cache: Mutex<ServerCache>,  // Thread-safe access
    cache_path: PathBuf,
}

impl VpnClient {
    pub fn new() -> Self {
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("protonvpn-tui")
            .join("server_cache.toml");

        let cache = ServerCache::load(cache_path.clone()).unwrap_or_default();

        Self {
            cli_path: "protonvpn".to_string(),
            cache: Mutex::new(cache),
            cache_path,
        }
    }

    // Helper to access cache with lock
    fn with_cache<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut ServerCache) -> T,
    {
        let mut cache = self.cache.lock().map_err(|e| {
            AppError::ConfigError(format!("Failed to lock cache: {}", e))
        })?;
        Ok(f(&mut cache))
    }
}
```

Update all cache access methods from `&mut self` to `&self`:

```rust
// OLD:
pub fn get_countries(&mut self) -> AppResult<HashMap<String, String>>

// NEW:
pub fn get_countries(&self) -> AppResult<HashMap<String, String>>

// OLD:
pub fn set_connected(&mut self, server: String, ip: Option<String>)

// NEW:
pub fn set_connected(&self, server: String, ip: Option<String>) -> AppResult<()>
```

Example update pattern:

```rust
// OLD:
pub fn get_countries(&mut self) -> AppResult<HashMap<String, String>> {
    if self.cache.countries.is_empty() {
        return self.refresh_countries();
    }
    Ok(self.cache.countries.clone())
}

// NEW:
pub fn get_countries(&self) -> AppResult<HashMap<String, String>> {
    let is_empty = self.with_cache(|c| c.countries.is_empty())?;
    if is_empty {
        return self.refresh_countries();
    }
    self.with_cache(|c| c.countries.clone())
}

// OLD:
pub fn set_connected(&mut self, server: String, ip: Option<String>) {
    self.cache.connected_server = Some(server);
    self.cache.connected_ip = ip;
    self.cache.connected_at = Some(Utc::now());
}

// NEW:
pub fn set_connected(&self, server: String, ip: Option<String>) -> AppResult<()> {
    self.with_cache(|c| {
        c.connected_server = Some(server);
        c.connected_ip = ip;
        c.connected_at = Some(Utc::now());
    })?;
    self.save_cache()
}
```

**Step 2: Update `src/vpn/state.rs`**

Change all `&mut self` methods to `&self`:

```rust
impl VpnState {
    pub fn new() -> Self {
        Self {
            client: VpnClient::new(),
        }
    }

    // Before: pub fn refresh_servers(&mut self)
    pub fn refresh_servers(&self) -> AppResult<Vec<Server>> {
        self.client.refresh_servers()
    }

    // Already &self - no change needed
    pub fn get_servers(&self) -> Vec<Server> {
        self.client.get_servers()
    }

    // All other methods: &mut self -> &self
    pub fn connect(&self, server: &str) -> AppResult<(String, Option<String>)> {
        self.client.connect(server)
    }

    pub fn connect_random(&self) -> AppResult<(String, Option<String>)> {
        self.client.connect_random()
    }

    pub fn connect_city(&self, city: &str) -> AppResult<(String, Option<String>)> {
        self.client.connect_city(city)
    }

    pub fn disconnect(&self) -> AppResult<()> {
        self.client.disconnect()
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }

    pub fn get_connected_server(&self) -> Option<String> {
        self.client.get_connected_server()
    }

    pub fn get_vpn_ip(&self) -> Option<String> {
        self.client.get_vpn_ip()
    }

    pub fn matches_ip(&self, ip: &str) -> bool {
        self.client.matches_ip(ip)
    }
}
```

**Step 3: Update `src/state/async_tasks.rs`**

Use `Arc<VpnState>` for all spawn functions:

```rust
use std::sync::Arc;

pub fn spawn_refresh_servers(
    vpn_state: Arc<VpnState>,  // Arc wrapper
    sender: mpsc::Sender<AsyncResult<Vec<Server>>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.refresh_servers();  // No mut needed
        let _ = sender.send(result);
    });
}

pub fn spawn_connect(
    vpn_state: Arc<VpnState>,
    server_id: String,
    sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.connect(&server_id);
        let _ = sender.send(result);
    });
}

pub fn spawn_disconnect(
    vpn_state: Arc<VpnState>,
    sender: mpsc::Sender<AsyncResult<()>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.disconnect();
        let _ = sender.send(result);
    });
}

pub fn spawn_connect_random(
    vpn_state: Arc<VpnState>,
    sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.connect_random();
        let _ = sender.send(result);
    });
}

pub fn spawn_cities(
    vpn_state: Arc<VpnState>,
    country_code: String,
    sender: mpsc::Sender<AsyncResult<Vec<City>>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.list_cities_with_features(&country_code);
        let _ = sender.send(result);
    });
}

pub fn spawn_connect_city(
    vpn_state: Arc<VpnState>,
    city: String,
    sender: mpsc::Sender<AsyncResult<(String, Option<String>)>>,
) {
    std::thread::spawn(move || {
        let result = vpn_state.connect_city(&city);
        let _ = sender.send(result);
    });
}
```

**Step 4: Update `src/state/app_state.rs`**

Change `vpn_state` field to use `Arc`:

```rust
// In struct definition
vpn_state: Arc<VpnState>,

// In new()
pub fn new() -> Self {
    Self {
        // ...
        vpn_state: Arc::new(VpnState::new()),
        // ...
    }
}

// When spawning async tasks
self.async_manager.spawn_refresh_servers(self.vpn_state.clone(), tx);
self.async_manager.spawn_connect(self.vpn_state.clone(), server_id, tx);
self.async_manager.spawn_connect_random(self.vpn_state.clone(), tx);
self.async_manager.spawn_disconnect(self.vpn_state.clone(), tx);
self.async_manager.spawn_cities(self.vpn_state.clone(), country_code, tx);
self.async_manager.spawn_connect_city(self.vpn_state.clone(), city, tx);
```

**Step 5: Verify all changes compile**

Run `cargo check` and fix any compilation errors. Common issues:
- Missing `Arc` import
- Methods still using `&mut self` that weren't updated
- Lock guard lifetime issues

---

## Acceptance Criteria

- [x] `refresh_servers` does not clear `connected_server`/`ip`
- [x] `connect` preserves countries data in cache
- [x] In-memory cache stays consistent with disk cache
- [x] Async operations don't cause data loss
- [x] All VpnClient methods use `&self` with internal Mutex
- [x] VpnState uses `&self` for all methods
- [x] app_state uses `Arc<VpnState>` for shared state
- [x] `cargo check` passes without errors
- [x] `cargo test` passes

---

## Related Issues

- issue002: Servers view with city details
- issue003: Cities view bugs
