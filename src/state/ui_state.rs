//! UI state types

use crate::state::AppView;
use crate::state::Pane;
use crate::state::ServerFilter;
use crate::state::ServerSort;
use crate::state::SortDirection;
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

/// UI State - contains all UI-related fields
/// This struct can be extracted to ui_state.rs in Phase 2
#[derive(Clone)]
pub struct UiState {
    pub current_view: AppView,
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

    // === Getters ===

    pub fn get_current_view(&self) -> AppView {
        self.current_view
    }

    pub fn set_current_view(&mut self, view: AppView) {
        self.current_view = view;
    }

    pub fn get_pane_focus(&self) -> Pane {
        self.pane_focus
    }

    pub fn get_selected_server(&self) -> Option<usize> {
        self.selected_server
    }

    pub fn get_selected_city(&self) -> Option<usize> {
        self.selected_city
    }

    pub fn get_settings_selected(&self) -> Option<usize> {
        self.settings_selected
    }

    pub fn is_settings_expanded(&self) -> bool {
        self.settings_expanded
    }

    pub fn get_settings_option_selected(&self) -> usize {
        self.settings_option_selected
    }

    pub fn get_logs_selected(&self) -> Option<usize> {
        self.logs_selected
    }

    pub fn get_search_query(&self) -> &SearchQuery {
        &self.search_query
    }

    pub fn get_filter(&self) -> ServerFilter {
        self.filter
    }

    pub fn get_sort(&self) -> ServerSort {
        self.sort
    }

    pub fn get_sort_direction(&self) -> SortDirection {
        self.sort_direction
    }

    pub fn is_dark_theme(&self) -> bool {
        self.is_dark_theme
    }

    pub fn get_input_mode(&self) -> InputMode {
        self.input_mode
    }

    pub fn get_dns_input(&self) -> &str {
        &self.dns_input
    }

    // === Setters ===

    pub fn set_settings_expanded(&mut self, expanded: bool) {
        self.settings_expanded = expanded;
    }

    pub fn set_settings_option_selected(&mut self, index: usize) {
        self.settings_option_selected = index;
    }

    pub fn set_pane_focus(&mut self, focus: Pane) {
        self.pane_focus = focus;
    }

    pub fn set_filter(&mut self, filter: ServerFilter) {
        self.filter = filter;
    }

    pub fn set_sort(&mut self, sort: ServerSort) {
        self.sort = sort;
    }

    pub fn set_sort_direction(&mut self, direction: SortDirection) {
        self.sort_direction = direction;
    }

    pub fn toggle_theme(&mut self) {
        self.is_dark_theme = !self.is_dark_theme;
    }

    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    pub fn set_dns_input(&mut self, input: String) {
        self.dns_input = input;
    }

    pub fn clear_dns_input(&mut self) {
        self.dns_input.clear();
    }

    pub fn push_dns_char(&mut self, c: char) {
        self.dns_input.push(c);
    }

    pub fn pop_dns_char(&mut self) {
        self.dns_input.pop();
    }

    pub fn reset_settings_selection(&mut self) {
        self.settings_expanded = false;
        self.settings_option_selected = 0;
    }

    /// Switch to next view
    pub fn switch_view(&mut self) {
        self.current_view = self.current_view.next();
    }
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
