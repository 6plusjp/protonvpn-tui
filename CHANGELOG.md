# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-03-21

### Added

- Initial release of ProtonVPN TUI
- Interactive terminal UI for Proton VPN using ratatui and crossterm
- Server browsing with country → city hierarchy
- Fuzzy search for servers
- Multiple connection types:
  - Fastest server
  - Specific country or city
  - Random server
  - P2P-optimized server
  - Tor server
  - Secure Core server
- VPN settings management:
  - Kill Switch
  - IPv6 protection
  - Moderate NAT
  - VPN Accelerator
  - Port Forwarding
  - NetShield (malware/ads/trackers blocking)
  - Custom DNS servers
  - Mask IP (hide IP address in header)
- Connection history and notification logs
- Multiple theme support:
  - System (terminal colors)
  - Catppuccin Mocha (dark)
  - Catppuccin Latte (light)
  - Dracula
  - Nord
  - Gruvbox
  - Tokyo Night
- Vim-style keybindings and navigation
- Configuration file support (`~/.config/protonvpn-tui/config.toml`)
