# issue092: Config file handling issues - double load, no external edit detection, unsafe writes

**Status: OPEN** 🔴

## Summary

Multiple issues found in config.toml handling: double load on startup, no external edit detection, and unsafe write patterns in other file operations.

## Problems

### 1. Double Load on Startup

`UserConfig::load()` is called in `main.rs`, then called again inside `AppState::from_config()`.

```rust
// main.rs:49-50
let user_config = UserConfig::load();
let mut app = TuiApp::new(user_config)?;

// app.rs:44-46 - config is destructured
let key_bindings = config.keybindings.into();  // config consumed
let ui_config = config.ui;
let mut state = AppState::from_config(&key_bindings, &ui_config);

// app_state.rs:128 - loads again!
let user_config = UserConfig::load();  // ← unnecessary 2nd read
```

**Impact**: Wasted file I/O. Rare possibility of returning different results on second read.

### 2. No External Edit Detection

`ProtonSettings` has `invalidate_cache()`/`clear_cache()`, but `UserConfig` has **no reload mechanism**.

- Editing `config.toml` externally → changes not reflected until app restart
- `AppState` has `user_config: UserConfig` field but no way to reload it

### 3. Same Unsafe Pattern in Other File Writes

| File | Method | Issue |
|---|---|---|
| `vpn/cache.rs:51` | `ServerCache::save()` | Direct overwrite (no READ) |
| `state/log_persistence.rs:52` | `save_notification_log()` | Direct overwrite (no READ) |

These use the same unsafe pattern as `UserConfig::save()` (overwrite without reading first).

## Proposed Solutions

### 1. Fix Double Load

```rust
// main.rs
let user_config = UserConfig::load();
let mut app = TuiApp::new(user_config)?;  // pass directly

// app.rs - TuiApp::new()
pub fn new(config: UserConfig) -> io::Result<Self> {
    let key_bindings = config.keybindings.into();
    let ui_config = config.ui;
    let mut state = AppState::from_config(&key_bindings, &ui_config, config);  // pass config
    // ...
}

// app_state.rs - AppState::from_config()
pub fn from_config(key_bindings: &KeyBindings, ui_config: &UiConfig, user_config: UserConfig) -> Self {
    // let user_config = UserConfig::load(); ← remove
    // use user_config parameter instead
}
```

### 2. External Edit Detection

```rust
impl AppState {
    pub fn reload_user_config(&mut self) {
        self.user_config = UserConfig::load();
        // Update UI state as well
        self.ui_state.theme_mode = ThemeMode::from_str(&self.user_config.ui.theme);
        self.ui_state.show_footer = self.user_config.ui.footer;
        self.ui_state.favorite_countries = self.user_config.ui.favorites.iter().cloned().collect();
    }
}
```

### 3. Fix Other File Writes

Apply Read-Modify-Write pattern to `ServerCache::save()` and `save_notification_log()`. Consider atomic writes (temp file + rename) for data safety.

## Files Involved

- `src/main.rs` - initial load
- `src/ui/app.rs` - TuiApp constructor
- `src/state/app_state.rs` - AppState::from_config(), save_* methods
- `src/config/user_config.rs` - UserConfig::save(), UserConfig::load()
- `src/vpn/cache.rs` - ServerCache::save()
- `src/state/log_persistence.rs` - save_notification_log()

## Related Issues

- issue048: config.toml content deletion (Read-Modify-Write fix applied)
