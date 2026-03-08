# issue021: Feature - Server list fallback for first run / cache miss

## Summary

Add a hardcoded fallback country list for when ProtonVPN CLI is unavailable, ensuring the app remains usable.

## Problem

On first run (or after cache clear), the app fetches server data from `protonvpn` CLI. If the CLI:
- Is not installed
- Is not configured
- Fails for any reason

The user sees an empty server list and cannot use the VPN functionality at all.

## Solution

### 1. Fallback Countries (countries only)

Hardcode all countries as fallback data:

```rust
// src/vpn/cache.rs

/// Fallback countries data - used when CLI is unavailable
/// Format: country code → country name
pub const FALLBACK_COUNTRIES: &[(&str, &str)] = &[
    ("AF", "Afghanistan"),
    ("AL", "Albania"),
    ("DZ", "Algeria"),
    // ... (all ~190 countries from `protonvpn countries`)
];
```

**Rationale:**
- Countries rarely change (no new countries added frequently)
- This is just for display; VPN connection still requires CLI
- All countries included (not just popular ones) for completeness

### 2. Cities NOT included in fallback

Cities are NOT included in fallback because:
- They have features that vary per server (P2P, Secure Core, etc.)
- They change more frequently than countries
- Users must explicitly select a country to fetch cities anyway

Users will fetch cities on-demand via CLI when they select a country.

### 3. CLI Failure Handling

When `protonvpn countries` fails:

```
┌─────────────────────────────────────────────────────┐
│ Countries              Cities                       │
├─────────────────────────┼───────────────────────────┤
│ ★ Japan                │ Tokyo                      │
│   United States        │                            │
│   Germany              │                            │
│   ...                  │                            │
├─────────────────────────┴───────────────────────────┤
│ ⚠ ProtonVPN CLI unavailable.                       │
│   VPN functionality disabled.                       │
│   Please check ProtonVPN CLI installation.         │
└─────────────────────────────────────────────────────┘
```

Error message: "ProtonVPN CLI unavailable. VPN functionality disabled. Please check ProtonVPN CLI installation."

**Rationale:** This TUI is a wrapper for the CLI - without CLI, VPN cannot function. Clear error message is essential.

### 4. Implementation

#### src/vpn/cache.rs
- Add `FALLBACK_COUNTRIES` constant with all ~190 countries

#### src/vpn/client.rs
- Modify `refresh_countries()` to:
  1. Try CLI first
  2. On success: parse and cache result
  3. On failure: return fallback with error indicator

#### src/state/app_state.rs  
- Handle CLI failure in refresh flow
- Show error notification when using fallback

### 5. Data Source

Countries extracted from `protonvpn countries` output (see below):

```
Country                 Code
----------------------  ------
Afghanistan             AF
Albania                 AL
...
Japan                   JP
United States           US
...
```

---

## Acceptance Criteria

- [x] `FALLBACK_COUNTRIES` constant contains all ~190 countries
- [x] Fallback used when CLI fails or is unavailable
- [x] Clear error notification: "ProtonVPN CLI unavailable. VPN functionality disabled."
- [x] Cities remain empty in fallback mode (fetched on-demand)
- [x] Fallback NOT cached (CLI retried on next startup)

---

## Implementation Notes

### Files Changed

- `src/vpn/cache.rs` - Added `FALLBACK_COUNTRIES` (190 countries) and `cli_unavailable` flag
- `src/vpn/client.rs` - Modified `refresh_countries()` to use fallback on CLI failure
- `src/vpn/state.rs` - Added `is_cli_unavailable()` method
- `src/state/app_state.rs` - Added error notification when CLI is unavailable
