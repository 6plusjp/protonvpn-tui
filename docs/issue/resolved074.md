# issue074: Configuration files (.gitignore, rustfmt.toml)

## Summary

**RESOLVED** - Configuration files were incomplete. Updated `.gitignore` with common patterns and created `rustfmt.toml`.

**Resolved**: 2026-03-25
**Resolved by**: Sisyphus (AI agent)

## Problem

### 1. .gitignore — Minimal

Only contained `/target`. Missing:
- IDE files (.idea/, *.swp, .vscode/)
- OS files (.DS_Store, Thumbs.db)
- Log files (*.log)
- Environment files (.env)

### 2. rustfmt.toml — Missing

`cargo fmt` used defaults but project-specific rules weren't documented.

## Solution

### 1. Updated .gitignore

```gitignore
# Build
/target

# IDE
.idea/
*.swp
*.swo
*~
.vscode/

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Environment
.env
.env.*

# Temporary
/tmp/
```

### 2. Created rustfmt.toml

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

Note: Removed unstable options (`imports_granularity`, `group_imports`, `wrap_comments`, `normalize_comments`) that require nightly.

## Acceptance Criteria

- [x] `.gitignore` includes IDE, OS, log, and environment patterns
- [x] `rustfmt.toml` created with stable formatting rules
- [x] `cargo fmt --check` passes with no warnings
