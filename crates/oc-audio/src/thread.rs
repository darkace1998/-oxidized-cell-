//! Audio thread

/// Audio thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioThreadState {
    Stopped,
    Running,
}

/// Audio thread
pub struct AudioThread {
    state: AudioThreadState,
    volume: f32,
}

impl AudioThread {
    pub fn new() -> Self {
        Self {
            state: AudioThreadState::Stopped,
            volume: 1.0,
        }
    }

    pub fn start(&mut self) {
        self.state = AudioThreadState::Running;
    }

    pub fn stop(&mut self) {
        self.state = AudioThreadState::Stopped;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }
}

impl Default for AudioThread {
    fn default() -> Self {
        Self::new()
    }
}
