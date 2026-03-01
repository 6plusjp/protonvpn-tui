//! Main application state

use super::{AppView, ConnectionState};
use crate::config::Settings;

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current connection state
    pub connection: ConnectionState,
    /// Current UI view
    pub current_view: AppView,
    /// Search query for server filtering
    pub search_query: String,
    /// Application configuration
    pub config: Settings,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            current_view: AppView::Connect,
            search_query: String::new(),
            config: Settings::default(),
        }
    }

    pub fn with_config(mut self, config: Settings) -> Self {
        self.config = config;
        self
    }

    pub fn switch_view(&mut self) {
        self.current_view = self.current_view.next();
    }
}
