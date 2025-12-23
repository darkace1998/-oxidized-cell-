//! Audio mixer

/// Simple audio mixer that averages multiple input streams and applies a
/// master volume. All samples are expected to be interleaved f32 PCM in the
/// range [-1.0, 1.0].
#[derive(Debug, Clone)]
pub struct Mixer {
    volume: f32,
}

impl Mixer {
    /// Create a new mixer with unity volume.
    pub fn new() -> Self {
        Self { volume: 1.0 }
    }

    /// Set master volume (clamped between 0.0 and 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Get the current volume.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// Mix multiple streams into a single buffer. Streams are averaged to
    /// avoid clipping and then scaled by the master volume.
    pub fn mix(&self, inputs: &[Vec<f32>]) -> Vec<f32> {
        if inputs.is_empty() {
            return Vec::new();
        }

        let max_len = inputs.iter().map(|v| v.len()).max().unwrap_or(0);
        let mut output = vec![0.0f32; max_len];

        for input in inputs {
            for (idx, sample) in input.iter().enumerate() {
                output[idx] += *sample;
            }
        }

        let count = inputs.len() as f32;
        for sample in &mut output {
            *sample = (*sample / count) * self.volume;
            *sample = sample.clamp(-1.0, 1.0);
        }

        output
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_and_scales() {
        let mut mixer = Mixer::new();
        mixer.set_volume(0.5);
        let a = vec![1.0, 1.0];
        let b = vec![-1.0, -1.0];
        let mixed = mixer.mix(&[a, b]);
        assert_eq!(mixed.len(), 2);
        // (1 + -1) / 2 = 0, scaled by 0.5 -> 0
        assert!((mixed[0] - 0.0).abs() < f32::EPSILON);
    }
}
