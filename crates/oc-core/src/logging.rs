//! Logging infrastructure for oxidized-cell emulator

use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use crate::config::{Config, LogLevel};

/// Initialize the logging system based on configuration
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
