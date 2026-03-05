# Rust Design Rules - LLM Reference

## Project Structure

```
protonvpn-tui/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── app.rs               # Main app state & logic
│   ├── error.rs             # Custom error types
│   ├── commands/            # Command handlers
│   │   ├── mod.rs
│   │   ├── connect.rs
│   │   ├── disconnect.rs
│   │   └── ...
│   ├── state/               # App state management
│   │   ├── mod.rs
│   │   └── app_state.rs
│   ├── ui/                  # TUI components
│   │   ├── mod.rs
│   │   ├── components/      # Reusable widgets
│   │   │   ├── mod.rs
│   │   │   ├── server_list.rs
│   │   │   ├── status_bar.rs
│   │   │   └── ...
│   │   ├── views/           # Full views
│   │   │   ├── mod.rs
│   │   │   ├── connect.rs
│   │   │   ├── stats.rs
│   │   │   └── settings.rs
│   │   └── styles.rs        # Theme & styles
│   ├── vpn/                 # VPN backend
│   │   ├── mod.rs
│   │   ├── client.rs        # protonvpn-cli wrapper
│   │   ├── types.rs         # Data types
│   │   └── state.rs         # Connection state
│   └── config/              # Configuration
│       ├── mod.rs
│       └── settings.rs
├── Cargo.toml
└── README.md
```

---

## Error Handling

### Rule: Use `thiserror` for library errors, `anyhow` for application errors

**ProtonVPN-TUIはCLIアプリなので `anyhow` をメインで使用**

```rust
// error.rs - Main error type for the application
use anyhow::{Context, Result};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("VPN connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication error: {0}")]
    AuthFailed(String),
    
    #[error("Server not found: {0}")]
    ServerNotFound(String),
    
    #[error("Command execution failed: {0}")]
    CommandFailed(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

// For library-like reusable components, define specific errors
#[derive(Debug, thiserror::Error)]
pub enum VpnError {
    #[error("Not connected")]
    NotConnected,
    
    #[error("Already connected")]
    AlreadyConnected,
    
    #[error("Invalid server: {0}")]
    InvalidServer(String),
}
```

### Rule: Always use `?` operator, never `.unwrap()` in production

```rust
// GOOD
fn connect_to_server(server: &str) -> AppResult<Connection> {
    let output = Command::new("protonvpn-cli")
        .args(["-c", server])
        .output()
        .context("Failed to execute protonvpn-cli")?;
    
    parse_connection(output)
}

// BAD
fn connect_to_server(server: &str) -> Connection {
    let output = Command::new("protonvpn-cli")
        .args(["-c", server])
        .output()
        .unwrap(); // Never do this!
    output
}
```

### Rule: Add context to errors

```rust
// GOOD - shows WHERE the error happened
let config = fs::read_to_string("config.json")
    .context("Failed to read config file")?;

// GOOD - shows WHAT was being processed
let server = servers.iter()
    .find(|s| s.id == id)
    .context(format!("Server with id {} not found", id))?;
```

---

## State Management

### Rule: Use enums for finite states

```rust
// state.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { server: String, ip: String },
    Disconnecting,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppView {
    Connect,
    Stats,
    Settings,
    Help,
}

// App state combines all state
pub struct AppState {
    pub connection: ConnectionState,
    pub current_view: AppView,
    pub servers: Vec<Server>,
    pub selected_server: Option<usize>,
    pub search_query: String,
    pub config: Config,
}
```

### Rule: Use builder pattern for complex state initialization

```rust
impl AppState {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            current_view: AppView::Connect,
            servers: Vec::new(),
            selected_server: None,
            search_query: String::new(),
            config: Config::default(),
        }
    }
    
    pub fn with_servers(mut self, servers: Vec<Server>) -> Self {
        self.servers = servers;
        self
    }
    
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
}
```

---

## Async / Concurrency

### Rule: Use `tokio` for async runtime in CLI apps

```rust
// Cargo.toml
tokio = { version = "1", features = ["full"] }

#[tokio::main]
async fn main() -> AppResult<()> {
    // App initialization
    let mut app = AppState::new();
    
    // Run TUI in tokio task
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    
    tokio::spawn(async move {
        run_tui(tx, rx).await;
    });
    
    // Background VPN polling
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // Check connection status
    }
}
```

### Rule: Use channels for TUI ↔ Backend communication

```rust
// Commands sent from TUI to VPN backend
enum VpnCommand {
    Connect(String),
    Disconnect,
    GetServers,
    GetStats,
}

// Events sent from VPN backend to TUI
enum VpnEvent {
    StateChanged(ConnectionState),
    StatsUpdated(ConnectionStats),
    Error(String),
}
```

---

## CLI / Command Pattern

### Rule: Use `clap` for argument parsing

```rust
// Cargo.toml
clap = { version = "4", features = ["derive"] }

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "protonvpn-tui")]
#[command(about = "Proton VPN TUI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    #[arg(short, long, default_value = "false")]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Connect to a server
    Connect { server: Option<String> },
    
    /// Disconnect from VPN
    Disconnect,
    
    /// Show connection statistics
    Stats,
    
    /// List available servers
    Servers,
}
```

---

## TUI / crossterm

### Rule: Separate UI logic from business logic

```rust
// ui/components/server_list.rs
pub struct ServerList {
    items: Vec<Server>,
    selected: usize,
    scroll: usize,
}

impl ServerList {
    pub fn new(servers: Vec<Server>) -> Self {
        Self {
            items: servers,
            selected: 0,
            scroll: 0,
        }
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ServerListAction> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = self.selected.saturating_add(1);
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                None
            }
            KeyCode::Enter => Some(ServerListAction::Select(self.selected)),
            _ => None,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Render logic here
    }
}
```

### Rule: Use `crossterm` events, not polling

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

fn handle_events(app: &mut AppState) -> bool {
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        match event::read().unwrap() {
            Event::Key(KeyEvent { kind: KeyEventKind::Press, code, .. }) => {
                match code {
                    KeyCode::Char('q') => return false, // Exit
                    KeyCode::Char('c') if event::KeyModifiers::CONTROL => return false,
                    _ => app.handle_key(code),
                }
            }
            _ => {}
        }
    }
    true
}
```

---

## Data Types

### Rule: Use `serde` for serialization

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: String,
    pub load: u8,        // 0-100%
    pub ping: Option<u32>,
    pub features: ServerFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerFeatures {
    pub secure_core: bool,
    pub p2p: bool,
    pub tor: bool,
    pub streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_at: DateTime<Utc>,
    pub server_ip: String,
    pub protocol: String,
}
```

### Rule: Derive common traits

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
// Debug - for debugging
// Clone - for ownership transfer
// PartialEq/Eq - for comparison
// Hash - for use in HashMap
// Default - for default values
pub struct Config {
    pub theme: Theme,
    pub keybindings: KeyBindings,
}
```

---

## Logging

### Rule: Use `tracing` for structured logging

```rust
// Cargo.toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

use tracing::{info, error, warn, debug};

// Initialize
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env()
        .add_directive("protonvpn_tui=info".parse().unwrap()))
    .init();

// Usage
fn connect_vpn(server: &str) -> AppResult<Connection> {
    info!("Connecting to server: {}", server);
    
    let result = do_connect(server);
    
    match result {
        Ok(conn) => {
            info!("Successfully connected to {}", server);
            Ok(conn)
        }
        Err(e) => {
            error!("Failed to connect to {}: {}", server, e);
            Err(e)
        }
    }
}
```

---

## Configuration

### Rule: Use config files with `serde`

```rust
// config/settings.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub general: GeneralSettings,
    
    #[serde(default)]
    pub connection: ConnectionSettings,
    
    #[serde(default)]
    pub ui: UiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            connection: ConnectionSettings::default(),
            ui: UiSettings::default(),
        }
    }
}

impl Settings {
    pub fn load(path: PathBuf) -> AppResult<Self> {
        let content = fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&content)?;
        Ok(settings)
    }
    
    pub fn save(&self, path: PathBuf) -> AppResult<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
```

---

## Testing

### Rule: Write tests for business logic

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_connection_state_transitions() {
        let state = ConnectionState::Disconnected;
        
        // Cannot disconnect when already disconnected
        assert!(!state.can_disconnect());
        
        let connected = ConnectionState::Connected { 
            server: "JP#1".into(),
            ip: "1.2.3.4".into() 
        };
        
        assert!(connected.can_disconnect());
    }
    
    #[test]
    fn test_server_filtering() {
        let servers = vec![
            Server { country: "JP".into(), .. },
            Server { country: "US".into(), .. },
            Server { country: "JP".into(), .. },
        ];
        
        let filtered: Vec<_> = servers.iter()
            .filter(|s| s.country == "JP")
            .collect();
        
        assert_eq!(filtered.len(), 2);
    }
}
```

---

## Dependencies (Minimal Set)

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# TUI
crossterm = "0.27"
crossterm = { version = "0.27", features = ["serde"] }

# Async
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
toml = "0.8"

# DateTime
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"
assert_cmd = "1"
```

---

## Key Patterns Summary

| Pattern | Usage |
|---------|-------|
| `thiserror` | Custom error types (library-like components) |
| `anyhow` | Application-wide error handling |
| `enum` | Finite states (ConnectionState, AppView) |
| `tokio::mpsc` | TUI ↔ Backend communication |
| `clap` | CLI argument parsing |
| `serde` | Data serialization (JSON/TOML) |
| `tracing` | Structured logging |
| Builder pattern | Complex initialization |

---

## Vim-style Keybindings Reference

```rust
// Key binding constants
const KEY_CONNECT: char = 'c';
const KEY_DISCONNECT: char = 'd';
const KEY_QUIT: char = 'q';
const KEY_HELP: char = '?';
const KEY_SEARCH: char = '/';
const KEY_NEXT: char = 'n';
const KEY_PREV: char = 'N';
const KEY_STATS: char = 's';
const KEY_SERVERS: char = 'r';
const KEY_TAB: char = '\t';

// Movement
const KEY_UP: char = 'k';
const KEY_DOWN: char = 'j';
const KEY_LEFT: char = 'h';
const KEY_RIGHT: char = 'l';
```
