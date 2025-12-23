//! SPU Memory Flow Controller (MFC)
//!
//! The MFC handles DMA transfers between SPU local storage and main memory.

use std::collections::VecDeque;

/// MFC command opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MfcCommand {
    /// Put (local to main)
    Put = 0x20,
    /// Put with barrier
    PutB = 0x21,
    /// Put with fence
    PutF = 0x22,
    /// Put unconditional
    PutU = 0x28,
    /// Get (main to local)
    Get = 0x40,
    /// Get with barrier
    GetB = 0x41,
    /// Get with fence
    GetF = 0x42,
    /// Get unconditional
    GetU = 0x48,
    /// Get Lock Line Unconditional (atomic reservation)
    GetLLAR = 0xD0,
    /// Put Lock Line Conditional (atomic store)
    PutLLC = 0xB4,
    /// Put Lock Line Unconditional
    PutLLUC = 0xB0,
    /// Barrier
    Barrier = 0xC0,
    /// Unknown/Invalid
    Unknown = 0xFF,
}

impl From<u8> for MfcCommand {
    fn from(value: u8) -> Self {
        match value {
            0x20 => Self::Put,
            0x21 => Self::PutB,
            0x22 => Self::PutF,
            0x28 => Self::PutU,
            0x40 => Self::Get,
            0x41 => Self::GetB,
            0x42 => Self::GetF,
            0x48 => Self::GetU,
            0xD0 => Self::GetLLAR,
            0xB4 => Self::PutLLC,
            0xB0 => Self::PutLLUC,
            0xC0 => Self::Barrier,
            _ => Self::Unknown,
        }
    }
}

/// MFC DMA command
#[derive(Debug, Clone)]
pub struct MfcDmaCommand {
    /// Local storage address
    pub lsa: u32,
    /// Effective address (main memory)
    pub ea: u64,
    /// Transfer size
    pub size: u32,
    /// Tag ID (0-31)
    pub tag: u8,
    /// Command opcode
    pub cmd: MfcCommand,
}

/// MFC state
pub struct Mfc {
    /// Command queue
    queue: VecDeque<MfcDmaCommand>,
    /// Tag group completion status (bit per tag)
    tag_status: u32,
    /// Atomic reservation address
    reservation_addr: u64,
    /// Atomic reservation data (128 bytes)
    reservation_data: [u8; 128],
    /// Reservation valid flag
    reservation_valid: bool,
}

impl Mfc {
    /// Create a new MFC
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(16),
            tag_status: 0xFFFFFFFF, // All tags initially complete
            reservation_addr: 0,
            reservation_data: [0; 128],
            reservation_valid: false,
        }
    }

    /// Queue a DMA command
    pub fn queue_command(&mut self, cmd: MfcDmaCommand) {
        // Mark tag as pending
        self.tag_status &= !(1 << cmd.tag);
        self.queue.push_back(cmd);
    }

    /// Check if queue is empty
    pub fn is_queue_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get next pending command
    pub fn pop_command(&mut self) -> Option<MfcDmaCommand> {
        self.queue.pop_front()
    }

    /// Mark a tag as complete
    pub fn complete_tag(&mut self, tag: u8) {
        self.tag_status |= 1 << tag;
    }

    /// Get tag status (bitmask of completed tags)
    pub fn get_tag_status(&self) -> u32 {
        self.tag_status
    }

    /// Check if specific tags are complete
    pub fn check_tags(&self, mask: u32) -> bool {
        (self.tag_status & mask) == mask
    }

    /// Set atomic reservation
    pub fn set_reservation(&mut self, addr: u64, data: &[u8]) {
        self.reservation_addr = addr & !127; // Align to 128 bytes
        self.reservation_data[..data.len().min(128)].copy_from_slice(&data[..data.len().min(128)]);
        self.reservation_valid = true;
    }

    /// Get reservation address
    pub fn get_reservation_addr(&self) -> u64 {
        self.reservation_addr
    }

    /// Get reservation data
    pub fn get_reservation_data(&self) -> &[u8; 128] {
        &self.reservation_data
    }

    /// Check if reservation is valid
    pub fn has_reservation(&self) -> bool {
        self.reservation_valid
    }

    /// Clear reservation
    pub fn clear_reservation(&mut self) {
        self.reservation_valid = false;
    }

    /// Get queue size
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is full (16 entries max)
    pub fn is_queue_full(&self) -> bool {
        self.queue.len() >= 16
    }
}

impl Default for Mfc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfc_creation() {
        let mfc = Mfc::new();
        assert!(mfc.is_queue_empty());
        assert_eq!(mfc.get_tag_status(), 0xFFFFFFFF);
    }

    #[test]
    fn test_mfc_command_queue() {
        let mut mfc = Mfc::new();

        let cmd = MfcDmaCommand {
            lsa: 0x1000,
            ea: 0x20000000,
            size: 0x4000,
            tag: 0,
            cmd: MfcCommand::Get,
        };

        mfc.queue_command(cmd);
        assert!(!mfc.is_queue_empty());
        assert_eq!(mfc.get_tag_status() & 1, 0); // Tag 0 pending

        let popped = mfc.pop_command().unwrap();
        assert_eq!(popped.lsa, 0x1000);
        assert!(mfc.is_queue_empty());

        mfc.complete_tag(0);
        assert_eq!(mfc.get_tag_status() & 1, 1); // Tag 0 complete
    }

    #[test]
    fn test_mfc_reservation() {
        let mut mfc = Mfc::new();

        assert!(!mfc.has_reservation());

        let data = [0x42u8; 128];
        mfc.set_reservation(0x1000, &data);

        assert!(mfc.has_reservation());
        assert_eq!(mfc.get_reservation_addr(), 0x1000);
        assert_eq!(mfc.get_reservation_data()[0], 0x42);

        mfc.clear_reservation();
        assert!(!mfc.has_reservation());
    }
}
