use crate::vpn::City;
use crate::vpn::Server;

pub struct ServerDataState {
    pub servers: Vec<Server>,
    pub is_initialized: bool,
    pub current_cities: Vec<City>,
    pub current_country_code: Option<String>,
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
}
