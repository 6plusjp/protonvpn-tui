# issue001: Settings view - Enter key triggers config toggle via `protonvpn config set`

## Summary

Toggle VPN settings directly from the Settings view by pressing Enter, which invokes `protonvpn config set`.

## Problem

Currently, the Settings view only displays configuration options but lacks interactive controls. Users must use the CLI to change settings like kill-switch, NetShield, etc.

## Solution

Implement `protonvpn config set` integration:

1. Add command functions to execute `protonvpn config set <option> <value>` via the VPN client
2. Bind Enter key in Settings view to toggle the selected setting
3. Provide visual feedback via notification

## Technical Notes

### Already Implemented
- `protonvpn config set` toggles: kill-switch, ipv6, moderate-nat, vpn-accelerator, port-forwarding
- NetShield cycles through: off → malware-only → malware-ads-trackers → off
- Custom DNS (index 2) - requires additional configuration

### Custom DNS Implementation Options

`protonvpn config set custom-dns` requires IP addresses:
```bash
protonvpn config set custom-dns on --dns 1.1.1.1,9.9.9.9
protonvpn config set custom-dns off
```

| Option | Description | Complexity |
|--------|-------------|------------|
| **A) Toggle only** | Toggle on/off with current DNS servers (read from config) | Simple |
| **B) Prompt input** | Ask user to enter DNS IPs via TUI input | Medium |
| **C) Preset list** | Choose from predefined DNS presets (Cloudflare, Google, etc.) | Medium |

### Option A Details (Recommended)
- Read current DNS IPs from `~/.config/Proton/VPN/settings.json`
- Toggle on/off using those IPs
- No additional UI required

### Option B Details
- Add TUI text input component
- User enters DNS IPs when enabling
- Requires new input view/state

### Option C Details
- Add DNS preset selection (e.g., Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9)
- Show preset list on Enter key in DNS row
- Requires selection list UI

## Implementation Findings (2026-03-06)

### Current State Analysis

**Already implemented (verified in code):**
- `VpnClient::toggle_*` methods in `src/vpn/client.rs` (lines 493-534)
- `AppState::toggle_settings()` in `src/state/app_state.rs` (lines 705-784)
- Enter key handler in `src/ui/app.rs` (lines 140-152)
- Notification feedback on success/error
- Cache invalidation via `OnceLock::new()`

**Missing UX elements:**
- No keyboard hints in Settings footer (only shows "settings")

### Design Decisions

1. **Input-box (filter) styling**: Filter input at bottom uses `theme.key_hint` for border - this is consistent with the design system

2. **Settings footer**: Add keyboard hints following other views pattern:
   - `[j/k]` - move
   - `[Enter]` - toggle / input (for DNS)
   - `[Space]` - turn off (for DNS)

3. **Custom DNS**: Implemented Option B (Prompt input)
   - DNS enabled + Enter → input prompt for new IPs
   - DNS enabled + Space → disable DNS directly
   - DNS disabled + Enter → input prompt for new IPs
   - DNS disabled + Space → show "already off"

### Implementation Details (2026-03-06)

**Added:**
- `InputMode` enum in `src/state/app_state.rs` (Normal, Filter, DnsInput)
- `dns_input` field to store DNS IPs during input
- `render_dns_input()` in `src/ui/app.rs`
- `apply_dns_setting()` method in AppState
- `set_custom_dns()` / `disable_custom_dns()` in VpnClient/VpnState
- `toggle_settings_off()` for Space key (turns off DNS directly)
- Fixed `ProtonCustomDns.ip_list` structure to match actual JSON format

**Keyboard controls:**
- `Enter` in Settings: Toggle (for DNS: always show input prompt)
- `Space` in Settings: Turn off (for DNS: disable directly)
- `c`: Connect (removed from Settings)

## Acceptance Criteria

- [x] `protonvpn config set` commands implemented in VPN client (basic toggles)
- [x] Pressing Enter in Settings view triggers the toggle action
- [x] User receives feedback via notification (success/error)
- [x] Settings cache invalidates after change
- [x] Custom DNS toggle implemented (Option B: Prompt input)
- [x] Keyboard hints added to Settings footer
