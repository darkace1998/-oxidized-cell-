//! SPU thread state

use std::sync::Arc;
use oc_memory::MemoryManager;
use crate::channels::SpuChannels;
use crate::mfc::Mfc;

/// SPU local storage size (256 KB)
pub const SPU_LS_SIZE: usize = 256 * 1024;

/// SPU thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpuThreadState {
    /// Thread is stopped
    Stopped,
    /// Thread is running
    Running,
    /// Thread is waiting on channel
    Waiting,
    /// Thread is halted (stop instruction)
    Halted,
}

/// SPU register set (128 x 128-bit)
#[derive(Clone)]
pub struct SpuRegisters {
    /// General Purpose Registers (128-bit each)
    pub gpr: [[u32; 4]; 128],
    /// Program Counter (instruction address in local storage)
    pub pc: u32,
}

impl Default for SpuRegisters {
    fn default() -> Self {
        Self {
            gpr: [[0; 4]; 128],
            pc: 0,
        }
    }
}

impl SpuRegisters {
    /// Read a register as 4 x u32
    #[inline]
    pub fn read_u32x4(&self, index: usize) -> [u32; 4] {
        self.gpr[index]
    }

    /// Write a register as 4 x u32
    #[inline]
    pub fn write_u32x4(&mut self, index: usize, value: [u32; 4]) {
        self.gpr[index] = value;
    }

    /// Read preferred slot (word 0) as u32
    #[inline]
    pub fn read_preferred_u32(&self, index: usize) -> u32 {
        self.gpr[index][0]
    }

    /// Write to preferred slot (word 0)
    #[inline]
    pub fn write_preferred_u32(&mut self, index: usize, value: u32) {
        self.gpr[index] = [value, 0, 0, 0];
    }
}

/// SPU thread
pub struct SpuThread {
    /// SPU ID (0-5 for PS3)
    pub id: u32,
    /// Thread name
    pub name: String,
    /// Register state
    pub regs: SpuRegisters,
    /// Local storage (256 KB)
    pub local_storage: Box<[u8; SPU_LS_SIZE]>,
    /// Thread state
    pub state: SpuThreadState,
    /// MFC (Memory Flow Controller)
    pub mfc: Mfc,
    /// SPU Channels
    pub channels: SpuChannels,
    /// Reference to main memory
    memory: Arc<MemoryManager>,
    /// Interrupt enabled
    pub interrupt_enabled: bool,
    /// Stop and signal value
    pub stop_signal: u32,
}

impl SpuThread {
    /// Create a new SPU thread
    pub fn new(id: u32, memory: Arc<MemoryManager>) -> Self {
        Self {
            id,
            name: format!("SPU Thread {}", id),
            regs: SpuRegisters::default(),
            local_storage: Box::new([0; SPU_LS_SIZE]),
            state: SpuThreadState::Stopped,
            mfc: Mfc::new(),
            channels: SpuChannels::new(),
            memory,
            interrupt_enabled: false,
            stop_signal: 0,
        }
    }

    /// Get the current program counter
    pub fn pc(&self) -> u32 {
        self.regs.pc
    }

    /// Set the program counter
    pub fn set_pc(&mut self, addr: u32) {
        self.regs.pc = addr & (SPU_LS_SIZE as u32 - 1);
    }

    /// Advance the program counter by 4 bytes
    pub fn advance_pc(&mut self) {
        self.regs.pc = (self.regs.pc + 4) & (SPU_LS_SIZE as u32 - 1);
    }

    /// Read from local storage (u32, big-endian)
    #[inline]
    pub fn ls_read_u32(&self, addr: u32) -> u32 {
        let addr = (addr & (SPU_LS_SIZE as u32 - 1)) as usize;
        u32::from_be_bytes([
            self.local_storage[addr],
            self.local_storage[addr + 1],
            self.local_storage[addr + 2],
            self.local_storage[addr + 3],
        ])
    }

    /// Write to local storage (u32, big-endian)
    #[inline]
    pub fn ls_write_u32(&mut self, addr: u32, value: u32) {
        let addr = (addr & (SPU_LS_SIZE as u32 - 1)) as usize;
        let bytes = value.to_be_bytes();
        self.local_storage[addr] = bytes[0];
        self.local_storage[addr + 1] = bytes[1];
        self.local_storage[addr + 2] = bytes[2];
        self.local_storage[addr + 3] = bytes[3];
    }

    /// Read from local storage (128-bit, big-endian)
    #[inline]
    pub fn ls_read_u128(&self, addr: u32) -> [u32; 4] {
        let addr = (addr & (SPU_LS_SIZE as u32 - 1) & !0xF) as usize;
        [
            u32::from_be_bytes([
                self.local_storage[addr],
                self.local_storage[addr + 1],
                self.local_storage[addr + 2],
                self.local_storage[addr + 3],
            ]),
            u32::from_be_bytes([
                self.local_storage[addr + 4],
                self.local_storage[addr + 5],
                self.local_storage[addr + 6],
                self.local_storage[addr + 7],
            ]),
            u32::from_be_bytes([
                self.local_storage[addr + 8],
                self.local_storage[addr + 9],
                self.local_storage[addr + 10],
                self.local_storage[addr + 11],
            ]),
            u32::from_be_bytes([
                self.local_storage[addr + 12],
                self.local_storage[addr + 13],
                self.local_storage[addr + 14],
                self.local_storage[addr + 15],
            ]),
        ]
    }

    /// Write to local storage (128-bit, big-endian)
    #[inline]
    pub fn ls_write_u128(&mut self, addr: u32, value: [u32; 4]) {
        let addr = (addr & (SPU_LS_SIZE as u32 - 1) & !0xF) as usize;
        for (i, word) in value.iter().enumerate() {
            let bytes = word.to_be_bytes();
            let offset = addr + i * 4;
            self.local_storage[offset] = bytes[0];
            self.local_storage[offset + 1] = bytes[1];
            self.local_storage[offset + 2] = bytes[2];
            self.local_storage[offset + 3] = bytes[3];
        }
    }

    /// Get reference to main memory
    pub fn memory(&self) -> &Arc<MemoryManager> {
        &self.memory
    }

    /// Start the thread
    pub fn start(&mut self) {
        self.state = SpuThreadState::Running;
    }

    /// Stop the thread
    pub fn stop(&mut self) {
        self.state = SpuThreadState::Stopped;
    }

    /// Check if thread is running
    pub fn is_running(&self) -> bool {
        self.state == SpuThreadState::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory() -> Arc<MemoryManager> {
        MemoryManager::new().unwrap()
    }

    #[test]
    fn test_spu_thread_creation() {
        let mem = create_test_memory();
        let thread = SpuThread::new(0, mem);
        
        assert_eq!(thread.id, 0);
        assert_eq!(thread.state, SpuThreadState::Stopped);
        assert_eq!(thread.pc(), 0);
    }

    #[test]
    fn test_local_storage_access() {
        let mem = create_test_memory();
        let mut thread = SpuThread::new(0, mem);

        thread.ls_write_u32(0x100, 0x12345678);
        assert_eq!(thread.ls_read_u32(0x100), 0x12345678);
    }

    #[test]
    fn test_local_storage_u128() {
        let mem = create_test_memory();
        let mut thread = SpuThread::new(0, mem);

        let value = [0x11111111, 0x22222222, 0x33333333, 0x44444444];
        thread.ls_write_u128(0x100, value);
        assert_eq!(thread.ls_read_u128(0x100), value);
    }

    #[test]
    fn test_pc_wrap() {
        let mem = create_test_memory();
        let mut thread = SpuThread::new(0, mem);

        thread.set_pc(SPU_LS_SIZE as u32 + 0x100);
        assert_eq!(thread.pc(), 0x100);
    }
}
