# issue055: Architecture - server_data.rs is dead code

## Summary

`src/state/server_data.rs` defines `ServerDataState` but it is **never actually used** by the business logic.

## Problem

The code in `app_state.rs` uses its own direct fields instead of `ServerDataState`:

```rust
// server_data.rs - defines but never used
pub struct ServerDataState {
    pub servers: Vec<Server>,           // ← UNUSED
    pub current_cities: Vec<City>,      // ← UNUSED  
    pub current_country_code: Option<String>, // ← UNUSED
}

// app_state.rs - uses these direct fields instead
pub servers: Vec<Server>,              // ← ACTUALLY USED
pub current_cities: Vec<City>,         // ← ACTUALLY USED
pub current_country_code: Option<String>, // ← ACTUALLY USED
```

## Files Affected

- `src/state/server_data.rs` - entire file is dead code
- `src/state/mod.rs` - exports `ServerDataState`
- `src/state/app_state.rs` - defines `server_data: ServerDataState` field but never uses it

## Recommendation

1. Remove `server_data: ServerDataState` field from `AppState`
2. Remove `server_data.rs` file
3. Update `state/mod.rs` to remove the export
4. Verify with `cargo check --all-targets`

## Severity

🔴 CRITICAL - Dead code that adds unnecessary complexity and confusion

## Labels

`architecture` `dead-code` `refactoring`
