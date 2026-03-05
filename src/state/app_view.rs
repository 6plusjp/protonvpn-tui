//! Application views

use serde::{Deserialize, Serialize};

/// Represents the current UI view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AppView {
    /// Server list / country selection view
    #[default]
    Servers,
    /// Statistics view (btop-like)
    Stats,
    /// Settings view
    Settings,
    /// Help view (only accessible via ?)
    Help,
    /// City list for selected country
    Cities,
}

impl AppView {
    /// Get next view in cycle (excludes Help)
    pub fn next(&self) -> Self {
        match self {
            Self::Servers => Self::Stats,
            Self::Stats => Self::Settings,
            Self::Settings => Self::Servers,
            Self::Help => Self::Servers,
            Self::Cities => Self::Servers,
        }
    }

    /// Get previous view in cycle (excludes Help)
    pub fn prev(&self) -> Self {
        match self {
            Self::Servers => Self::Settings,
            Self::Stats => Self::Servers,
            Self::Settings => Self::Stats,
            Self::Help => Self::Servers,
            Self::Cities => Self::Servers,
        }
    }
}
