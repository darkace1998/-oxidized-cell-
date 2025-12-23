//! Null backend for testing

use super::GraphicsBackend;

/// Null graphics backend (does nothing)
pub struct NullBackend;

impl NullBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphicsBackend for NullBackend {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn begin_frame(&mut self) {}

    fn end_frame(&mut self) {}

    fn clear(&mut self, _color: [f32; 4], _depth: f32, _stencil: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_backend() {
        let mut backend = NullBackend::new();
        assert!(backend.init().is_ok());
        backend.begin_frame();
        backend.clear([0.0, 0.0, 0.0, 1.0], 1.0, 0);
        backend.end_frame();
        backend.shutdown();
    }
}
