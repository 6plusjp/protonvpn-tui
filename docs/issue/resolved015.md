# issue015: Countries Pane - Highlight Priority & City Parsing Bug

## Status: RESOLVED (2026-03-07)

## Summary

Three issues in the Countries/Cities Pane were resolved:

1. ✅ **RESOLVED**: Highlight priority - check `is_selected` before `is_connected`
2. ✅ **RESOLVED**: City names with spaces - use 2+ consecutive whitespaces as delimiter
3. ✅ **RESOLVED**: Connect parsing - use stdout only (not combined with stderr)

---

## Issue 1: Highlight Priority

### Location
`src/ui/views/servers_view.rs` - lines 99-105

### Problem
When a country is both selected (keyboard navigation) AND connected, the "connected" highlight was shown instead of "selected" highlight.

### Fix
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

### Commit
`aa0f6df` - fix(issue015-1): swap highlight priority

---

## Issue 2: City Name Parsing Bug

### Location
`src/vpn/client.rs` - `parse_cities_with_features` function

### Problem
Multi-word city names like "Tel Aviv" were incorrectly parsed because the code split on the first whitespace:
- Input: `"Tel Aviv  P2P, Secure Core"`
- Wrong: `name="Tel"`, `features="Aviv P2P..."`

### Fix
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

### Commit
`5e49d6f` - fix(issue015-2): parse city names using 2+ consecutive whitespaces

---

## Issue 3: Connect Output Parsing Error

### Location
`src/vpn/client.rs` - `connect` function

### Problem
When `protonvpn connect` executes, stdout contains success message but stderr may contain error/traceback:

```
stdout: "Connected to JP#255 in Tokyo, Japan. Your new IP address is 159.26.119.30."
stderr: "ERROR | exception calling callback..."
```

When combined: `"Connected to JP#255... 159.26.119.30.ERROR | exception..."`

This caused IP parsing to fail because "ERROR" appeared after the IP address.

### Fix
Use only stdout for parsing (success messages go to stdout):

```rust
// Before (wrong):
let combined = format!("{} {}", stdout, stderr);
let (server_id, ip, ...) = self.parse_connect_output(&combined);

// After (correct):
let (server_id, ip, ...) = self.parse_connect_output(&stdout);
```

### Commit
`7a6f256` - fix(issue015-3): use stdout only for parsing connect output

---

## Tags

- bug
- ui
- parsing
- resolved
