//! ProtonVPN TUI - Library root

pub mod commands;
pub mod config;
pub mod constants;
pub mod error;
pub mod paths;
pub mod state;
pub mod ui;
pub mod vpn;

pub use error::{AppError, AppResult};
pub use state::{AppState, AppView, ConnectionState};
