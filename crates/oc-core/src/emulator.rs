//! Main emulator structure

use crate::config::Config;
use crate::error::Result;

/// Emulator state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulatorState {
    /// Emulator is stopped
    Stopped,
    /// Emulator is running
    Running,
    /// Emulator is paused
    Paused,
}

/// Main emulator structure
pub struct Emulator {
    /// Current emulator state
    state: EmulatorState,
    /// Configuration
    config: Config,
}

impl Emulator {
    /// Create a new emulator instance
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            state: EmulatorState::Stopped,
            config,
        })
    }

    /// Get the current state
    pub fn state(&self) -> EmulatorState {
        self.state
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Start the emulator
    pub fn start(&mut self) -> Result<()> {
        self.state = EmulatorState::Running;
        tracing::info!("Emulator started");
        Ok(())
    }

    /// Pause the emulator
    pub fn pause(&mut self) -> Result<()> {
        if self.state == EmulatorState::Running {
            self.state = EmulatorState::Paused;
            tracing::info!("Emulator paused");
        }
        Ok(())
    }

    /// Resume the emulator
    pub fn resume(&mut self) -> Result<()> {
        if self.state == EmulatorState::Paused {
            self.state = EmulatorState::Running;
            tracing::info!("Emulator resumed");
        }
        Ok(())
    }

    /// Stop the emulator
    pub fn stop(&mut self) -> Result<()> {
        self.state = EmulatorState::Stopped;
        tracing::info!("Emulator stopped");
        Ok(())
    }

    /// Check if the emulator is running
    pub fn is_running(&self) -> bool {
        self.state == EmulatorState::Running
    }

    /// Check if the emulator is paused
    pub fn is_paused(&self) -> bool {
        self.state == EmulatorState::Paused
    }

    /// Check if the emulator is stopped
    pub fn is_stopped(&self) -> bool {
        self.state == EmulatorState::Stopped
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            state: EmulatorState::Stopped,
            config: Config::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_creation() {
        let config = Config::default();
        let emu = Emulator::new(config).unwrap();
        assert_eq!(emu.state(), EmulatorState::Stopped);
    }

    #[test]
    fn test_emulator_state_transitions() {
        let mut emu = Emulator::default();

        assert!(emu.is_stopped());

        emu.start().unwrap();
        assert!(emu.is_running());

        emu.pause().unwrap();
        assert!(emu.is_paused());

        emu.resume().unwrap();
        assert!(emu.is_running());

        emu.stop().unwrap();
        assert!(emu.is_stopped());
    }
}
