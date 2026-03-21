//! Server filtering cache

use crate::vpn::Server;
use std::cell::RefCell;

pub struct FilteredServerCache {
    cache: RefCell<Option<(Vec<Server>, u64)>>,
    version: u64,
}

impl FilteredServerCache {
    pub fn new() -> Self {
        Self {
            cache: RefCell::new(None),
            version: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.version = self.version.wrapping_add(1);
    }

    pub fn get_cached(&self) -> Option<Vec<Server>> {
        self.cache.borrow().as_ref().and_then(|(servers, v)| {
            if *v == self.version {
                Some(servers.clone())
            } else {
                None
            }
        })
    }

    pub fn set_cached(&self, servers: Vec<Server>) {
        *self.cache.borrow_mut() = Some((servers, self.version));
    }
}

impl Default for FilteredServerCache {
    fn default() -> Self {
        Self::new()
    }
}
