# ProtonVPN CLI Reference

**Current Version**: 0.1.8 (March 23, 2026) / **Stable**: 1.0.0

## Overview

This document provides a comprehensive reference for the official ProtonVPN CLI (`protonvpn`), based on the [proton-vpn-cli](https://github.com/ProtonVPN/proton-vpn-cli) repository.

**Repository**: https://github.com/ProtonVPN/proton-vpn-cli  
**CLI Entry Point**: `protonvpn`  
**Language**: Python

## Supported Distributions

- **Officially Supported**: Debian, Ubuntu, Fedora
- **Community Supported**: Arch Linux (via `pacman -S proton-vpn-cli`)

## Requirements

- gnome-keyring (may work with KDE's KWallet)
- NetworkManager
- **Note**: CLI does NOT work on headless setups

---

## Command Summary

| Command | Description |
|---------|-------------|
| `protonvpn signin` | Sign in with Proton VPN credentials |
| `protonvpn signout` | Disconnect and remove credentials |
| `protonvpn info` | Display Proton VPN account information |
| `protonvpn connect` | Connect to VPN server |
| `protonvpn disconnect` | Disconnect from VPN |
| `protonvpn status` | Display connection status (v0.1.8+) |
| `protonvpn countries list` | Discover available countries |
| `protonvpn cities list <COUNTRY>` | Discover available cities for a country |
| `protonvpn config` | Configure VPN settings |
| `protonvpn config list` | List all configured settings (v0.1.7+) |

---

## Account Commands

### signin

Sign in with Proton VPN credentials.

```bash
protonvpn signin [username]
```

**Arguments**:
- `username` (required): Your Proton VPN username

**Behavior**:
1. Prompts for password (hidden input)
2. If 2FA is enabled, prompts for 2FA token
3. Stores credentials in system keyring

**Example**:
```bash
$ protonvpn signin myuser
Password: 
# If 2FA enabled:
2FA Token: 123456
```

**Error Cases**:
- Already signed in: `"Already signed in, please sign out first before changing accounts."`
- Authentication failed: `"Authentication failed. Please check your username and password and try again."`
- 2FA failed: `"2FA Authentication failed. Please try again."`

---

### signout

Disconnect VPN and remove stored credentials.

```bash
protonvpn signout
```

**Behavior**:
1. Disconnects if currently connected
2. Removes credentials from keyring
3. Resets most settings to defaults

**Example**:
```bash
$ protonvpn signout
# No output on success
```

---

### info

Display Proton VPN account information.

```bash
protonvpn info
```

**Output Example**:
```
Proton VPN Free
Username: myuser
Email: user@example.com
Plan: Plus
Status: Active
```

---

### status

Display current connection status. **(Added in v0.1.8)**

```bash
protonvpn status
```

**Output Format v1 (Current CLI)**:
```
Status: Connected
Server: JP#443 in Tokyo, Japan
Load: 15%
Protocol: wireguard
```

**Output Format v2 (Older CLI / Tests)**:
```
Status: Connected
Server: JP#374
Country: Japan
City: Tokyo
IP: 159.26.119.144
Uptime: 00:15:32
```

**With server list update message**:
```
Server list is outdated, updating... This may take a moment.
Status: Connected
Server: JP#443 in Tokyo, Japan
Load: 15%
Protocol: wireguard
```

**Output Fields**:
| Field | Example | Description | Format |
|-------|---------|-------------|--------|
| `Status` | `Connected` | Connection state | Both |
| `Server` | `JP#443 in Tokyo, Japan` | Server ID and location | v1 |
| `Server` | `JP#374` | Server ID only | v2 |
| `Country` | `Japan` | Country name | v2 only |
| `City` | `Tokyo` | City name | v2 only |
| `IP` | `159.26.119.144` | IP address | v2 only |
| `Load` | `15%` | Server load percentage | v1 only |
| `Protocol` | `wireguard` | VPN protocol (lowercase) | v1 only |
| `Uptime` | `00:15:32` | Connection time (HH:MM:SS) | v2 only |
| `Time` | `1:23:45` | Alternative uptime field | v2 variant |

**Note**: 
- Format varies by CLI version
- `Uptime`/`Time` fields are NOT present in current CLI versions (v0.1.8+)
- The TUI parses `Uptime:` and `Time:` fields if present (see `parse_status_uptime()`)

**Behavior**:
- Shows connection details if connected
- May show server list update message before status
- Returns nothing meaningful if not connected

---

## Connection Commands

### connect

Connect to Proton VPN server.

```bash
protonvpn connect [server_name]
```

**Arguments**:
- `server_name` (optional): Specific server ID (e.g., `CH#242`)

**Options**:
| Option | Description |
|--------|-------------|
| `--country CODE` | Connect to fastest server in specified country (country code or full name) |
| `--city NAME` | Connect to fastest server in specified city |
| `--p2p` | Connect to fastest P2P-optimized server |
| `-sc, --securecore` | Connect to fastest Secure Core server |
| `--tor` | Connect to fastest Tor server |
| `--random` | Connect to a random available server |

**Usage Examples**:

```bash
# Connect to fastest available server
protonvpn connect

# Connect to specific country
protonvpn connect --country US
protonvpn connect --country "United States"

# Connect to specific city
protonvpn connect --city miami
protonvpn connect --city "New York"

# Connect to specific server
protonvpn connect CH#242

# Connect to P2P-optimized server
protonvpn connect --p2p

# Connect to Secure Core server
protonvpn connect --securecore

# Connect to Tor server
protonvpn connect --tor

# Connect to random server
protonvpn connect --random
```

**Success Output Example**:
```
Connected to US#45 in Los Angeles, United States.
Your new IP address is 123.45.67.89.
```

**Output Formats by Server Type**:

| Server Type | Output Format |
|------------|---------------|
| Regular | `Connected to JP#374 in Tokyo, Japan.` |
| Secure Core | `Connected to CH#123 in Zurich, via CH.` |

The Secure Core format uses `" via "` to indicate the exit country.

**Warning** (when using OpenVPN):
```
OpenVPN is not fully supported in CLI and you may experience instability. 
For best results, use WireGuard.
```

**Error Cases**:
- Not authenticated: `"Authentication required. Please sign in with 'protonvpn signin' before connecting."`
- Invalid server: `"Invalid server ID 'XX#999'. Please use a valid server ID from the server list."`
- Invalid country: `"Invalid country code 'XX'. Please use a valid country code."`
- Free tier limitation: `"Server selection by ID is not available on the free plan."`
- No servers found: `"No servers found matching criteria. Try broadening your filters."`

---

### disconnect

Disconnect from Proton VPN.

```bash
protonvpn disconnect
```

**Behavior**:
1. Terminates VPN connection
2. Waits for post-disconnect tasks (kill switch)

**Example**:
```bash
$ protonvpn disconnect
# No output on success
```

---

## Discovery Commands

### countries list

List available countries.

```bash
protonvpn countries list
```

**Output Example**:
```
Server list is outdated, updating... This may take a moment.
Country                 Code
----------------------  ------
United States           US
United Kingdom          GB
Germany                 DE
Japan                   JP
...
```

---

### cities list

List cities in a specific country.

```bash
protonvpn cities list <country_input>
```

**Arguments**:
- `country_input` (required): Country code (e.g., `US`) or country name

**Usage Examples**:
```bash
# List cities in Portugal
protonvpn cities list PT

# List cities in United States
protonvpn cities list US
```

**Output Example**:
```
Cities in United States:

City           Features
Los Angeles    P2P, Secure Core
New York       P2P
Miami          P2P, Tor
Seattle        Secure Core
...
```

---

## Configuration Commands

### config

Configure Proton VPN settings. This is a parent command with subcommands.

```bash
protonvpn config <subcommand>
```

#### config list

Show current configuration for all settings. **(Added in v0.1.7)**

```bash
protonvpn config list
```

**Output Example**:
```
Current configuration
Setting                  Value
vpn-accelerator         off
moderate-nat            off
ipv6                    off
anonymous-crash-reports off
port-forwarding         off
custom-dns              off
kill-switch             off
netshield               off
```

---

#### config set

Change specific settings.

```bash
protonvpn config set <setting> <value>
```

##### Boolean Settings

| Setting | Description | Free Tier |
|---------|-------------|-----------|
| `vpn-accelerator` | VPN Accelerator | No |
| `moderate-nat` | Moderate NAT | No |
| `ipv6` | IPv6 support | Yes |
| `anonymous-crash-reports` | Anonymous crash reports | Yes |
| `port-forwarding` | Port forwarding | No |

**Usage**:
```bash
# Enable
protonvpn config set vpn-accelerator on
protonvpn config set moderate-nat on
protonvpn config set ipv6 on
protonvpn config set anonymous-crash-reports on
protonvpn config set port-forwarding on

# Disable
protonvpn config set vpn-accelerator off
protonvpn config set moderate-nat off
protonvpn config set ipv6 off
protonvpn config set anonymous-crash-reports off
protonvpn config set port-forwarding off
```

---

##### Kill Switch

```bash
protonvpn config set kill-switch <mode>
```

**Modes**:
- `off` - Disabled
- `standard` - Standard kill switch (blocks internet only while VPN is active)

**Usage**:
```bash
protonvpn config set kill-switch standard
protonvpn config set kill-switch off
```

**Note**: Unlike other boolean settings, kill-switch uses `standard` instead of `on`.

---

##### NetShield

```bash
protonvpn config set netshield <mode>
```

**Modes** (subscription required):
- `off` - Disabled
- `malware-only` - Block malware only
- `malware-ads-trackers` - Block malware, ads, and trackers

**Usage**:
```bash
protonvpn config set netshield off
protonvpn config set netshield malware-only
protonvpn config set netshield malware-ads-trackers
```

---

##### Custom DNS

```bash
protonvpn config set custom-dns <state> --dns <dns_servers>
```

**Arguments**:
- `state`: `on` or `off`
- `--dns`: Comma-separated DNS server IPs (required when enabling)

**Usage**:
```bash
# Enable with DNS servers
protonvpn config set custom-dns on --dns 1.1.1.1,9.9.9.9

# Disable
protonvpn config set custom-dns off
```

**Error Cases**:
- Missing DNS: `"When enabling Custom DNS feature you must provide a list of comma separated DNS's."`
- Invalid DNS: `"Invalid DNS address 'x.x.x.x'. Please provide a valid IPv4 address."`

---

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Show detailed output during command execution |
| `-h, --help` | Show help message |

---

## Server Features

The CLI supports connecting to servers with specific features:

| Feature | Flag | Description |
|---------|------|-------------|
| P2P | `--p2p` | P2P-optimized servers |
| Secure Core | `-sc, --securecore` | Secure Core (double VPN) servers |
| Tor | `--tor` | Tor-over-VPN servers |
| Random | `--random` | Random server selection |

---

## File Locations

- **CLI Logs**: `~/.cache/Proton/VPN/logs/`
- **User Settings**: `~/.config/Proton/VPN/`

---

## Limitations (as of v0.1.8)

- Cannot run alongside Proton VPN GUI app
- No server list command (use connection options or `protonvpn countries list` / `protonvpn cities list` instead)
- No headless support

---

## Release History

| Version | Date | Notable Changes |
|---------|------|-----------------|
| **1.0.0** | March 23, 2026 | First stable release |
| **0.1.8** | March 23, 2026 | Added `protonvpn status` command; improved help/error messages |
| **0.1.7** | March 2, 2026 | Added `protonvpn config list`; settings changes don't require reconnect |
| **0.1.5** | January 26, 2026 | Added: anonymous crash reports, custom DNS, IPv6, kill switch, NAT, NetShield, port forwarding, VPN accelerator |
| **0.1.4** | January 19, 2026 | P2P/Secure Core/Tor server selection; country/city browsing |
| **0.1.2** | November 14, 2025 | Initial release |

---

## References

- Official GitHub: https://github.com/ProtonVPN/proton-vpn-cli
- Official Documentation: https://protonvpn.com/support/linux-cli
- Release Notes: https://protonvpn.com/support/release-notes-linux-cli
- Server List: https://protonvpn.com/vpn-servers
