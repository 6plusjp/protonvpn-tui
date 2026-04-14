# Resolved 107: Refactor - Extract String Constants

## Summary

Move duplicated `"on"` and `"off"` string literals to `constants.rs`.

## Problem

The strings `"on"` and `"off"` appear in 4+ locations throughout the codebase:

| File | Line |
|------|------|
| `src/config/settings.rs` | 101-107 |
| `src/ui/views/settings_view.rs` | 111-201 |
| `src/ui/input/tools.rs` | 172 |
| `src/vpn/client.rs` | 626, 642, 654, 663, 679 |

These represent VPN setting values and should be centralized constants.

## Solution

Add to `src/constants.rs`:

```rust
pub const FEATURE_ON: &str = "on";
pub const FEATURE_OFF: &str = "off";
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/constants.rs` | Add constants |
| `src/config/settings.rs` | Use constants |
| `src/ui/views/settings_view.rs` | Use constants |
| `src/ui/input/tools.rs` | Use constants |
| `src/vpn/client.rs` | Use constants |

### Acceptance Criteria

- [x] Constants defined in `constants.rs`
- [x] All 4 files updated
- [x] `cargo test` passes
- [x] Easier to change values globally if needed

## Implementation Notes

### Architecture

| Location | Import Path |
|----------|-----------|
| `constants.rs` | Definition (`pub mod settings`) |
| `config/settings.rs` | `use crate::constants::settings::{FEATURE_OFF, FEATURE_ON}` |
| `vpn/client.rs` | `use crate::constants::settings::{FEATURE_OFF, FEATURE_ON}` |
| `ui/input/tools.rs` | `use crate::constants::settings::{FEATURE_OFF, FEATURE_ON}` |
| `ui/views/settings_view.rs` | `use crate::constants::settings::{FEATURE_OFF, FEATURE_ON}` |

### Files Modified

| File | Changes |
|------|---------|
| `src/constants.rs` | Added `pub mod settings { FEATURE_ON, FEATURE_OFF }` |
| `src/config/settings.rs` | Uses constants in `selectable_options()` |
| `src/vpn/client.rs` | Uses constants in toggle/set functions |
| `src/ui/input/tools.rs` | Uses constants in Footer toggle |
| `src/ui/views/settings_view.rs` | Uses constants in value formatting |

### Test Results

- `cargo test`: 230 passed
- `cargo check`: Clean