//! Null audio backend

/// Null audio backend (no sound output)
pub struct NullAudioBackend;

impl NullAudioBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}
