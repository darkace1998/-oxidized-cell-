//! Controller/gamepad handling (cellPad)

use bitflags::bitflags;

bitflags! {
    /// PS3 controller button flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PadButtons: u32 {
        const SELECT   = 0x0001;
        const L3       = 0x0002;
        const R3       = 0x0004;
        const START    = 0x0008;
        const DPAD_UP  = 0x0010;
        const DPAD_RIGHT = 0x0020;
        const DPAD_DOWN  = 0x0040;
        const DPAD_LEFT  = 0x0080;
        const L2       = 0x0100;
        const R2       = 0x0200;
        const L1       = 0x0400;
        const R1       = 0x0800;
        const TRIANGLE = 0x1000;
        const CIRCLE   = 0x2000;
        const CROSS    = 0x4000;
        const SQUARE   = 0x8000;
    }
}

/// Controller state
#[derive(Debug, Clone, Default)]
pub struct PadState {
    /// Button state (bitflags)
    pub buttons: u32,
    /// Left analog X (0-255, 128 = center)
    pub left_x: u8,
    /// Left analog Y (0-255, 128 = center)
    pub left_y: u8,
    /// Right analog X (0-255, 128 = center)
    pub right_x: u8,
    /// Right analog Y (0-255, 128 = center)
    pub right_y: u8,
    /// Pressure sensitivity for buttons (0-255 each)
    pub pressure: [u8; 12],
}

impl PadState {
    pub fn new() -> Self {
        Self {
            buttons: 0,
            left_x: 128,
            left_y: 128,
            right_x: 128,
            right_y: 128,
            pressure: [0; 12],
        }
    }

    pub fn is_button_pressed(&self, button: PadButtons) -> bool {
        (self.buttons & button.bits()) != 0
    }

    pub fn set_button(&mut self, button: PadButtons, pressed: bool) {
        if pressed {
            self.buttons |= button.bits();
        } else {
            self.buttons &= !button.bits();
        }
    }

    /// Update the left analog stick values (0-255).
    pub fn set_left_stick(&mut self, x: u8, y: u8) {
        self.left_x = x;
        self.left_y = y;
    }

    /// Update the right analog stick values (0-255).
    pub fn set_right_stick(&mut self, x: u8, y: u8) {
        self.right_x = x;
        self.right_y = y;
    }
}

/// Pad handler for a single controller
pub struct Pad {
    /// Controller port (0-6)
    pub port: u8,
    /// Current state
    pub state: PadState,
    /// Connected flag
    pub connected: bool,
}

impl Pad {
    pub fn new(port: u8) -> Self {
        Self {
            port,
            state: PadState::new(),
            connected: false,
        }
    }

    pub fn connect(&mut self) {
        self.connected = true;
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        self.state = PadState::new();
    }

    /// Apply a button press or release
    pub fn set_button(&mut self, button: PadButtons, pressed: bool) {
        self.state.set_button(button, pressed);
    }

    /// Update analog stick positions.
    pub fn set_left_stick(&mut self, x: u8, y: u8) {
        self.state.set_left_stick(x, y);
    }

    pub fn set_right_stick(&mut self, x: u8, y: u8) {
        self.state.set_right_stick(x, y);
    }
}

/// Manages a collection of pads.
pub struct PadManager {
    pads: Vec<Pad>,
}

impl PadManager {
    pub fn new(count: usize) -> Self {
        let mut pads = Vec::with_capacity(count);
        for idx in 0..count {
            pads.push(Pad::new(idx as u8));
        }
        Self { pads }
    }

    pub fn get_pad_mut(&mut self, port: u8) -> Option<&mut Pad> {
        self.pads.get_mut(port as usize)
    }

    pub fn connect_all(&mut self) {
        for pad in &mut self.pads {
            pad.connect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_creation() {
        let pad = Pad::new(0);
        assert_eq!(pad.port, 0);
        assert!(!pad.connected);
    }

    #[test]
    fn test_pad_state() {
        let mut state = PadState::new();
        assert!(!state.is_button_pressed(PadButtons::CROSS));
        
        state.set_button(PadButtons::CROSS, true);
        assert!(state.is_button_pressed(PadButtons::CROSS));
        
        state.set_button(PadButtons::CROSS, false);
        assert!(!state.is_button_pressed(PadButtons::CROSS));
    }

    #[test]
    fn test_analog_updates() {
        let mut state = PadState::new();
        state.set_left_stick(200, 100);
        assert_eq!(state.left_x, 200);
        assert_eq!(state.left_y, 100);
    }
}
