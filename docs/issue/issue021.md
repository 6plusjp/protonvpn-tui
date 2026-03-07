# issue021: Feature - Server list fallback for first run / cache miss

## Summary

Add a hardcoded fallback server list for initial app startup when no cached data exists.

## Problem

On first run (or after cache clear), the app has no server data. Currently:
1. App fetches server list from `protonvpn` CLI
2. This can be slow or fail if CLI isn't configured
3. User sees empty/incomplete UI until data loads

## Solution

Hardcode a minimal server list as fallback:

```rust
// src/vpn/server_list.rs or similar

pub const FALLBACK_SERVERS: &[Server] = &[
    // Popular servers by country
    Server { id: "US", name: "United States", cities: vec![
        City { id: "US-NY", name: "New York", ... },
        City { id: "US-LA", name: "Los Angeles", ... },
        City { id: "US-CH", name: "Chicago", ... },
    ]},
    Server { id: "JP", name: "Japan", cities: vec![
        City { id: "JP-TK", name: "Tokyo", ... },
        City { id: "JP-OS", name: "Osaka", ... },
    ]},
    Server { id: "DE", name: "Germany", cities: vec![
        City { id: "DE-BE", name: "Berlin", ... },
        City { id: "DE-FR", name: "Frankfurt", ... },
    ]},
    Server { id: "GB", name: "United Kingdom", cities: vec![
        City { id: "GB-LN", name: "London", ... },
    ]},
    Server { id: "CH", name: "Switzerland", cities: vec![
        City { id: "CH-ZH", name: "Zurich", ... },
    ]},
    Server { id: "NL", name: "Netherlands", cities: vec![
        City { id: "NL-AM", name: "Amsterdam", ... },
    ]},
    Server { id: "SG", name: "Singapore", cities: vec![
        City { id: "SG-SG", name: "Singapore", ... },
    ]},
    Server { id: "AU", name: "Australia", cities: vec![
        City { id: "AU-SY", name: "Sydney", ... },
    ]},
    Server { id: "CA", name: "Canada", cities: vec![
        City { id: "CA-TO", name: "Toronto", ... },
    ]},
    Server { id: "FR", name: "France", cities: vec![
        City { id: "FR-PA", name: "Paris", ... },
    ]},
];
```

## Usage

1. On startup, check if cache exists
2. If no cache, load fallback servers immediately (instant UI population)
3. In background, fetch fresh data from `protonvpn` CLI
4. Once fetched, replace fallback with real data

## Benefits

- **Instant UI**: User sees server list immediately
- **Offline-friendly**: Works without ProtonVPN CLI
- **Better UX**: Shows popular servers first
- **Graceful degradation**: Falls back when API fails

## Data Source

Can extract from:
- `protonvpn servers -i` output
- [ProtonVPN server list](https://protonvpn.com/servers) (manual copy)

---

## Acceptance Criteria

- [ ] Fallback server list exists for top 10+ countries
- [ ] App loads fallback on first run / cache miss
- [ ] Background fetch replaces fallback with real data
- [ ] No UI flash or jarring transitions
- [ ] Works offline (shows fallback only)
