# issue018: Bug - Data loading race conditions (load_cities + logs persistence)

## Summary

Two related data handling bugs that cause incorrect state updates.

---

## Bug 1: load_cities race condition

### Problem

When rapidly selecting different countries, `load_cities` doesn't properly overwrite results due to a race condition.

### Root Cause

In `src/state/app_state.rs`:

1. `fetch_cities()` (line 588-608) sets `current_country_code` BEFORE the async fetch completes
2. When async result arrives in `sync_connection_state()` (line 426-451), the condition at line 431 checks `if self.current_country_code.is_some()` — but this is ALWAYS true because `fetch_cities` already set it
3. The condition should check if the incoming cities match the CURRENT selected country, not just if ANY country is selected

### Race Condition Scenario

```
Timeline:
1. User selects Country A → fetch_cities("A") called
2. User quickly selects Country B → fetch_cities("B") called
   - Cache load for B sets current_country_code = Some("B")
3. Async result for A arrives (from background thread)
4. Condition current_country_code.is_some() is TRUE (it's "B")
5. Cities for A incorrectly overwrite Cities for B
```

### Current Code (Problematic)

```rust
// src/state/app_state.rs - fetch_cities() line 588-608
pub fn fetch_cities(&mut self, country_code: &str) {
    let country_code = country_code.to_string();

    // ⚠️ PROBLEM: Sets current_country_code BEFORE async task
    if let Ok(cities) = self.vpn_state.list_cities_with_features(&country_code) {
        self.current_cities = cities.clone();
        self.current_country_code = Some(country_code.clone());  // ← Set too early!
    }

    // ... spawn async task
}

// src/state/app_state.rs - sync_connection_state() line 426-451
if let Ok(result) = rx.try_recv() {
    match result {
        Ok(cities) => {
            // ⚠️ PROBLEM: Always true because fetch_cities already set it
            if self.current_country_code.is_some() {  // ← Always TRUE!
                self.current_cities = cities.clone();
            }
        }
    }
}
```

### Fix Implementation

**Step 1: Add pending tracking field to AppState**

In `src/state/app_state.rs`, add a new field to track which country the pending request is for:

```rust
// Around line 152, add this field:
pub(crate) pending_cities_country: Option<String>,
```

Initialize it in `AppState::new()`:

```rust
// Around line 194, add:
pending_cities_country: None,
```

**Step 2: Modify fetch_cities() to track pending country**

Replace the current `fetch_cities()` implementation (line 588-608):

```rust
pub fn fetch_cities(&mut self, country_code: &str) {
    let country_code = country_code.to_string();

    // Load cached cities first (non-blocking)
    if let Ok(cities) = self.vpn_state.list_cities_with_features(&country_code) {
        self.current_cities = cities.clone();
        self.current_country_code = Some(country_code.clone());
    }

    // ⚠️ NEW: Track which country we're waiting for
    self.pending_cities_country = Some(country_code.clone());

    self.invalidate_filtered_cache();

    self.show_notification(
        format!("Loading cities for {}...", country_code),
        NotificationType::Info,
    );

    let (tx, rx) = create_channel();
    self.pending_cities = Some(rx);
    self.async_manager
        .spawn_cities(self.vpn_state.clone(), country_code, tx);
}
```

**Step 3: Modify sync_connection_state() handler**

Replace the cities handling block (line 426-451):

```rust
// Check for pending cities fetch result
if let Some(rx) = self.pending_cities.as_mut() {
    if let Ok(result) = rx.try_recv() {
        // ⚠️ IMPORTANT: Get the country_code from the pending tracking,
        // NOT from the async result (async_tasks.rs only returns Vec<City>)
        let pending_country = self.pending_cities_country.clone();

        match result {
            Ok(cities) => {
                // ⚠️ FIXED: Check if pending country matches current selection
                // This prevents stale async results from overwriting newer selections
                if let Some(pending) = pending_country {
                    // Only update if the pending request matches current_country_code
                    // (i.e., user hasn't selected a different country while waiting)
                    if self.current_country_code.as_deref() == Some(&pending) {
                        self.current_cities = cities.clone();
                    }
                    // If current_country_code differs from pending, it means:
                    // - User selected a new country after fetch started
                    // - The stale result should be discarded
                }
                self.show_notification(
                    format!("Loaded {} cities", cities.len()),
                    NotificationType::Success,
                );
                notification_shown = true;
            }
            Err(e) => {
                self.show_notification(
                    format!("Failed to load cities: {}", e),
                    NotificationType::Error,
                );
                notification_shown = true;
            }
        }
        self.pending_cities = None;
        self.pending_cities_country = None;  // ⚠️ NEW: Clear pending
        self.invalidate_filtered_cache();
    }
}
```

**Note**: The async result only contains `Vec<City>`, not the country_code. We compare against `self.current_country_code` instead, which is updated when user selects a new country.

### Alternative Simpler Fix (Using sequence number)

If you prefer not to track pending country, use a sequence number approach:

```rust
// Add to AppState (around line 152):
pending_cities_seq: u64,

// Initialize in AppState::new():
pending_cities_seq: 0,

// In fetch_cities, increment before spawning:
self.pending_cities_seq += 1;
let seq = self.pending_cities_seq;
// Store seq in pending_cities if needed, or just use the field

// In sync_connection_state handler:
// When checking result, verify seq matches current pending_cities_seq
// If different, it means a newer request was made - discard stale result
```

---

## Bug 2: Logs not persisted (resets on restart)

### Problem

The Logs view shows `notification_log` which is stored in-memory only. When the application restarts, all logs are lost.

### Root Cause

In `src/ui/views/logs_view.rs`, the view reads directly from `state.notification_log` which is a `Vec<Notification>` in memory. There is no persistence layer.

### Current Code

```rust
// src/ui/views/logs_view.rs line 20-21
let items: Vec<ListItem> = state
    .notification_log  // ← Only in-memory, no file persistence
    .iter()
    .map(|n| { ... })
    .collect();
```

```rust
// src/state/app_state.rs line 170
pub notification_log: Vec<Notification>,  // ← Defined here, not persisted
```

### Fix Implementation

**Step 1: Create log persistence module**

Create `src/state/log_persistence.rs`:

```rust
use crate::state::{Notification, NotificationType};
use std::fs;
use std::path::PathBuf;

const LOG_FILE_NAME: &str = "logs.json";

/// Get log file path using XDG Base Directory standard
/// Log files belong in XDG_STATE_HOME (or fallback to data_local_dir)
/// 
/// - Linux: ~/.local/state/protonvpn-tui/logs.json (if XDG_STATE_HOME set)
///           ~/.local/share/protonvpn-tui/logs.json (fallback)
/// - macOS: ~/.local/share/protonvpn-tui/logs.json
/// - Windows: %LOCALAPPDATA%/protonvpn-tui/logs.json
fn get_log_file_path() -> PathBuf {
    // dirs crate doesn't have state_dir() in v5.0, use data_local_dir() as XDG fallback
    // This maps to: XDG_DATA_HOME ($HOME/.local/share) by default
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("protonvpn-tui").join(LOG_FILE_NAME)
}

pub fn load_notification_log() -> Vec<Notification> {
    let path = get_log_file_path();
    if !path.exists() {
        return Vec::new();
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_default()
        }
        Err(e) => {
            tracing::warn!("Failed to read log file: {}", e);
            Vec::new()
        }
    }
}

pub fn save_notification_log(log: &[Notification]) {
    let path = get_log_file_path();
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    match serde_json::to_string_pretty(log) {
        Ok(content) => {
            if let Err(e) = fs::write(&path, content) {
                tracing::warn!("Failed to write log file: {}", e);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to serialize log: {}", e);
        }
    }
}
```

Add the module to `src/state/mod.rs`:

```rust
mod log_persistence;
pub use log_persistence::*;
```

**Step 2: Modify AppState to use persisted logs**

In `src/state/app_state.rs`:

Add import:

```rust
use crate::state::log_persistence;
```

Modify `AppState::new()` to load persisted logs:

```rust
impl AppState {
    pub fn new() -> Self {
        // ... existing fields ...
        
        // ⚠️ NEW: Load persisted notification log
        notification_log: log_persistence::load_notification_log(),
        
        // ... rest ...
    }
}
```

Modify `show_notification()` to also save to file:

```rust
pub fn show_notification(&mut self, message: String, notification_type: NotificationType) {
    // ... existing code ...
    
    self.notification_log.push(Notification { ... });
    if self.notification_log.len() > MAX_NOTIFICATION_LOG {
        self.notification_log.remove(0);
    }
    
    // ⚠️ NEW: Persist to file
    log_persistence::save_notification_log(&self.notification_log);
}
```

**Step 3: Add serde derives to Notification**

In `src/state/app_state.rs`, add Serialize/Deserialize to `Notification` and `NotificationType`:

```rust
// Around line 104-109 - Add serde derives
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotificationType {
    Info,
    Success,
    Error,
}

// Around line 112-116 - Add serde derives
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
}
```

**Step 4: (Optional) Add dependencies**

These dependencies already exist in `Cargo.toml`:
- `serde = { version = "=1.0", features = ["derive"] }`
- `serde_json = "=1.0"`
- `dirs = "=5.0"`

No additional dependencies needed.

---

## Testing

### Test Bug 1 Fix (Race Condition)

1. Start the app
2. Navigate to a country (e.g., Japan)
3. Quickly press `j` to move to next country (e.g., Germany)
4. Wait for both cities to load
5. Verify the cities shown match the LAST selected country, not the first

### Test Bug 2 Fix (Log Persistence)

1. Start the app
2. Perform some actions (connect, disconnect, etc.)
3. Check Logs view shows entries
4. Quit the app
5. Start the app again
6. Check Logs view still shows previous entries

---

## Acceptance Criteria

- [x] load_cities properly tracks pending country and only updates when matching
- [x] Logs persist across app restarts
- [x] No race conditions when rapidly switching countries
- [x] Application builds without errors (`cargo build`)
- [x] No clippy warnings (`cargo clippy`)
