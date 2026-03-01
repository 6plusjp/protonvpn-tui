//! Application views

use serde::{Deserialize, Serialize};

/// Represents the current UI view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AppView {
    /// Server list / Connect view
    #[default]
    Connect,
    /// Statistics view (btop-like)
    Stats,
    /// Settings view
    Settings,
    /// Help view
    Help,
}

impl AppView {
    pub fn next(&self) -> Self {
        match self {
            Self::Connect => Self::Stats,
            Self::Stats => Self::Settings,
            Self::Settings => Self::Help,
            Self::Help => Self::Connect,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Connect => Self::Help,
            Self::Stats => Self::Connect,
            Self::Settings => Self::Stats,
            Self::Help => Self::Settings,
        }
    }
}
