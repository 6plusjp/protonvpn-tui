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
| Current | 24 |
| **Resolved** | **-7** |

#### Current Violations (24)

**get_ (12)**:
| File | Count | Functions |
|------|-------|-----------|
| `vpn/client.rs` | 2 | `get_connected_server_info()`, `get_connection_protocol()` |
| `ui/app.rs` | 3 | `get_theme()`, `get_footer_action_hints()`, `get_global_connect_hints()` |
| `config/settings.rs` | 1 | `get_selectable_option_command()` |
| `ui/views/settings_view.rs` | 2 | `get_setting_value()`, `get_setting_label()` |
| `state/app_state.rs` | 2 | `get_theme()`, `get_selection_bounds()` |
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

#### Resolved (7)

Previously resolved via past PRs.

---

### 2. unwrap() Usage

| File | Count | Notes |
|------|-------|-------|
| `config/user_config.rs` | 1 | ~~`toml::from_str(...).unwrap()`~~ → Fixed with `expect()` + justification |
| `vpn/async_tasks.rs` | 5 | Mutex lock - **Justified** (synchronization) |
| `vpn/types.rs` | 1 | `parts.last().unwrap()` - Needs review |
| `state/connection.rs` | 4 | Mutex/Condvar wait - **Justified** |

**Resolved**: 1 (`user_config.rs:363`)

---

### 3. main.rs Documentation

| Item | Status |
|------|--------|
| `//!` doc comment | ❌ Missing |

---

## Prioritized Tasks

1. ~~**[Todo] Small**: Add `//!` doc comment to `src/main.rs`~~ ✓ Resolved
2. ~~**[Todo] Medium**: Replace `unwrap()` in `user_config.rs:363` with proper error handling~~ ✓ Resolved
3. **[Todo] Large**: Convert 24 getter/setters to pub fields (phase gradually)

---

## Related

- [@docs/policy/coding-standards.md](docs/policy/coding-standards.md)
- [@docs/policy/rust-maintainability.md](docs/policy/rust-maintainability.md)
