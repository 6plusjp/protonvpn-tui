//! ProtonVPN TUI - Binary entry point

use protonvpn_tui::config::UserConfig;
use protonvpn_tui::constants::paths::APP_LOG_FILE;
use protonvpn_tui::ui::app::TuiApp;
use std::io;

fn main() -> io::Result<()> {
    // Initialize logging to file (won't interfere with TUI)
    let default_filter = tracing_subscriber::EnvFilter::from_default_env();
    let directive = "protonvpn_tui=warn".parse::<tracing_subscriber::filter::Directive>();
    let env_filter = match directive {
        Ok(d) => default_filter.add_directive(d),
        Err(e) => {
            eprintln!("Warning: Failed to parse RUST_LOG directive: {}", e);
            default_filter.add_directive(tracing_subscriber::filter::LevelFilter::WARN.into())
        }
    };

    // Log to file in home directory or temp
    let log_path = std::env::var("PROTONVPN_TUI_LOG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join(APP_LOG_FILE));

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_ansi(false)
            .with_writer(file)
            .init();
    } else {
        // Fallback to stderr if file fails
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(false)
            .with_ansi(false)
            .init();
    }

    tracing::info!("Starting ProtonVPN TUI");

    let user_config = UserConfig::load();
    let mut app = TuiApp::new(user_config)?;
    app.run()?;

    tracing::info!("Shutting down ProtonVPN TUI");
    Ok(())
}
