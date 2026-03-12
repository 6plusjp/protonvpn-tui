//! UI state types

use crate::vpn::Server;
use std::fmt;
use std::sync::RwLock;

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

/// Server cache wrapper - encapsulates cached filtered servers and version
/// Simplifies cache invalidation and retrieval
pub struct ServerCache {
    cache: RwLock<Option<(Vec<Server>, u64)>>,
    version: u64,
}

impl ServerCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(None),
            version: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }

    pub fn get_cached(&self) -> Option<Vec<Server>> {
        match self.cache.read() {
            Ok(c) => c
                .as_ref()
                .map(|(servers, v)| {
                    if *v == self.version {
                        Some(servers.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or(None),
            Err(_) => None,
        }
    }

    pub fn set_cached(&self, servers: Vec<Server>) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some((servers, self.version));
        }
    }

    pub fn try_get_cached(&self) -> Option<Vec<Server>> {
        self.get_cached()
    }

    pub fn try_set_cached(&self, servers: Vec<Server>) {
        self.set_cached(servers);
    }
}

impl Default for ServerCache {
    fn default() -> Self {
        Self::new()
    }
}
