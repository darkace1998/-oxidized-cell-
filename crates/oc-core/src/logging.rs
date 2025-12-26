//! Logging infrastructure for oxidized-cell emulator

use std::sync::OnceLock;
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, reload, EnvFilter};

use crate::config::{Config, LogLevel};

/// Handle to reload the log filter at runtime
type ReloadHandle = reload::Handle<EnvFilter, tracing_subscriber::Registry>;

/// Global reload handle for runtime log level changes
static RELOAD_HANDLE: OnceLock<ReloadHandle> = OnceLock::new();

/// Convert our LogLevel to tracing Level
fn log_level_to_tracing(level: LogLevel) -> Option<Level> {
    match level {
        LogLevel::Off => None,
        LogLevel::Error => Some(Level::ERROR),
        LogLevel::Warn => Some(Level::WARN),
        LogLevel::Info => Some(Level::INFO),
        LogLevel::Debug => Some(Level::DEBUG),
        LogLevel::Trace => Some(Level::TRACE),
    }
}

/// Initialize the logging system with reloadable filter
/// 
/// Call this once at startup. Use `set_log_level` to change level at runtime.
pub fn init_with_reload(initial_level: LogLevel) {
    let level = log_level_to_tracing(initial_level).unwrap_or(Level::INFO);
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    
    let (filter_layer, reload_handle) = reload::Layer::new(filter);
    
    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        );
    
    if subscriber.try_init().is_ok() {
        let _ = RELOAD_HANDLE.set(reload_handle);
        tracing::info!("Logging initialized with level: {:?}", initial_level);
    }
}

/// Set the log level at runtime
/// 
/// Returns true if the level was successfully changed
pub fn set_log_level(level: LogLevel) -> bool {
    if let Some(handle) = RELOAD_HANDLE.get() {
        if let Some(tracing_level) = log_level_to_tracing(level) {
            let new_filter = EnvFilter::from_default_env()
                .add_directive(tracing_level.into());
            
            if handle.reload(new_filter).is_ok() {
                tracing::info!("Log level changed to: {:?}", level);
                return true;
            }
        } else {
            // LogLevel::Off - set to a very restrictive filter
            let new_filter = EnvFilter::new("off");
            if handle.reload(new_filter).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Initialize the logging system based on configuration (legacy)
pub fn init(config: &Config) {
    let level = match config.debug.log_level {
        LogLevel::Off => return,
        LogLevel::Error => Level::ERROR,
        LogLevel::Warn => Level::WARN,
        LogLevel::Info => Level::INFO,
        LogLevel::Debug => Level::DEBUG,
        LogLevel::Trace => Level::TRACE,
    };

    let filter = EnvFilter::from_default_env().add_directive(level.into());

    let subscriber = tracing_subscriber::registry().with(filter).with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true),
    );

    if config.debug.log_to_file {
        if let Ok(file) = std::fs::File::create(&config.debug.log_path) {
            let file_layer = fmt::layer().with_writer(file).with_ansi(false);
            let _ = subscriber.with(file_layer).try_init();
        } else {
            let _ = subscriber.try_init();
        }
    } else {
        let _ = subscriber.try_init();
    }
}

/// Initialize logging with default settings (for tests and quick starts)
pub fn init_default() {
    let filter = EnvFilter::from_default_env()
        .add_directive(Level::INFO.into());

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init();
}

// Convenience macros for component-specific logging

/// Log a PPU trace message
#[macro_export]
macro_rules! ppu_trace {
    ($($arg:tt)*) => {
        tracing::trace!(target: "ppu", $($arg)*)
    };
}

/// Log a PPU debug message
#[macro_export]
macro_rules! ppu_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "ppu", $($arg)*)
    };
}

/// Log an SPU trace message
#[macro_export]
macro_rules! spu_trace {
    ($($arg:tt)*) => {
        tracing::trace!(target: "spu", $($arg)*)
    };
}

/// Log an SPU debug message
#[macro_export]
macro_rules! spu_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "spu", $($arg)*)
    };
}

/// Log an RSX trace message
#[macro_export]
macro_rules! rsx_trace {
    ($($arg:tt)*) => {
        tracing::trace!(target: "rsx", $($arg)*)
    };
}

/// Log an RSX debug message
#[macro_export]
macro_rules! rsx_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "rsx", $($arg)*)
    };
}

/// Log a kernel trace message
#[macro_export]
macro_rules! kernel_trace {
    ($($arg:tt)*) => {
        tracing::trace!(target: "kernel", $($arg)*)
    };
}

/// Log a kernel debug message
#[macro_export]
macro_rules! kernel_debug {
    ($($arg:tt)*) => {
        tracing::debug!(target: "kernel", $($arg)*)
    };
}
