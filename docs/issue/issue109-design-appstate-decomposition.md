# issue109: Design - AppState Decomposition

## Summary

Design and implement the decomposition of the monolithic 12-field `AppState` into composed sub-state structs.

## Problem

`src/state/app_state.rs` contains a struct with 12+ fields, violating Single Responsibility Principle and making testing difficult.

**Current fields:**
```rust
pub struct AppState {
    pub vpn_state: Arc<VpnClient>,
    pub connection_manager: ConnectionManager,
    pub servers: Vec<Server>,
    pub server_cache: FilteredServerCache,
    pub is_initialized: bool,
    pub current_cities: Vec<City>,
    pub current_country_code: Option<String>,
    pub ui_state: UiState,
    pub notification_state: NotificationState,
    pub config_state: ConfigState,
    pub key_bindings: KeyBindings,
    pub user_config: UserConfig,
    pub keymap: KeyMap,
}
```

## Analysis Result

### Usage Frequency (from grep)

| Field | Access Count | Category |
|-------|-------------|----------|
| `ui_state` | ~198 | UI |
| `vpn_state` | ~39 | VPN |
| `notification_state` | ~18 | Notification |
| `connection_manager` | ~10 | VPN |
| `current_cities` | ~8 | Data |
| others | <5 | Config |

### Existing Sub-states

| Struct | Fields | Status |
|--------|-------|--------|
| `UiState` | ~20 | Already separated |
| `NotificationState` | ~10 | Already separated |
| `ConfigState` | ~5 | Partial |
| `ConnectionManager` | ~10 | Already separated |

### Async Patterns

- `vpn_state: Arc<VpnClient>` - shared, thread-safe
- `ConnectionManager` - contains `AsyncNotifier` (Mutex + Condvar)

## Adopted Design: Option B (Sub-states)

```rust
pub struct AppState {
    pub vpn: VpnState,                    // VPN connection + async management
    pub data: DataState,               // server/city data
    pub ui: UiState,                  // UI state (highest access - keep as-is)
    pub notifications: NotificationState, // notifications (keep as-is)
    pub config: ConfigState,          // settings + KeyBindings + Keymap
}
```

### VpnState (NEW)

```rust
pub struct VpnState {
    pub client: Arc<VpnClient>,           // shared VPN client
    pub connection: ConnectionManager,   // async connection management
    pub connected_at: Option<chrono::DateTime<chrono::Utc>>,
}
```

**Rationale**: `vpn_state` (~39 accesses) and `connection_manager` (~10 accesses) are always accessed together.

### DataState (NEW)

```rust
pub struct DataState {
    pub servers: Vec<Server>,             // raw server list
    pub cache: FilteredServerCache, // filtered cache
    pub cities: Vec<City>,            // current cities for selected country
    pub country: Option<CountryCode>, // selected country
    pub initialized: bool,
}
```

**Rationale**: Consolidate scattered server-related data.

### UiState (KEEP as-is)

Already properly separated with ~20 fields. Access frequency is highest (~198), so splitting would require massive caller changes.

### NotificationState (KEEP as-is)

Already properly separated with ~18 accesses. Good granularity.

### ConfigState (EXPANDED)

```rust
pub struct ConfigState {
    pub user: UserConfig,      // user settings
    pub bindings: KeyBindings, // key bindings
    pub keymap: KeyMap,      // key map
}
```

**Rationale**: All config-related fields consolidated.

## Implementation Plan

### Phase 1: ConfigState Consolidation (Low Risk)

- [ ] Move `key_bindings` and `keymap` into `ConfigState`
- [ ] Update all callers

### Phase 2: DataState Creation

- [ ] Create `DataState` struct
- [ ] Move `servers`, `server_cache`, `current_cities`, `current_country_code`, `is_initialized`
- [ ] Update all callers

### Phase 3: VpnState Creation

- [ ] Create `VpnState` struct
- [ ] Move `vpn_state` and `connection_manager`
- [ ] Add `connected_at` field
- [ ] Update all callers

### Phase 4: AppState Refactor

- [ ] Re-compose AppState with new sub-states
- [ ] Add migration comments
- [ ] Add tests for each sub-state

## Files to Update

| File | Changes |
|------|--------|
| `src/state/app_state.rs` | Re-compose struct |
| `src/state/mod.rs` | Export new types |
| `src/state/vpn_state.rs` | New file |
| `src/state/data_state.rs` | New file |
| `src/ui/app.rs` | Update accesses |
| All `src/state/*.rs` | Update field accesses |

## Verification

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] All field accesses updated

## Notes

### Why UiState/NotificationState Not Split

1. **UiState**: Highest access frequency (~198). Splitting would require updating 100+ call sites.
2. Already properly encapsulated in existing sub-state.

### Test Strategy

Each sub-state should be independently testable with mocks:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_vpn_state_connection() {
        let vpn = VpnState::new();
        // test connection logic
    }

    #[test]
    fn test_data_state_filter() {
        let data = DataState::new();
        // test filtering logic
    }
}
```