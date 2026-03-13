//! Centralized render logic with Renderable trait.
//!
//! This module provides a unified interface for all UI views,
//! replacing scattered render functions with a consistent trait-based approach.

use crate::state::AppState;
use ratatui::{layout::Rect, Frame};

/// Trait for renderable view components.
///
/// All views implement this trait to provide a consistent render API.
/// This unifies the previously inconsistent function signatures across
/// different view modules.
pub trait Renderable {
    /// Render the view to the frame.
    ///
    /// # Arguments
    /// * `state` - Mutable reference to application state
    /// * `f` - Mutable reference to the terminal frame
    /// * `area` - The rectangular area to render within
    fn render(&mut self, state: &mut AppState, f: &mut Frame<'_>, area: Rect);
}

/// View variants representing all available UI views.
///
/// This enum wraps all view-specific state, providing a single type
/// that can implement Renderable. Each variant holds the necessary
/// state for rendering that particular view.
pub enum View {
    Servers(ServersViewState),
    Tools(ToolsViewState),
    Help,
}

/// State for the Servers view (countries + cities panes).
#[derive(Default)]
pub struct ServersViewState {
    pub countries_list_state: ratatui::widgets::TableState,
    pub cities_list_state: ratatui::widgets::TableState,
}

/// State for the Tools view (settings + logs panes).
#[derive(Default)]
pub struct ToolsViewState {
    pub settings_list_state: ratatui::widgets::ListState,
    pub logs_list_state: ratatui::widgets::TableState,
}

impl Renderable for View {
    fn render(&mut self, state: &mut AppState, f: &mut Frame<'_>, area: Rect) {
        match self {
            View::Servers(view_state) => {
                crate::ui::views::servers_view::render_servers_view(
                    state,
                    &mut view_state.countries_list_state,
                    &mut view_state.cities_list_state,
                    f,
                    area,
                );
            }
            View::Tools(view_state) => {
                crate::ui::views::tools_view::render_tools_view(
                    state,
                    &mut view_state.settings_list_state,
                    &mut view_state.logs_list_state,
                    f,
                    area,
                );
            }
            View::Help => {
                crate::ui::views::help_view::render_help_view(state, f, area);
            }
        }
    }
}
