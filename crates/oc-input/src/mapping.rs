//! Input mapping

use oc_core::config::KeyboardMapping;

use crate::pad::{PadButtons, PadState};

/// Maps host input events to PS3 pad buttons.
#[derive(Debug, Clone)]
pub struct InputMapping {
    pub keyboard: KeyboardMapping,
}

impl InputMapping {
    pub fn new(keyboard: KeyboardMapping) -> Self {
        Self { keyboard }
    }

    /// Translate a keyboard key into a PS3 button.
    pub fn map_key(&self, key: &str) -> Option<PadButtons> {
        let key = key.to_lowercase();
        macro_rules! cmp {
            ($btn:ident, $field:ident) => {
                if key == self.keyboard.$field.to_lowercase() {
                    return Some(PadButtons::$btn);
                }
            };
        }

        cmp!(CROSS, cross);
        cmp!(CIRCLE, circle);
        cmp!(SQUARE, square);
        cmp!(TRIANGLE, triangle);
        cmp!(L1, l1);
        cmp!(L2, l2);
        cmp!(L3, l3);
        cmp!(R1, r1);
        cmp!(R2, r2);
        cmp!(R3, r3);
        cmp!(START, start);
        cmp!(SELECT, select);
        cmp!(DPAD_UP, dpad_up);
        cmp!(DPAD_DOWN, dpad_down);
        cmp!(DPAD_LEFT, dpad_left);
        cmp!(DPAD_RIGHT, dpad_right);

        None
    }

    /// Apply a keyboard press/release to a pad state.
    pub fn apply_key(&self, state: &mut PadState, key: &str, pressed: bool) {
        if let Some(button) = self.map_key(key) {
            state.set_button(button, pressed);
        }
    }
}

impl Default for InputMapping {
    fn default() -> Self {
        Self {
            keyboard: KeyboardMapping::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_keys_to_buttons() {
        let mapping = InputMapping::default();
        let mut state = PadState::new();

        mapping.apply_key(&mut state, "X", true);
        assert!(state.is_button_pressed(PadButtons::CROSS));

        mapping.apply_key(&mut state, "X", false);
        assert!(!state.is_button_pressed(PadButtons::CROSS));
    }
}
