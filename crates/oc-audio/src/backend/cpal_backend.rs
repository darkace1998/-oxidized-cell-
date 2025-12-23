//! cpal audio backend

use std::sync::Arc;

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::AudioBackend;

/// Minimal CPAL backend wrapper. If a default output device cannot be opened,
/// the backend will gracefully degrade into a no-op backend while still
/// keeping the last submitted samples for verification.
pub struct CpalBackend {
    device: Option<cpal::Device>,
    config: Option<cpal::StreamConfig>,
    last_samples: Option<Vec<f32>>,
}

impl CpalBackend {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device();
        let config = device
            .as_ref()
            .and_then(|d| d.default_output_config().ok())
            .map(|cfg| cfg.config());

        Self {
            device,
            config,
            last_samples: None,
        }
    }

    /// Get the configured sample rate if a device is available.
    pub fn sample_rate(&self) -> Option<u32> {
        self.config.as_ref().map(|c| c.sample_rate.0)
    }

    /// Returns a copy of the last samples submitted to the backend.
    pub fn last_samples(&self) -> Option<&[f32]> {
        self.last_samples.as_deref()
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for CpalBackend {
    fn start(&mut self) -> Result<()> {
        // Nothing to pre-start; the stream is created on demand.
        Ok(())
    }

    fn stop(&mut self) {}

    fn play_samples(&mut self, samples: &[f32]) -> Result<()> {
        self.last_samples = Some(samples.to_vec());

        // In this lightweight stub we only record the samples; actual playback
        // is handled by higher-level integration or alternate backends.
        Ok(())
    }
}
