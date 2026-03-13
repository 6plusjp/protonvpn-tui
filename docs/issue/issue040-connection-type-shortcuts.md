# Issue 039: Connection Type Shortcuts & Improved Sorting UI

## Summary

Add keyboard shortcuts for quick connection types (fastest, P2P, Tor, SecureCore) and implement improved sorting UI with number keys and arrow keys.

## Research Findings

### 1. Keyboard Shortcuts for Connection Types

**Current Implementation** (`src/config/settings.rs:240-258`):
- `c` - Connect to selected server
- `d` - Disconnect
- `r` - Refresh server list
- `x` - Random connect (equivalent to `--random`)
- `s` - Cycle sort (toggle sort direction)

**Missing Shortcuts** (requested):
| Key | Connection Type | CLI Flag |
|-----|----------------|----------|
| `s` | SecureCore | `-sc` or `--securecore` |
| `f` | Fastest | `--fastest` or `-f` |
| `p` | P2P/Torrent | `--p2p` |
| `t` | Tor | `--tor` |

**ProtonVPN CLI Support** (`src/vpn/client.rs`):
- Currently only implements:
  - `connect_country()` - `protonvpn connect --country <code>`
  - `connect_random()` - `protonvpn connect --random`
  - `connect_city()` - `protonvpn connect --city <name>`

---

### 2. Connection Persistence (`connection_persistence.json`)

**Current State**:
- App stores cache at `~/.cache/protonvpn-tui/server_cache.toml` (custom path)
- Uses `~/.config/Proton/VPN/settings.json` for Proton settings (read-only)
- NO interaction with `~/.cache/Proton/VPN/connection_persistence.json`
- **On startup, if VPN is connected externally, shows "Unknown" as server** (`src/state/app_state.rs:580`)

**ProtonVPN CLI Persistence** (`~/.cache/Proton/VPN/connection/connection_persistence.json`):
```json
{
  "server": {
    "server_name": "JP#422",
    "server_ip": "212.102.51.122",
    "protocol": "wireguard"
  }
}
```

**Proposed Changes**:
1. Add function to read `connection_persistence.json`
2. Parse `server.server_name` field
3. Use parsed server name instead of "Unknown" on startup

**Implementation Location** (`src/state/app_state.rs:577-586`):
```rust
// Current:
if self.connection_manager.connection == ConnectionState::Disconnected
    && self.vpn_state.is_connected()
{
    self.connection_manager.connection = ConnectionState::Connected {
        server: "Unknown".to_string(),  // <-- CHANGE THIS
        ip: String::new(),
        city: None,
        country: None,
    };
}

// New:
if self.connection_manager.connection == ConnectionState::Disconnected
    && self.vpn_state.is_connected()
{
    let server_name = self.vpn_state.get_connected_server_name()
        .unwrap_or_else(|| "Unknown".to_string());
    self.connection_manager.connection = ConnectionState::Connected {
        server: server_name,
        ip: String::new(),
        city: None,
        country: None,
    };
}
```

---

### 3. Sorting UI Improvements (NEW)

**Current Implementation**:
- `s` key: Toggle sort direction (Asc ↔ Desc)
- `f` key: Cycle sort field (Code → Country → Code)
- No visual indicator for sort field numbers

**Proposed Changes**:
- Header display: `Code¹ Country²` (superscript numbers)
- `1` key: Sort by Code (configurable)
- `2` key: Sort by Country (configurable)
- `←` / `→` keys: Toggle sort direction (fixed, not configurable)
- `s` key: Reassigned to SecureCore connection

**KeyBindings Changes** (`src/config/settings.rs`):
```rust
// Current:
cycle_sort: KeyBinding::new('s', KeyModifier::None),      // REMOVE
cycle_sort_field: KeyBinding::new('f', KeyModifier::None), // REMOVE

// New:
sort_by_code: KeyBinding::new('1', KeyModifier::None),      // ADD
sort_by_country: KeyBinding::new('2', KeyModifier::None),  // ADD
securecore: KeyBinding::new('s', KeyModifier::None),        // ADD (replaces cycle_sort)
connect_fastest: KeyBinding::new('f', KeyModifier::None),  // ADD (replaces cycle_sort_field)
connect_p2p: KeyBinding::new('p', KeyModifier::None),      // ADD
connect_tor: KeyBinding::new('t', KeyModifier::None),     // ADD
```

**Table Header Display** (`src/ui/components/pane_table.rs`):
- Add method to render sort field with superscript numbers
- Highlight active sort field
- Show direction indicator (↑/↓)

---

## Implementation Plan

### Phase 1: Sorting UI Improvements
- [ ] Update `KeyBindings` struct - remove `cycle_sort`, `cycle_sort_field`, add `sort_by_code`, `sort_by_country`
- [ ] Add connection type keys: `securecore`, `connect_fastest`, `connect_p2p`, `connect_tor`
- [ ] Add key event handlers for `1`, `2`, `ArrowLeft`, `ArrowRight`
- [ ] Update table header rendering to show `Code¹ Country²` with indicators
- [ ] Update help view

### Phase 2: Connection Type Shortcuts
- [ ] Add `VpnClient` methods:
  - `connect_fastest()` → `protonvpn connect --fastest`
  - `connect_p2p()` → `protonvpn connect --p2p`
  - `connect_tor()` → `protonvpn connect --tor`
  - `connect_securecore()` → `protonvpn connect --securecore`
- [ ] Add async task variants in `async_tasks.rs`
- [ ] Wire up key handlers

### Phase 3: Connection Persistence (Previous Server)
- [ ] Add function to read `~/.cache/Proton/VPN/connection/connection_persistence.json`
- [ ] Parse `server.server_name` field
- [ ] Add `get_connected_server_name()` method to `VpnClient`
- [ ] Update startup logic in `check_pending_async_events()` to use parsed server name

### Phase 4: Auto-reconnect (Future - Optional)
- [ ] Read `connection_persistence.json` on startup
- [ ] Implement auto-reconnect option

---

## Related Files

- `src/config/settings.rs` - Key binding config
- `src/ui/app.rs` - Key event handling
- `src/ui/components/pane_table.rs` - Table header rendering
- `src/ui/views/help_view.rs` - Help display
- `src/vpn/client.rs` - VPN CLI wrapper, read persistence file
- `src/vpn/async_tasks.rs` - Async job definitions
- `src/vpn/types.rs` - Add persistence data types
- `src/state/app_state.rs` - App state management, startup connection check
- `src/state/server_sort.rs` - Sort state definitions
- `src/state/ui_state.rs` - UI state

---

## Notes

- SecureCore and Tor require Plus/Professional plan (check for 401 error)
- Number keys (1, 2) are configurable via KeyBindings
- Arrow keys (←, →) are fixed, not configurable
- Current `f` key for cycle_sort_field will be reused for `connect_fastest`
- Correct CLI flag for SecureCore is `--securecore` (not `--sc`)
