//! Audio backends

use anyhow::Result;

pub mod cpal_backend;
pub mod null;

pub use cpal_backend::CpalBackend;
pub use null::NullAudioBackend;

/// Simple backend interface used by the audio thread
pub trait AudioBackend: Send {
    /// Prepare the backend for playback
    fn start(&mut self) -> Result<()>;
    /// Stop playback and release resources
    fn stop(&mut self);
    /// Submit a buffer of interleaved f32 samples
    fn play_samples(&mut self, samples: &[f32]) -> Result<()>;
}
