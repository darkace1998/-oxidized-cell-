//! Audio thread

use anyhow::Result;

use crate::{
    backend::{AudioBackend, NullAudioBackend},
    mixer::Mixer,
};

/// Audio thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioThreadState {
    Stopped,
    Running,
}

/// Audio thread responsible for mixing and submitting audio to a backend.
pub struct AudioThread {
    state: AudioThreadState,
    mixer: Mixer,
    backend: Box<dyn AudioBackend>,
    last_buffer: Vec<f32>,
}

impl AudioThread {
    /// Create a new audio thread using the provided backend.
    pub fn with_backend(backend: Box<dyn AudioBackend>) -> Self {
        Self {
            state: AudioThreadState::Stopped,
            mixer: Mixer::new(),
            backend,
            last_buffer: Vec::new(),
        }
    }

    /// Create a new audio thread using the null backend.
    pub fn new() -> Self {
        Self::with_backend(Box::new(NullAudioBackend::new()))
    }

    /// Start playback.
    pub fn start(&mut self) -> Result<()> {
        self.backend.start()?;
        self.state = AudioThreadState::Running;
        Ok(())
    }

    /// Stop playback.
    pub fn stop(&mut self) {
        self.backend.stop();
        self.state = AudioThreadState::Stopped;
        self.last_buffer.clear();
    }

    /// Mix the provided input streams and submit them to the backend.
    pub fn submit_streams(&mut self, streams: &[Vec<f32>]) -> Result<()> {
        let buffer = self.mixer.mix(streams);
        self.backend.play_samples(&buffer)?;
        self.last_buffer = buffer;
        Ok(())
    }

    /// Adjust output volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.mixer.set_volume(volume);
    }

    /// Current volume.
    pub fn volume(&self) -> f32 {
        self.mixer.volume()
    }

    /// Get current state.
    pub fn state(&self) -> AudioThreadState {
        self.state
    }

    /// Access the last buffer that was submitted (useful for tests).
    pub fn last_buffer(&self) -> &[f32] {
        &self.last_buffer
    }
}

impl Default for AudioThread {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_and_stores_buffer() {
        let mut thread = AudioThread::new();
        thread.start().unwrap();
        thread.set_volume(1.0);

        let buf_a = vec![0.5f32, 0.5];
        thread.submit_streams(&[buf_a]).unwrap();

        assert_eq!(thread.state(), AudioThreadState::Running);
        assert_eq!(thread.last_buffer().len(), 2);

        thread.stop();
        assert_eq!(thread.state(), AudioThreadState::Stopped);
    }
}
