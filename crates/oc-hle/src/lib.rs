//! HLE (High Level Emulation) modules for oxidized-cell
//!
//! This crate provides HLE implementations of PS3 system libraries.

pub mod module;
pub mod context;

// Graphics Modules
pub mod cell_gcm_sys;
pub mod cell_gif_dec;
pub mod cell_png_dec;
pub mod cell_jpg_dec;

// System Modules
pub mod cell_sysutil;
pub mod cell_game;
pub mod cell_save_data;

// Multimedia Modules
pub mod cell_dmux;
pub mod cell_vdec;
pub mod cell_adec;
pub mod cell_vpost;

// Network Modules
pub mod cell_net_ctl;
pub mod cell_http;
pub mod cell_ssl;

// Utilities Modules
pub mod cell_font;
pub mod cell_spurs;
pub mod libsre;

// Other System Modules
pub mod cell_audio;
pub mod cell_fs;
pub mod cell_pad;

pub use module::ModuleRegistry;
pub use context::{HleContext, HLE_CONTEXT, get_hle_context, get_hle_context_mut, reset_hle_context};
