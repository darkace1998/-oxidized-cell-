//! Input handling for oxidized-cell
//!
//! This crate provides comprehensive input handling for PS3 emulation including:
//! - DualShock 3 controller with Sixaxis motion and vibration
//! - PlayStation Move motion controller
//! - USB and Bluetooth controller support
//! - Guitar Hero / Rock Band instruments
//! - PlayStation Eye camera
//! - Microphone input
//! - Keyboard and mouse

// Core input modules
pub mod keyboard;
pub mod mapping;
pub mod mouse;
pub mod pad;

// Controller modules
pub mod bluetooth;
pub mod dualshock3;
pub mod usb;

// Special peripherals
pub mod camera;
pub mod instruments;
pub mod microphone;
pub mod move_controller;

// Re-exports for convenient access
pub use pad::Pad;

// DualShock 3
pub use dualshock3::{DualShock3, DualShock3Manager, SixaxisData, VibrationState};

// USB controllers
pub use usb::{UsbController, UsbControllerManager, UsbDeviceInfo};

// Bluetooth
pub use bluetooth::{BluetoothAdapter, BluetoothDevice, BluetoothManager};

// PlayStation Move
pub use move_controller::{MoveController, MoveManager, MoveMotionData, SphereColor};

// Instruments
pub use instruments::{
    DrumController, DrumPads, DrumType, GuitarController, GuitarFrets, GuitarType,
    InstrumentManager, TurntableController,
};

// Camera
pub use camera::{Camera, CameraManager, CameraResolution, CameraSettings, CameraType};

// Microphone
pub use microphone::{Microphone, MicrophoneConfig, MicrophoneManager, SampleRate};
