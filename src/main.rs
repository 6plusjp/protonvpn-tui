use protonvpn_tui::ui::app::TuiApp;
use std::io;

fn main() -> io::Result<()> {
    // Initialize logging to stderr (won't interfere with TUI)
    let default_filter = tracing_subscriber::EnvFilter::from_default_env();
    let directive = "protonvpn_tui=info".parse::<tracing_subscriber::filter::Directive>();
    let env_filter = match directive {
        Ok(d) => default_filter.add_directive(d),
        Err(e) => {
            eprintln!("Warning: Failed to parse RUST_LOG directive: {}", e);
            default_filter.add_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    tracing::info!("Starting ProtonVPN TUI");

    // Initialize and run TUI
    let mut app = TuiApp::new()?;
    app.run()?;

    tracing::info!("Shutting down ProtonVPN TUI");
    Ok(())
}
