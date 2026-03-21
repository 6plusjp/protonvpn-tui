//! ProtonVPN TUI library
//!
//! Terminal UI for Proton VPN built with [ratatui](https://ratatui.rs/) and [crossterm](https://docs.rs/crossterm).
//!
//! This crate provides the core functionality for the ProtonVPN TUI application,
//! including VPN connection management, server browsing, and terminal rendering.

pub mod config;
pub mod constants;
pub mod error;
pub mod paths;
pub mod state;
pub mod ui;
pub mod vpn;

pub use error::{AppError, AppResult};
pub use state::{AppState, AppView, ConnectionState};
