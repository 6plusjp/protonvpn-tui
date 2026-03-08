# resolved020: Feature - Simplify disconnect detection

## Summary

Use `protonvpn disconnect` command result directly to determine disconnection status, instead of checking `ip addr`.

## Problem

Previously, the disconnect logic used `ip addr` to verify disconnection, which is:
- Too heavy (requires parsing network interfaces)
- Platform-specific
- Unnecessary since `protonvpn disconnect` already returns appropriate exit codes

## Solution

1. Parse `protonvpn disconnect` output/exit code directly
2. Remove `ip addr` checking logic
3. Trust the CLI's response

### Implementation (src/vpn/client.rs)

```rust
pub fn disconnect(&self) -> AppResult<()> {
    let output = Command::new(&self.cli_path)
        .args(["disconnect"])
        .output()
        .map_err(|e| {
            tracing::warn!("Failed to execute disconnect command: {}", e);
            AppError::ConnectionFailed(format!("Failed to execute disconnect: {}", e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}{}", stdout, stderr);

    // Exit code 0 = successfully disconnected
    // Exit code 1 = already disconnected or error
    if output.status.success() {
        self.with_cache(|c| c.set_disconnected())?;
        self.save_cache()?;
        tracing::info!("Successfully disconnected from VPN");
        return Ok(());
    }

    // Handle "already disconnected" gracefully - not an error
    if combined_output.to_lowercase().contains("already disconnected")
        || combined_output.to_lowercase().contains("not connected")
    {
        self.with_cache(|c| c.set_disconnected())?;
        self.save_cache()?;
        tracing::info!("Already disconnected from VPN");
        return Ok(());
    }

    Err(AppError::ConnectionFailed(format!(
        "Failed to disconnect: {}",
        combined_output.trim()
    )))
}
```

## Changes

- Removed `ip addr` checking retry loop
- Removed unused imports: `DISCONNECT_RETRY_COUNT`, `DISCONNECT_RETRY_DELAY_MS`
- Added direct exit code and output message parsing
- Handle "already disconnected" gracefully (not an error)

## Note

Disconnect delay may occur due to ProtonVPN CLI processing time, not the application code.

## Acceptance Criteria

- [x] `protonvpn disconnect` result determines disconnection status
- [x] No `ip addr` checking for disconnect verification
- [x] "Already disconnected" handled gracefully
- [x] Appropriate user feedback via notification
