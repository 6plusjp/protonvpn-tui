use crate::state::AppView;
use crate::state::Pane;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
use crate::vpn::Server;
use std::fmt;
use std::sync::RwLock;

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
    pub logs_selected: Option<usize>,
    pub search_query: SearchQuery,
    pub filter: ServerFilter,
    pub sort: ServerSort,
    pub sort_direction: SortDirection,
    pub is_dark_theme: bool,
    pub input_mode: InputMode,
    pub dns_input: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    pub fn new() -> Self {
        Self {
            current_view: AppView::Servers,
            previous_view: AppView::Servers,
            selected_server: Some(0),
            selected_city: Some(0),
            pane_focus: Pane::Countries,
            settings_selected: Some(0),
            settings_expanded: false,
            settings_option_selected: 0,
            logs_selected: Some(0),
            search_query: SearchQuery::new(),
            filter: ServerFilter::default(),
            sort: ServerSort::default(),
            sort_direction: SortDirection::default(),
            is_dark_theme: true,
            input_mode: InputMode::Normal,
            dns_input: String::new(),
        }
    }

    pub fn toggle_theme(&mut self) {
        self.is_dark_theme = !self.is_dark_theme;
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

impl Default for ServerCache {
    fn default() -> Self {
        Self::new()
    }
}
