use protonvpn_tui::ui::app::TuiApp;
use std::io;

fn main() -> io::Result<()> {
    // Initialize logging to stderr (won't interfere with TUI)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(
                "protonvpn_tui=info"
                    .parse()
                    .expect("Failed to parse log directive"),
            ),
        )
        .with_target(false)
        .init();

    tracing::info!("Starting ProtonVPN TUI");

    // Initialize and run TUI
    let mut app = TuiApp::new()?;
    app.run()?;

    tracing::info!("Shutting down ProtonVPN TUI");
    Ok(())
}
