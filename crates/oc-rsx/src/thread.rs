//! RSX thread (command processor)

use std::sync::Arc;
use oc_memory::MemoryManager;
use crate::state::RsxState;
use crate::fifo::CommandFifo;

/// RSX thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RsxThreadState {
    Stopped,
    Running,
    Idle,
}

/// RSX command processor thread
pub struct RsxThread {
    /// Thread state
    pub state: RsxThreadState,
    /// Graphics state
    pub gfx_state: RsxState,
    /// Command FIFO
    pub fifo: CommandFifo,
    /// Memory manager reference
    memory: Arc<MemoryManager>,
}

impl RsxThread {
    /// Create a new RSX thread
    pub fn new(memory: Arc<MemoryManager>) -> Self {
        Self {
            state: RsxThreadState::Stopped,
            gfx_state: RsxState::new(),
            fifo: CommandFifo::new(),
            memory,
        }
    }

    /// Process commands from FIFO
    pub fn process_commands(&mut self) {
        while let Some(cmd) = self.fifo.pop() {
            self.execute_command(cmd.method, cmd.data);
        }
    }

    /// Execute a single RSX command
    fn execute_command(&mut self, method: u32, data: u32) {
        tracing::trace!("RSX method 0x{:04x} = 0x{:08x}", method, data);
        
        match method {
            // NV4097_SET_SURFACE_COLOR_TARGET
            0x0194 => {
                self.gfx_state.surface_color_target = data;
            }
            // NV4097_SET_SURFACE_PITCH_A
            0x0280 => {
                self.gfx_state.surface_pitch[0] = data;
            }
            // NV4097_SET_CONTEXT_DMA_COLOR_A
            0x0184 => {
                self.gfx_state.context_dma_color[0] = data;
            }
            // NV4097_SET_SURFACE_COLOR_AOFFSET
            0x0190 => {
                self.gfx_state.surface_offset_color[0] = data;
            }
            // NV4097_SET_SURFACE_FORMAT
            0x0180 => {
                self.gfx_state.surface_format = data;
            }
            // NV4097_SET_SURFACE_CLIP_HORIZONTAL
            0x018C => {
                self.gfx_state.surface_clip_x = (data & 0xFFFF) as u16;
                self.gfx_state.surface_clip_width = ((data >> 16) & 0xFFFF) as u16;
            }
            // NV4097_SET_SURFACE_CLIP_VERTICAL
            0x0188 => {
                self.gfx_state.surface_clip_y = (data & 0xFFFF) as u16;
                self.gfx_state.surface_clip_height = ((data >> 16) & 0xFFFF) as u16;
            }
            // NV4097_CLEAR_SURFACE
            0x1D94 => {
                self.clear_surface(data);
            }
            // NV4097_SET_BEGIN_END
            0x1808 => {
                if data != 0 {
                    self.gfx_state.primitive_type = data;
                } else {
                    // End primitive
                    self.flush_vertices();
                }
            }
            _ => {
                // Many methods are not yet implemented
            }
        }
    }

    /// Clear the surface
    fn clear_surface(&mut self, _mask: u32) {
        // Clear color/depth/stencil based on mask
        tracing::trace!("Clear surface");
    }

    /// Flush accumulated vertices
    fn flush_vertices(&mut self) {
        // Draw accumulated vertices
        tracing::trace!("Flush vertices");
    }

    /// Get memory manager reference
    pub fn memory(&self) -> &Arc<MemoryManager> {
        &self.memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsx_thread_creation() {
        let memory = MemoryManager::new().unwrap();
        let thread = RsxThread::new(memory);
        assert_eq!(thread.state, RsxThreadState::Stopped);
    }
}
