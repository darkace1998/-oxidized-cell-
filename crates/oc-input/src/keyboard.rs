//! Keyboard handling (cellKb)

use std::collections::HashSet;

use crate::{mapping::InputMapping, pad::PadState};

/// Tracks currently pressed keyboard keys.
#[derive(Default, Debug, Clone)]
pub struct KeyboardState {
    pressed: HashSet<String>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }

    pub fn press(&mut self, key: &str) {
        self.pressed.insert(key.to_string());
    }

    pub fn release(&mut self, key: &str) {
        self.pressed.remove(key);
    }

    pub fn is_pressed(&self, key: &str) -> bool {
        self.pressed.contains(key)
    }

    /// Apply the current keyboard state to a pad state using the provided
    /// mapping.
    pub fn update_pad(&self, pad_state: &mut PadState, mapping: &InputMapping) {
        for key in &self.pressed {
            mapping.apply_key(pad_state, key, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_pressed_keys() {
        let mut kb = KeyboardState::new();
        let mapping = InputMapping::default();
        let mut pad = PadState::new();

        kb.press("X");
        kb.update_pad(&mut pad, &mapping);
        assert!(kb.is_pressed("X"));
        assert!(pad.is_button_pressed(crate::pad::PadButtons::CROSS));

        kb.release("X");
        assert!(!kb.is_pressed("X"));
    }
}
