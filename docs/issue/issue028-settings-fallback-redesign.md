# issue028: Bug - settings.json not found on first run + Settings UI redesign

## Summary

Two related issues:
1. Missing `settings.json` handling on first run
2. Redesign settings UI from toggle to expandable selection

---

## Part A: settings.json not found on first run

### Problem

`~/.config/Proton/VPN/settings.json` is not generated until the user runs `protonvpn config set` with some value. On first run (or fresh installation), this file doesn't exist, causing:

1. `ProtonSettings::load()` returns `None`
2. Settings UI shows "Loading settings..." or incorrect defaults
3. User sees "off" which incorrectly implies the setting is configured

### Root Cause

The `protonvpn` CLI only creates `settings.json` after the user explicitly configures at least one setting:

```bash
protonvpn config set killswitch on  # Creates settings.json
```

Without this, the config directory exists but `settings.json` is missing.

### Solution

**IMPORTANT**: Never default to "off" - that implies the user has explicitly disabled a feature. When settings cannot be loaded, display "unknown" to indicate the setting has not been configured.

```rust
impl ProtonSettings {
    /// Load Proton VPN settings from ~/.config/Proton/VPN/settings.json
    /// Falls back to None if file doesn't exist (NOT to defaults)
    pub fn load() -> Option<Self> {
        let config_path = dirs::config_dir()?
            .join("Proton")
            .join("VPN")
            .join("settings.json");
        
        if !config_path.exists() {
            tracing::debug!("settings.json not found");
            return None;
        }
        
        let content = std::fs::read_to_string(config_path).ok()?;
        let settings: Self = serde_json::from_str(&content).ok()?;
        Some(settings)
    }
}
```

### UI Display Logic

| Value | Display |
|-------|---------|
| `Some(0)` or `Some(false)` | "off" (user explicitly disabled) |
| `Some(1)` or `Some(true)` | "on" (user explicitly enabled) |
| `Some(2)` etc | "value" (numeric value) |
| `None` | "unknown" (not configured) |

**Key**: Always use `None` check first, not `unwrap_or_else(|| "off")`.

### Acceptance Criteria - Part A

- [ ] App works on first run without `settings.json`
- [ ] Unconfigured settings show "unknown" (not "off")
- [ ] Logs indicate when settings cannot be loaded

---

## Part B: Settings UI Redesign (toggle → expandable selection)

### Problem

Currently, pressing a key toggles a setting on/off. This is:
- Not discoverable (user doesn't know what keys work)
- Error-prone (easy to accidentally toggle)
- Limited (can only handle binary on/off)

### Solution

Replace toggle behavior with an expandable selection menu:

**Current (toggle)**:
```
[ ] Kill Switch      on
[ ] IPv6             off
[ ] DNS              default
```
Press `k` → toggles killswitch

**New (expandable)**:
```
> Kill Switch        on      (Enter to change)
  IPv6               off
  DNS                default
```
Press `Enter` on Kill Switch → expands to:
```
  Kill Switch        on
  ├─► off
  │   on
  └─ unknown
```
Press `j/k` to move, `Enter` to select, `Esc` to cancel

### Keyboard Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in list |
| `k` / `↑` | Move up in list |
| `Enter` | Expand selected setting / Confirm selection |
| `Esc` | Collapse / Cancel selection |
| `g` / `G` | Go to top / bottom |

### Interaction Flow

1. **Browse mode** (default): Navigate with j/k
2. **Expand**: Press `Enter` on selected setting → shows options
3. **Select**: Press `Enter` on option → executes `protonvpn config set ...` and collapses
4. **Cancel**: Press `Esc` → collapses without changing

### Example: Changing Kill Switch

```
Before: Kill Switch: unknown
        IPv6:       unknown

User presses Enter on "Kill Switch"

Expanded: Kill Switch: unknown
          ├─► off
          │   on  
          └─   unknown
                   (cursor on "off")

User presses Enter

→ Executes: protonvpn config set killswitch off
→ Collapses: Kill Switch: off
```

### Example: Changing NetShield (3 options)

```
Before: NetShield: unknown

User presses Enter on "NetShield"

Expanded: NetShield: unknown
          ├─► off
          │   malware-only
          └─   malware-ads-trackers

User presses j to move to "malware-only", Enter

→ Executes: protonvpn config set netshield 1
→ Collapses: NetShield: malware-only
```

### Technical Implementation

```rust
// State for settings view
pub struct SettingsViewState {
    pub selected: usize,           // Currently selected setting
    pub expanded: bool,            // Is options list expanded?
    pub options_selected: usize,   // Selected option in expanded list
}

impl AppState {
    pub fn handle_settings_input(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        match (self.settings_view.expanded, key.code) {
            // Browse mode
            (false, KeyCode::Enter) => {
                self.settings_view.expanded = true;
                self.settings_view.options_selected = 0; // Reset to first option
                Some(SettingsAction::Expand)
            }
            (false, KeyCode::Char('j') | KeyCode::Down) => {
                self.settings_view.selected = self.settings_view.selected.saturating_add(1);
                None
            }
            (false, KeyCode::Char('k') | KeyCode::Up) => {
                self.settings_view.selected = self.settings_view.selected.saturating_sub(1);
                None
            }
            
            // Expanded mode
            (true, KeyCode::Enter) => {
                // Execute protonvpn config set based on selected option
                Some(SettingsAction::Select(self.settings_view.selected, self.settings_view.options_selected))
            }
            (true, KeyCode::Esc) => {
                self.settings_view.expanded = false;
                Some(SettingsAction::Collapse)
            }
            (true, KeyCode::Char('j') | KeyCode::Down) => {
                self.settings_view.options_selected = self.settings_view.options_selected.saturating_add(1);
                None
            }
            (true, KeyCode::Char('k') | KeyCode::Up) => {
                self.settings_view.options_selected = self.settings_view.options_selected.saturating_sub(1);
                None
            }
            
            // Global
            (true | false, KeyCode::Char('q') | KeyCode::Esc) => {
                Some(SettingsAction::Exit)
            }
            _ => None,
        }
    }
}
```

### Settings and Their Options

| Setting | Options |
|---------|---------|
| Kill Switch | off, on, unknown |
| IPv6 | disabled, enabled, unknown |
| DNS | default, custom (IP list), unknown |
| NetShield | off, malware-only, malware-ads-trackers, unknown |
| Moderate NAT | off, on, unknown |
| VPN Accelerator | off, on, unknown |
| Port Forwarding | off, on, unknown |
| Theme | Dark, Light |

### Acceptance Criteria - Part B

- [ ] Settings list displays with expandable options
- [ ] Enter expands selected setting
- [ ] j/k navigates options when expanded
- [ ] Enter on option executes `protonvpn config set`
- [ ] Esc collapses without changes
- [ ] Visual indication of expanded state (e.g., arrow, highlight)
- [ ] All settings have appropriate options

---

## Related

- `src/ui/views/settings_view.rs` - Settings rendering
- `src/config/settings.rs` - Settings loading logic
- `src/vpn/client.rs` - Command execution
