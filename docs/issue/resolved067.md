# issue067: app.rs exceeds 1500 lines

## Summary

`src/ui/app.rs` at 1551 lines handles event loop, key handling, and all rendering, violating single-responsibility principle.

## Problem

Current `app.rs` responsibilities:

| Responsibility | Lines | % |
|----------------|-------|---|
| Event loop (`run()`) | 57-119 | 4% |
| Key handling (`handle_*`) | 121-650 | 34% |
| Header rendering | 650-850 | 13% |
| Footer rendering | 850-950 | 6% |
| Notification popup | 950-1100 | 10% |
| Connection actions | 1100-1400 | 19% |
| Settings actions | 1400-1551 | 10% |

### Code Smell Indicators

1. **File too large**: 1551 lines exceeds typical 300-500 line target
2. **Multiple concerns**: Event handling, rendering, and business logic mixed
3. **Hard to navigate**: Finding specific functionality requires scrolling
4. **Merge conflicts**: High likelihood with multiple contributors

## Solution

### Proposed Split

```
src/ui/
├── app.rs              # Event loop + TuiApp struct (~150 lines)
├── input/
│   ├── mod.rs
│   ├── common.rs       # handle_common_keys() (~100 lines)
│   ├── servers.rs      # handle_servers_key() (~100 lines)
│   └── tools.rs        # handle_tools_key() (~100 lines)
└── renderers/
    ├── mod.rs
    ├── header.rs       # render_header() (~200 lines)
    ├── footer.rs       # render_footer() (~100 lines)
    └── notification.rs # render_notification_popup() (~150 lines)
```

### Migration Strategy

1. **Phase 1**: Extract `render_header()` → `renderers/header.rs`
2. **Phase 2**: Extract `render_footer()` → `renderers/footer.rs`
3. **Phase 3**: Extract `handle_*_key()` → `input/*.rs`
4. **Phase 4**: Keep `app.rs` as thin orchestrator

### Example After Refactoring

```rust
// app.rs (simplified)
mod input;
mod renderers;

impl TuiApp {
    pub fn run(&mut self) -> io::Result<()> {
        loop {
            // Event processing
            if event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    let action = input::handle_key(&mut self.state, key);
                    self.handle_action(action);
                }
            }
            
            // Render
            terminal.draw(|f| {
                renderers::header::render(&self.state, f, header_area);
                self.current_view.render(&mut self.state, f, main_area);
                renderers::footer::render(&self.state, f, footer_area);
                renderers::notification::render(&self.state, f, popup_area);
            })?;
        }
    }
}
```

## Files Affected

- `src/ui/app.rs` — Slim down to orchestrator
- `src/ui/input/mod.rs` — New: key handling modules
- `src/ui/input/common.rs` — Common key handling
- `src/ui/input/servers.rs` — Servers view keys
- `src/ui/input/tools.rs` — Tools view keys
- `src/ui/renderers/mod.rs` — New: render modules
- `src/ui/renderers/header.rs` — Header rendering
- `src/ui/renderers/footer.rs` — Footer rendering
- `src/ui/renderers/notification.rs` — Notification popup

## Severity

🟢 **LOW** — Code quality improvement, no functional impact.

## Labels

`refactor` `code-quality` `ui`

---

# Implementation Results

## Status: ✅ COMPLETED

### Final File Structure

```
src/ui/
├── app.rs              # 203 lines (87% reduction from 1528)
├── input/
│   ├── mod.rs          # 14 lines
│   ├── app_action.rs   # 6 lines
│   ├── input_state.rs  # 28 lines
│   ├── handler.rs      # 26 lines
│   ├── common.rs       # 376 lines
│   ├── servers.rs      # 87 lines
│   ├── tools.rs        # 236 lines
│   ├── help.rs         # 14 lines
│   └── filter.rs       # 54 lines
└── renderers/
    ├── mod.rs          # 6 lines
    ├── header.rs       # 149 lines
    ├── footer.rs       # 261 lines
    ├── notification.rs # 73 lines
    └── input.rs        # 87 lines
```

### Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| app.rs lines | 1528 | 203 | **87% reduction** |
| Total lines | 1528 | 1609 | +81 (module overhead) |
| Modules | 0 | 12 | +12 |

### Verification

- ✅ `cargo check` — Compilation successful
- ✅ `cargo test` — 56 unit tests + integration tests passed
- ✅ `cargo clippy` — No warnings
- ✅ All behavior preserved (no functional changes)
