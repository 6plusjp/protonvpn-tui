# issue091: Extract duplicated test utilities to common module

**Status: RESOLVED** ✅

## Summary

テストヘルパー関数 (`make_servers()`, `setup()`) が複数ファイルに重複している。重複排除して保守性を向上。

## Problems

### 1. `make_servers()` が4ファイルに重複

```rust
// src/state/server_ops.rs:272
// src/state/navigation.rs:261
// src/state/app_state.rs:240
// src/state/settings_ops.rs:244

fn make_servers() -> Vec<Server> {
    // Same 15-line function
    vec![
        Server {
            id: "JP#1".into(),
            code: "JP".into(),
            name: "Japan".into(),
            city: Some("Tokyo".into()),
            // ... (15 lines identical)
        }
    ]
}
```

### 2. `setup()` が5ファイルに重複

```rust
// src/state/server_ops.rs
// src/state/navigation.rs
// src/state/app_state.rs
// src/state/settings_ops.rs
// src/state/notifications.rs

fn setup() {
    log_persistence::set_test_mode(true);
}
```

## Solution

### Create shared test module

```rust
// src/test_helpers.rs (new file)

#[cfg(test)]
mod test_helpers {
    use super::*;
    use crate::vpn::types::{City, Server, ServerFeatures};
    
    /// Create test server data - used across all tests
    pub fn make_servers() -> Vec<Server> {
        vec![
            Server {
                id: "JP#1".into(),
                code: "JP".into(),
                name: "Japan".into(),
                city: Some("Tokyo".into()),
                features: ServerFeatures {
                    // ...
                },
                // ...
            },
            // ... more servers for testing
        ]
    }
    
    /// Setup test environment - call before each test
    pub fn setup() {
        log_persistence::set_test_mode(true);
    }
    
    /// Create test cities for a country
    pub fn make_cities() -> Vec<City> {
        // ...
    }
}
```

### Update existing test files

```rust
// Before
mod tests {
    fn make_servers() -> Vec<Server> { /* ... */ }
    fn setup() { /* ... */ }
    
    #[test]
    fn test_something() {
        setup();
        // ...
    }
}

// After
mod tests {
    use super::test_helpers::*;
    
    #[test]
    fn test_something() {
        setup();
        // ...
    }
}
```

## Resolution

**Implemented**: Created shared test utilities module.

### Changes:

1. **Created** `src/test_helpers.rs`:
   - `pub fn make_servers() -> Vec<Server>` - shared test server data
   - `pub fn make_servers_with_features() -> Vec<Server>` - servers with features
   - `pub fn setup()` - test environment setup

2. **Updated** files to use shared helpers:
   - `src/state/server_ops.rs` - use test_helpers
   - `src/state/navigation.rs` - use test_helpers  
   - `src/state/app_state.rs` - use test_helpers
   - `src/state/settings_ops.rs` - use test_helpers
   - `src/state/notifications.rs` - use test_helpers

3. **Added** `pub mod test_helpers;` to `src/lib.rs`

### Result:
- Removed ~60 lines of duplicated code
- Single source of truth for test data

---

## Files Modified

- `src/lib.rs` - added test_helpers module
- `src/test_helpers.rs` - new file
- `src/state/server_ops.rs` - use shared helpers
- `src/state/navigation.rs` - use shared helpers
- `src/state/app_state.rs` - use shared helpers
- `src/state/settings_ops.rs` - use shared helpers
- `src/state/notifications.rs` - use shared helpers

## References

- Related: issue090 (test coverage gaps)
