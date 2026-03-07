# issue015: Countries Pane - Highlight Priority & City Parsing Bug

## Summary

Two issues in the Countries Pane:

1. Highlight priority is incorrect - `isconnected` should have lower priority than `isselected`
2. City names with spaces are incorrectly parsed due to whitespace handling

## Issue 1: Highlight Priority

### Problem Statement

Currently in the Countries Pane, when a country is both connected and selected, the highlight logic may prioritize `isconnected` over `isselected`. The correct priority should be:

- `isselected` > `isconnected`

This means if a country is selected (e.g., via keyboard navigation), it should be highlighted even if it's also the connected country.

### Expected Behavior

| State                                  | Highlight                             |
| -------------------------------------- | ------------------------------------- |
| isselected = true                      | Selected highlight (highest priority) |
| isconnected = true, isselected = false | Connected highlight                   |
| Neither                                | Default                               |

### Affected Files

- `src/ui/views/countries_pane.rs` - Highlight logic in render

---

## Issue 2: City Name Parsing Bug

### Problem Statement

When parsing the output of `protonvpn cities --country IL`, city names containing spaces are incorrectly split:

```
$ protonvpn cities --country IL

Cities in Israel:
City      Features
--------  ----------------
Tel Aviv  P2P, Secure Core
```

**Current (incorrect) parsed result:**

- city: "Tel"
- features: "Aviv P2P, Secure Core"

**Expected (correct) result:**

- city: "Tel Aviv"
- features: "P2P, Secure Core"

### Root Cause

The parsing logic likely uses whitespace as a delimiter without accounting for multi-word city names. The features column is left-aligned, so "Tel Aviv" is followed by spaces, then "P2P, Secure Core".

### Affected Files

- `src/vpn/client.rs` - City list parsing logic
- `src/vpn/types.rs` - City struct definition if applicable

---

## Issue 3: Connect Output Parsing Error

### Problem Statement

The `parse_connect_output` function in `src/vpn/client.rs` fails to correctly parse certain `protonvpn connect` command outputs. The current parsing logic expects a specific format:

```
Server list is outdated, updating... This may take a moment.
Connected to JP#378 in Tokyo, Japan. Your new IP address is 159.26.119.143.
```

But the actual output from `protonvpn connect` may vary or contain edge cases that are not handled correctly.

### Affected Files

- `src/vpn/client.rs` - `parse_connect_output` function (lines ~172-228)

---

## Tags

- bug
- ui
- parsing
