//! cellAudio HLE helpers for the audio subsystem.

use anyhow::Result;

use crate::{
    backend::{CpalBackend, NullAudioBackend},
    thread::AudioThread,
};

/// Lightweight cellAudio facade that owns an audio thread and backend.
pub struct CellAudio {
    thread: AudioThread,
}

impl CellAudio {
    /// Create a new cellAudio instance using the provided backend.
    pub fn with_null_backend() -> Self {
        Self {
            thread: AudioThread::with_backend(Box::new(NullAudioBackend::new())),
        }
    }

    /// Try to construct a CPAL backend, falling back to a null backend if
    /// no output device is available.
    pub fn with_cpal_backend() -> Self {
        let backend = CpalBackend::new();
        Self {
            thread: AudioThread::with_backend(Box::new(backend)),
        }
    }

    /// Start audio playback.
    pub fn init(&mut self) -> Result<()> {
        self.thread.start()
    }

    /// Stop audio playback.
    pub fn shutdown(&mut self) {
        self.thread.stop();
    }

    /// Submit a set of interleaved streams to be mixed and played.
    pub fn play_streams(&mut self, streams: &[Vec<f32>]) -> Result<()> {
        self.thread.submit_streams(streams)
    }

    /// Adjust output volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.thread.set_volume(volume);
    }

    /// Read back the last mixed buffer (useful for verification).
    pub fn last_buffer(&self) -> &[f32] {
        self.thread.last_buffer()
    }
}

impl Default for CellAudio {
    fn default() -> Self {
        Self::with_null_backend()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_and_plays() {
        let mut cell_audio = CellAudio::default();
        cell_audio.init().unwrap();

        let left = vec![0.25f32, 0.25];
        let right = vec![0.75f32, 0.75];

        cell_audio.play_streams(&[left, right]).unwrap();
        assert_eq!(cell_audio.last_buffer().len(), 2);

        cell_audio.shutdown();
    }
}
