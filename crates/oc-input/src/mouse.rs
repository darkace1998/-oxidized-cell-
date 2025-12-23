//! Mouse handling (cellMouse)

use bitflags::bitflags;

use crate::pad::{PadButtons, PadState};

bitflags! {
    /// Mouse button state flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct MouseButtons: u8 {
        const LEFT   = 0x01;
        const RIGHT  = 0x02;
        const MIDDLE = 0x04;
    }
}

/// Captures mouse position and buttons.
#[derive(Debug, Clone, Copy, Default)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
    pub buttons: MouseButtons,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            delta_x: 0,
            delta_y: 0,
            buttons: MouseButtons::empty(),
        }
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.delta_x = dx;
        self.delta_y = dy;
        self.x += dx;
        self.y += dy;
    }

    pub fn press(&mut self, button: MouseButtons) {
        self.buttons.insert(button);
    }

    pub fn release(&mut self, button: MouseButtons) {
        self.buttons.remove(button);
    }

    /// Map primary mouse buttons to pad buttons (useful for quick testing)
    pub fn apply_to_pad(&self, pad: &mut PadState) {
        pad.set_button(PadButtons::CROSS, self.buttons.contains(MouseButtons::LEFT));
        pad.set_button(PadButtons::CIRCLE, self.buttons.contains(MouseButtons::RIGHT));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_mouse_buttons() {
        let mut mouse = MouseState::new();
        mouse.press(MouseButtons::LEFT);
        assert!(mouse.buttons.contains(MouseButtons::LEFT));
        mouse.release(MouseButtons::LEFT);
        assert!(!mouse.buttons.contains(MouseButtons::LEFT));
    }
}
