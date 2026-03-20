//! Server filtering cache

use crate::vpn::Server;
use std::sync::RwLock;

/// Cache for filtered server lists
///
/// Invalidates when filter/sort/favorite settings change.
pub struct FilteredServerCache {
    cache: RwLock<Option<(Vec<Server>, u64)>>,
    version: u64,
}

impl FilteredServerCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(None),
            version: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.version = self.version.wrapping_add(1);
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
}

impl Default for FilteredServerCache {
    fn default() -> Self {
        Self::new()
    }
}
