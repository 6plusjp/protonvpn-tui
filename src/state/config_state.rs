use crate::config::ProtonSettings;

pub struct ConfigState {
    pub proton_settings_cache: Option<ProtonSettings>,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            proton_settings_cache: ProtonSettings::load(),
        }
    }

    pub fn clear_cache(&mut self) {
        self.proton_settings_cache = ProtonSettings::load();
    }

    pub fn invalidate_cache(&mut self) {
        self.proton_settings_cache = None;
    }
}
