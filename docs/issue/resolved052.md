# issue052-favorite-countries-pinned-to-top

## Summary

Allow users to favorite countries and have them pinned to the top of the server list. Pressing `<C-f>` on a selected country should toggle its favorite status, and favorited countries should always appear at the top of the list regardless of sort order.

## Status

**[Resolved]**

## Resolution

Implemented with a dedicated status column (1 char width) for indicators:

| State | Display |
|-------|---------|
| Selected | `>` |
| Not selected + Favorite | `*` |
| Not selected + Not favorite | ` ` |

### Changes Made

| File | Change |
|------|--------|
| `src/ui/components/pane_table.rs` | Added empty status column to CountriesTable; updated sort key indices (¹→column 1, ²→column 2) |
| `src/ui/views/servers_view.rs` | Added indicator logic in `render_countries_pane()`; removed `highlight_symbol("> ")`; added status column Cell |

### Implementation Details

- Status column: 1 char width, empty header
- Code column: 6 char width (accommodates "Code ▲/▼")
- Sort key indices updated to account for new column 0 (status)

## Feature Requirements

### Core Behavior

1. **Toggle Favorite with `<C-f>`**
   - When a country is selected in the Countries pane, pressing `<C-f>` toggles its favorite status
   - If not favorited → add to favorites
   - If favorited → remove from favorites
   - Show notification confirming action

2. **Favorites Always on Top (Sorted)**
   - Favorited countries appear at the top of the server list
   - Favorites are sorted by the **current sort selection** (not forced to country name)
   - After favorites, remaining countries are sorted by the **current sort selection**
   - When user changes sort (press `1` or `2`), both favorites and non-favorites are re-sorted

   **Example** (Sort by Country):
   ```
   ★ DE (Germany)      ← Favorite, sorted by country
   ★ JP (Japan)       ← Favorite, sorted by country
   AU (Australia)      ← Not favorite, sorted by country
   US (United States)  ← Not favorite, sorted by country
   ```

   **Example** (Sort by Code):
   ```
   ★ DE (Germany)      ← Favorite, sorted by code
   ★ JP (Japan)       ← Favorite, sorted by code
   AU (Australia)      ← Not favorite, sorted by code
   US (United States)  ← Not favorite, sorted by code
   ```

3. **Visual Indicator**
   - Favorited countries show `*` indicator in the code column
   - Indicator is shown **only when the row is not selected**
   - When selected, the `> ` highlight symbol takes precedence (no `*` shown)
   - Indicator position: same column as `> ` would appear (code column, left side)

   **Display Rules**:
   | State | Display |
   |-------|---------|
   | Selected (any) | `> US` — highlight_symbol only |
   | Not selected + Favorite | `* US` |
   | Not selected + Not favorite | `US` |

   **Implementation**:
   - Since `> ` is applied via ratatui's `highlight_symbol`, we add `* ` only to the Cell content when **not selected**
   - When selected, highlight_symbol replaces the cell content, so `* ` is naturally hidden

### User Experience

| Key | Action |
|-----|--------|
| `<C-f>` | Toggle favorite on selected country |

### Persistence

- Favorites should be saved to config file
- Load favorites on app startup

## Technical Implementation

### Files to Modify

| File | Change |
|------|--------|
| `src/config/settings.rs` | Add `toggle_favorite` to `KeyBindings` struct with default `<C-f>` |
| `src/state/ui_state.rs` | Add `favorite_countries: HashSet<String>` field |
| `src/state/app_state.rs` | Add methods: `toggle_favorite()`, `is_favorite()`, update `compute_filtered_servers()` |
| `src/ui/app.rs` | Handle `<C-f>` key in `handle_servers_key()` |
| `src/ui/views/servers_view.rs` | Add favorite indicator to row rendering |
| `src/config/user_config.rs` | Add favorites to `UserConfig` for persistence |

### Key Implementation Details

#### 1. KeyBinding (settings.rs)

```rust
pub struct KeyBindings {
    // ... existing fields
    pub toggle_favorite: KeyBinding,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            // ... existing defaults
            toggle_favorite: KeyBinding::new('f', KeyModifier::Control),
        }
    }
}
```

#### 2. UI State (ui_state.rs)

```rust
pub struct UIState {
    // ... existing fields
    pub favorite_countries: HashSet<String>,
}
```

#### 3. AppState Methods (app_state.rs)

```rust
impl AppState {
    pub fn toggle_favorite(&mut self, country_code: &str) {
        if self.ui_state.favorite_countries.contains(country_code) {
            self.ui_state.favorite_countries.remove(country_code);
            self.show_notification(
                format!("Removed {} from favorites", country_code),
                NotificationType::Info,
                None,
            );
        } else {
            self.ui_state.favorite_countries.insert(country_code.to_string());
            self.show_notification(
                format!("Added {} to favorites", country_code),
                NotificationType::Success,
                None,
            );
        }
        self.server_cache.invalidate();
    }

    pub fn is_favorite(&self, country_code: &str) -> bool {
        self.ui_state.favorite_countries.contains(country_code)
    }
}
```

#### 4. Sorting Logic (app_state.rs)

Modify `compute_filtered_servers()` to partition into favorites/non-favorites, then apply current sort to each group:

```rust
pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
    // ... existing filter logic ...

    // Partition into favorites and non-favorites
    let mut favorites: Vec<Server> = result
        .iter()
        .filter(|s| self.is_favorite(&s.code))
        .cloned()
        .collect();
    
    let mut non_favorites: Vec<Server> = result
        .iter()
        .filter(|s| !self.is_favorite(&s.code))
        .cloned()
        .collect();

    // Apply current sort to favorites
    self.sort_servers(&mut favorites);

    // Apply current sort to non-favorites
    self.sort_servers(&mut non_favorites);

    // Combine: favorites first, then non-favorites
    favorites.append(&mut non_favorites);
    favorites
}
```

**Important**: The `sort_servers()` method should use the **current sort setting** (`self.ui_state.sort` and `self.ui_state.sort_direction`).

#### 5. Key Handling (app.rs)

In `handle_servers_key()`:

```rust
let is_toggle_favorite = bindings
    .toggle_favorite
    .matches(key_event.code, key_event.modifiers);

// ... in match statement:
_ if is_toggle_favorite => {
    if let Some(idx) = self.state.ui_state.selected_server {
        if let Some(server) = self.state.filtered_servers().get(idx) {
            self.state.toggle_favorite(&server.code);
        }
    }
    None
}
```

#### 6. Visual Indicator (servers_view.rs)

In the row rendering for `render_countries_pane()`, add `* ` prefix to code Cell when **not selected** and **is favorite**:

```rust
// Line 130-134: Modify the Cell for code column
let code_display = if is_favorite && !is_selected {
    format!("* {}", server.code)
} else {
    server.code.clone()
};

Row::new(vec![
    Cell::from(code_display).style(code_style),
    Cell::from(server.country.clone()).style(country_style),
    Cell::from(cities_str).style(cities_style),
])
.style(row_style)
```

**Key logic**:
- `is_favorite = state.is_favorite(&server.code)`
- `is_selected = state.ui_state.selected_server == Some(idx)`
- When `is_selected` is true, ratatui's `highlight_symbol("> ")` takes over the cell, so we don't add `*`
- When `is_selected` is false and `is_favorite` is true, we add `* ` prefix

### Persistence

Add favorites to `UserConfig` in `user_config.rs`:

```rust
pub struct UserConfig {
    // ... existing fields
    pub favorites: Vec<String>,
}
```

Load/save in the appropriate places.

## Notes

- Favorites are identified by country code (e.g., "US", "JP", "DE")
- The `<C-f>` key is currently unbound (no conflict with existing keys)
- Notification should confirm when adding/removing from favorites
- Consider adding favorite count to footer or header for visibility
