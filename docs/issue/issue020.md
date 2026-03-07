# issue020: Feature - Simplify disconnect detection

## Summary

Use `protonvpn disconnect` command result directly to determine disconnection status, instead of checking `ip addr`.

## Problem

Currently, the disconnect logic may be using `ip addr` to verify disconnection, which is:
- Too heavy (requires parsing network interfaces)
- Platform-specific
- Unnecessary since `protonvpn disconnect` already returns appropriate exit codes

## Current Implementation (to verify)

Check `src/vpn/client.rs` and `src/vpn/state.rs` for current disconnect handling:

```bash
# protonvpn disconnect exit codes:
# 0 = successfully disconnected
# 1 = already disconnected or error
```

## Solution

1. Parse `protonvpn disconnect` output/exit code directly
2. Remove `ip addr` checking logic (if exists)
3. Trust the CLI's response

### Implementation

```rust
pub fn disconnect(&self) -> Result<(), VpnError> {
    let output = Command::new("protonvpn")
        .args(["disconnect"])
        .output()?;
    
    // Check exit code or output message
    if output.status.success() || 
       output.stdout.contains("disconnected") || 
       output.stderr.contains("disconnected") {
        Ok(())
    } else {
        Err(VpnError::DisconnectionFailed(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}
```

## Additional Considerations

- Handle "already disconnected" gracefully (not an error)
- Show appropriate notification message based on result

---

## Acceptance Criteria

- [ ] `protonvpn disconnect` result determines disconnection status
- [ ] No `ip addr` checking for disconnect verification
- [ ] "Already disconnected" handled gracefully
- [ ] Appropriate user feedback via notification
