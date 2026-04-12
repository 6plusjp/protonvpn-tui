# issue027: Refactor - Settings Feature Access Duplication

## Summary

Consolidate duplicated `ProtonFeatures` access patterns across 4 files into a single helper method on `ProtonSettings`.

## Problem

The same `.and_then()` chaining pattern for accessing `ProtonFeatures` fields appears in **4 different locations**:

### Current Pattern (4 files)
```rust
// tools.rs, settings_view.rs, settings_ops.rs, settings.rs
SettingKey::NetShield => match ps.features.as_ref().and_then(|f| f.netshield) { ... }
SettingKey::ModerateNat => ps.features.as_ref().and_then(|f| f.moderate_nat).map(|v| if v { 1 } else { 0 })
SettingKey::VpnAccelerator => ps.features.as_ref().and_then(|f| f.vpn_accelerator).map(...)
SettingKey::PortForwarding => ps.features.as_ref().and_then(|f| f.port_forwarding).map(...)
```

### Affected Files
| File | Lines | Pattern |
|------|-------|----------|
| `src/ui/input/tools.rs` | 268-328 | Settings value conversion |
| `src/ui/views/settings_view.rs` | 91-208 | Settings value rendering |
| `src/state/settings_ops.rs` | 54-100 | Settings toggle operations |
| `src/config/settings.rs` | 224-250 | Settings count calculation |

## Solution

Create a helper method on `ProtonSettings` to centralize feature access:

```rust
impl ProtonSettings {
    /// Get a feature value by setter key, returning T
    pub fn get_feature<T, F>(key: SettingKey, f: F) -> Option<T>
    where
        F: FnOnce(&ProtonFeatures) -> Option<T>;

    /// Get feature as i32 (for CLI args: on=1, off=0)
    pub fn get_feature_as_i32(key: SettingKey) -> Option<i32>;

    /// Check if any feature is enabled
    pub fn has_any_feature(&self) -> bool;
}
```

### New Location
- Add to `src/config/settings.rs` (where `ProtonSettings` is defined)

### Files to Modify

| File | Changes |
|------|---------|
| `src/config/settings.rs` | Add helper methods to `ProtonSettings` impl |
| `src/ui/input/tools.rs` | Use helper methods |
| `src/ui/views/settings_view.rs` | Use helper methods |
| `src/state/settings_ops.rs` | Use helper methods |

### Acceptance Criteria

- [ ] Single source of truth for feature access
- [ ] All 4 files refactored to use helper methods
- [ ] No runtime behavior change
- [ ] `cargo test` passes
- [ ] Reduced code duplication (estimated ~40 lines)