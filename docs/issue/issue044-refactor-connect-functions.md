# issue044 - Refactor connect_country and connect_city to use connect_with_args

## Summary

Refactor `connect_country` and `connect_city` to use `connect_with_args`.

## Current State

`src/vpn/client.rs`:
- `connect_fastest()` - ✓ uses `connect_with_args`
- `connect_random()` - ✓ uses `connect_with_args`
- `connect_p2p()` - ✓ uses `connect_with_args`
- `connect_tor()` - ✓ uses `connect_with_args`
- `connect_securecore()` - ✓ uses `connect_with_args`
- `connect_country(target)` - direct implementation, requires argument
- `connect_city(city_arg)` - direct implementation, requires argument

## Problem

`connect_country` and `connect_city` require arguments:
- `protonvpn connect --country US` - needs country argument
- `protonvpn connect --city Tokyo` - needs city argument

Current `connect_with_args` only supports a single flag:
```rust
fn connect_with_args(&self, flag: &str, fallback_name: &str) -> AppResult<ConnectResult>
```

## Proposal

Extend `connect_with_args` to also accept optional arguments:

```rust
fn connect_with_args(&self, flag: &str, arg: Option<&str>, fallback_name: &str) -> AppResult<ConnectResult>
```

Or:

```rust
fn connect_with_args(&self, flag: &str, fallback_name: &str, args: &[&str]) -> AppResult<ConnectResult>
```

## Related

- resolved040 - Add keyboard shortcuts
