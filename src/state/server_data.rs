//! Server data state management

use crate::vpn::City;
use crate::vpn::Server;

/// Server data state - contains all server-related fields
/// This struct can be extracted to server_data.rs in Phase 2.3
pub struct ServerDataState {
    pub(crate) servers: Vec<Server>,
    pub(crate) is_initialized: bool,
    pub(crate) current_cities: Vec<City>,
    pub(crate) current_country_code: Option<String>,
}

impl Default for ServerDataState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerDataState {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            is_initialized: false,
            current_cities: Vec::new(),
            current_country_code: None,
        }
    }

    // === Getters ===

    /// Get all servers
    pub fn get_servers(&self) -> &Vec<Server> {
        &self.servers
    }

    /// Check if state is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Get current cities
    pub fn get_current_cities(&self) -> &Vec<City> {
        &self.current_cities
    }

    /// Get current country code
    pub fn get_current_country_code(&self) -> &Option<String> {
        &self.current_country_code
    }

    /// Get mutable reference to current cities
    pub fn current_cities_mut(&mut self) -> &mut Vec<City> {
        &mut self.current_cities
    }

    /// Get mutable reference to current country code
    pub fn current_country_code_mut(&mut self) -> &mut Option<String> {
        &mut self.current_country_code
    }

    // === Setters ===

    /// Set servers
    pub fn set_servers(&mut self, servers: Vec<Server>) {
        self.servers = servers;
    }

    /// Set initialized state
    pub fn set_initialized(&mut self, initialized: bool) {
        self.is_initialized = initialized;
    }

    /// Clear current cities
    pub fn clear_cities(&mut self) {
        self.current_cities.clear();
    }

    /// Set current country code
    pub fn set_country_code(&mut self, code: Option<String>) {
        self.current_country_code = code;
    }
}
