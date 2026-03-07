# issue018: Bug - Data loading race conditions (load_cities + logs persistence)

## Summary

Two related data handling bugs that cause incorrect state updates.

## Bug 1: load_cities race condition

### Problem

When rapidly selecting different countries, `load_cities` doesn't properly overwrite results due to a race condition.

### Root Cause

In `src/state/app_state.rs`:

1. `fetch_cities()` sets `current_country_code` BEFORE the async fetch completes (line 594)
2. When async result arrives, the condition at line 431 checks `if self.current_country_code.is_some()` - but this is ALWAYS true because `fetch_cities` already set it
3. The condition should check if the incoming cities match the CURRENT selected country, not just if ANY country is selected

### Race Condition Scenario

1. User selects Country A → `fetch_cities("A")` called
2. User quickly selects Country B → `fetch_cities("B")` called
   - Cache load for B sets `current_country_code = Some("B")`
3. Async result for A arrives
4. Condition `current_country_code.is_some()` is TRUE (it's "B")
5. Cities for A incorrectly overwrite Cities for B

### Fix Required

Track which country_code the async request was for:

```rust
// In fetch_cities, store the pending country:
self.pending_country_code = Some(country_code.clone());

// In handler, check match:
if let Some(pending) = self.pending_country_code {
    if pending == country_code {  // Only update if matches
        self.current_cities = cities;
    }
}
self.pending_country_code = None;
```

---

## Bug 2: Logs not persisted (resets on restart)

### Problem

The Logs view shows `notification_log` which is stored in-memory only. When the application restarts, all logs are lost, making it appear as if logs are "resetting."

### Root Cause

In `src/ui/views/logs_view.rs`, the view reads directly from `state.notification_log` which is a `Vec<Notification>` in memory. There is no persistence layer.

### Solution Options

| Option | Description | Complexity |
|--------|-------------|------------|
| **A) File-based logging** | Write logs to a file (e.g., `~/.local/share/protonvpn-tui/logs.json`) | Medium |
| **B) Append to file on each notification** | Real-time file append | Medium |
| **C) Save on app exit** | Persist on shutdown | Low |

### Recommended: Option C (Save on exit)

- Load logs from file on startup
- Append new notifications to both memory and file
- Save to file when app exits (or periodically)

---

## Acceptance Criteria

- [ ] load_cities properly tracks pending country and only updates when matching
- [ ] Logs persist across app restarts
- [ ] No race conditions when rapidly switching countries
