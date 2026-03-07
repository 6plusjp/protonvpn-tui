# issue015: Countries Pane - Highlight Priority & City Parsing Bug

## Status: PARTIALLY RESOLVED (2026-03-07)

## Summary

Three issues identified in the Countries/Cities Pane:

1. ✅ **RESOLVED**: Highlight priority - check `is_selected` before `is_connected`
2. ✅ **RESOLVED**: City names with spaces - use 2+ consecutive whitespaces as delimiter
3. ⚠️ **NOT RESOLVED**: Connect output parsing - still has issues

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

## Issue 3: Connect Output Parsing Error ⚠️ NOT RESOLVED

### Location
`src/vpn/client.rs` - `parse_connect_output` function

### Current Status
The fix was applied to find "Connected to" line first, but there are still edge cases that need investigation.

### Problem Description
When parsing `protonvpn connect` output:
- Traceback with IP-like numbers may cause wrong IP extraction
- "Server list outdated" message appearing first may cause issues

### Next Steps
Need to investigate actual failing cases to understand what's still broken.

---

## Tags

- bug
- ui
- parsing
- partially-resolved
