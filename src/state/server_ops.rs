//! Server filtering, sorting, and related operations

use crate::state::{NotificationType, ServerFilter, ServerSort, SortDirection};
use crate::vpn::Server;

impl crate::state::AppState {
    pub const MAX_PENDING_CITY_FETCHES: usize = 5;

    /// Get filtered and sorted server list
    pub fn filtered_servers(&self) -> Vec<Server> {
        if let Some(cached) = self.server_cache.get_cached() {
            return cached;
        }

        let result = self.compute_filtered_servers();
        self.server_cache.set_cached(result.clone());
        result
    }

    pub(crate) fn compute_filtered_servers(&self) -> Vec<Server> {
        let query = &self.ui_state.search_query.query_lower;

        let servers = self.vpn_state.servers();

        let fuzzy_variants = if query.is_empty() {
            vec![]
        } else {
            Self::compute_fuzzy_variants(&servers, query)
        };

        let result: Vec<Server> = if query.is_empty() {
            servers.clone()
        } else {
            let query_len = query.len();
            let skip_city = query_len <= 2;

            servers
                .iter()
                .filter(|server| {
                    let q = query.as_str();
                    let matches_code = server.code_lower.contains(q);
                    let matches_country = server.country_lower.contains(q);
                    let matches_fuzzy =
                        Self::fuzzy_match_with_variants(&server.country_lower, q, &fuzzy_variants);

                    if skip_city {
                        match self.ui_state.filter {
                            ServerFilter::Code => matches_code || matches_fuzzy,
                            ServerFilter::Country => matches_country || matches_fuzzy,
                            ServerFilter::City => false,
                        }
                    } else {
                        let matches_city = server.cities.iter().any(|c| {
                            let search = if c.name_lower.is_empty() {
                                c.name.to_lowercase()
                            } else {
                                c.name_lower.clone()
                            };
                            search.contains(q)
                        });

                        match self.ui_state.filter {
                            ServerFilter::Code => matches_code || matches_fuzzy,
                            ServerFilter::Country => matches_country || matches_fuzzy,
                            ServerFilter::City => matches_city || matches_fuzzy,
                        }
                    }
                })
                .cloned()
                .collect()
        };

        let mut favorites: Vec<Server> = Vec::new();
        let mut non_favorites: Vec<Server> = Vec::new();

        for server in result {
            if self.ui_state.favorite_countries.contains(&server.code) {
                favorites.push(server);
            } else {
                non_favorites.push(server);
            }
        }

        let favorites_sorted =
            Self::sort_servers(favorites, self.ui_state.sort, self.ui_state.sort_direction);
        let non_favorites_sorted = Self::sort_servers(
            non_favorites,
            self.ui_state.sort,
            self.ui_state.sort_direction,
        );

        favorites_sorted
            .into_iter()
            .chain(non_favorites_sorted)
            .collect()
    }

    fn sort_servers(
        servers: Vec<Server>,
        sort: ServerSort,
        direction: SortDirection,
    ) -> Vec<Server> {
        let mut result = servers;
        match (sort, direction) {
            (ServerSort::Code, SortDirection::Asc) => {
                result.sort_by(|a, b| a.code_lower.cmp(&b.code_lower))
            }
            (ServerSort::Code, SortDirection::Desc) => {
                result.sort_by(|a, b| b.code_lower.cmp(&a.code_lower))
            }
            (ServerSort::Country, SortDirection::Asc) => {
                result.sort_by(|a, b| a.country_lower.cmp(&b.country_lower))
            }
            (ServerSort::Country, SortDirection::Desc) => {
                result.sort_by(|a, b| b.country_lower.cmp(&a.country_lower))
            }
        }
        result
    }

    fn fuzzy_match_with_variants(text_lower: &str, query: &str, variants: &[String]) -> bool {
        if text_lower.starts_with(query) {
            return true;
        }

        variants.iter().any(|v| text_lower.contains(v))
    }

    fn compute_fuzzy_variants(servers: &[Server], query: &str) -> Vec<String> {
        let mut variants = vec![query.to_string()];

        let no_vowels: String = query
            .chars()
            .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
            .collect();
        if !no_vowels.is_empty() && no_vowels != query {
            variants.push(no_vowels);
        }

        for server in servers {
            if server.code_lower == query {
                variants.push(server.country_lower.clone());
                break;
            }
        }

        variants
    }

    pub fn cycle_filter(&mut self) {
        self.ui_state.filter = self.ui_state.filter.next();
        self.server_cache.invalidate();
    }

    pub fn cycle_sort(&mut self) {
        self.ui_state.sort_direction = self.ui_state.sort_direction.toggle();
        self.server_cache.invalidate();
    }

    pub fn cycle_sort_field(&mut self) {
        self.ui_state.sort = self.ui_state.sort.next();
        self.server_cache.invalidate();
    }

    pub fn toggle_favorite(&mut self, country_code: &str) {
        self.server_cache.invalidate();

        if self.ui_state.favorite_countries.contains(country_code) {
            self.ui_state.favorite_countries.remove(country_code);
            self.show_notification(
                format!("Removed {} from favorites", country_code),
                NotificationType::Info,
                None,
            );
        } else {
            self.ui_state
                .favorite_countries
                .insert(country_code.to_string());
            self.show_notification(
                format!("Added {} to favorites", country_code),
                NotificationType::Info,
                None,
            );
        }
        self.save_favorites();
    }

    pub fn is_favorite(&self, country_code: &str) -> bool {
        self.ui_state.favorite_countries.contains(country_code)
    }

    pub fn set_sort_by_code(&mut self) {
        self.ui_state.sort = ServerSort::Code;
        self.server_cache.invalidate();
        self.switch_cities_to_selected();
    }

    pub fn set_sort_by_country(&mut self) {
        self.ui_state.sort = ServerSort::Country;
        self.server_cache.invalidate();
        self.switch_cities_to_selected();
    }

    pub fn toggle_sort_direction(&mut self) {
        self.ui_state.sort_direction = self.ui_state.sort_direction.toggle();
        self.server_cache.invalidate();
        self.switch_cities_to_selected();
    }

    pub(crate) fn fetch_cities(&mut self, country_code: &str, force: bool) {
        let country_code = country_code.to_string();

        if let Some(cities) = self.vpn_state.cached_cities(&country_code) {
            self.current_cities = cities;
            return;
        }

        let already_loading = self
            .connection_manager
            .loading_cities
            .contains(&country_code);

        if already_loading && !force {
            return;
        }

        if !force
            && !already_loading
            && self.connection_manager.loading_cities.len() >= Self::MAX_PENDING_CITY_FETCHES
        {
            return;
        }

        self.show_notification(
            format!("Loading cities for {}...", country_code),
            NotificationType::Info,
            Some(format!("cities:{}", country_code)),
        );

        self.connection_manager
            .loading_cities
            .insert(country_code.clone());
        self.connection_manager.async_manager.spawn_cities(
            self.vpn_state.clone(),
            country_code,
            self.connection_manager.async_notifier.clone(),
        );
    }

    pub fn reload_cities(&mut self) {
        if let Some(country_code) = self.current_country_code.clone() {
            self.current_cities.clear();

            if let Err(e) = self.vpn_state.clear_cities_cache(&country_code) {
                tracing::warn!("Failed to clear cities cache: {}", e);
            }

            self.fetch_cities(&country_code, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::log_persistence;
    use crate::state::AppState;
    use crate::vpn::{City, Server, VpnClient};

    fn setup() {
        log_persistence::set_test_mode(true);
    }

    fn make_servers() -> Vec<Server> {
        vec![
            Server {
                code: "JP".to_string(),
                code_lower: "jp".to_string(),
                country: "Japan".to_string(),
                country_lower: "japan".to_string(),
                cities: vec![
                    City::new("Tokyo".to_string()),
                    City::new("Osaka".to_string()),
                ],
            },
            Server {
                code: "US".to_string(),
                code_lower: "us".to_string(),
                country: "United States".to_string(),
                country_lower: "united states".to_string(),
                cities: vec![City::new("New York".to_string())],
            },
            Server {
                code: "DE".to_string(),
                code_lower: "de".to_string(),
                country: "Germany".to_string(),
                country_lower: "germany".to_string(),
                cities: vec![City::new("Berlin".to_string())],
            },
            Server {
                code: "GB".to_string(),
                code_lower: "gb".to_string(),
                country: "United Kingdom".to_string(),
                country_lower: "united kingdom".to_string(),
                cities: vec![City::new("London".to_string())],
            },
            Server {
                code: "FR".to_string(),
                code_lower: "fr".to_string(),
                country: "France".to_string(),
                country_lower: "france".to_string(),
                cities: vec![City::new("Paris".to_string())],
            },
        ]
    }

    #[test]
    fn test_cycle_filter() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        assert_eq!(state.ui_state.filter, ServerFilter::Code);
        state.cycle_filter();
        assert_eq!(state.ui_state.filter, ServerFilter::Country);
        state.cycle_filter();
        assert_eq!(state.ui_state.filter, ServerFilter::City);
        state.cycle_filter();
        assert_eq!(state.ui_state.filter, ServerFilter::Code);
    }

    #[test]
    fn test_cycle_sort() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        assert_eq!(state.ui_state.sort_direction, SortDirection::Asc);
        state.cycle_sort();
        assert_eq!(state.ui_state.sort_direction, SortDirection::Desc);
        state.cycle_sort();
        assert_eq!(state.ui_state.sort_direction, SortDirection::Asc);
    }

    #[test]
    fn test_cycle_sort_field() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        assert_eq!(state.ui_state.sort, ServerSort::Code);
        state.cycle_sort_field();
        assert_eq!(state.ui_state.sort, ServerSort::Country);
        state.cycle_sort_field();
        assert_eq!(state.ui_state.sort, ServerSort::Code);
    }

    #[test]
    fn test_toggle_favorite_add_and_remove() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        assert!(!state.is_favorite("JP"));
        state.toggle_favorite("JP");
        assert!(state.is_favorite("JP"));
        state.toggle_favorite("JP");
        assert!(!state.is_favorite("JP"));
    }

    #[test]
    fn test_is_favorite() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.favorite_countries.insert("US".to_string());
        assert!(state.is_favorite("US"));
        assert!(!state.is_favorite("JP"));
    }

    #[test]
    fn test_set_sort_by_code() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.sort = ServerSort::Country;
        state.set_sort_by_code();
        assert_eq!(state.ui_state.sort, ServerSort::Code);
    }

    #[test]
    fn test_set_sort_by_country() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.sort = ServerSort::Code;
        state.set_sort_by_country();
        assert_eq!(state.ui_state.sort, ServerSort::Country);
    }

    #[test]
    fn test_toggle_sort_direction() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        assert_eq!(state.ui_state.sort_direction, SortDirection::Asc);
        state.toggle_sort_direction();
        assert_eq!(state.ui_state.sort_direction, SortDirection::Desc);
        state.toggle_sort_direction();
        assert_eq!(state.ui_state.sort_direction, SortDirection::Asc);
    }

    #[test]
    fn test_favorites_sorted_and_come_first_by_code_asc() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.sort = ServerSort::Code;
        state.ui_state.sort_direction = SortDirection::Asc;

        state.toggle_favorite("FR");
        state.toggle_favorite("JP");

        let servers = state.filtered_servers();

        assert_eq!(servers[0].code, "FR");
        assert_eq!(servers[1].code, "JP");
        assert_eq!(servers[2].code, "DE");
        assert_eq!(servers[3].code, "GB");
        assert_eq!(servers[4].code, "US");
    }

    #[test]
    fn test_favorites_sorted_and_come_first_by_country_asc() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.sort = ServerSort::Country;
        state.ui_state.sort_direction = SortDirection::Asc;

        state.toggle_favorite("FR");
        state.toggle_favorite("JP");

        let servers = state.filtered_servers();

        assert_eq!(servers[0].code, "FR");
        assert_eq!(servers[1].code, "JP");
        assert_eq!(servers[2].code, "DE");
    }

    #[test]
    fn test_favorites_descending_order() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.sort = ServerSort::Code;
        state.ui_state.sort_direction = SortDirection::Desc;

        state.toggle_favorite("FR");
        state.toggle_favorite("JP");

        let servers = state.filtered_servers();

        assert_eq!(servers[0].code, "JP");
        assert_eq!(servers[1].code, "FR");
    }

    #[test]
    fn test_toggle_favorite_updates_position_correctly() {
        setup();
        let mut state = AppState::new();
        state.vpn_state = std::sync::Arc::new(VpnClient::with_test_servers(make_servers()));
        state.ui_state.favorite_countries.clear();
        state.ui_state.sort = ServerSort::Code;
        state.ui_state.sort_direction = SortDirection::Asc;

        state.ui_state.selected_server = Some(4);
        state.toggle_favorite("FR");

        let servers = state.filtered_servers();

        assert_eq!(servers[0].code, "FR");
        assert_eq!(servers[1].code, "DE");
        assert_eq!(servers[2].code, "GB");
        assert_eq!(servers[3].code, "JP");
        assert_eq!(servers[4].code, "US");
    }
}
