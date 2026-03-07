# issue015: Countries Pane - Highlight Priority & City Parsing Bug

## Status: INVESTIGATED (2026-03-07)

## Summary

Three issues identified in the Countries/Cities Pane:

1. ✅ **CONFIRMED**: Highlight priority is incorrect - `isconnected` should have lower priority than `isselected`
2. ✅ **CONFIRMED**: City names with spaces are incorrectly parsed due to whitespace handling
3. ✅ **CONFIRMED**: Connect output parsing - IP extracted from wrong line (traceback/outdated message) instead of "Connected to" line

---

## Issue 1: Highlight Priority ✅ CONFIRMED BUG

### Location
`src/ui/views/servers_view.rs` - lines 99-105

### Current Code (BUGGY)
```rust
if is_connected {
    connected_list_item(&row, theme)
} else if is_selected {  // BUG: checks is_connected first
    styled_list_item(&row, true, is_focused, theme)
} else {
    styled_list_item(&row, false, is_focused, theme)
}
```

### Problem
When a country is both selected (keyboard navigation) AND connected, the "connected" highlight is shown instead of "selected" highlight. The priority should be `isselected > isconnected`.

### Expected Behavior
| State                                  | Highlight                              |
| -------------------------------------- | -------------------------------------- |
| isselected = true                      | Selected highlight (highest priority) |
| isconnected = true, isselected = false | Connected highlight                   |
| Neither                                | Default                                |

### Fix Required
Swap the condition order - check `is_selected` first:

```rust
if is_selected {
    styled_list_item(&row, true, is_focused, theme)
} else if is_connected {
    connected_list_item(&row, theme)
} else {
    styled_list_item(&row, false, is_focused, theme)
}
```

---

## Issue 2: City Name Parsing Bug ✅ CONFIRMED BUG

### Location
`src/vpn/client.rs` - `parse_cities_with_features` function (lines 412-417)

### Current Code (BUGGY)
```rust
// Split name (before first whitespace) from features (after)
let name_end = line.find(|c: char| c.is_whitespace());
let (name, features_str) = match name_end {
    Some(pos) => (line[..pos].to_string(), line[pos..].trim()),
    None => (line.to_string(), ""),
};
```

### Problem
Uses `find(|c: char| c.is_whitespace())` which finds the FIRST whitespace. For input `"Tel Aviv  P2P, Secure Core"`:

- **Current (wrong)**: name=`"Tel"`, features=`"Aviv P2P, Secure Core"`
- **Expected**: name=`"Tel Aviv"`, features=`"P2P, Secure Core"`

### Root Cause
The CLI output uses fixed-width columns:
```
City      Features
--------  ----------------
Tel Aviv  P2P, Secure Core
```

The parsing should account for the column alignment, not just split on first whitespace.

### Fix Required
Implement column-aware parsing. The Features column starts at a predictable position (after the city name padding). Parse based on column positions instead of whitespace.

---

## Issue 3: Connect Output Parsing Error ✅ ROOT CAUSE IDENTIFIED

### Location
`src/vpn/client.rs` - `parse_connect_output` function (lines 172-228)

### Root Cause
Current implementation iterates through ALL lines and checks for IP patterns FIRST, before finding "Connected to". This causes issues when:

1. **Traceback contains numbers/IPs**: Error messages may contain IP-like patterns (e.g., `InvalidData(123.45.67.89)`)
2. **"Server list outdated" first**: When this message appears first, the parser may incorrectly handle subsequent lines
3. **Wrong priority**: IP is extracted from wrong line (traceback) instead of the actual "Connected to" line

### Current Code (BUGGY)
```rust
for line in output.lines() {
    let line = line.trim();

    // BUG: Checks IP patterns FIRST on all lines
    if line.contains("IP address") || line.contains("IP:") || line.contains("address is") {
        // Extract IP from ANY matching line - potentially wrong!
    }

    // Then checks "Connected to" - too late if IP already found
    if line.starts_with("Connected to ") {
        // Extract server_id, city, country
    }
}
```

### Problem Scenarios

**Scenario 1**: Traceback with IP-like number:
```
Traceback (most recent call last):
  ...
local_agent.LocalAgentError: Tokio(Custom { kind: InvalidData, error: InvalidData(123.45.67.89) })

Connected to JP#374 in Tokyo, Japan. Your new IP address is 159.26.119.144.
```
→ May extract wrong IP or fail

**Scenario 2**: Server list message first:
```
Server list is outdated, updating... This may take a moment.
Connected to JP#378 in Tokyo, Japan. Your new IP address is 159.26.119.143.
```
→ May confuse the parser

### Fix Required

Change priority: Find "Connected to" line FIRST, then extract all info from that single line:

```rust
pub(crate) fn parse_connect_output(&self, output: &str) -> (...) {
    // Find the "Connected to" line FIRST
    let connected_line = output.lines()
        .find(|line| line.trim().starts_with("Connected to "));

    if let Some(line) = connected_line {
        let line = line.trim();
        // Parse server_id, city, country, IP ALL from this single line
        // - Extract server_id after "Connected to "
        // - Extract city/country after " in "
        // - Extract IP from "Your new IP address is X" or "IP address: X"
    }

    // Ignore all other lines (traceback, server list messages, etc.)
}
```

### Key Changes
1. Find "Connected to " line first (single source of truth)
2. Extract server_id, city, country from that line
3. Extract IP from that same line (not from other lines)
4. Ignore all other lines completely (traceback, outdated messages, etc.)

---

## Tags

- bug
- ui
- parsing
- confirmed
- priority-high
