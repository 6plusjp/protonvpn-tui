//! Application views

use serde::{Deserialize, Serialize};

/// Represents the current UI view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AppView {
    /// Server list / country selection view
    #[default]
    Servers,
    /// Tools view
    Tools,
    /// Help view (only accessible via ?)
    Help,
}

/// Represents which pane has focus in split-pane view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Pane {
    /// Left pane - countries list (Servers view)
    #[default]
    Countries,
    /// Right pane - cities list (Servers view)
    Cities,
    /// Left pane - settings list (Tools view)
    Settings,
    /// Right pane - logs table (Tools view)
    Logs,
}

impl Pane {
    /// Toggle between panes within the current view
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Countries => Self::Cities,
            Self::Cities => Self::Countries,
            Self::Settings => Self::Logs,
            Self::Logs => Self::Settings,
        };
    }

    /// Check if this pane is a left pane
    pub fn is_left(&self) -> bool {
        matches!(self, Self::Countries | Self::Settings)
    }
}

impl AppView {
    /// Get next view in cycle (excludes Help)
    pub fn next(&self) -> Self {
        match self {
            Self::Servers => Self::Tools,
            Self::Tools => Self::Servers,
            Self::Help => Self::Servers,
        }
    }

    /// Get previous view in cycle (excludes Help)
    pub fn prev(&self) -> Self {
        match self {
            Self::Servers => Self::Tools,
            Self::Tools => Self::Servers,
            Self::Help => Self::Servers,
        }
    }

    /// Get default pane for this view
    pub fn default_pane(&self) -> Pane {
        match self {
            Self::Servers => Pane::Countries,
            Self::Tools => Pane::Settings,
            Self::Help => Pane::Countries,
        }
    }
}
