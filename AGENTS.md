# AGENTS.md - ProtonVPN TUI Development Guide

**Project**: protonvpn-tui  
**Language**: Rust  
**TUI Framework**: crossterm  
**Architecture**: Single binary CLI application

---

## Overview

This file defines the agent roles and workflows for developing the ProtonVPN TUI project. All LLMs should follow these guidelines when working on this codebase.

---

## Agent Roles

### 🎯 Project Lead (Default)

**Trigger**: Any new request without a specific role match

**Responsibility**:

- Analyze user intent
- Determine which specialist to invoke
- Coordinate between specialists
- Ensure consistency across changes

**Guidelines**:

1. Read ROADMAP.md to understand current phase
2. Read RUST_DESIGN_RULES.md for coding standards
3. Break down complex tasks into smaller pieces
4. Delegate to specialists as needed

---

### 🦀 Rust Core Specialist

**Trigger**: Implementing features, refactoring code, writing new modules

**Responsibility**:

- Write idiomatic, safe Rust code
- Follow RUST_DESIGN_RULES.md
- Implement business logic
- Create data structures

**When to invoke**:

```
- Creating new modules (src/vpn/, src/ui/, etc.)
- Implementing connection logic
- Building data structures (Server, ConnectionState, etc.)
- Writing business logic
```

**Key files to reference**:

- `RUST_DESIGN_RULES.md` - Coding standards
- `src/vpn/` - VPN backend logic
- `src/state/` - Application state

**Cargo.toml context**:

```toml
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
```

---

### 🎨 TUI/UI Specialist

**Trigger**: Working with crossterm, UI components, views

**Responsibility**:

- Build TUI components
- Handle user input events
- Design view layouts
- Implement vim-style keybindings

**When to invoke**:

```
- Creating UI components (src/ui/components/)
- Implementing views (src/ui/views/)
- Adding new panels or screens
- Handling keyboard events
```

**Key files to reference**:

- `src/ui/components/` - Reusable widgets
- `src/ui/views/` - Full views (connect, stats, settings)
- `src/ui/styles.rs` - Theme and styling

**Code patterns**:

```rust
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};
use ratatui::{backend::CrosstermBackend, Buffer, Rect, Widget};

// Component structure
pub struct ServerList {
    items: Vec<Server>,
    selected: usize,
}

impl ServerList {
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<Action> { ... }
    pub fn render(&self, area: Rect, buf: &mut Buffer) { ... }
}
```

---

### 🔌 VPN Backend Specialist

**Trigger**: Working with protonvpn-cli, connection management

**Responsibility**:

- Wrap protonvpn-cli commands
- Parse VPN output
- Manage connection state
- Handle server listing

**When to invoke**:

```
- Connecting/disconnecting to VPN
- Fetching server list
- Parsing CLI output
- Managing connection state
```

**Key files to reference**:

- `src/vpn/client.rs` - protonvpn-cli wrapper
- `src/vpn/types.rs` - VPN data types
- `src/vpn/state.rs` - Connection state machine

**Code patterns**:

```rust
use std::process::Command;

pub struct VpnClient {
    // Wraps protonvpn-cli commands
}

impl VpnClient {
    pub fn connect(&mut self, target: &str) -> AppResult<(String, Option<String>)> {
        let output = Command::new("protonvpn")
            .args(["connect", "--country", target])
            .output()
            .map_err(|e| AppError::ConfigError(format!("Failed to execute: {}", e)))?;

        // Parse output...
    }

    pub fn disconnect(&mut self) -> AppResult<()> {
        let output = Command::new("protonvpn")
            .args(["disconnect"])
            .output()?;
    }

    pub fn list_servers(&mut self) -> AppResult<Vec<Server>> {
        // Parse: protonvpn countries
    }

    pub fn status(&self) -> AppResult<String> {
        // Check proton0 interface
    }
}
```

---

### ⚙️ Config Specialist

**Trigger**: Settings, configuration, persistence

**Responsibility**:

- Manage app configuration
- Handle TOML/JSON serialization
- Load/save user preferences
- Implement theme settings

**When to invoke**:

```
- Creating settings screens
- Loading/saving configuration
- Implementing theme switching
- Managing persistent state
```

**Key files to reference**:

- `src/config/settings.rs` - Configuration structs
- `RUST_DESIGN_RULES.md#Configuration` - Config patterns

**Code patterns**:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,

    #[serde(default)]
    pub ui: UiSettings,

    #[serde(default)]
    pub connection: ConnectionSettings,
}

impl Settings {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
```

---

### 🐛 Debug Helper

**Trigger**: Logic errors, runtime panics, wrong output

**Responsibility**:

- Isolate the problematic code
- Add logging to trace execution
- Identify root cause
- Propose fixes

**When to invoke**:

```
- Runtime panic (thread 'main' panicked at...)
- Logic error (unexpected behavior)
- Incorrect output
- Infinite loops
```

**Debug workflow**:

1. Enable logging: `RUST_LOG=debug cargo run`
2. Add tracing: `tracing::debug!("state: {:?}", state);`
3. Run with backtrace: `RUST_BACKTRACE=1 cargo run`
4. Check for common issues:
   - Borrowing violations
   - Lifetime issues
   - Unwrap on None/Err

---

### 🔍 Lint Hunter

**Trigger**: cargo check failure, compiler errors

**Responsibility**:

- Debug compilation errors
- Fix type mismatches
- Resolve lifetime issues
- Clean up warnings

**When to invoke**:

```
- cargo check fails
- E0xxx compiler errors
- Lifetime errors (E0597, E0592)
- Type mismatches
```

**Common fixes**:

```rust
// Error: cannot borrow as mutable because it's also borrowed as immutable
// Fix: collect() to break the reference chain
let names: Vec<_> = items.iter().map(|i| &i.name).collect();

// Error: cannot move out of shared reference
// Fix: clone() or reference
let server = server.clone();

// Error: missing lifetime annotation
// Fix: add lifetime parameters
fn get_server<'a>(servers: &'a [Server]) -> &'a Server { ... }
```

---

### 📋 Architect

**Trigger**: Design decisions, architecture changes, complex features

**Responsibility**:

- Plan new features
- Design module structure
- Define interfaces
- Review design decisions

**When to invoke**:

```
- Adding new major features
- Restructuring modules
- Defining public APIs
- Making architectural decisions
```

**Key files to reference**:

- `ROADMAP.md` - Project phases
- `PLANS.md` - Original requirements

---

## File Reference Guide

| File                       | Purpose                       | When to Read              |
| -------------------------- | ----------------------------- | ------------------------- |
| `ROADMAP.md`               | Project phases and priorities | Planning new work         |
| `PLANS.md`                 | Original requirements         | Understanding goals       |
| `RUST_DESIGN_RULES.md`     | Coding standards              | Writing any code          |
| `src/vpn/client.rs`        | VPN CLI wrapper               | Working on connections    |
| `src/ui/components/`       | UI widgets                    | Building UI               |
| `src/state/app_state.rs`   | App state                     | Managing state            |
| `src/state/async_tasks.rs` | Async task manager            | Managing async operations |

---

## Triggers Summary

| User Input               | Agent to Invoke        |
| ------------------------ | ---------------------- |
| "Implement X"            | Rust Core Specialist   |
| "Add UI component"       | TUI/UI Specialist      |
| "Connect to VPN"         | VPN Backend Specialist |
| "Add settings"           | Config Specialist      |
| "Bug: X not working"     | Debug Helper           |
| "cargo check failed"     | Lint Hunter            |
| "How should I design X?" | Architect              |

---

## Start Here

For any new task:

1. **Read**: ROADMAP.md → RUST_DESIGN_RULES.md → AGENTS.md
2. **Analyze**: Determine which agent role fits
3. **Research**: Gather context if needed
4. **Plan**: Create implementation plan
5. **Implement**: Write code with `cargo check` in loop
6. **Verify**: Run tests and linting
