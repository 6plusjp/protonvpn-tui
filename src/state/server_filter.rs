//! Server filter mode

use serde::{Deserialize, Serialize};

/// Filter mode for server search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ServerFilter {
    #[default]
    Code,
    Country,
    City,
}

impl ServerFilter {
    pub fn next(&self) -> Self {
        match self {
            ServerFilter::Code => ServerFilter::Country,
            ServerFilter::Country => ServerFilter::City,
            ServerFilter::City => ServerFilter::Code,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ServerFilter::Code => "Code",
            ServerFilter::Country => "Country",
            ServerFilter::City => "City",
        }
    }
}
