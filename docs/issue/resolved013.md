# resolved_issue013: UI Bug Fixes - Servers View

## Summary

Fix 4 UI bugs in the Servers View related to country/city selection, title rendering, focus highlighting, and scroll behavior.

## Status: RESOLVED ✅

---

## Bug 1: Selected Country Exists but Cities Not Reflected at Startup

**Status: RESOLVED**

Added `load_cities_for_selected_server()` helper method called from `set_servers()` to automatically load cities when servers are set.

---

## Bug 2: Cities Title Shows "Cities - Cities" When Cities Not Loaded

**Status: RESOLVED**

Title now shows three states:
- "Select country" (when no country selected)
- "XX - Loading..." (when country selected but cities empty)
- "XX - Cities" (when cities loaded)

---

## Bug 3: Pane Focus + Selected Should Highlight Foreground

**Status: RESOLVED**

- Added `is_focused: bool` parameter to `styled_list_item()`
- Selected + focused: foreground + background (inverted)
- Selected + not focused: foreground only

---

## Bug 4: CitiesPane Scroll Issues with 2+ Cities

**Status: RESOLVED**

Split single `list_state` into:
- `countries_list_state` - for Countries pane
- `cities_list_state` - for Cities pane

---

## Tags

- bug
- ui
- resolved
