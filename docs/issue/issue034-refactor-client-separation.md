# issue034 - Refactor vpn/client.rs: Separate CLI Wrapper from Parsing/Cache Logic

## Summary

The `vpn/client.rs` file currently has multiple responsibilities:
1. **CLI execution** (legitimate: wrapping `protonvpn` commands)
2. **Parsing CLI output** (should be in `types.rs`)
3. **Cache management** (should be in `cache.rs`)
4. **State queries** (should be in `cache.rs`)

This violates the Single Responsibility Principle and makes the code harder to maintain and test.

## Related Issues

- (None directly related)

---

## Problems Identified

### Current State: All Functions in client.rs

**Location**: `src/vpn/client.rs`

The file contains ~25 public functions, many of which don't directly interact with the CLI:

```
src/vpn/
├── client.rs   # 725 lines - OVERLOADED
├── cache.rs    # ServerCache struct only (235 lines)
├── types.rs    # Data types only (66 lines)
└── state.rs   # Thin VpnClient wrapper (147 lines)
```

### Functions That Don't Execute CLI

| Function | Current Location | Should Be In | Status |
|----------|-----------------|--------------|--------|
| `with_cache()` | client.rs | cache.rs | Not moved yet |
| `save_cache()` | client.rs | cache.rs | Not moved yet |
| `parse_countries()` | client.rs | types.rs | Not moved yet |
| `parse_cities_with_features()` | client.rs | types.rs | Not moved yet |
| `parse_connect_output()` | client.rs | types.rs | Not moved yet |
| `get_connected_server()` | client.rs | cache.rs | Not moved yet |
| `get_vpn_ip()` | client.rs | cache.rs | Not moved yet |
| `matches_ip()` | client.rs | cache.rs | Not moved yet |
| `countries_to_servers()` | client.rs | types.rs | Not moved yet |
| `check_cli_error()` | client.rs | client.rs | Legitimate - stays |
| `is_connected()` | client.rs | client.rs | Network check - stays |

**Removed (unused)**:
- `get_countries()` - ✅ Deleted
- `list_countries()` - ✅ Deleted
- `list_cities()` - ✅ Deleted  
- `status()` - ✅ Deleted

---

## Proposed Solution

### Target Architecture

```
src/vpn/
├── client.rs   # CLI execution only (~200 lines)
│   ├── connect(), connect_random(), connect_city()
│   ├── disconnect()
│   ├── list_countries(), list_cities(), list_servers()
│   ├── refresh_countries(), refresh_servers()
│   ├── config_set(), toggle_*(), set_*()
│   └── check_cli_error()
│
├── cache.rs    # Cache management (~300 lines)
│   ├── ServerCache struct
│   ├── with_cache(), save_cache()
│   ├── get_connected_server(), get_vpn_ip()
│   ├── matches_ip(), status()
│   └── FALLBACK_COUNTRIES
│
├── types.rs    # Data types + parsing (~200 lines)
│   ├── ServerFeatures, City, Server, ConnectionStats
│   ├── parse_countries()
│   ├── parse_cities_with_features()
│   ├── parse_connect_output()
│   └── countries_to_servers()
│
└── state.rs    # Thin wrapper (unchanged)
```

### Migration Plan

**Step 1**: Move parsing functions to `types.rs`

```rust
// src/vpn/types.rs

impl VpnClient {
    pub fn parse_countries(output: &str) -> HashMap<String, String> { ... }
    pub fn parse_cities_with_features(output: &str) -> Vec<City> { ... }
    pub fn parse_connect_output(output: &str) -> (String, Option<String>, Option<String>, Option<String>) { ... }
}

// Free function (preferred)
pub fn parse_countries(output: &str) -> HashMap<String, String> { ... }
pub fn parse_cities_with_features(output: &str) -> Vec<City> { ... }
pub fn parse_connect_output(output: &str) -> (String, Option<String>, Option<String>, Option<String>) { ... }
pub fn countries_to_servers(countries: &HashMap<String, String>, cities: &HashMap<String, Vec<City>>) -> Vec<Server> { ... }
```

**Step 2**: Move cache-related functions to `cache.rs`

```rust
// src/vpn/cache.rs

impl ServerCache {
    pub fn with_cache<F, T>(&mut self, f: F) -> AppResult<T>
    where F: FnOnce(&mut Self) -> T { ... }
    
    // Already has: save(), load(), is_stale()
    // Already has: set_connected(), set_disconnected(), matches_ip()
    
    pub fn get_connected_server(&self) -> Option<String> { ... }
    pub fn get_vpn_ip(&self) -> Option<String> { ... }
    pub fn status(&self) -> String { ... }
}
```

**Step 3**: Update `VpnClient` to delegate

```rust
// src/vpn/client.rs - simplified

impl VpnClient {
    // CLI execution only
    pub fn connect(&self, target: &str) -> AppResult<(String, Option<String>)> { ... }
    pub fn disconnect(&self) -> AppResult<()> { ... }
    // etc.
}
```

---

## Benefits

1. **Single Responsibility**: Each module has one clear purpose
2. **Testability**: Parsing functions can be tested without CLI
3. **Maintainability**: Smaller, focused files
4. **Reusability**: Cache logic can be used independently

---

## Status: In Progress

### Completed (2026-03-11)

- [x] Remove unused functions from client.rs:
  - `get_countries()` - unused
  - `list_countries()` - unused  
  - `list_cities()` - use `list_cities_with_features()` instead
  - `status()` - unused
- [x] Update related tests

### Pending

- [ ] Move parsing functions to types.rs
- [ ] Move cache management functions to cache.rs  
- [ ] Update VpnClient to delegate to appropriate modules
- [ ] Run cargo check and cargo test

---

## Priority

| Priority | Item | Effort | Status |
|----------|------|--------|--------|
| ~~Medium~~ **Done** | Remove unused functions | Low | ✅ Done |
| Medium | Move parsing to types.rs | Low | Pending |
| Medium | Move cache logic to cache.rs | Low | Pending |
| Low | Update tests | Low | Pending |
| Low | Verify build passes | Low | Pending |

---

## Notes

- `is_connected()` checks `proton0` interface - belongs in client.rs (network check, not CLI)
- `check_cli_error()` validates CLI output - belongs in client.rs
- Consider making parsing functions free functions (not impl methods) for purity
- This refactoring is non-breaking - just internal reorganization
