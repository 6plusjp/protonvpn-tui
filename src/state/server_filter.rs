//! Server filter mode

use serde::{Deserialize, Serialize};

/// Filter mode for server search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServerFilter {
    #[default]
    Id,
    Country,
    City,
}

impl ServerFilter {
    pub fn next(&self) -> Self {
        match self {
            ServerFilter::Id => ServerFilter::Country,
            ServerFilter::Country => ServerFilter::City,
            ServerFilter::City => ServerFilter::Id,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServerFilter::Id => "ID",
            ServerFilter::Country => "Country",
            ServerFilter::City => "City",
        }
    }
}
