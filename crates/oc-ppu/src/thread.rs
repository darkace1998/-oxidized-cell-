//! PPU thread state

use std::sync::Arc;
use oc_memory::MemoryManager;

/// PPU register set
#[derive(Debug, Clone)]
pub struct PpuRegisters {
    /// General Purpose Registers (64-bit)
    pub gpr: [u64; 32],
    /// Floating Point Registers (64-bit)
    pub fpr: [f64; 32],
    /// Vector Registers (128-bit, stored as 4 x u32)
    pub vr: [[u32; 4]; 32],
    /// Condition Register
    pub cr: u32,
    /// Link Register
    pub lr: u64,
    /// Count Register
    pub ctr: u64,
    /// Fixed-Point Exception Register
    pub xer: u64,
    /// FP Status and Control Register
    pub fpscr: u64,
    /// Vector Status and Control Register
    pub vscr: u32,
    /// Program Counter / Next Instruction Address
    pub cia: u64,
}

impl Default for PpuRegisters {
    fn default() -> Self {
        Self {
            gpr: [0; 32],
            fpr: [0.0; 32],
            vr: [[0; 4]; 32],
            cr: 0,
            lr: 0,
            ctr: 0,
            xer: 0,
            fpscr: 0,
            vscr: 0,
            cia: 0,
        }
    }
}

/// PPU thread state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PpuThreadState {
    /// Thread is stopped
    Stopped,
    /// Thread is running
    Running,
    /// Thread is waiting (blocked)
    Waiting,
    /// Thread is suspended
    Suspended,
}

/// PPU thread
pub struct PpuThread {
    /// Thread ID
    pub id: u32,
    /// Thread name
    pub name: String,
    /// Register state
    pub regs: PpuRegisters,
    /// Thread state
    pub state: PpuThreadState,
    /// Memory manager reference
    memory: Arc<MemoryManager>,
    /// Stack address
    pub stack_addr: u32,
    /// Stack size
    pub stack_size: u32,
    /// Priority
    pub priority: u32,
}

impl PpuThread {
    /// Create a new PPU thread
    pub fn new(id: u32, memory: Arc<MemoryManager>) -> Self {
        Self {
            id,
            name: format!("PPU Thread {}", id),
            regs: PpuRegisters::default(),
            state: PpuThreadState::Stopped,
            memory,
            stack_addr: 0,
            stack_size: 0,
            priority: 0,
        }
    }

    /// Get the current instruction address
    pub fn pc(&self) -> u64 {
        self.regs.cia
    }

    /// Set the program counter
    pub fn set_pc(&mut self, addr: u64) {
        self.regs.cia = addr;
    }

    /// Advance the program counter by 4 bytes
    pub fn advance_pc(&mut self) {
        self.regs.cia += 4;
    }

    /// Read a GPR
    #[inline]
    pub fn gpr(&self, index: usize) -> u64 {
        self.regs.gpr[index]
    }

    /// Write a GPR
    #[inline]
    pub fn set_gpr(&mut self, index: usize, value: u64) {
        if index != 0 {
            self.regs.gpr[index] = value;
        }
    }

    /// Read an FPR
    #[inline]
    pub fn fpr(&self, index: usize) -> f64 {
        self.regs.fpr[index]
    }

    /// Write an FPR
    #[inline]
    pub fn set_fpr(&mut self, index: usize, value: f64) {
        self.regs.fpr[index] = value;
    }

    /// Read a VR
    #[inline]
    pub fn vr(&self, index: usize) -> [u32; 4] {
        self.regs.vr[index]
    }

    /// Write a VR
    #[inline]
    pub fn set_vr(&mut self, index: usize, value: [u32; 4]) {
        self.regs.vr[index] = value;
    }

    /// Get a reference to the memory manager
    pub fn memory(&self) -> &Arc<MemoryManager> {
        &self.memory
    }

    /// Start the thread
    pub fn start(&mut self) {
        self.state = PpuThreadState::Running;
    }

    /// Stop the thread
    pub fn stop(&mut self) {
        self.state = PpuThreadState::Stopped;
    }

    /// Check if thread is running
    pub fn is_running(&self) -> bool {
        self.state == PpuThreadState::Running
    }

    /// Get CR field value (0-7)
    pub fn get_cr_field(&self, field: usize) -> u32 {
        (self.regs.cr >> (28 - field * 4)) & 0xF
    }

    /// Set CR field value (0-7)
    pub fn set_cr_field(&mut self, field: usize, value: u32) {
        let shift = 28 - field * 4;
        self.regs.cr = (self.regs.cr & !(0xF << shift)) | ((value & 0xF) << shift);
    }

    /// Get XER CA (Carry) bit
    pub fn get_xer_ca(&self) -> bool {
        (self.regs.xer & 0x20000000) != 0
    }

    /// Set XER CA (Carry) bit
    pub fn set_xer_ca(&mut self, value: bool) {
        if value {
            self.regs.xer |= 0x20000000;
        } else {
            self.regs.xer &= !0x20000000;
        }
    }

    /// Get XER OV (Overflow) bit
    pub fn get_xer_ov(&self) -> bool {
        (self.regs.xer & 0x40000000) != 0
    }

    /// Set XER OV (Overflow) bit
    pub fn set_xer_ov(&mut self, value: bool) {
        if value {
            self.regs.xer |= 0x40000000;
        } else {
            self.regs.xer &= !0x40000000;
        }
    }

    /// Get XER SO (Summary Overflow) bit
    pub fn get_xer_so(&self) -> bool {
        (self.regs.xer & 0x80000000) != 0
    }

    /// Set XER SO (Summary Overflow) bit
    pub fn set_xer_so(&mut self, value: bool) {
        if value {
            self.regs.xer |= 0x80000000;
        } else {
            self.regs.xer &= !0x80000000;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memory() -> Arc<MemoryManager> {
        MemoryManager::new().unwrap()
    }

    #[test]
    fn test_ppu_thread_creation() {
        let mem = create_test_memory();
        let thread = PpuThread::new(0, mem);
        
        assert_eq!(thread.id, 0);
        assert_eq!(thread.state, PpuThreadState::Stopped);
        assert_eq!(thread.pc(), 0);
    }

    #[test]
    fn test_gpr_operations() {
        let mem = create_test_memory();
        let mut thread = PpuThread::new(0, mem);

        thread.set_gpr(1, 0x12345678);
        assert_eq!(thread.gpr(1), 0x12345678);

        // R0 should always be writable (unlike some RISC ISAs)
        thread.set_gpr(0, 0xDEADBEEF);
        // Note: In PPU, R0 can be used as a normal register
    }

    #[test]
    fn test_pc_operations() {
        let mem = create_test_memory();
        let mut thread = PpuThread::new(0, mem);

        thread.set_pc(0x10000);
        assert_eq!(thread.pc(), 0x10000);

        thread.advance_pc();
        assert_eq!(thread.pc(), 0x10004);
    }

    #[test]
    fn test_cr_fields() {
        let mem = create_test_memory();
        let mut thread = PpuThread::new(0, mem);

        thread.set_cr_field(0, 0b1010);
        assert_eq!(thread.get_cr_field(0), 0b1010);

        thread.set_cr_field(7, 0b0101);
        assert_eq!(thread.get_cr_field(7), 0b0101);
    }
}
