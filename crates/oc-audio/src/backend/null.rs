//! Null audio backend

use anyhow::Result;

use super::AudioBackend;

/// Null audio backend (no sound output)
#[derive(Default)]
pub struct NullAudioBackend {
    pub(crate) started: bool,
    pub(crate) last_samples: Option<Vec<f32>>,
}

impl NullAudioBackend {
    pub fn new() -> Self {
        Self {
            started: false,
            last_samples: None,
        }
    }

    /// Returns the last samples that were submitted (useful for tests)
    pub fn last_samples(&self) -> Option<&[f32]> {
        self.last_samples.as_deref()
    }
}

impl AudioBackend for NullAudioBackend {
    fn start(&mut self) -> Result<()> {
        self.started = true;
        Ok(())
    }

    fn stop(&mut self) {
        self.started = false;
        self.last_samples = None;
    }

    fn play_samples(&mut self, samples: &[f32]) -> Result<()> {
        if self.started {
            self.last_samples = Some(samples.to_vec());
        }
        Ok(())
    }
}
