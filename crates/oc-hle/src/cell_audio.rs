//! cellAudio HLE

use std::sync::{Mutex, OnceLock};

use oc_audio::cell_audio::CellAudio;

use crate::module::HleModule;

// Simple global cellAudio instance so the lightweight HLE entry points can
// control the audio thread.
static AUDIO: OnceLock<Mutex<CellAudio>> = OnceLock::new();

fn audio_instance() -> &'static Mutex<CellAudio> {
    AUDIO.get_or_init(|| Mutex::new(CellAudio::with_cpal_backend()))
}

fn cell_audio_init(_args: &[u64]) -> i64 {
    let _ = audio_instance().lock().map(|mut audio| audio.init());
    0
}

fn cell_audio_quit(_args: &[u64]) -> i64 {
    if let Ok(mut audio) = audio_instance().lock() {
        audio.shutdown();
    }
    0
}

fn cell_audio_set_volume(args: &[u64]) -> i64 {
    if let Some(raw) = args.first() {
        let volume_bits = *raw as u32;
        let volume = f32::from_bits(volume_bits);
        if let Ok(mut audio) = audio_instance().lock() {
            audio.set_volume(volume);
        }
    }
    0
}

/// Register the minimal cellAudio module.
pub fn module() -> HleModule {
    let mut module = HleModule::new("cellAudio");
    // NIDs are placeholders sufficient for dispatch in tests.
    module.register(0xD8B9C89B, cell_audio_init); // cellAudioInit
    module.register(0x2A978EF8, cell_audio_quit); // cellAudioQuit
    module.register(0x77FEAE3C, cell_audio_set_volume); // cellAudioSetVolume
    module
}
