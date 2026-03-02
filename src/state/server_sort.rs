//! Server sort mode

use serde::{Deserialize, Serialize};

/// Sort mode for server list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServerSort {
    #[default]
    Name,
    Country,
    City,
    Id,
}

impl ServerSort {
    pub fn next(&self) -> Self {
        match self {
            ServerSort::Name => ServerSort::Country,
            ServerSort::Country => ServerSort::City,
            ServerSort::City => ServerSort::Id,
            ServerSort::Id => ServerSort::Name,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServerSort::Name => "Name",
            ServerSort::Country => "Country",
            ServerSort::City => "City",
            ServerSort::Id => "ID",
        }
    }
}
