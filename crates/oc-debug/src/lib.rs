//! Debugging tools for oxidized-cell PS3 emulator
//!
//! This crate provides debugging infrastructure for:
//! - PPU debugging (instruction tracing, register inspection, breakpoints)
//! - SPU debugging (local storage viewer, register viewer, channel monitor)
//! - RSX debugging (command buffer viewer, state inspector)
//! - Performance profiling (CPU/GPU profiling, hotspot analysis)

pub mod ppu_debugger;
pub mod spu_debugger;
pub mod rsx_debugger;
pub mod profiler;
pub mod breakpoint;
pub mod disassembler;

pub use ppu_debugger::PpuDebugger;
pub use spu_debugger::SpuDebugger;
pub use rsx_debugger::RsxDebugger;
pub use profiler::Profiler;
pub use breakpoint::{Breakpoint, BreakpointManager};
pub use disassembler::{PpuDisassembler, SpuDisassembler};
