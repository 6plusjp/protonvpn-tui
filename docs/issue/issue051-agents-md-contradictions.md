# issue051-agents-md-contradictions

## Summary

Track contradictions between AGENTS.md/Policy and actual implementation.

**Status: [Open]**

---

## Current State (2026-03-17)

### 1. Getter/Setter Violations (Ongoing)

| Period | Violations |
|--------|-------------|
| Initial | 31 |
| Current | 20 |
| **Resolved** | **-11** |

#### Current Violations (20)

**get_ (8)**:
| File | Count | Functions |
|------|-------|-----------|
| `vpn/client.rs` | 2 | `get_connected_server_info()`, `get_connection_protocol()` |
| `ui/app.rs` | 2 | `get_footer_action_hints()`, `get_global_connect_hints()` |
| `config/settings.rs` | 1 | `get_selectable_option_command()` |
| `ui/views/settings_view.rs` | 1 | `get_setting_value()` |
| `state/log_persistence.rs` | 1 | `get_log_file_path()` |
| `state/ui_state.rs` | 1 | `get_cached()` |

**set_ (12)**:
| File | Count | Functions |
|------|-------|-----------|
| `vpn/client.rs` | 3 | `set_config()`, `set_netshield()`, `set_custom_dns()` |
| `vpn/cache.rs` | 2 | `set_connected()`, `set_disconnected()` |
| `state/app_state.rs` | 4 | `set_search_query()`, `set_servers()`, `set_sort_by_code()`, `set_sort_by_country()` |
| `state/ui_state.rs` | 2 | `set_view()`, `set_cached()` |
| `state/log_persistence.rs` | 1 | `set_test_mode()` |

#### Resolved (11)

**Previously resolved (7)**:
- Past PRs

**Today's fixes (2026-03-17) (4)**:
- `ui/views/settings_view.rs::get_setting_label()` → Moved to `SettingKey::label()`
- `state/app_state.rs::get_selection_bounds()` → Renamed to `selection_bounds()`
- `ui/app.rs::get_theme()` → Renamed to `theme()`
- `state/app_state.rs::get_theme()` → Renamed to `theme()`

---

### 2. unwrap() Usage

| File | Count | Notes |
|------|-------|-------|
| `config/user_config.rs` | 1 | ~~`toml::from_str(...).unwrap()`~~ → Fixed with `expect()` + justification |
| `vpn/async_tasks.rs` | 5 | Mutex lock - **Justified** (synchronization) |
| `vpn/types.rs` | 1 | ~~`parts.last().unwrap()`~~ → Fixed with `expect()` + justification |
| `state/connection.rs` | 4 | Mutex/Condvar wait - **Justified** |

**Resolved**: 2 (`user_config.rs:363`, `vpn/types.rs:109`)

---

### 3. main.rs Documentation

| Item | Status |
|------|--------|
| `//!` doc comment | ✅ Resolved |

---

## Prioritized Tasks

1. ~~**[Todo] Small**: Add `//!` doc comment to `src/main.rs`~~ ✓ Resolved
2. ~~**[Todo] Medium**: Replace `unwrap()` in `user_config.rs:363` with proper error handling~~ ✓ Resolved
3. ~~**[Todo] Small**: Fix `get_setting_label()` → `SettingKey::label()`~~ ✓ Resolved (2026-03-17)
4. ~~**[Todo] Small**: Fix `get_selection_bounds()` → `selection_bounds()`~~ ✓ Resolved (2026-03-17)
5. ~~**[Todo] Small**: Fix `parts.last().unwrap()` in `vpn/types.rs:109`~~ ✓ Resolved (2026-03-17)
6. **[Todo] Large**: Convert 22 getter/setters to pub fields (phase gradually)

### Additional Findings (Non-violations)

The following were identified as trivial wrappers but do NOT violate the policy (no `get_`/`set_` prefix):

| File | Function | Status |
|------|----------|--------|
| `config/settings.rs` | `index()` | Unused (can be removed) |
| `config/settings.rs` | `selectable_option_count()` | In use, but trivial |
| `ui/app.rs` | `get_theme()` | Trivial but duplicated in `state/app_state.rs` |
| `state/app_state.rs` | `get_theme()` | Trivial but duplicated in `ui/app.rs` |

---

## Related

- [@docs/policy/coding-standards.md](docs/policy/coding-standards.md)
- [@docs/policy/rust-maintainability.md](docs/policy/rust-maintainability.md)
