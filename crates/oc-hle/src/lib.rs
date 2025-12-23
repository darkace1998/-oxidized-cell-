//! HLE (High Level Emulation) modules for oxidized-cell
//!
//! This crate provides HLE implementations of PS3 system libraries.

pub mod module;

// HLE module stubs
pub mod cell_audio;
pub mod cell_font;
pub mod cell_fs;
pub mod cell_game;
pub mod cell_gcm_sys;
pub mod cell_http;
pub mod cell_net_ctl;
pub mod cell_pad;
pub mod cell_png_dec;
pub mod cell_save_data;
pub mod cell_spurs;
pub mod cell_sysutil;

pub use module::ModuleRegistry;
