# issue076: Test coverage gaps for views and renderers

## Summary

Several UI components lack test coverage despite having substantial logic.

## Problem

Current test coverage is good for core logic but has gaps in UI layer:

### Well-Tested Modules
| Module | Tests |
|--------|-------|
| vpn/types.rs | 23 parsing tests |
| state/app_state.rs | 16+ filter/sort/notification tests |
| state/notifications.rs | 8 tests |
| ui/keymap.rs | 7 tests |
| config/user_config.rs | 3 tests |

### Untested Modules
| Module | Notes |
|--------|-------|
| ui/views/servers_view.rs | No tests |
| ui/views/tools_view.rs | No tests |
| ui/views/settings_view.rs | No tests |
| ui/views/logs_view.rs | No tests |
| ui/renderers/footer.rs | No tests |
| ui/renderers/header.rs | No tests |
| ui/renderers/notification.rs | No tests |
| ui/input/handler.rs | No tests |
| ui/input/filter.rs | No tests |
| state/navigation.rs | No tests |
| state/server_ops.rs | No tests |
| state/settings_ops.rs | No tests |

## Solution

Priority testing targets:

### P1: State operations (pure logic, easy to test)
- `state/navigation.rs` — Selection/pane navigation logic
- `state/server_ops.rs` — Server filtering/sorting operations
- `state/settings_ops.rs` — Settings management

### P2: Input handling (logic-heavy)
- `ui/input/handler.rs` — Key event routing
- `ui/input/filter.rs` — Filter logic

### P3: Renderers (can test output structure)
- `ui/renderers/footer.rs` — Hint generation logic

## Acceptance Criteria

- [ ] Unit tests added for state/navigation.rs
- [ ] Unit tests added for state/server_ops.rs
- [ ] Unit tests added for state/settings_ops.rs
- [ ] Unit tests added for ui/input/filter.rs
- [ ] `cargo test` passes with new tests
