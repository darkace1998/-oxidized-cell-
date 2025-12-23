//! PS3 memory map constants

/// Main memory base address
pub const MAIN_MEM_BASE: u32 = 0x0000_0000;
/// Main memory size (256 MB)
pub const MAIN_MEM_SIZE: u32 = 0x1000_0000;

/// User memory base address
pub const USER_MEM_BASE: u32 = 0x2000_0000;
/// User memory size (256 MB)
pub const USER_MEM_SIZE: u32 = 0x1000_0000;

/// RSX mapped memory base
pub const RSX_MAP_BASE: u32 = 0x3000_0000;
/// RSX mapped memory size
pub const RSX_MAP_SIZE: u32 = 0x1000_0000;

/// RSX I/O (control registers) base
pub const RSX_IO_BASE: u32 = 0x4000_0000;
/// RSX I/O size
pub const RSX_IO_SIZE: u32 = 0x0010_0000;

/// RSX local memory (VRAM) base
pub const RSX_MEM_BASE: u32 = 0xC000_0000;
/// RSX local memory size (256 MB)
pub const RSX_MEM_SIZE: u32 = 0x1000_0000;

/// Stack area base
pub const STACK_BASE: u32 = 0xD000_0000;
/// Stack area size
pub const STACK_SIZE: u32 = 0x1000_0000;

/// SPU local storage base
pub const SPU_BASE: u32 = 0xE000_0000;
/// SPU local storage size per SPU (256 KB)
pub const SPU_LS_SIZE: u32 = 0x0004_0000;

/// Standard page size (4 KB)
pub const PAGE_SIZE: u32 = 0x1000;
/// Large page size (1 MB)
pub const LARGE_PAGE_SIZE: u32 = 0x10_0000;

/// Reservation granularity for SPU atomics (128 bytes = cache line)
pub const RESERVATION_GRANULARITY: u32 = 128;

/// Total address space size (4 GB, 32-bit)
pub const ADDRESS_SPACE_SIZE: usize = 0x1_0000_0000;

/// Number of pages in the address space
pub const NUM_PAGES: usize = ADDRESS_SPACE_SIZE / PAGE_SIZE as usize;

/// Number of reservations in the address space
pub const NUM_RESERVATIONS: usize = ADDRESS_SPACE_SIZE / RESERVATION_GRANULARITY as usize;
