//! Oxidized-Cell - PS3 Emulator
//!
//! Main entry point for the emulator application.

use oc_core::config::Config;
use oc_ui::app;

fn main() -> eframe::Result<()> {
    // Load config to get initial log level
    let config = Config::load().unwrap_or_default();
    
    // Initialize logging with reloadable filter
    oc_core::logging::init_with_reload(config.debug.log_level);

    tracing::info!("Starting Oxidized-Cell PS3 Emulator");

    // Run the application
    app::run()
}
