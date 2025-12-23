//! Page flags and management

use bitflags::bitflags;

bitflags! {
    /// Page protection and attribute flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PageFlags: u32 {
        /// Page is readable
        const READ    = 0b0001;
        /// Page is writable
        const WRITE   = 0b0010;
        /// Page is executable
        const EXECUTE = 0b0100;
        /// Page is memory-mapped I/O
        const MMIO    = 0b1000;

        /// Read and write access
        const RW  = Self::READ.bits() | Self::WRITE.bits();
        /// Read, write, and execute access
        const RWX = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
        /// Read and execute access
        const RX  = Self::READ.bits() | Self::EXECUTE.bits();
    }
}

impl Default for PageFlags {
    fn default() -> Self {
        Self::empty()
    }
}
