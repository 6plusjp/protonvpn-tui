# issue072: Documentation - README and AGENTS.md out of sync with codebase

## Summary

**RESOLVED** - README.md and AGENTS.md files contained stale references that didn't match the actual codebase structure.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

Pre-publication review revealed multiple documentation drift issues:

### 1. README.md

- **Line 31**: Clone URL was placeholder `https://github.com/yourusername/protonvpn-tui`
- **Architecture section** (lines 153-174): Outdated directory tree missing `input/`, `renderers/`, `keymap.rs`, `error.rs`, `constants.rs`, `paths.rs`, state/ detail

### 2. Root AGENTS.md

| AGENTS.md Listed | Actual Status |
|-----------------|---------------|
| `src/commands/` (placeholder) | Deleted (resolved056.md) |
| `src/state/connection.rs` | Renamed → `connection_manager.rs` (resolved058.md) |
| `src/state/server_data.rs` | Deleted (resolved055.md) |
| (not listed) | `src/state/app_state_impl.rs` added |
| (not listed) | `src/state/event_handler.rs` added |
| (not listed) | `src/state/navigation.rs` added |
| (not listed) | `src/state/server_ops.rs` added |
| (not listed) | `src/state/settings_ops.rs` added |
| `components/styles.rs` | Not existed |

### 3. src/ui/AGENTS.md

- Referenced `App` struct (actual: `TuiApp`)
- Missing `input/`, `renderers/`, `keymap.rs` modules
- Public API example showed `pub use app::App` — didn't match actual exports

### 4. src/state/AGENTS.md

- Missing `server_cache.rs`

## Solution

### README.md

1. Fixed clone URL to `https://github.com/6plusjp/protonvpn-tui`
2. Updated Architecture section to match actual `src/` structure with full tree

### Root AGENTS.md

1. Removed `src/commands/` (deleted)
2. Updated `connection.rs` → `connection_manager.rs`
3. Removed `server_data.rs` (deleted)
4. Added new state modules: `app_state_impl.rs`, `event_handler.rs`, `navigation.rs`, `server_ops.rs`, `settings_ops.rs`, `server_cache.rs`
5. Removed `components/styles.rs` (not existed)

### src/ui/AGENTS.md

- Rewritten to match current structure: `TuiApp`, `input/`, `renderers/`, `keymap.rs`
- Removed outdated Public API section

### src/state/AGENTS.md

- Added missing `server_cache.rs`

## Acceptance Criteria

- [x] README.md clone URL points to actual repository
- [x] README.md Architecture section matches actual `src/` structure
- [x] Root AGENTS.md structure matches actual files (no stale refs, all new files listed)
- [x] src/ui/AGENTS.md matches current ui module structure
- [x] src/state/AGENTS.md matches current state module structure
- [x] `cargo check` passes (no code changes expected)

## Verification

```bash
cargo check  # Passed - no errors
```

## Resolution

### Files Modified

1. **README.md**:
   - Line 31: Clone URL `yourusername` → `6plusjp`
   - Architecture section: Full rewrite with accurate directory tree

2. **AGENTS.md** (root):
   - Removed `commands/` entry
   - Updated state section: `connection.rs` → `connection_manager.rs`, removed `server_data.rs`, added 6 new modules
   - Removed non-existent `components/styles.rs`

3. **src/ui/AGENTS.md**:
   - Complete rewrite: `TuiApp`, `input/`, `renderers/`, `keymap.rs` documented

4. **src/state/AGENTS.md**:
   - Added `server_cache.rs` to structure

### References

- docs/issue/resolved056.md — commands/ deletion
- docs/issue/resolved058-connection-naming-confusion.md — connection.rs → connection_manager.rs
- docs/issue/resolved055.md — server_data.rs deletion
