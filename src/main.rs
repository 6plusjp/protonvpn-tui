//! ProtonVPN TUI - Entry point

use protonvpn_tui::AppState;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("protonvpn_tui=info".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting ProtonVPN TUI");

    // Initialize application state
    let _app = AppState::new();

    tracing::info!("Application initialized successfully");
}
