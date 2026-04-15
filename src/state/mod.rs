//! Application state management

mod app_state;
pub use app_state::*;

mod connection_state;
pub use connection_state::*;

mod app_view;
pub use app_view::*;

mod server_filter;
pub use server_filter::*;

mod server_sort;
pub use server_sort::*;

mod log_persistence;
pub use log_persistence::*;

mod ui_state;
pub use ui_state::*;

mod notifications;
pub use notifications::*;

mod connection_manager;
pub use connection_manager::*;

mod config_state;
pub use config_state::*;

mod server_cache;
pub use server_cache::*;

// Extracted modules from app_state.rs
mod app_state_impl;
mod event_handler;
mod navigation;
pub use navigation::SelectionTarget;
mod server_ops;
mod settings_ops;
