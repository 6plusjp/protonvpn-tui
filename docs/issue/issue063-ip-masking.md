# issue063: Feature - Header IP masking for security

## Summary

Add option to mask IP addresses displayed in the header for security purposes.

## Problem

Currently, the connected IP address is displayed in plain text in the header.

Example: `ip: 159.26.119.143`

This creates a security risk when recording screens, giving demos, or sharing screenshots, as the real IP address is exposed.

## Current Implementation

**Display location**: `src/ui/app.rs:1149-1153`

```rust
if !ip.is_empty() {
    spans.push(Span::styled("  ", Style::default().fg(theme.dim)));
    spans.push(Span::styled("ip:", Style::default().fg(theme.dim)));
    spans.push(Span::styled(ip, Style::default().fg(theme.secondary)));
}
```

**IP source**: `ConnectionState::Connected { ip, .. }` — `src/state/connection_state.rs`

## Solution

Add option to mask IP display as `***.**.***.***`.

### Display Mockup

```
Default:  JP#374  ip: 159.26.119.143  loc: Tokyo,Japan
Masked:   JP#374  ip: ***.**.***.***  loc: Tokyo,Japan
```

### Implementation

**Config setting**: Add `mask_ip: bool` to `UserConfig.ui` (default: `false`)
**Mask format**: `***.**.***.***` (full mask — IP structure hidden)

### Display Mockup

```
Default:  JP#374  ip: 159.26.119.143  loc: Tokyo,Japan
Masked:   JP#374  ip: ***.**.***.***  loc: Tokyo,Japan
```

## Files Affected

- `src/ui/app.rs:1149-1153` — masking display logic
- `src/config/user_config.rs` — add `mask_ip: bool` field
- `src/config/settings.rs` — add `SettingKey::MaskIp` if exposed in settings UI

## Recommendation

1. Add `mask_ip: bool` to `UserConfig.ui` (default: `false`)
2. In `render_header`, replace IP with `***.**.***.***` when `mask_ip` is `true`
3. Optionally expose as toggle in Settings view (Tools → Settings)

## Severity

🟡 MEDIUM — Security concern, prevents IP exposure in screenshots/demos

## Labels

`feature` `security` `ui`
