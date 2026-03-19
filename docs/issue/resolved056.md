# issue056: Architecture - Empty commands/ module

## Summary

The `src/commands/` directory contained only a placeholder comment with no implementation.

## Problem

```rust
//! Command handlers
// CLI command implementations will go here
```

This was 3 lines of dead code with no functionality.

## Resolution (2026-03-19)

**Removed** the entire `src/commands/` directory:
- Deleted `src/commands/mod.rs`
- Removed `pub mod commands;` from `src/lib.rs`
- Verified no references to `commands` module existed in codebase

**Rationale**: This TUI application does not require CLI subcommands. VPN commands are handled by the `src/vpn/` module directly.

## Files Changed

- `src/lib.rs` - removed `pub mod commands;`
- `src/commands/` - deleted directory

## Severity

🟢 RESOLVED - Dead code removed
