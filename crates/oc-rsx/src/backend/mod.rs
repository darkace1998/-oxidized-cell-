//! RSX rendering backends

pub mod null;
pub mod vulkan;

/// Graphics backend trait
pub trait GraphicsBackend {
    /// Initialize the backend
    fn init(&mut self) -> Result<(), String>;
    
    /// Shutdown the backend
    fn shutdown(&mut self);
    
    /// Begin a frame
    fn begin_frame(&mut self);
    
    /// End a frame and present
    fn end_frame(&mut self);
    
    /// Clear the screen
    fn clear(&mut self, color: [f32; 4], depth: f32, stencil: u8);
}
