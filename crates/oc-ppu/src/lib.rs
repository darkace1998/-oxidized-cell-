//! PPU (PowerPC Processing Unit) emulation for oxidized-cell
//!
//! This crate implements the Cell BE PPU, which is based on the PowerPC 970
//! architecture with VMX/AltiVec SIMD support.

pub mod decoder;
pub mod instructions;
pub mod interpreter;
pub mod thread;
pub mod vmx;

pub use decoder::PpuDecoder;
pub use interpreter::PpuInterpreter;
pub use thread::PpuThread;
