use crate::state::AppView;
use crate::state::Pane;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
use crate::ui::styles::ThemeMode;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Filter,
    DnsInput,
}

#[derive(Clone)]
pub struct UiState {
    pub current_view: AppView,
    pub previous_view: AppView,
    pub selected_server: Option<usize>,
    pub selected_city: Option<usize>,
    pub pane_focus: Pane,
    pub settings_selected: Option<usize>,
    pub settings_expanded: bool,
    pub settings_option_selected: usize,
    pub settings_last_key_g: bool,
    pub logs_selected: Option<usize>,
    pub search_query: SearchQuery,
    pub filter: ServerFilter,
    pub sort: ServerSort,
    pub sort_direction: SortDirection,
    pub theme_mode: ThemeMode,
    pub show_footer: bool,
    pub mask_ip: bool,
    pub input_mode: InputMode,
    pub dns_input: String,
    pub favorite_countries: HashSet<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_view: AppView::Servers,
            previous_view: AppView::Servers,
            selected_server: Some(0),
            selected_city: Some(0),
            pane_focus: Pane::Countries,
            settings_selected: Some(0),
            settings_expanded: false,
            settings_option_selected: 0,
            settings_last_key_g: false,
            logs_selected: Some(0),
            search_query: SearchQuery::new(),
            filter: ServerFilter::default(),
            sort: ServerSort::default(),
            sort_direction: SortDirection::default(),
            theme_mode: ThemeMode::default(),
            show_footer: true,
            mask_ip: false,
            input_mode: InputMode::Normal,
            dns_input: String::new(),
            favorite_countries: HashSet::new(),
        }
    }
}

impl UiState {
    pub fn from_config(theme: &str, show_footer: bool, mask_ip: bool) -> Self {
        let theme_mode = match theme {
            "CatppuccinMocha" => ThemeMode::CatppuccinMocha,
            "CatppuccinLatte" => ThemeMode::CatppuccinLatte,
            "Dracula" => ThemeMode::Dracula,
            "Nord" => ThemeMode::Nord,
            "Gruvbox" => ThemeMode::Gruvbox,
            "TokyoNight" => ThemeMode::TokyoNight,
            _ => ThemeMode::System,
        };
        Self {
            current_view: AppView::Servers,
            previous_view: AppView::Servers,
            selected_server: Some(0),
            selected_city: Some(0),
            pane_focus: Pane::Countries,
            settings_selected: Some(0),
            settings_expanded: false,
            settings_option_selected: 0,
            settings_last_key_g: false,
            logs_selected: Some(0),
            search_query: SearchQuery::new(),
            filter: ServerFilter::default(),
            sort: ServerSort::default(),
            sort_direction: SortDirection::default(),
            theme_mode,
            show_footer,
            mask_ip,
            input_mode: InputMode::Normal,
            dns_input: String::new(),
            favorite_countries: HashSet::new(),
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::System => ThemeMode::CatppuccinMocha,
            ThemeMode::CatppuccinMocha => ThemeMode::CatppuccinLatte,
            ThemeMode::CatppuccinLatte => ThemeMode::Dracula,
            ThemeMode::Dracula => ThemeMode::Nord,
            ThemeMode::Nord => ThemeMode::Gruvbox,
            ThemeMode::Gruvbox => ThemeMode::TokyoNight,
            ThemeMode::TokyoNight => ThemeMode::System,
        };
    }

    pub fn reset_settings_selection(&mut self) {
        self.settings_expanded = false;
        self.settings_option_selected = 0;
    }

    pub fn switch_view(&mut self) {
        self.previous_view = self.current_view;
        self.current_view = self.current_view.next();
    }

    pub fn set_view(&mut self, view: AppView) {
        self.previous_view = self.current_view;
        self.current_view = view;
        self.pane_focus = view.default_pane();
    }
}

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
