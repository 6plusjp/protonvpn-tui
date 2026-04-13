# ProtonVPN CLI Reference

**Current Version**: 1.0.0 (Stable) / **Release Date**: March 23, 2026

## Overview

This document provides a comprehensive reference for the official ProtonVPN CLI (`protonvpn`), based on the [proton-vpn-cli](https://github.com/ProtonVPN/proton-vpn-cli) repository.

**Repository**: https://github.com/ProtonVPN/proton-vpn-cli  
**CLI Entry Point**: `protonvpn`  
**Language**: Python

## Capabilities

- Connect to **19,000+ servers in 139 countries**
- Select servers by country, city, or server ID
- Secure Core, P2P, and Tor server support
- Kill switch, NetShield ad-blocker, port forwarding, Custom DNS, IPv6, VPN Accelerator, NAT type
- WireGuard protocol support

## Supported Distributions

- **Officially Supported**: Debian, Ubuntu, Fedora
- **Community Supported**: Arch Linux (via `pacman -S proton-vpn-cli`)

## Requirements

- gnome-keyring (may work with KDE's KWallet)
- NetworkManager
- **Note**: CLI does NOT work on headless setups

## Limitations

- Cannot run alongside the Proton VPN GUI app
- Headless setups are not currently supported
- Split tunneling is not yet available

---

## Command Summary

| Command                           | Description                                           |
| --------------------------------- | ----------------------------------------------------- | --- |
| `protonvpn signin`                | Sign in with Proton VPN credentials                   |
| `protonvpn signout`               | Disconnect and remove credentials                     |
| `protonvpn info`                  | Display Proton VPN account information                |
| `protonvpn connect`               | Connect to VPN server                                 |
| `protonvpn disconnect`            | Disconnect from VPN                                   |
| `protonvpn status`                | Display connection status                             |
| `protonvpn servers`               | View available servers (prints link to protonvpn.com) |
| `protonvpn countries list`        | Discover available countries                          |
| `protonvpn cities list <COUNTRY>` | Discover available cities for a country               |
| `protonvpn config`                | Configure VPN settings                                |
| `protonvpn config list`           | List all configured settings                          |     |

---

## Account Commands

### signin

Sign in with Proton VPN credentials.

```bash
protonvpn signin USERNAME
```

**Arguments**:

- `USERNAME` (required): Proton account username (e.g., `user@proton.me`)

**Behavior**:

1. Prompts for password (hidden input)
2. If 2FA is enabled, prompts for 2FA token
3. Stores credentials in system keyring

**Example**:

```bash
$ protonvpn signin user@proton.me
Password: 
Successfully signed in as 'user'
```

**Error Cases**:

- Already signed in: `"Already signed in, please sign out first before changing accounts."`
- Authentication failed: `"Authentication failed. Please check your username and password and try again."`
- 2FA failed: `"2FA Authentication failed. Please try again."`

---

### signout

Sign out from Proton VPN and clear local credentials.

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
VPN connection terminated and you've been successfully signed out.

# To sign in again:
protonvpn signin
```

---

### info

Display Proton VPN account information.

```bash
protonvpn info
```

**Output Example**:

```
Account: 'user'
```

---

### status

Display current connection status.

```bash
protonvpn status
```

**Output Example**:

```
Status: Connected
Server: JP#443 in Tokyo, Japan
Load: 6%
Protocol: wireguard
```

**Output Fields**:
| Field | Example | Description |
|-------|---------|-------------|
| `Status` | `Connected` | Connection state (Connected/Disconnected) |
| `Server` | `JP#443 in Tokyo, Japan` | Server ID and location |
| `Load` | `6%` | Server load percentage |
| `Protocol` | `wireguard` | VPN protocol (wireguard/openvpn) |

**With server list update message**:

```
Server list is outdated, updating... This may take a moment.
Status: Connected
Server: JP#443 in Tokyo, Japan
Load: 6%
Protocol: wireguard
```

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
Server list is outdated, updating... This may take a moment.
Connected to JP#318 in Tokyo, Japan.
Your new IP address is 159.26.119.63.

Port forwarding is active on this server.
To get your forwarded port, run the natpmpc setup script.
Guide: https://protonvpn.com/support/port-forwarding-manual-setup#linux
```

**Output Formats by Server Type**:

| Server Type | Output Format                            |
| ----------- | ---------------------------------------- |
| Regular     | `Connected to JP#374 in Tokyo, Japan.`   |
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

Disconnect from current VPN server.

```bash
protonvpn disconnect
```

**Behavior**:

1. Terminates active VPN connection
2. Restores original network configuration

**Example**:

```bash
$ protonvpn disconnect
Disconnected.

# Check connection status:
protonvpn status
```

---

## Discovery Commands

### servers

View available servers. Prints a link to the full server list on protonvpn.com.

```bash
protonvpn servers
```

**Output**:

```
To view detailed server information including specific server IDs, visit:  https://account.proton.me/vpn/WireGuard
```

---

### countries list

List available countries.

```bash
protonvpn countries list
```

**Output Example**:

```
Country                           Code
--------------------------------  ------
Afghanistan                       AF
Albania                           AL
...
Japan                             JP
...
United States                     US
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
Cities in Japan:

City    Features
------  ----------
Osaka   P2P
Tokyo   P2P
```

---

## Configuration Commands

### config

Configure Proton VPN settings. This is a parent command with subcommands.

```bash
protonvpn config <subcommand>
```

#### config list

Show current configuration for all settings.

```bash
protonvpn config list
```

**Output Example**:

```
Current configuration
Setting                  Value
-----------------------  ------------
netshield                malware-only
kill-switch              off
port-forwarding          on
custom-dns               off
vpn-accelerator          on
moderate-nat             off
ipv6                     on
anonymous-crash-reports  off

Use 'protonvpn config set <setting> <value>' to change settings.
Use 'protonvpn config set <setting> --help' for available values.
```

---

#### config set

Change specific settings.

```bash
protonvpn config set <setting> <value>
```

##### VPN Accelerator

```bash
protonvpn config set vpn-accelerator {on|off}
```

Enable or disable VPN Accelerator for improved connection speeds.

**Values**:

- `off` - Disable VPN Accelerator
- `on` - Enable VPN Accelerator (recommended)

**How It Works**:
VPN Accelerator uses advanced technologies to increase VPN speeds by up to 400% without compromising security. It optimizes the connection between your device and Proton VPN servers.

**Recommendation**: Keep enabled for best performance. Only disable if experiencing connection issues with specific networks.

**Usage**:

```bash
protonvpn config set vpn-accelerator on
protonvpn config set vpn-accelerator off
```

---

##### Moderate NAT

```bash
protonvpn config set moderate-nat {on|off}
```

Enable or disable Moderate NAT for improved gaming and P2P performance.

**Values**:

- `off` - Disable Moderate NAT (Strict NAT)
- `on` - Enable Moderate NAT

**How It Works**:
NAT (Network Address Translation) type affects your ability to connect to other players in online games and peers in P2P networks.

- Strict NAT (off): Maximum security, may limit connections in games/P2P
- Moderate NAT (on): Better connectivity for gaming and P2P, still secure

**Usage**:

```bash
protonvpn config set moderate-nat on
protonvpn config set moderate-nat off
```

---

##### Port Forwarding

```bash
protonvpn config set port-forwarding {on|off}
```

Enable or disable port forwarding for P2P applications.

**Values**:

- `off` - Disable port forwarding
- `on` - Enable port forwarding

**How It Works**:
When enabled, the VPN client requests a forwarded port during connection to P2P-capable servers. The port assignment requires an external script to maintain the lease and retrieve the port number. Without the script, the assigned port expires.

**Setup Guide**: https://protonvpn.com/support/port-forwarding-manual-setup#linux

**Usage**:

```bash
protonvpn config set port-forwarding on
protonvpn config set port-forwarding off
```

---

##### IPv6

```bash
protonvpn config set ipv6 {on|off}
```

Enable or disable IPv6 support.

**Values**:

- `off` - Disable IPv6 (default)
- `on` - Enable IPv6

**How It Works**:
When enabled, IPv6 traffic will be routed through the VPN tunnel.

**Usage**:

```bash
protonvpn config set ipv6 on
protonvpn config set ipv6 off
```

---

##### Anonymous Crash Reports

```bash
protonvpn config set anonymous-crash-reports {on|off}
```

Enable or disable anonymous crash reports.

**Values**:

- `off` - Disable crash reporting
- `on` - Enable anonymous crash reporting

**How It Works**:
When enabled, the CLI sends anonymous crash reports to help us:

- Fix bugs and improve stability
- Detect firewall interference
- Avoid VPN blocks

**Privacy**:

- Reports are completely anonymous
- No personal information is collected
- Helps improve the CLI for everyone

**Usage**:

```bash
protonvpn config set anonymous-crash-reports on
protonvpn config set anonymous-crash-reports off
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

## Quick Start

```bash
# 1. Sign in (prompts for password)
protonvpn signin USERNAME

# 2. Connect to fastest server
protonvpn connect

# 3. Disconnect
protonvpn disconnect
```

---

## Global Options

| Option          | Description                                   |
| --------------- | --------------------------------------------- |
| `-v, --verbose` | Show detailed output during command execution |
| `-h, --help`    | Show help message                             |

---

## Server Features

The CLI supports connecting to servers with specific features:

| Feature     | Flag                | Description                      |
| ----------- | ------------------- | -------------------------------- |
| P2P         | `--p2p`             | P2P-optimized servers            |
| Secure Core | `-sc, --securecore` | Secure Core (double VPN) servers |
| Tor         | `--tor`             | Tor-over-VPN servers             |
| Random      | `--random`          | Random server selection          |

---

## File Locations

- **CLI Logs**: `~/.cache/Proton/VPN/logs/`
- **User Settings**: `~/.config/Proton/VPN/`

---

## Limitations

- Cannot run alongside the Proton VPN GUI app
- No headless support
- Split tunneling is not yet available

---

## Release History

| Version   | Date              | Notable Changes                                                                                                 |
| --------- | ----------------- | --------------------------------------------------------------------------------------------------------------- |
| **1.0.0** | March 23, 2026    | First stable release                                                                                            |
| **0.1.8** | March 23, 2026    | Added `protonvpn status` command; improved help/error messages                                                  |
| **0.1.7** | March 2, 2026     | Added `protonvpn config list`; settings changes don't require reconnect                                         |
| **0.1.5** | January 26, 2026  | Added: anonymous crash reports, custom DNS, IPv6, kill switch, NAT, NetShield, port forwarding, VPN accelerator |
| **0.1.4** | January 19, 2026  | P2P/Secure Core/Tor server selection; country/city browsing                                                     |
| **0.1.2** | November 14, 2025 | Initial release                                                                                                 |

---

## References

- Official GitHub: https://github.com/ProtonVPN/proton-vpn-cli
- Official Documentation: https://protonvpn.com/support/linux-cli
- Release Notes: https://protonvpn.com/support/release-notes-linux-cli
- Server List: https://protonvpn.com/vpn-servers
