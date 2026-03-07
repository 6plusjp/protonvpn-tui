# issue015: Countries Pane - Highlight Priority & City Parsing Bug

## Status: RESOLVED (2026-03-07)

## Summary

Three issues identified in the Countries/Cities Pane:

1. ✅ **RESOLVED**: Highlight priority - check `is_selected` before `is_connected`
2. ✅ **RESOLVED**: City names with spaces - use 2+ consecutive whitespaces as delimiter
3. ✅ **RESOLVED**: Connect parsing - use stdout only (not combined with stderr)

---

## Issue 1: Highlight Priority ✅ RESOLVED

### Location
`src/ui/views/servers_view.rs` - lines 99-105

### Fix Applied
Swapped condition order - check `is_selected` first:

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

## Issue 2: City Name Parsing Bug ✅ RESOLVED

### Location
`src/vpn/client.rs` - `parse_cities_with_features` function

### Fix Applied
Changed to find 2+ consecutive whitespaces as delimiter:

```rust
let mut chars = line.char_indices().peekable();
let mut name_end = None;

while let Some((start, c)) = chars.next() {
    if c.is_whitespace() {
        let mut consecutive = 1;
        while let Some(&(_, next_c)) = chars.peek() {
            if next_c.is_whitespace() {
                consecutive += 1;
                chars.next();
            } else {
                break;
            }
        }
        if consecutive >= 2 {
            name_end = Some(start);
            break;
        }
    }
}
```

---

## Issue 3: Connect Output Parsing Error ⚠️ RESOLVED

### Location
`src/vpn/client.rs` - `connect` function (lines 79-103)

### Problem Description
When `protonvpn connect` is executed, the output may contain both success message AND error/traceback:

```
stdout: "Connected to JP#255 in Tokyo, Japan. Your new IP address is 159.26.119.30."
stderr: "ERROR | exception calling callback..."
```

When combined: `"Connected to JP#255... 159.26.119.30.ERROR | exception..."`

This causes IP parsing to fail because "ERROR" appears after the IP address.

### Root Cause
The code was using `stdout + stderr` combined for parsing:
```rust
let combined = format!("{} {}", stdout, stderr);
let (server_id, ip, _city, _country) = self.parse_connect_output(&combined);
```

### Fix Applied
Use only stdout for parsing (success messages go to stdout):
```rust
let (server_id, ip, _city, _country) = self.parse_connect_output(&stdout);
```

### Expected Behavior
- `check_cli_error` uses combined output (to detect errors)
- `parse_connect_output` uses stdout only (success messages are in stdout)

---

## Tags

- bug
- ui
- parsing
- partially-resolved
