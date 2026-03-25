# issue066: Code duplication in toggle/connect methods

## Summary

Multiple nearly identical toggle and connect methods violate DRY principle.

## Problem

### 1. Toggle Methods (`src/vpn/client.rs:425-472`)

Six toggle methods with identical structure:

```rust
pub fn toggle_killswitch(&self, current: Option<i32>) -> AppResult<String> { ... }
pub fn toggle_ipv6(&self, current: Option<bool>) -> AppResult<String> { ... }
pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> { ... }
pub fn toggle_vpn_accelerator(&self, current: Option<bool>) -> AppResult<String> { ... }
pub fn toggle_port_forwarding(&self, current: Option<bool>) -> AppResult<String> { ... }
pub fn toggle_anonymous_crash_reports(&self, current: Option<bool>) -> AppResult<String> { ... }
```

### 2. Connect Methods (`src/state/app_state_impl.rs:84-187`)

Five connect methods with identical boilerplate:

```rust
pub fn connect_random(&mut self) { ... }      // Lines 84-103
pub fn connect_fastest(&mut self) { ... }     // Lines 105-124
pub fn connect_p2p(&mut self) { ... }        // Lines 126-145
pub fn connect_tor(&mut self) { ... }        // Lines 147-166
pub fn connect_securecore(&mut self) { ... } // Lines 168-187
```

Each follows the same pattern:
1. Check connection state
2. Set ConnectionState::Connecting
3. Show notification
4. Create channel
5. Spawn async task

## Solution

### For Toggle Methods

Extract generic toggle helper:

```rust
impl VpnClient {
    fn toggle_bool_setting(&self, setting: &str, current: Option<bool>) -> AppResult<String> {
        let new_value = if current == Some(true) { "off" } else { "on" };
        self.set_config(setting, new_value)
    }
    
    pub fn toggle_ipv6(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("ipv6", current)
    }
    
    pub fn toggle_moderate_nat(&self, current: Option<bool>) -> AppResult<String> {
        self.toggle_bool_setting("moderate-nat", current)
    }
    // ... etc
}
```

### For Connect Methods

Extract common connect pattern:

```rust
impl AppState {
    fn connect_special(&mut self, flag: &str, name: &str, notification_key: &str) {
        if self.connection_manager.connection.is_connecting() {
            self.show_notification("Still connecting...".into(), NotificationType::Info, None);
            return;
        }
        
        self.connection_manager.previous_connection = Some(self.connection_manager.connection.clone());
        self.connection_manager.connection = ConnectionState::Connecting;
        self.show_notification(format!("Connecting to {}...", name), NotificationType::Info, Some("connect".into()));
        
        let (tx, rx) = create_channel();
        self.connection_manager.pending_connect.insert((), rx);
        self.connection_manager.async_manager.spawn_connect_special(self.vpn_state.clone(), flag, tx);
    }
    
    pub fn connect_random(&mut self) {
        self.connect_special("--random", "Random Server", "connect");
    }
    
    pub fn connect_p2p(&mut self) {
        self.connect_special("--p2p", "P2P Server", "connect");
    }
    // ... etc
}
```

## Files Affected

- `src/vpn/client.rs` — Add `toggle_bool_setting()` helper
- `src/state/app_state_impl.rs` — Add `connect_special()` helper

## Severity

🟢 **LOW** — Code quality improvement, no functional impact.

## Labels

`refactor` `code-quality` `dry`
