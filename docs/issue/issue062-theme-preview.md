# issue062: Feature - Theme preview in settings view

## Summary

Add color palette preview when selecting a theme in the settings view.

## Problem

Currently, the theme selection UI shows only a text list of theme names ("Nord", "Dracula", etc.).
Users cannot visually compare themes without selecting and applying each one.

First-time users especially have no way to judge a theme's appearance before committing to it.

## Current Implementation

- **Options**: `src/config/settings.rs:108-116` — `selectable_options()` returns text list
- **Theme definition**: `src/ui/styles.rs` — `Theme` struct with 9 colors (background, foreground, primary, secondary, accent, error, success, warning, dim)
- **Display**: `src/ui/views/settings_view.rs` — lists SettingKey::Theme options as plain text

```
Theme: [Nord] ← currently selected
       Catppuccin Mocha
       Catppuccin Latte
       Dracula
       ...
```

## Solution

Temporarily apply the highlighted theme when navigating the theme list. The UI updates in real-time as the user moves through options, allowing visual comparison before committing.

### Behavior

- Navigating theme options (j/k) → UI temporarily switches to that theme
- Confirming selection (Enter) → theme persists
- Moving focus away from Theme setting → revert to confirmed theme

### Implementation Approach

1. Track `preview_theme: Option<ThemeMode>` in `UiState`
2. On theme option highlight, set `preview_theme = Some(selected_mode)`
3. In `render()`, use `preview_theme` if set, otherwise use confirmed theme
4. On focus leave or confirm, clear `preview_theme`

## Files Affected

- `src/state/ui_state.rs` — add `preview_theme: Option<ThemeMode>` field
- `src/ui/app.rs` — use `preview_theme` in render if set
- `src/state/app_state.rs` — handle preview in theme navigation methods

## Recommendation

1. Add `preview_theme: Option<ThemeMode>` to `UiState`
2. In navigation methods (`settings_select_next/prev`), set `preview_theme` when on Theme setting
3. In `AppState::theme()`, return `Theme::from_mode(preview_theme.unwrap_or(confirmed_theme))`
4. On confirm or focus leave, clear `preview_theme`

## Severity

🟢 LOW — UX improvement, not functionally critical

## Labels

`feature` `ui` `theme`
