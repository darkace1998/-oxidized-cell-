//! Tests for 32-bit address space validation

use oc_memory::{
    constants::*, MemoryManager, PageFlags,
};

#[test]
fn test_address_space_boundaries() {
    let mem = MemoryManager::new().unwrap();
    
    // Test that we can access main memory
    let addr = MAIN_MEM_BASE;
    mem.write::<u32>(addr, 0xDEADBEEF).unwrap();
    assert_eq!(mem.read::<u32>(addr).unwrap(), 0xDEADBEEF);
    
    // Test upper boundary of main memory
    let addr = MAIN_MEM_BASE + MAIN_MEM_SIZE - 4;
    mem.write::<u32>(addr, 0xCAFEBABE).unwrap();
    assert_eq!(mem.read::<u32>(addr).unwrap(), 0xCAFEBABE);
    
    // Test user memory boundaries
    let alloc_addr = mem.allocate(0x1000, 0x1000, PageFlags::RW).unwrap();
    assert!(alloc_addr >= USER_MEM_BASE);
    assert!(alloc_addr < USER_MEM_BASE + USER_MEM_SIZE);
    
    mem.write::<u64>(alloc_addr, 0x1234567890ABCDEF).unwrap();
    assert_eq!(mem.read::<u64>(alloc_addr).unwrap(), 0x1234567890ABCDEF);
}

#[test]
fn test_memory_region_isolation() {
    let mem = MemoryManager::new().unwrap();
    
    // Write to main memory
    let main_addr = MAIN_MEM_BASE + 0x1000;
    mem.write::<u32>(main_addr, 0x11111111).unwrap();
    
    // Write to user memory
    let user_addr = mem.allocate(0x1000, 0x1000, PageFlags::RW).unwrap();
    mem.write::<u32>(user_addr, 0x22222222).unwrap();
    
    // Write to stack
    let stack_addr = STACK_BASE + 0x1000;
    mem.write::<u32>(stack_addr, 0x33333333).unwrap();
    
    // Verify values are independent
    assert_eq!(mem.read::<u32>(main_addr).unwrap(), 0x11111111);
    assert_eq!(mem.read::<u32>(user_addr).unwrap(), 0x22222222);
    assert_eq!(mem.read::<u32>(stack_addr).unwrap(), 0x33333333);
}

#[test]
fn test_overlapping_allocations_prevention() {
    let mem = MemoryManager::new().unwrap();
    
    // Allocate a large block
    let size = 0x100000; // 1 MB
    let addr1 = mem.allocate(size, 0x1000, PageFlags::RW).unwrap();
    
    // Write a pattern to the first allocation
    for i in 0..100 {
        mem.write::<u32>(addr1 + i * 4, i).unwrap();
    }
    
    // Allocate another block
    let addr2 = mem.allocate(size, 0x1000, PageFlags::RW).unwrap();
    
    // Ensure they don't overlap
    assert!(addr2 >= addr1 + size || addr1 >= addr2 + size);
    
    // Write to second allocation
    mem.write::<u32>(addr2, 0xFFFFFFFF).unwrap();
    
    // Verify first allocation is unchanged
    assert_eq!(mem.read::<u32>(addr1).unwrap(), 0);
    assert_eq!(mem.read::<u32>(addr1 + 4).unwrap(), 1);
}

#[test]
fn test_32bit_address_wraparound() {
    let mem = MemoryManager::new().unwrap();
    
    // Test that we properly handle 32-bit addresses (no 64-bit overflow)
    let max_valid_addr = STACK_BASE + STACK_SIZE - 4;
    mem.write::<u32>(max_valid_addr, 0x12345678).unwrap();
    assert_eq!(mem.read::<u32>(max_valid_addr).unwrap(), 0x12345678);
}

#[test]
fn test_memory_region_permissions() {
    let mem = MemoryManager::new().unwrap();
    
    // Main memory should be RWX
    let main_addr = MAIN_MEM_BASE + 0x1000;
    mem.write::<u32>(main_addr, 0x12345678).unwrap();
    assert_eq!(mem.read::<u32>(main_addr).unwrap(), 0x12345678);
    
    // Stack should be RW (not executable)
    let stack_addr = STACK_BASE + 0x1000;
    mem.write::<u32>(stack_addr, 0xABCDEF00).unwrap();
    assert_eq!(mem.read::<u32>(stack_addr).unwrap(), 0xABCDEF00);
}

#[test]
fn test_unaligned_access() {
    let mem = MemoryManager::new().unwrap();
    
    let addr = MAIN_MEM_BASE + 1; // Unaligned address
    
    // Should work due to unaligned read/write support
    mem.write::<u32>(addr, 0x12345678).unwrap();
    assert_eq!(mem.read::<u32>(addr).unwrap(), 0x12345678);
    
    mem.write::<u64>(addr, 0xDEADBEEFCAFEBABE).unwrap();
    assert_eq!(mem.read::<u64>(addr).unwrap(), 0xDEADBEEFCAFEBABE);
}

#[test]
fn test_page_aligned_allocations() {
    let mem = MemoryManager::new().unwrap();
    
    // All allocations should be page-aligned
    for _ in 0..10 {
        let addr = mem.allocate(0x5555, 0x1000, PageFlags::RW).unwrap();
        assert_eq!(addr % PAGE_SIZE, 0, "Allocation not page-aligned");
    }
}

#[test]
fn test_allocation_size_rounding() {
    let mem = MemoryManager::new().unwrap();
    
    // Allocate a non-page-aligned size
    let addr1 = mem.allocate(0x1001, 0x1000, PageFlags::RW).unwrap();
    let addr2 = mem.allocate(0x1000, 0x1000, PageFlags::RW).unwrap();
    
    // Should be separated by at least 2 pages (0x2000) since 0x1001 rounds up to 0x2000
    assert!(addr2 >= addr1 + 0x2000);
}

#[test]
fn test_big_endian_operations() {
    let mem = MemoryManager::new().unwrap();
    
    let addr = MAIN_MEM_BASE + 0x1000;
    
    // Test BE16
    mem.write_be16(addr, 0x1234).unwrap();
    assert_eq!(mem.read_be16(addr).unwrap(), 0x1234);
    
    // Test BE32
    mem.write_be32(addr + 2, 0x12345678).unwrap();
    assert_eq!(mem.read_be32(addr + 2).unwrap(), 0x12345678);
    
    // Test BE64
    mem.write_be64(addr + 8, 0xDEADBEEFCAFEBABE).unwrap();
    assert_eq!(mem.read_be64(addr + 8).unwrap(), 0xDEADBEEFCAFEBABE);
}
