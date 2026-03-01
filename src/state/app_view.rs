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
    /// Help view (only accessible via ?)
    Help,
}

impl AppView {
    /// Get next view in cycle (excludes Help)
    pub fn next(&self) -> Self {
        match self {
            Self::Connect => Self::Stats,
            Self::Stats => Self::Settings,
            Self::Settings => Self::Connect,
            Self::Help => Self::Connect,
        }
    }

    /// Get previous view in cycle (excludes Help)
    pub fn prev(&self) -> Self {
        match self {
            Self::Connect => Self::Settings,
            Self::Stats => Self::Connect,
            Self::Settings => Self::Stats,
            Self::Help => Self::Connect,
        }
    }
}
