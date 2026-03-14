use crate::config::{KeyBinding, KeyBindings, KeyModifier};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default = "default_true")]
    pub footer: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: String::new(),
            footer: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingConfig {
    pub code: char,
    pub modifiers: Vec<String>,
}

impl From<KeyBindingConfig> for KeyBinding {
    fn from(cfg: KeyBindingConfig) -> Self {
        let modifiers = match cfg.modifiers.first().map(|s| s.as_str()) {
            Some("Control") => KeyModifier::Control,
            Some("Alt") => KeyModifier::Alt,
            Some("Shift") => KeyModifier::Shift,
            _ => KeyModifier::None,
        };
        KeyBinding::new(cfg.code, modifiers)
    }
}

impl From<&KeyBinding> for KeyBindingConfig {
    fn from(binding: &KeyBinding) -> Self {
        let modifiers = match binding.modifiers {
            KeyModifier::Control => vec!["Control".to_string()],
            KeyModifier::Alt => vec!["Alt".to_string()],
            KeyModifier::Shift => vec!["Shift".to_string()],
            KeyModifier::None => vec![],
        };
        Self {
            code: binding.code,
            modifiers,
        }
    }
}

impl Default for KeyBindingConfig {
    fn default() -> Self {
        Self {
            code: '\0',
            modifiers: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindingsConfig {
    pub navigation_down: KeyBindingConfig,
    pub navigation_up: KeyBindingConfig,
    pub page_down: KeyBindingConfig,
    pub page_up: KeyBindingConfig,
    pub go_first: KeyBindingConfig,
    pub go_last: KeyBindingConfig,
    pub connect: KeyBindingConfig,
    pub disconnect: KeyBindingConfig,
    pub refresh: KeyBindingConfig,
    pub random_connect: KeyBindingConfig,
    pub pane_next: KeyBindingConfig,
    pub pane_prev: KeyBindingConfig,
    pub sort_by_code: KeyBindingConfig,
    pub sort_by_country: KeyBindingConfig,
    pub connect_fastest: KeyBindingConfig,
    pub connect_p2p: KeyBindingConfig,
    pub connect_tor: KeyBindingConfig,
    pub securecore: KeyBindingConfig,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        Self {
            navigation_down: KeyBindingConfig {
                code: 'j',
                modifiers: vec![],
            },
            navigation_up: KeyBindingConfig {
                code: 'k',
                modifiers: vec![],
            },
            page_down: KeyBindingConfig {
                code: 'd',
                modifiers: vec!["Control".to_string()],
            },
            page_up: KeyBindingConfig {
                code: 'u',
                modifiers: vec!["Control".to_string()],
            },
            go_first: KeyBindingConfig {
                code: 'g',
                modifiers: vec![],
            },
            go_last: KeyBindingConfig {
                code: 'G',
                modifiers: vec!["Shift".to_string()],
            },
            connect: KeyBindingConfig {
                code: 'c',
                modifiers: vec![],
            },
            disconnect: KeyBindingConfig {
                code: 'd',
                modifiers: vec![],
            },
            refresh: KeyBindingConfig {
                code: 'r',
                modifiers: vec![],
            },
            random_connect: KeyBindingConfig {
                code: 'x',
                modifiers: vec![],
            },
            pane_next: KeyBindingConfig {
                code: 'l',
                modifiers: vec![],
            },
            pane_prev: KeyBindingConfig {
                code: 'h',
                modifiers: vec![],
            },
            sort_by_code: KeyBindingConfig {
                code: '1',
                modifiers: vec![],
            },
            sort_by_country: KeyBindingConfig {
                code: '2',
                modifiers: vec![],
            },
            connect_fastest: KeyBindingConfig {
                code: 'f',
                modifiers: vec![],
            },
            connect_p2p: KeyBindingConfig {
                code: 'p',
                modifiers: vec![],
            },
            connect_tor: KeyBindingConfig {
                code: 't',
                modifiers: vec![],
            },
            securecore: KeyBindingConfig {
                code: 's',
                modifiers: vec![],
            },
        }
    }
}

impl From<KeyBindingsConfig> for KeyBindings {
    fn from(cfg: KeyBindingsConfig) -> Self {
        KeyBindings {
            navigation_down: cfg.navigation_down.into(),
            navigation_up: cfg.navigation_up.into(),
            page_down: cfg.page_down.into(),
            page_up: cfg.page_up.into(),
            go_first: cfg.go_first.into(),
            go_last: cfg.go_last.into(),
            connect: cfg.connect.into(),
            disconnect: cfg.disconnect.into(),
            refresh: cfg.refresh.into(),
            random_connect: cfg.random_connect.into(),
            pane_next: cfg.pane_next.into(),
            pane_prev: cfg.pane_prev.into(),
            sort_by_code: cfg.sort_by_code.into(),
            sort_by_country: cfg.sort_by_country.into(),
            connect_fastest: cfg.connect_fastest.into(),
            connect_p2p: cfg.connect_p2p.into(),
            connect_tor: cfg.connect_tor.into(),
            securecore: cfg.securecore.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub keybindings: KeyBindingsConfig,
}

impl UserConfig {
    fn get_config_dir() -> Option<PathBuf> {
        if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg_config).join("protonvpn-tui"));
        }
        dirs::config_dir().map(|p| p.join("protonvpn-tui"))
    }

    fn get_config_path() -> Option<PathBuf> {
        Self::get_config_dir().map(|p| p.join("config.toml"))
    }

    pub fn load() -> Self {
        let config_path = match Self::get_config_path() {
            Some(path) => path,
            None => {
                tracing::debug!("Could not determine config directory, using defaults");
                return Self::default();
            }
        };

        if !config_path.exists() {
            tracing::debug!(
                "Config file not found at {}, using defaults",
                config_path.display()
            );
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str::<UserConfig>(&content) {
                Ok(config) => {
                    tracing::info!("Loaded user config from {}", config_path.display());
                    config
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse config file {}: {}, using defaults",
                        config_path.display(),
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!(
                    "Failed to read config file {}: {}, using defaults",
                    config_path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn config_dir() -> Option<PathBuf> {
        Self::get_config_dir()
    }

    pub fn save(&self) {
        let config_path = match Self::get_config_path() {
            Some(path) => path,
            None => {
                tracing::warn!("Could not determine config path for saving");
                return;
            }
        };

        let defaults = Self::default();

        let mut toml_string = String::new();

        let save_theme = !self.ui.theme.is_empty();
        let save_footer = self.ui.footer != defaults.ui.footer;

        if save_theme || save_footer {
            toml_string.push_str("[ui]\n");
            if save_theme {
                toml_string.push_str(&format!("theme = \"{}\"\n", self.ui.theme));
            }
            if save_footer {
                toml_string.push_str(&format!("footer = {}\n", self.ui.footer));
            }
        }

        // Save keybindings if different from defaults
        let keybindings_different = self.keybindings.navigation_down.code
            != defaults.keybindings.navigation_down.code
            || self.keybindings.navigation_up.code != defaults.keybindings.navigation_up.code
            || self.keybindings.connect.code != defaults.keybindings.connect.code
            || self.keybindings.disconnect.code != defaults.keybindings.disconnect.code;

        if keybindings_different {
            if !toml_string.is_empty() {
                toml_string.push('\n');
            }
            toml_string.push_str("[keybindings]\n");

            if self.keybindings.navigation_down.code != defaults.keybindings.navigation_down.code {
                toml_string.push_str(&format!(
                    "navigation_down = {{ code = \"{}\", modifiers = {:?} }}\n",
                    self.keybindings.navigation_down.code,
                    self.keybindings.navigation_down.modifiers
                ));
            }
            if self.keybindings.navigation_up.code != defaults.keybindings.navigation_up.code {
                toml_string.push_str(&format!(
                    "navigation_up = {{ code = \"{}\", modifiers = {:?} }}\n",
                    self.keybindings.navigation_up.code, self.keybindings.navigation_up.modifiers
                ));
            }
            if self.keybindings.connect.code != defaults.keybindings.connect.code {
                toml_string.push_str(&format!(
                    "connect = {{ code = \"{}\", modifiers = {:?} }}\n",
                    self.keybindings.connect.code, self.keybindings.connect.modifiers
                ));
            }
            if self.keybindings.disconnect.code != defaults.keybindings.disconnect.code {
                toml_string.push_str(&format!(
                    "disconnect = {{ code = \"{}\", modifiers = {:?} }}\n",
                    self.keybindings.disconnect.code, self.keybindings.disconnect.modifiers
                ));
            }
        }

        if toml_string.is_empty() {
            tracing::debug!("No settings to save (all defaults)");
            return;
        }

        if let Some(config_dir) = config_path.parent() {
            if let Err(e) = std::fs::create_dir_all(config_dir) {
                tracing::warn!("Failed to create config directory: {}", e);
                return;
            }
        }

        if let Err(e) = std::fs::write(&config_path, toml_string) {
            tracing::warn!("Failed to write config: {}", e);
        } else {
            tracing::debug!("Saved config to {}", config_path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert_eq!(config.ui.theme, "");
        assert_eq!(config.ui.footer, true);
    }

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[ui]
theme = "Nord"
footer = false

[keybindings]
navigation_down = { code = "j", modifiers = [] }
navigation_up = { code = "k", modifiers = [] }
"#;
        let config: UserConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.ui.theme, "Nord");
        assert_eq!(config.ui.footer, false);
        assert_eq!(config.keybindings.navigation_down.code, 'j');
    }
}
