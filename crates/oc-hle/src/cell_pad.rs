//! cellPad HLE

use std::sync::{Mutex, OnceLock};

use oc_input::pad::PadManager;

use crate::module::HleModule;

static PAD_MANAGER: OnceLock<Mutex<PadManager>> = OnceLock::new();

fn pad_manager() -> &'static Mutex<PadManager> {
    PAD_MANAGER.get_or_init(|| Mutex::new(PadManager::new(4)))
}

fn cell_pad_init(_args: &[u64]) -> i64 {
    if let Ok(mut mgr) = pad_manager().lock() {
        mgr.connect_all();
    }
    0
}

fn cell_pad_end(_args: &[u64]) -> i64 {
    if let Ok(mut mgr) = pad_manager().lock() {
        for idx in 0..4 {
            if let Some(pad) = mgr.get_pad_mut(idx) {
                pad.disconnect();
            }
        }
    }
    0
}

/// Minimal cellPadGetData implementation: args[0] = port, args[1] = button bits.
fn cell_pad_get_data(args: &[u64]) -> i64 {
    let port = args.get(0).copied().unwrap_or(0) as u8;
    let buttons = args.get(1).copied().unwrap_or(0) as u32;

    if let Ok(mut mgr) = pad_manager().lock() {
        if let Some(pad) = mgr.get_pad_mut(port) {
            pad.connect();
            pad.state.buttons = buttons;
            return 0;
        }
    }

    -1
}

/// Register cellPad module with a handful of essential entry points.
pub fn module() -> HleModule {
    let mut module = HleModule::new("cellPad");
    module.register(0x578E3C98, cell_pad_init); // cellPadInit
    module.register(0x3733EA3C, cell_pad_end); // cellPadEnd
    module.register(0x1CF98800, cell_pad_get_data); // cellPadGetData
    module
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_init_connects_devices() {
        cell_pad_init(&[]);
        let mgr = pad_manager();
        {
            let mut guard = mgr.lock().unwrap();
            let pad0 = guard.get_pad_mut(0).unwrap();
            assert!(pad0.connected);
        }
        // Simulate button state update
        cell_pad_get_data(&[0, 0x4000]);
        {
            let mut guard = mgr.lock().unwrap();
            let pad0 = guard.get_pad_mut(0).unwrap();
            assert_eq!(pad0.state.buttons, 0x4000);
        }
        cell_pad_end(&[]);
    }
}
