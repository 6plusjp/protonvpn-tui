//! UI state types

use std::fmt;

/// Input mode for text input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Filter input mode (/)
    Filter,
    /// DNS input mode (for custom DNS)
    DnsInput,
}

/// Search query wrapper - encapsulates query and lowercase version
/// Ensures query_lower is always in sync with query
#[derive(Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub query_lower: String,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, query: String) {
        self.query_lower = query.to_lowercase();
        self.query = query;
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.query_lower.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.query
    }
}

impl fmt::Debug for SearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchQuery")
            .field("query", &self.query)
            .finish()
    }
}
