# issue052-favorite-countries-pinned-to-top

## Summary

Allow users to favorite countries and have them pinned to the top of the server list. Pressing `<C-f>` on a selected country should toggle its favorite status, and favorited countries should always appear at the top of the list regardless of sort order.

## Status

**[Open]**

## Feature Requirements

### Core Behavior

1. **Toggle Favorite with `<C-f>`**
   - When a country is selected in the Countries pane, pressing `<C-f>` toggles its favorite status
   - If not favorited → add to favorites
   - If favorited → remove from favorites
   - Show notification confirming action

2. **Favorites Always on Top**
   - Favorited countries appear at the top of the server list
   - This ordering takes precedence over other sort methods (code, country name)
   - Secondary sort within favorites: by country name (ascending)
   - Secondary sort within non-favorites: by current sort selection

3. **Visual Indicator**
   - Favorited countries show a indicator (e.g., `★` or `*`) in the country list
   - The indicator should be visible in the first column (server code)

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

#### 4. Filtering Logic (app_state.rs)

Modify `compute_filtered_servers()` to sort favorites to top:

```rust
pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
    // ... existing filter logic ...

    // After filtering, partition into favorites and non-favorites
    let favorites: Vec<Server> = result
        .iter()
        .filter(|s| self.is_favorite(&s.code))
        .cloned()
        .collect();
    
    let non_favorites: Vec<Server> = result
        .iter()
        .filter(|s| !self.is_favorite(&s.code))
        .cloned()
        .collect();

    // Sort favorites by country name (ascending)
    let mut favorites = favorites;
    favorites.sort_by(|a, b| a.country_lower.cmp(&b.country_lower));

    // Combine: favorites first, then non-favorites with current sort
    let mut final_result = favorites;
    final_result.extend(non_favorites);
    
    final_result
}
```

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

In the row rendering, add indicator:

```rust
let favorite_indicator = if state.is_favorite(&server.code) {
    "★ ".to_string()
} else {
    "  ".to_string()
};

Row::new(vec![
    Cell::from(format!("{}{}", favorite_indicator, server.code)).style(code_style),
    // ... rest of cells
])
```

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
