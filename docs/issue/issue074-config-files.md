# issue074: Configuration files (.gitignore, rustfmt.toml)

## Summary

Configuration files are incomplete — `.gitignore` is minimal and `rustfmt.toml` is missing despite being referenced.

## Problem

### 1. .gitignore — Minimal

Current content:
```
/target
```

Missing common Rust patterns:
- IDE files (`.idea/`, `*.swp`, `*.swo`)
- OS files (`.DS_Store`, `Thumbs.db`)
- Log files (`*.log`)
- Environment files (`.env`)

### 2. rustfmt.toml — Missing

- `Cargo.toml` exclude list references `rustfmt.toml` but the file doesn't exist
- `cargo fmt` uses Rust defaults, but project-specific formatting rules aren't documented
- Without explicit config, contributors might have different editor settings

## Solution

### 1. Update .gitignore

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

# cargo-edit
Cargo.lock

# Temporary
/tmp/
```

### 2. Create rustfmt.toml

```toml
# Rustfmt configuration for protonvpn-tui

# Use 2021 edition style
edition = "2021"

# Maximum line width
max_width = 100

# Indentation
tab_spaces = 4

# Imports
imports_granularity = "Module"
group_imports = "StdExternalCrate"

# Comments
wrap_comments = true
normalize_comments = true

# Formatting
use_field_init_shorthand = true
use_try_shorthand = true
```

## Acceptance Criteria

- [ ] `.gitignore` includes IDE, OS, log, and environment patterns
- [ ] `rustfmt.toml` created with project-specific formatting rules
- [ ] `cargo fmt --check` still passes (no formatting changes)
- [ ] Exclude list in `Cargo.toml` is correct
