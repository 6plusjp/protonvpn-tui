# issue071: Cargo.toml dependency issues (version conflicts and pinning)

## Summary

**RESOLVED** - Dependency configuration has a crossterm version conflict, overly strict version pins, and minor clippy warnings.

**Resolved**: 2026-03-24
**Resolved by**: Sisyphus (AI agent)

## Problem

### 1. Crossterm Version Conflict (CRITICAL)

```toml
[dependencies]
crossterm = "=0.27"  # Direct dependency, exact pin
ratatui = { version = "0.30", default-features = false, features = ["crossterm"] }
```

- Ratatui's crossterm backend pulls in `crossterm v0.29.0`
- Direct dependency pins to `crossterm v0.27.0`
- Result: **Two versions** of crossterm in dependency tree
- Impact: Wasted binary size, potential subtle incompatibilities

### 2. Over-Strict Version Pins

All dependencies use exact pinning (`=X.Y`), preventing security updates:

| Dependency | Current | Recommendation |
|------------|---------|----------------|
| clap | `=4.5` | `^4.5` |
| serde | `=1.0` | `^1.0` |
| serde_json | `=1.0` | `^1.0` |
| thiserror | `=1.0` | `^1.0` |
| anyhow | `=1.0` | `^1.0` |
| tracing | `=0.1` | `^0.1` |
| tracing-subscriber | `=0.3` | `^0.3` |
| toml | `=0.8` | `^0.8` |
| chrono | `=0.4` | `^0.4` |
| dirs | `=5.0` | `^5.0` |
| once_cell | `=1.19` | `^1.19` |
| tempfile | `=3.0` | `^3.0` |
| assert_cmd | `=1.0` | `^1.0` |

Note: `ratatui` already correctly uses `^0.30` (semver range).

### 3. Clippy Warnings (2 warnings)

**File**: `src/config/user_config.rs`

```rust
// Line 623: Replace assert_eq! with bool literal
assert_eq!(config.ui.footer, true);   // ❌ Clippy warning
assert!(config.ui.footer);             // ✅ Correct

// Line 636: Replace assert_eq! with bool literal
assert_eq!(config.ui.footer, false);   // ❌ Clippy warning
assert!(!config.ui.footer);            // ✅ Correct
```

## Solution

### 1. Fix Crossterm Conflict

Option A (Recommended): Remove direct crossterm dependency, let ratatui manage it
```toml
[dependencies]
# crossterm = "=0.27"  # REMOVE THIS LINE
ratatui = { version = "^0.30", default-features = false, features = ["crossterm"] }
```

Option B: Use semver range to allow ratatui's version
```toml
crossterm = "0.27"  # Drop = prefix
```

### 2. Update Version Pins to Semver Ranges

Change all `=X.Y` to `^X.Y` in Cargo.toml:
```toml
[dependencies]
clap = { version = "^4.5", features = ["derive"] }
# ... etc
```

### 3. Fix Clippy Warnings

```bash
cargo clippy --fix --lib -p protonvpn-tui --tests
```

Or manually fix in `src/config/user_config.rs`:
- Line 623: `assert!(config.ui.footer);`
- Line 636: `assert!(!config.ui.footer);`

## Acceptance Criteria

- [x] Crossterm version conflict resolved (single version in tree)
- [x] Cargo.toml uses semver ranges (`^X.Y`) instead of exact pins (`=X.Y`)
- [x] `cargo clippy` passes with zero warnings
- [x] `cargo build --release` compiles cleanly
- [x] `cargo test` passes

## Verification

```bash
# Check dependency tree (should show only one crossterm version)
cargo tree -d -p crossterm

# Verify no clippy warnings
cargo clippy --all-targets --all-features -- -D warnings

# Verify build and tests
cargo build --release && cargo test
```

## Resolution

### Changes Made

1. **Cargo.toml**:
   - Updated crossterm from `=0.27` to `0.29` (matches ratatui's dependency)
   - Removed exact pin prefix (`=`) from all dependencies:
     - clap: `=4.5` → `4.5`
     - serde: `=1.0` → `1.0`
     - serde_json: `=1.0` → `1.0`
     - thiserror: `=1.0` → `1.0`
     - anyhow: `=1.0` → `1.0`
     - tracing: `=0.1` → `0.1`
     - tracing-subscriber: `=0.3` → `0.3`
     - toml: `=0.8` → `0.8`
     - chrono: `=0.4` → `0.4`
     - dirs: `=5.0` → `5.0`
     - once_cell: `=1.19` → `1.19`
     - tempfile: `=3.0` → `3.0`
     - assert_cmd: `=1.0` → `1.0`

2. **src/config/user_config.rs**:
   - Line 623: `assert_eq!(config.ui.footer, true)` → `assert!(config.ui.footer)`
   - Line 636: `assert_eq!(config.ui.footer, false)` → `assert!(!config.ui.footer)`

### Verification Results

- ✅ `cargo tree -d -p crossterm`: No duplicate versions
- ✅ `cargo clippy --all-targets --all-features -- -D warnings`: Zero warnings
- ✅ `cargo build --release`: Compiled successfully
- ✅ `cargo test`: 168 tests passed (55 unit, 113 integration)
