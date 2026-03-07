//! Application views

use serde::{Deserialize, Serialize};

/// Represents the current UI view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AppView {
    /// Server list / country selection view
    #[default]
    Servers,
    /// Settings view
    Settings,
    /// Logs view (notification history)
    Logs,
    /// Help view (only accessible via ?)
    Help,
    /// City list for selected country (deprecated - use split pane instead)
    #[allow(dead_code)]
    Cities,
}

/// Represents which pane has focus in split-pane view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Pane {
    /// Left pane - countries list
    #[default]
    Countries,
    /// Right pane - cities list
    Cities,
}

impl Pane {
    /// Toggle between panes
    pub fn toggle(&mut self) {
        *self = match self {
            Self::Countries => Self::Cities,
            Self::Cities => Self::Countries,
        };
    }
}

impl AppView {
    /// Get next view in cycle (excludes Help)
    pub fn next(&self) -> Self {
        match self {
            Self::Servers => Self::Settings,
            Self::Settings => Self::Logs,
            Self::Logs => Self::Servers,
            Self::Help => Self::Servers,
            Self::Cities => Self::Servers,
        }
    }

    /// Get previous view in cycle (excludes Help)
    pub fn prev(&self) -> Self {
        match self {
            Self::Servers => Self::Logs,
            Self::Settings => Self::Servers,
            Self::Logs => Self::Settings,
            Self::Help => Self::Servers,
            Self::Cities => Self::Servers,
        }
    }
}
