use crate::config::{KeyBinding, KeyBindings, KeyModifier};
use crate::paths;
use crate::ui::{KeyArrow, KeyMap, KeyMatcher};
use crossterm::event::KeyModifiers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default = "default_true")]
    pub footer: bool,
    #[serde(default)]
    pub favorites: Vec<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: String::new(),
            footer: true,
            favorites: Vec::new(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum KeyMatcherConfig {
    Char(char),
    CharWithMod { code: char, modifiers: Vec<String> },
    Arrow(String),
    DoubleChar(char),
}

impl KeyMatcherConfig {
    fn parse_modifiers(modifiers: &[String]) -> KeyModifier {
        match modifiers.first().map(|s| s.as_str()) {
            Some("Control") => KeyModifier::Control,
            Some("Alt") => KeyModifier::Alt,
            Some("Shift") => KeyModifier::Shift,
            _ => KeyModifier::None,
        }
    }
}

impl From<KeyBindingConfig> for KeyMatcherConfig {
    fn from(cfg: KeyBindingConfig) -> Self {
        let mods = KeyMatcherConfig::parse_modifiers(&cfg.modifiers);
        if mods == KeyModifier::None {
            KeyMatcherConfig::Char(cfg.code)
        } else {
            KeyMatcherConfig::CharWithMod {
                code: cfg.code,
                modifiers: cfg.modifiers,
            }
        }
    }
}

impl From<&KeyBinding> for KeyMatcherConfig {
    fn from(binding: &KeyBinding) -> Self {
        if binding.modifiers == KeyModifier::None {
            KeyMatcherConfig::Char(binding.code)
        } else {
            let modifiers = match binding.modifiers {
                KeyModifier::Control => vec!["Control".to_string()],
                KeyModifier::Alt => vec!["Alt".to_string()],
                KeyModifier::Shift => vec!["Shift".to_string()],
                KeyModifier::None => vec![],
            };
            KeyMatcherConfig::CharWithMod {
                code: binding.code,
                modifiers,
            }
        }
    }
}

impl KeyMatcherConfig {
    pub fn to_keymatcher(&self) -> KeyMatcher {
        match self {
            KeyMatcherConfig::Char(c) => KeyMatcher::Char(*c),
            KeyMatcherConfig::CharWithMod { code, modifiers } => {
                KeyMatcher::CharWithMod(*code, KeyMatcherConfig::parse_modifiers(modifiers).into())
            }
            KeyMatcherConfig::Arrow(dir) => {
                let arrow = match dir.to_lowercase().as_str() {
                    "up" => KeyArrow::Up,
                    "down" => KeyArrow::Down,
                    "left" => KeyArrow::Left,
                    "right" => KeyArrow::Right,
                    _ => KeyArrow::Down,
                };
                KeyMatcher::Arrow(arrow)
            }
            KeyMatcherConfig::DoubleChar(c) => KeyMatcher::DoubleChar(*c),
        }
    }

    pub fn from_keymatcher(m: &KeyMatcher) -> Self {
        match m {
            KeyMatcher::Char(c) => KeyMatcherConfig::Char(*c),
            KeyMatcher::CharWithMod(c, mods) => {
                let modifiers = match *mods {
                    KeyModifiers::CONTROL => vec!["Control".to_string()],
                    KeyModifiers::ALT => vec!["Alt".to_string()],
                    KeyModifiers::SHIFT => vec!["Shift".to_string()],
                    _ => vec![],
                };
                KeyMatcherConfig::CharWithMod {
                    code: *c,
                    modifiers,
                }
            }
            KeyMatcher::Arrow(dir) => {
                let dir_str = match dir {
                    KeyArrow::Up => "Up",
                    KeyArrow::Down => "Down",
                    KeyArrow::Left => "Left",
                    KeyArrow::Right => "Right",
                };
                KeyMatcherConfig::Arrow(dir_str.to_string())
            }
            KeyMatcher::DoubleChar(c) => KeyMatcherConfig::DoubleChar(*c),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct KeyBindingsConfig {
    pub navigation_down: Vec<KeyMatcherConfig>,
    pub navigation_up: Vec<KeyMatcherConfig>,
    pub page_down: Vec<KeyMatcherConfig>,
    pub page_up: Vec<KeyMatcherConfig>,
    pub go_first: Vec<KeyMatcherConfig>,
    pub go_last: Vec<KeyMatcherConfig>,
    pub connect: Vec<KeyMatcherConfig>,
    pub disconnect: Vec<KeyMatcherConfig>,
    pub refresh: Vec<KeyMatcherConfig>,
    pub random_connect: Vec<KeyMatcherConfig>,
    pub pane_next: Vec<KeyMatcherConfig>,
    pub pane_prev: Vec<KeyMatcherConfig>,
    pub sort_by_code: Vec<KeyMatcherConfig>,
    pub sort_by_country: Vec<KeyMatcherConfig>,
    pub connect_fastest: Vec<KeyMatcherConfig>,
    pub connect_p2p: Vec<KeyMatcherConfig>,
    pub connect_tor: Vec<KeyMatcherConfig>,
    pub securecore: Vec<KeyMatcherConfig>,
    pub search: Vec<KeyMatcherConfig>,
    pub cancel: Vec<KeyMatcherConfig>,
    pub help: Vec<KeyMatcherConfig>,
    pub quit: Vec<KeyMatcherConfig>,
    pub next_setting: Vec<KeyMatcherConfig>,
    pub prev_setting: Vec<KeyMatcherConfig>,
    pub toggle_setting: Vec<KeyMatcherConfig>,
    pub select_city: Vec<KeyMatcherConfig>,
    pub refresh_cities: Vec<KeyMatcherConfig>,
    pub toggle_favorite: Vec<KeyMatcherConfig>,
}

impl Default for KeyBindingsConfig {
    fn default() -> Self {
        let km = KeyMap::default();
        Self {
            navigation_down: km
                .down
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            navigation_up: km
                .up
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            page_down: km
                .page_down
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            page_up: km
                .page_up
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            go_first: km
                .go_first
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            go_last: km
                .go_last
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            connect: km
                .connect
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            disconnect: km
                .disconnect
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            refresh: km
                .refresh
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            random_connect: km
                .random_connect
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            pane_next: km
                .pane_next
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            pane_prev: km
                .pane_prev
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            sort_by_code: km
                .sort_by_code
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            sort_by_country: km
                .sort_by_country
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            connect_fastest: km
                .connect_fastest
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            connect_p2p: km
                .connect_p2p
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            connect_tor: km
                .connect_tor
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            securecore: km
                .securecore
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            search: km
                .search
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            cancel: km
                .cancel
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            help: km
                .help
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            quit: km
                .quit
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            next_setting: km
                .next_setting
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            prev_setting: km
                .prev_setting
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            toggle_setting: km
                .toggle_setting
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            select_city: km
                .select_city
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            refresh_cities: km
                .refresh_cities
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
            toggle_favorite: km
                .toggle_favorite
                .iter()
                .map(KeyMatcherConfig::from_keymatcher)
                .collect(),
        }
    }
}

impl From<KeyBindingsConfig> for KeyBindings {
    fn from(cfg: KeyBindingsConfig) -> Self {
        fn first_to_binding(configs: Vec<KeyMatcherConfig>) -> KeyBinding {
            configs
                .into_iter()
                .next()
                .map(|c| match c {
                    KeyMatcherConfig::Char(c) => KeyBinding::new(c, KeyModifier::None),
                    KeyMatcherConfig::CharWithMod { code, modifiers } => {
                        KeyBinding::new(code, KeyMatcherConfig::parse_modifiers(&modifiers))
                    }
                    KeyMatcherConfig::Arrow(_) | KeyMatcherConfig::DoubleChar(_) => {
                        KeyBinding::new(' ', KeyModifier::None)
                    }
                })
                .unwrap_or_else(|| KeyBinding::new(' ', KeyModifier::None))
        }
        KeyBindings {
            navigation_down: first_to_binding(cfg.navigation_down),
            navigation_up: first_to_binding(cfg.navigation_up),
            page_down: first_to_binding(cfg.page_down),
            page_up: first_to_binding(cfg.page_up),
            go_first: first_to_binding(cfg.go_first),
            go_last: first_to_binding(cfg.go_last),
            connect: first_to_binding(cfg.connect),
            disconnect: first_to_binding(cfg.disconnect),
            refresh: first_to_binding(cfg.refresh),
            random_connect: first_to_binding(cfg.random_connect),
            pane_next: first_to_binding(cfg.pane_next),
            pane_prev: first_to_binding(cfg.pane_prev),
            sort_by_code: first_to_binding(cfg.sort_by_code),
            sort_by_country: first_to_binding(cfg.sort_by_country),
            connect_fastest: first_to_binding(cfg.connect_fastest),
            connect_p2p: first_to_binding(cfg.connect_p2p),
            connect_tor: first_to_binding(cfg.connect_tor),
            securecore: first_to_binding(cfg.securecore),
            toggle_favorite: first_to_binding(cfg.toggle_favorite),
        }
    }
}

impl From<KeyBindingsConfig> for KeyMap {
    fn from(cfg: KeyBindingsConfig) -> Self {
        fn to_matchers(configs: Vec<KeyMatcherConfig>) -> Vec<KeyMatcher> {
            configs.into_iter().map(|c| c.to_keymatcher()).collect()
        }
        KeyMap {
            down: to_matchers(cfg.navigation_down),
            up: to_matchers(cfg.navigation_up),
            page_down: to_matchers(cfg.page_down),
            page_up: to_matchers(cfg.page_up),
            go_first: to_matchers(cfg.go_first),
            go_last: to_matchers(cfg.go_last),
            connect: to_matchers(cfg.connect),
            disconnect: to_matchers(cfg.disconnect),
            refresh: to_matchers(cfg.refresh),
            random_connect: to_matchers(cfg.random_connect),
            pane_next: to_matchers(cfg.pane_next),
            pane_prev: to_matchers(cfg.pane_prev),
            sort_by_code: to_matchers(cfg.sort_by_code),
            sort_by_country: to_matchers(cfg.sort_by_country),
            connect_fastest: to_matchers(cfg.connect_fastest),
            connect_p2p: to_matchers(cfg.connect_p2p),
            connect_tor: to_matchers(cfg.connect_tor),
            securecore: to_matchers(cfg.securecore),
            search: to_matchers(cfg.search),
            cancel: to_matchers(cfg.cancel),
            help: to_matchers(cfg.help),
            quit: to_matchers(cfg.quit),
            next_setting: to_matchers(cfg.next_setting),
            prev_setting: to_matchers(cfg.prev_setting),
            toggle_setting: to_matchers(cfg.toggle_setting),
            select_city: to_matchers(cfg.select_city),
            refresh_cities: to_matchers(cfg.refresh_cities),
            toggle_favorite: to_matchers(cfg.toggle_favorite),
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
    pub fn config_display_path() -> String {
        paths::config_display_path()
    }

    pub fn load() -> Self {
        let config_path = match paths::config_path() {
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

    pub fn config_dir() -> Option<std::path::PathBuf> {
        paths::config_dir()
    }

    pub fn save(&self) {
        let config_path = match paths::config_path() {
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
        let save_favorites = !self.ui.favorites.is_empty();

        if save_theme || save_footer || save_favorites {
            toml_string.push_str("[ui]\n");
            if save_theme {
                toml_string.push_str(&format!("theme = \"{}\"\n", self.ui.theme));
            }
            if save_footer {
                toml_string.push_str(&format!("footer = {}\n", self.ui.footer));
            }
            if save_favorites {
                let favs: Vec<String> = self
                    .ui
                    .favorites
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect();
                toml_string.push_str(&format!("favorites = [{}]\n", favs.join(", ")));
            }
        }

        // Save keybindings if different from defaults
        let keybindings_different = self.keybindings != defaults.keybindings;

        if keybindings_different {
            if !toml_string.is_empty() {
                toml_string.push('\n');
            }
            toml_string.push_str("[keybindings]\n");

            if self.keybindings.navigation_down != defaults.keybindings.navigation_down {
                toml_string.push_str(&format!(
                    "navigation_down = {}\n",
                    format_keymatchers(&self.keybindings.navigation_down)
                ));
            }
            if self.keybindings.navigation_up != defaults.keybindings.navigation_up {
                toml_string.push_str(&format!(
                    "navigation_up = {}\n",
                    format_keymatchers(&self.keybindings.navigation_up)
                ));
            }
            if self.keybindings.connect != defaults.keybindings.connect {
                toml_string.push_str(&format!(
                    "connect = {}\n",
                    format_keymatchers(&self.keybindings.connect)
                ));
            }
            if self.keybindings.disconnect != defaults.keybindings.disconnect {
                toml_string.push_str(&format!(
                    "disconnect = {}\n",
                    format_keymatchers(&self.keybindings.disconnect)
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

fn format_keymatchers(matchers: &[KeyMatcherConfig]) -> String {
    let items: Vec<String> = matchers
        .iter()
        .map(|m| match m {
            KeyMatcherConfig::Char(c) => format!("{{ type = \"Char\", value = \"{}\" }}", c),
            KeyMatcherConfig::CharWithMod { code, modifiers } => {
                let mods = modifiers
                    .iter()
                    .map(|m| format!("\"{}\"", m))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{{ type = \"CharWithMod\", value = {{ code = \"{}\", modifiers = [{}] }} }}",
                    code, mods
                )
            }
            KeyMatcherConfig::Arrow(dir) => {
                format!("{{ type = \"Arrow\", value = \"{}\" }}", dir)
            }
            KeyMatcherConfig::DoubleChar(c) => {
                format!("{{ type = \"DoubleChar\", value = \"{}\" }}", c)
            }
        })
        .collect();
    format!("[{}]", items.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UserConfig::default();
        assert_eq!(config.ui.theme, "");
        assert!(config.ui.footer);
    }

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[ui]
theme = "Nord"
footer = false
"#;
        let config: UserConfig =
            toml::from_str(toml_content).expect("test TOML is valid and should parse");
        assert_eq!(config.ui.theme, "Nord");
        assert!(!config.ui.footer);
    }

    #[test]
    fn test_keybindings_config_default() {
        let config = KeyBindingsConfig::default();
        assert!(!config.navigation_down.is_empty());
        if let Some(KeyMatcherConfig::Char(c)) = config.navigation_down.first() {
            assert_eq!(*c, 'j');
        } else {
            panic!("Expected Char key matcher for navigation_down");
        }
    }
}
