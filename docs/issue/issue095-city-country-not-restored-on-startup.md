# Issue 095: city/country Not Restored on Startup

## Summary

After reboot with VPN connected, the header displays server name and IP but not city/country/via (for Secure Core) because the CLI's persistence file doesn't contain this information.

## Problem Description

When the app starts with an existing VPN connection, `sync_connection_state()` reads from `connection_persistence.json` which only contains:
- `server_name` (e.g., "JP#374")
- `server_ip` (e.g., "159.26.119.144")
- `protocol` (e.g., "wireguard")

Missing:
- `city` (e.g., "Tokyo")
- `country` (e.g., "Japan")
- `via` (e.g., "Switzerland" for Secure Core)

### Header Display After Startup

| Phase | Server | IP | Location (city/country/via) |
|-------|--------|-----|----------------------------|
| After connect | CH-JP#2 | 37.19.x.x | Tokyo, via Switzerland |
| After reboot | CH-JP#2 | 37.19.x.x | **Not displayed** |

## Root Cause

The CLI's `connection_persistence.json` is managed by Proton's Python backend (`python-proton-vpn-api-core`) and only stores minimal connection info:

```python
@dataclass
class VPNServer:
    server_ip: str
    server_id: str          # e.g., "CH#10"
    server_name: str        # e.g., "CH#10"
    domain: str
    x25519pk: str
    openvpn_ports: ProtocolPorts
    wireguard_ports: ProtocolPorts
    has_ipv6_support: bool
    label: str = None
```

The `city`, `country`, `via` fields are not included.

## Investigation: Official ProtonVPN GTK App

The official GTK app faces the same limitation. It solves this by:

1. Storing `server_id` in persistence
2. Looking up `LogicalServer` from cached server list (fetched from Proton API)
3. Getting `city`, `exit_country`, `entry_country` from the lookup

```python
class LogicalServer:
    @property
    def city(self) -> str:              # "Tokyo"
    @property
    def exit_country(self) -> str:      # "JP"
    @property
    def exit_country_name(self) -> str: # "Japan"
    @property
    def entry_country(self) -> str:     # "CH" (for Secure Core)
    @property
    def entry_country_name(self) -> str: # "Switzerland"
```

### Comparison

| Aspect | GTK App | protonvpn-tui |
|--------|---------|---------------|
| Server list source | Proton API (LogicalServer) | CLI (`protonvpn countries/cities`) |
| Server list data | Full (city, country, features) | Partial (country, city names) |
| server_id lookup | ✅ Supported | ❌ Not implemented |
| city/country on reboot | Restored from server list | Not restored |

## CLI Status Command (v0.1.8+)

`protonvpn status` outputs location information:

```
Server: JP#374
Location: Tokyo, Japan
Protocol: WireGuard
Uptime: 00:15:32
```

This provides the missing `city` and `country` information.

## Proposed Solution

### Use `protonvpn status` to restore location (Recommended)

On startup, if VPN is connected, call `protonvpn status` and parse the location.

**Implementation:**

```rust
// Add to types.rs
pub struct StatusInfo {
    pub server: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub protocol: Option<String>,
    pub uptime: Option<Duration>,
}

pub fn parse_status_output(output: &str) -> Option<StatusInfo> {
    let mut server = None;
    let mut city = None;
    let mut country = None;
    let mut protocol = None;
    let mut uptime = None;

    for line in output.lines() {
        if let Some(val) = line.strip_prefix("Server:") {
            server = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("Location:") {
            // Parse "Tokyo, Japan" format
            let parts: Vec<&str> = val.trim().splitn(2, ',').collect();
            if parts.len() == 2 {
                city = Some(parts[0].trim().to_string());
                country = Some(parts[1].trim().to_string());
            } else {
                country = Some(val.trim().to_string());
            }
        } else if let Some(val) = line.strip_prefix("Protocol:") {
            protocol = Some(val.trim().to_string());
        } else if let Some(val) = line.strip_prefix("Uptime:") {
            // Parse "00:15:32" format
            uptime = parse_uptime(val.trim());
        }
    }

    server.map(|s| StatusInfo { server: s, city, country, protocol, uptime })
}
```

**Update `sync_connection_state()`:**

```rust
fn sync_connection_state(&mut self) -> bool {
    if self.connection_manager.connection.is_connected() {
        return false;
    }
    if self.connection_manager.connection == ConnectionState::Disconnected
        && self.vpn_state.is_connected()
    {
        // Try to get full info from protonvpn status
        let (server, ip, city, country, via, connected_at) =
            match self.vpn_state.get_status_info() {
                Some(status) => {
                    let connected_at = status.uptime.map(|u| {
                        Utc::now() - chrono::Duration::from_std(u).unwrap_or_default()
                    });
                    // Get IP from persistence file
                    let (_, ip) = self.vpn_state.get_connected_server_info()
                        .unwrap_or_default();
                    (status.server, ip, status.city, status.country, None, connected_at)
                }
                None => {
                    // Fallback to persistence file
                    let (server, ip) = self.vpn_state.get_connected_server_info()
                        .unwrap_or_else(|| ("Unknown".to_string(), String::new()));
                    (server, ip, None, None, None, None)
                }
            };

        // Update cache
        self.vpn_state.sync_cache_with_connection(
            &server, &ip, city.as_deref(), country.as_deref(), connected_at
        );

        self.connection_manager.connection = ConnectionState::Connected {
            server,
            ip,
            city,
            country,
            via,
        };
        return true;
    }
    false
}
```

**Benefits:**
- Uses authoritative CLI data
- Gets accurate city/country from `protonvpn status`
- Gets accurate uptime for session time
- No need to store city/country in cache

**Limitations:**
- `via` (entry country for Secure Core) is not in status output
- Requires CLI call on startup

### Fallback: Extract country from server_id

If CLI is unavailable, extract country code from server_id:

```rust
fn extract_country_code(server_id: &str) -> Option<String> {
    // "JP#374" → "JP", "CH-JP#2" → "CH" (entry country for Secure Core)
    server_id.split(|c| c == '#' || c == '-')
        .next()
        .map(|s| s.to_string())
}
```

**Pros**: Simple, no external dependencies
**Cons**: Only country code, no city, no entry country name

## Verification Steps

1. Connect to VPN (city/country shown in header)
2. Reboot computer
3. Launch ProtonVPN TUI
4. Verify city/country is displayed from `protonvpn status`

## Dependencies

- Issue 093: Session time fix
- Issue 094: Cache sync fix
- ProtonVPN CLI v0.1.8+ (for `protonvpn status` command)

## Affected Files

| File | Role |
|------|------|
| `src/vpn/cache.rs` | Add new fields |
| `src/vpn/client.rs` | Update `set_connected()` |
| `src/vpn/types.rs` | `parse_connect_output()` already extracts city/country |
| `src/ui/renderers/header.rs` | Use restored city/country |

## Verification Steps

1. Connect to VPN (city/country shown in header)
2. Reboot computer
3. Launch ProtonVPN TUI
4. Verify city/country is displayed (if implemented)

## Dependencies

- Issue 093: Session time fix
- Issue 094: Cache sync fix

## Priority

Low - cosmetic issue, functionality works

## Related

- Issue 093: Session time persists across computer reboot
- Issue 094: Cache not synced with actual connection on startup
- `src/vpn/cache.rs` - ServerCache struct
- `src/vpn/types.rs` - parse_connect_output()
- `src/ui/renderers/header.rs` - Header rendering
- ProtonVPN GTK App: https://github.com/ProtonVPN/proton-vpn-gtk-app
- ProtonVPN Core: https://github.com/ProtonVPN/python-proton-vpn-api-core
