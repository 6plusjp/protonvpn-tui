# issue087: Fix cargo doc warnings and improve API documentation

## Summary

Fix `cargo doc` warnings and improve public API documentation for crate publish readiness.

## Problems

### 1. HTML Warnings (3件)

```
warning: unclosed HTML tag `CC`
 --> src/vpn/client.rs:5:29
warning: unclosed HTML tag `city`
 --> src/vpn/client.rs:8:32
```

Doc comment内の `<CC>`, `<city>` がHTMLタグとして認識されている。

### 2. Public API Documentation不足

主要なpublic typeにdoc commentがない (58文件中16ファイルのみdocumentationあり):

- `vpn::VpnClient`, `Server`, `City`, `ServerFeatures`, `ServerCache`, `AsyncTaskManager`
- `state::AppState`, `AppView`, `ConnectionState`, `ServerFilter`, `ServerSort`
- `ui::Theme`, `ThemeMode`, `KeyMap`, `KeyMatcher`

Package metadataには `documentation = "https://docs.rs/protonvpn-tui"` が設定済み。

## Solution

### 1. Fix HTML Warnings (Quick Fix)

`src/vpn/client.rs` のdoc commentを修正:

```rust
// Before (bad)
/// - protonvpn cities list <CC>  -> list cities for a country

// After (good) - escape with backticks
/// - `protonvpn cities list <CC>` → list cities for a country
```

### 2. Add Doc Comments to Public API Types

主要なpublic typeにdoc commentを追加 (priority順):

| Priority | Type | File |
|----------|------|------|
| High | `VpnClient`, `Server`, `City`, `ServerFeatures` | `vpn/types.rs`, `vpn/client.rs` |
| High | `AppState`, `ConnectionState` | `state/app_state.rs` |
| Medium | `Theme`, `ThemeMode`, `KeyMap` | `ui/styles.rs`, `ui/keymap.rs` |
| Medium | `ServerFilter`, `ServerSort` | `state/server_filter.rs`, `state/server_sort.rs` |

## Verification

```bash
# Should pass without warnings
cargo doc --no-deps

# Should pass clippy
cargo clippy --no-deps
```

## Status

- [x] Fix HTML warnings in vpn/client.rs
- [x] Verify cargo doc passes without warnings
- [x] Add doc comments to public API types

### Added Documentation

Added doc comments to the following public types:

| Module | Types |
|--------|-------|
| `error.rs` | `AppError`, `VpnError` |
| `config/settings.rs` | `KeyModifier`, `KeyBindings`, `KeyBinding` |
| `config/user_config.rs` | `UiConfig`, `KeyBindingConfig`, `KeyMatcherConfig`, `UserConfig` |
| `ui/app.rs` | `TuiApp` |
| `ui/input/app_action.rs` | `AppAction` |

---

## Related

- Cargo.toml: `documentation = "https://docs.rs/protonvpn-tui"`
- 58 Rust source files in src/
- 16 files have doc comments (28%)