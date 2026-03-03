//! Server sort mode

use serde::{Deserialize, Serialize};

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Asc => SortDirection::Desc,
            SortDirection::Desc => SortDirection::Asc,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortDirection::Asc => "↑",
            SortDirection::Desc => "↓",
        }
    }
}

/// Sort mode for server list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServerSort {
    #[default]
    Id,
    Country,
}

impl ServerSort {
    /// Cycle through all sort combinations: Id+Asc → Id+Desc → Country+Asc → Country+Desc → Id+Asc
    pub fn next(&self) -> Self {
        match self {
            ServerSort::Id => ServerSort::Country,
            ServerSort::Country => ServerSort::Id,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServerSort::Id => "ID",
            ServerSort::Country => "Country",
        }
    }
}
