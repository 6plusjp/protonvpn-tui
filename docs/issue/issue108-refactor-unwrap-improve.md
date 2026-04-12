# issue033: Refactor - Improve unwrap() Error Messages

## Summary

Replace bare `.unwrap()` calls in `vpn/types.rs` with descriptive `.expect()` or proper error handling.

## Problem

`vpn/types.rs` has 14 `.unwrap()` calls concentrated in parsing functions (lines 734-935):

```rust
let uptime = uptime.unwrap();
let info = info.unwrap();
let server = server.unwrap();
```

These provide no context when they panic, making debugging difficult.

## Solution

Replace with descriptive `.expect()`:

```rust
let uptime = uptime.expect("parse_connect_output: missing required field 'uptime'");
let info = info.expect("parse_connect_output: missing required field 'info'");
```

Or better: use `ok()` with logging:

```rust
let uptime = uptime.ok().inspect(|_| tracing::debug!("uptime field missing in connect output"));
```

### Priority
- **Low**: This is a quality-of-life improvement, not a bug fix
- Focus after higher-impact refactors

### Files to Modify

| File | Changes |
|------|---------|
| `src/vpn/types.rs` | Add descriptive messages to 14 unwrap calls |

### Acceptance Criteria

- [ ] No behavior change at runtime (unless panic)
- [ ] Better error messages if panic occurs
- [ ] `cargo test` passes