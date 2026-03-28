#[cfg(test)]
pub mod test_helpers {
    use crate::vpn::{City, Server};

    pub fn make_servers() -> Vec<Server> {
        vec![
            Server {
                code: "JP".into(),
                code_lower: "jp".into(),
                country: "Japan".into(),
                country_lower: "japan".into(),
                cities: vec![City::new("Tokyo".into()), City::new("Osaka".into())],
            },
            Server {
                code: "US".into(),
                code_lower: "us".into(),
                country: "United States".into(),
                country_lower: "united states".into(),
                cities: vec![City::new("New York".into())],
            },
            Server {
                code: "DE".into(),
                code_lower: "de".into(),
                country: "Germany".into(),
                country_lower: "germany".into(),
                cities: vec![City::new("Berlin".into())],
            },
            Server {
                code: "GB".into(),
                code_lower: "gb".into(),
                country: "United Kingdom".into(),
                country_lower: "united kingdom".into(),
                cities: vec![City::new("London".into())],
            },
            Server {
                code: "FR".into(),
                code_lower: "fr".into(),
                country: "France".into(),
                country_lower: "france".into(),
                cities: vec![City::new("Paris".into())],
            },
        ]
    }

    pub fn make_servers_with_features() -> Vec<Server> {
        vec![
            Server {
                code: "JP".into(),
                code_lower: "jp".into(),
                country: "Japan".into(),
                country_lower: "japan".into(),
                cities: vec![City::with_features(
                    "Tokyo".into(),
                    vec!["P2P".into(), "Secure Core".into()],
                )],
            },
            Server {
                code: "CH".into(),
                code_lower: "ch".into(),
                country: "Switzerland".into(),
                country_lower: "switzerland".into(),
                cities: vec![City::with_features(
                    "Zurich".into(),
                    vec!["Secure Core".into()],
                )],
            },
        ]
    }

    pub fn setup() {
        crate::state::set_test_mode(true);
    }
}
