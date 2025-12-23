//! Test ROM execution framework for PPU instruction validation
//!
//! This module provides infrastructure for loading and executing test ROMs
//! to validate PPU instruction implementations against known-good behavior.

use oc_ppu::{PpuInterpreter, PpuThread};
use oc_memory::MemoryManager;
use oc_core::error::PpuError;
use std::sync::Arc;

/// Test ROM format
///
/// A simple format for test ROMs:
/// ```
/// Offset | Size | Description
/// -------|------|------------
/// 0x0000 | 4    | Magic number: "PPUT" (0x50505554)
/// 0x0004 | 4    | Version: 1
/// 0x0008 | 4    | Entry point address
/// 0x000C | 4    | Code size
/// 0x0010 | 4    | Initial register count
/// 0x0014 | 4    | Expected register count
/// 0x0018 | ...  | Initial register values (reg_num:u8, value:u64) * count
/// ...    | ...  | Code bytes
/// ...    | ...  | Expected register values (reg_num:u8, value:u64) * count
/// ```
const TEST_ROM_MAGIC: u32 = 0x50505554; // "PPUT"
const TEST_ROM_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct TestRom {
    /// Entry point address
    pub entry_point: u32,
    /// Code bytes
    pub code: Vec<u8>,
    /// Initial register states (register number, value)
    pub initial_regs: Vec<(u8, u64)>,
    /// Expected register states after execution (register number, value)
    pub expected_regs: Vec<(u8, u64)>,
}

impl TestRom {
    /// Load a test ROM from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 0x18 {
            return Err("Test ROM too small".to_string());
        }

        // Parse header
        let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        if magic != TEST_ROM_MAGIC {
            return Err(format!("Invalid magic: 0x{:08X}", magic));
        }

        let version = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        if version != TEST_ROM_VERSION {
            return Err(format!("Unsupported version: {}", version));
        }

        let entry_point = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let code_size = u32::from_be_bytes([data[12], data[13], data[14], data[15]]) as usize;
        let initial_reg_count = u32::from_be_bytes([data[16], data[17], data[18], data[19]]) as usize;
        let expected_reg_count = u32::from_be_bytes([data[20], data[21], data[22], data[23]]) as usize;

        let mut offset = 0x18;

        // Parse initial registers
        let mut initial_regs = Vec::new();
        for _ in 0..initial_reg_count {
            if offset + 9 > data.len() {
                return Err("Unexpected end of ROM (initial regs)".to_string());
            }
            let reg_num = data[offset];
            let value = u64::from_be_bytes([
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
                data[offset + 8],
            ]);
            initial_regs.push((reg_num, value));
            offset += 9;
        }

        // Parse code
        if offset + code_size > data.len() {
            return Err("Unexpected end of ROM (code)".to_string());
        }
        let code = data[offset..offset + code_size].to_vec();
        offset += code_size;

        // Parse expected registers
        let mut expected_regs = Vec::new();
        for _ in 0..expected_reg_count {
            if offset + 9 > data.len() {
                return Err("Unexpected end of ROM (expected regs)".to_string());
            }
            let reg_num = data[offset];
            let value = u64::from_be_bytes([
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
                data[offset + 8],
            ]);
            expected_regs.push((reg_num, value));
            offset += 9;
        }

        Ok(TestRom {
            entry_point,
            code,
            initial_regs,
            expected_regs,
        })
    }

    /// Create a simple test ROM with code and expected results
    pub fn create_simple(entry_point: u32, code: Vec<u8>, initial_regs: Vec<(u8, u64)>, expected_regs: Vec<(u8, u64)>) -> Vec<u8> {
        let mut rom = Vec::new();

        // Header
        rom.extend_from_slice(&TEST_ROM_MAGIC.to_be_bytes());
        rom.extend_from_slice(&TEST_ROM_VERSION.to_be_bytes());
        rom.extend_from_slice(&entry_point.to_be_bytes());
        rom.extend_from_slice(&(code.len() as u32).to_be_bytes());
        rom.extend_from_slice(&(initial_regs.len() as u32).to_be_bytes());
        rom.extend_from_slice(&(expected_regs.len() as u32).to_be_bytes());

        // Initial registers
        for (reg, value) in initial_regs {
            rom.push(reg);
            rom.extend_from_slice(&value.to_be_bytes());
        }

        // Code
        rom.extend_from_slice(&code);

        // Expected registers
        for (reg, value) in expected_regs {
            rom.push(reg);
            rom.extend_from_slice(&value.to_be_bytes());
        }

        rom
    }

    /// Execute the test ROM and verify results
    pub fn execute_and_verify(&self) -> Result<(), String> {
        // Create interpreter and thread
        let memory = MemoryManager::new().map_err(|e| format!("Failed to create memory: {}", e))?;
        let interpreter = PpuInterpreter::new(memory.clone());
        let mut thread = PpuThread::new(0, memory.clone());

        // Set initial registers
        for (reg, value) in &self.initial_regs {
            thread.set_gpr(*reg as usize, *value);
        }

        // Load code into memory
        memory.write_bytes(self.entry_point, &self.code)
            .map_err(|e| format!("Failed to write code: {}", e))?;

        // Set PC to entry point
        thread.set_pc(self.entry_point as u64);

        // Execute until we reach the end of the code or a breakpoint
        let max_instructions = 10000;
        let code_end = self.entry_point + self.code.len() as u32;
        
        for i in 0..max_instructions {
            let pc = thread.pc();
            
            // Check if we've reached the end of the code
            if pc >= code_end as u64 {
                break;
            }

            // Execute one instruction
            match interpreter.step(&mut thread) {
                Ok(_) => {},
                Err(PpuError::Breakpoint { .. }) => {
                    // Breakpoint hit, stop execution
                    break;
                }
                Err(e) => {
                    return Err(format!("Execution failed at PC=0x{:08X} (step {}): {}", pc, i, e));
                }
            }
        }

        // Verify expected registers
        for (reg, expected_value) in &self.expected_regs {
            let actual_value = thread.gpr(*reg as usize);
            if actual_value != *expected_value {
                return Err(format!(
                    "Register r{} mismatch: expected 0x{:016X}, got 0x{:016X}",
                    reg, expected_value, actual_value
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_format() {
        // Create a simple test ROM
        let code = vec![
            0x38, 0x60, 0x00, 0x2A, // addi r3, r0, 42
        ];
        
        let initial_regs = vec![];
        let expected_regs = vec![(3, 42)];
        
        let rom_bytes = TestRom::create_simple(0x2000_0000, code, initial_regs, expected_regs);
        
        // Parse it back
        let rom = TestRom::from_bytes(&rom_bytes).unwrap();
        
        assert_eq!(rom.entry_point, 0x2000_0000);
        assert_eq!(rom.code.len(), 4);
        assert_eq!(rom.expected_regs.len(), 1);
        assert_eq!(rom.expected_regs[0], (3, 42));
    }

    #[test]
    fn test_simple_add_rom() {
        // Test ROM: addi r3, r0, 42
        let code = vec![
            0x38, 0x60, 0x00, 0x2A, // addi r3, r0, 42
        ];
        
        let rom = TestRom {
            entry_point: 0x2000_0000,
            code,
            initial_regs: vec![],
            expected_regs: vec![(3, 42)],
        };
        
        rom.execute_and_verify().unwrap();
    }

    #[test]
    fn test_add_with_registers() {
        // Test ROM: addi r4, r0, 10; addi r5, r0, 20; add r3, r4, r5
        let code = vec![
            0x38, 0x80, 0x00, 0x0A, // addi r4, r0, 10
            0x38, 0xA0, 0x00, 0x14, // addi r5, r0, 20
            0x7C, 0x64, 0x2A, 0x14, // add r3, r4, r5
        ];
        
        let rom = TestRom {
            entry_point: 0x2000_0000,
            code,
            initial_regs: vec![],
            expected_regs: vec![(3, 30), (4, 10), (5, 20)],
        };
        
        rom.execute_and_verify().unwrap();
    }

    #[test]
    fn test_branch_and_link() {
        // Test ROM: bl +8; addi r3, r0, 99; addi r3, r0, 42
        // Should skip to the last instruction
        let code = vec![
            0x48, 0x00, 0x00, 0x09, // bl +8 (with link)
            0x38, 0x60, 0x00, 0x63, // addi r3, r0, 99 (skipped)
            0x38, 0x60, 0x00, 0x2A, // addi r3, r0, 42
        ];
        
        let rom = TestRom {
            entry_point: 0x2000_0000,
            code,
            initial_regs: vec![],
            expected_regs: vec![(3, 42)],
        };
        
        rom.execute_and_verify().unwrap();
    }

    #[test]
    fn test_load_store() {
        // Test ROM: setup base address, store and load
        let code = vec![
            0x3C, 0xA0, 0x20, 0x00, // addis r5, r0, 0x2000
            0x60, 0xA5, 0x10, 0x00, // ori r5, r5, 0x1000 (r5 = 0x20001000)
            0x38, 0x80, 0x12, 0x34, // addi r4, r0, 0x1234
            0x90, 0x85, 0x00, 0x00, // stw r4, 0(r5)
            0x80, 0x65, 0x00, 0x00, // lwz r3, 0(r5)
        ];
        
        let rom = TestRom {
            entry_point: 0x2000_0000,
            code,
            initial_regs: vec![],
            expected_regs: vec![(3, 0x1234), (4, 0x1234)],
        };
        
        rom.execute_and_verify().unwrap();
    }
}
