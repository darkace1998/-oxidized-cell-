//! PPU interpreter implementation
//!
//! This module implements the PPU instruction interpreter, dispatching decoded
//! instructions to the appropriate handlers in the instruction modules.

use std::sync::Arc;
use std::collections::HashSet;
use parking_lot::RwLock;
use oc_memory::MemoryManager;
use oc_core::error::PpuError;
use crate::decoder::{PpuDecoder, InstructionForm};
use crate::thread::PpuThread;
use crate::instructions::{float, system, vector};

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    /// Unconditional breakpoint - always breaks
    Unconditional,
    /// Conditional breakpoint - breaks when condition is met
    Conditional(BreakpointCondition),
}

/// Breakpoint condition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointCondition {
    /// Break when GPR equals value
    GprEquals { reg: usize, value: u64 },
    /// Break when instruction count reaches value
    InstructionCount { count: u64 },
}

/// Breakpoint information
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Address of the breakpoint
    pub addr: u64,
    /// Type of breakpoint
    pub bp_type: BreakpointType,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// Hit count
    pub hit_count: u64,
}

/// PPU interpreter for instruction execution
pub struct PpuInterpreter {
    /// Memory manager
    memory: Arc<MemoryManager>,
    /// Breakpoints (address -> breakpoint)
    breakpoints: RwLock<HashSet<u64>>,
    /// Breakpoint details
    breakpoint_details: RwLock<std::collections::HashMap<u64, Breakpoint>>,
    /// Total instruction count (for conditional breakpoints)
    instruction_count: parking_lot::Mutex<u64>,
}

impl PpuInterpreter {
    /// Create a new PPU interpreter
    pub fn new(memory: Arc<MemoryManager>) -> Self {
        Self {
            memory,
            breakpoints: RwLock::new(HashSet::new()),
            breakpoint_details: RwLock::new(std::collections::HashMap::new()),
            instruction_count: parking_lot::Mutex::new(0),
        }
    }

    /// Add a breakpoint at the specified address
    pub fn add_breakpoint(&self, addr: u64, bp_type: BreakpointType) {
        self.breakpoints.write().insert(addr);
        self.breakpoint_details.write().insert(
            addr,
            Breakpoint {
                addr,
                bp_type,
                enabled: true,
                hit_count: 0,
            },
        );
    }

    /// Remove a breakpoint at the specified address
    pub fn remove_breakpoint(&self, addr: u64) {
        self.breakpoints.write().remove(&addr);
        self.breakpoint_details.write().remove(&addr);
    }

    /// Enable a breakpoint
    pub fn enable_breakpoint(&self, addr: u64) {
        if let Some(bp) = self.breakpoint_details.write().get_mut(&addr) {
            bp.enabled = true;
        }
    }

    /// Disable a breakpoint
    pub fn disable_breakpoint(&self, addr: u64) {
        if let Some(bp) = self.breakpoint_details.write().get_mut(&addr) {
            bp.enabled = false;
        }
    }

    /// Clear all breakpoints
    pub fn clear_breakpoints(&self) {
        self.breakpoints.write().clear();
        self.breakpoint_details.write().clear();
    }

    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoint_details
            .read()
            .values()
            .cloned()
            .collect()
    }

    /// Check if we should break at this address
    #[inline]
    fn should_break(&self, thread: &PpuThread) -> bool {
        let pc = thread.pc();
        
        // Fast path: check if there's a breakpoint at this address
        if !self.breakpoints.read().contains(&pc) {
            return false;
        }

        // Check breakpoint condition
        let details = self.breakpoint_details.read();
        if let Some(bp) = details.get(&pc) {
            if !bp.enabled {
                return false;
            }

            match bp.bp_type {
                BreakpointType::Unconditional => true,
                BreakpointType::Conditional(condition) => match condition {
                    BreakpointCondition::GprEquals { reg, value } => {
                        thread.gpr(reg) == value
                    }
                    BreakpointCondition::InstructionCount { count } => {
                        *self.instruction_count.lock() >= count
                    }
                },
            }
        } else {
            false
        }
    }

    /// Execute a single instruction
    pub fn step(&self, thread: &mut PpuThread) -> Result<(), PpuError> {
        // Check for breakpoints before execution
        if self.should_break(thread) {
            // Update hit count
            let pc = thread.pc();
            if let Some(bp) = self.breakpoint_details.write().get_mut(&pc) {
                bp.hit_count += 1;
            }
            return Err(PpuError::Breakpoint { addr: pc });
        }

        // Increment instruction count for conditional breakpoints
        *self.instruction_count.lock() += 1;

        // Fetch instruction
        let pc = thread.pc() as u32;
        let opcode = self.memory.read_be32(pc).map_err(|_| PpuError::InvalidInstruction {
            addr: pc,
            opcode: 0,
        })?;

        // Decode instruction
        let decoded = PpuDecoder::decode(opcode);

        // Execute instruction
        self.execute(thread, opcode, decoded)?;

        Ok(())
    }

    /// Get the current instruction count
    pub fn instruction_count(&self) -> u64 {
        *self.instruction_count.lock()
    }

    /// Reset the instruction count
    pub fn reset_instruction_count(&self) {
        *self.instruction_count.lock() = 0;
    }

    /// Execute a decoded instruction
    #[inline]
    fn execute(&self, thread: &mut PpuThread, opcode: u32, decoded: crate::decoder::DecodedInstruction) -> Result<(), PpuError> {
        match decoded.form {
            InstructionForm::D => self.execute_d_form(thread, opcode, decoded.op),
            InstructionForm::I => self.execute_i_form(thread, opcode),
            InstructionForm::B => self.execute_b_form(thread, opcode),
            InstructionForm::X => self.execute_x_form(thread, opcode, decoded.xo),
            InstructionForm::XO => self.execute_xo_form(thread, opcode, decoded.xo),
            InstructionForm::XL => self.execute_xl_form(thread, opcode, decoded.xo),
            InstructionForm::M => self.execute_m_form(thread, opcode, decoded.op),
            InstructionForm::A => self.execute_a_form(thread, opcode, decoded.xo),
            InstructionForm::VA => self.execute_va_form(thread, opcode),
            InstructionForm::SC => self.execute_sc(thread, opcode),
            _ => {
                tracing::warn!("Unimplemented instruction form: {:?} at 0x{:08x}", decoded.form, thread.pc());
                thread.advance_pc();
                Ok(())
            }
        }
    }

    /// Execute D-form instructions (most common form - optimized hot path)
    #[inline]
    fn execute_d_form(&self, thread: &mut PpuThread, opcode: u32, op: u8) -> Result<(), PpuError> {
        let (rt, ra, d) = PpuDecoder::d_form(opcode);
        let d = d as i64;

        match op {
            // addi - Add Immediate
            14 => {
                let value = if ra == 0 {
                    d as u64
                } else {
                    (thread.gpr(ra as usize) as i64).wrapping_add(d) as u64
                };
                thread.set_gpr(rt as usize, value);
            }
            // addis - Add Immediate Shifted
            15 => {
                let value = if ra == 0 {
                    (d << 16) as u64
                } else {
                    (thread.gpr(ra as usize) as i64).wrapping_add(d << 16) as u64
                };
                thread.set_gpr(rt as usize, value);
            }
            // lwz - Load Word and Zero
            32 => {
                let ea = if ra == 0 { d as u64 } else { thread.gpr(ra as usize).wrapping_add(d as u64) };
                let value = self.memory.read_be32(ea as u32).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
                thread.set_gpr(rt as usize, value as u64);
            }
            // stw - Store Word
            36 => {
                let ea = if ra == 0 { d as u64 } else { thread.gpr(ra as usize).wrapping_add(d as u64) };
                let value = thread.gpr(rt as usize) as u32;
                self.memory.write_be32(ea as u32, value).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
            }
            // lbz - Load Byte and Zero
            34 => {
                let ea = if ra == 0 { d as u64 } else { thread.gpr(ra as usize).wrapping_add(d as u64) };
                let value: u8 = self.memory.read(ea as u32).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
                thread.set_gpr(rt as usize, value as u64);
            }
            // stb - Store Byte
            38 => {
                let ea = if ra == 0 { d as u64 } else { thread.gpr(ra as usize).wrapping_add(d as u64) };
                let value = thread.gpr(rt as usize) as u8;
                self.memory.write(ea as u32, value).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
            }
            // ori - OR Immediate
            24 => {
                let value = thread.gpr(rt as usize) | (d as u64 & 0xFFFF);
                thread.set_gpr(ra as usize, value);
            }
            // oris - OR Immediate Shifted
            25 => {
                let value = thread.gpr(rt as usize) | ((d as u64 & 0xFFFF) << 16);
                thread.set_gpr(ra as usize, value);
            }
            // andi. - AND Immediate
            28 => {
                let value = thread.gpr(rt as usize) & (d as u64 & 0xFFFF);
                thread.set_gpr(ra as usize, value);
                self.update_cr0(thread, value);
            }
            // cmpi - Compare Immediate (signed)
            11 => {
                let bf = (rt >> 2) & 7;
                let l = (rt & 1) != 0;
                let a = if l { thread.gpr(ra as usize) as i64 } else { thread.gpr(ra as usize) as i32 as i64 };
                let b = if l { d } else { d as i32 as i64 };
                let c = if a < b { 0b1000 } else if a > b { 0b0100 } else { 0b0010 };
                let c = c | if thread.get_xer_so() { 1 } else { 0 };
                thread.set_cr_field(bf as usize, c);
            }
            // cmpli - Compare Logical Immediate (unsigned)
            10 => {
                let bf = (rt >> 2) & 7;
                let l = (rt & 1) != 0;
                let a = if l { thread.gpr(ra as usize) } else { thread.gpr(ra as usize) as u32 as u64 };
                let b = if l { d as u64 & 0xFFFF } else { (d as u64 & 0xFFFF) as u32 as u64 };
                let c = if a < b { 0b1000 } else if a > b { 0b0100 } else { 0b0010 };
                let c = c | if thread.get_xer_so() { 1 } else { 0 };
                thread.set_cr_field(bf as usize, c);
            }
            _ => {
                tracing::warn!("Unimplemented D-form op {} at 0x{:08x}", op, thread.pc());
            }
        }

        thread.advance_pc();
        Ok(())
    }

    /// Execute I-form instructions (branches)
    #[inline]
    fn execute_i_form(&self, thread: &mut PpuThread, opcode: u32) -> Result<(), PpuError> {
        let (li, aa, lk) = PpuDecoder::i_form(opcode);

        if lk {
            thread.regs.lr = thread.pc() + 4;
        }

        let target = if aa {
            li as u64
        } else {
            (thread.pc() as i64 + li as i64) as u64
        };

        thread.set_pc(target);
        Ok(())
    }

    /// Execute B-form instructions (conditional branches)
    #[inline]
    fn execute_b_form(&self, thread: &mut PpuThread, opcode: u32) -> Result<(), PpuError> {
        let (bo, bi, bd, aa, lk) = PpuDecoder::b_form(opcode);

        let ctr_ok = if (bo & 0x04) != 0 {
            true
        } else {
            thread.regs.ctr = thread.regs.ctr.wrapping_sub(1);
            ((thread.regs.ctr != 0) as u8) ^ ((bo >> 1) & 1) != 0
        };

        let cond_ok = if (bo & 0x10) != 0 {
            true
        } else {
            let cr_bit = (thread.regs.cr >> (31 - bi)) & 1;
            (cr_bit as u8) == ((bo >> 3) & 1)
        };

        if ctr_ok && cond_ok {
            if lk {
                thread.regs.lr = thread.pc() + 4;
            }

            let target = if aa {
                bd as u64
            } else {
                (thread.pc() as i64 + bd as i64) as u64
            };

            thread.set_pc(target);
        } else {
            thread.advance_pc();
        }

        Ok(())
    }

    /// Execute X-form instructions
    #[inline]
    fn execute_x_form(&self, thread: &mut PpuThread, opcode: u32, xo: u16) -> Result<(), PpuError> {
        let (rt, ra, rb, _, rc) = PpuDecoder::x_form(opcode);

        match xo {
            // and - AND
            28 => {
                let value = thread.gpr(rt as usize) & thread.gpr(rb as usize);
                thread.set_gpr(ra as usize, value);
                if rc { self.update_cr0(thread, value); }
            }
            // or - OR
            444 => {
                let value = thread.gpr(rt as usize) | thread.gpr(rb as usize);
                thread.set_gpr(ra as usize, value);
                if rc { self.update_cr0(thread, value); }
            }
            // xor - XOR
            316 => {
                let value = thread.gpr(rt as usize) ^ thread.gpr(rb as usize);
                thread.set_gpr(ra as usize, value);
                if rc { self.update_cr0(thread, value); }
            }
            // nor - NOR
            124 => {
                let value = !(thread.gpr(rt as usize) | thread.gpr(rb as usize));
                thread.set_gpr(ra as usize, value);
                if rc { self.update_cr0(thread, value); }
            }
            // cmp - Compare (signed)
            0 => {
                let bf = (rt >> 2) & 7;
                let l = (rt & 1) != 0;
                let a = if l { thread.gpr(ra as usize) as i64 } else { thread.gpr(ra as usize) as i32 as i64 };
                let b = if l { thread.gpr(rb as usize) as i64 } else { thread.gpr(rb as usize) as i32 as i64 };
                let c = if a < b { 0b1000 } else if a > b { 0b0100 } else { 0b0010 };
                let c = c | if thread.get_xer_so() { 1 } else { 0 };
                thread.set_cr_field(bf as usize, c);
            }
            // cmpl - Compare Logical (unsigned)
            32 => {
                let bf = (rt >> 2) & 7;
                let l = (rt & 1) != 0;
                let a = if l { thread.gpr(ra as usize) } else { thread.gpr(ra as usize) as u32 as u64 };
                let b = if l { thread.gpr(rb as usize) } else { thread.gpr(rb as usize) as u32 as u64 };
                let c = if a < b { 0b1000 } else if a > b { 0b0100 } else { 0b0010 };
                let c = c | if thread.get_xer_so() { 1 } else { 0 };
                thread.set_cr_field(bf as usize, c);
            }
            // lwzx - Load Word and Zero Indexed
            23 => {
                let ea = if ra == 0 { thread.gpr(rb as usize) } else { thread.gpr(ra as usize).wrapping_add(thread.gpr(rb as usize)) };
                let value = self.memory.read_be32(ea as u32).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
                thread.set_gpr(rt as usize, value as u64);
            }
            // stwx - Store Word Indexed
            151 => {
                let ea = if ra == 0 { thread.gpr(rb as usize) } else { thread.gpr(ra as usize).wrapping_add(thread.gpr(rb as usize)) };
                let value = thread.gpr(rt as usize) as u32;
                self.memory.write_be32(ea as u32, value).map_err(|_| PpuError::InvalidInstruction {
                    addr: thread.pc() as u32,
                    opcode,
                })?;
            }
            // mfspr - Move From Special Purpose Register
            339 => {
                let spr = ((rb as u16) << 5) | (ra as u16);
                let value = match spr {
                    1 => thread.regs.xer,     // XER
                    8 => thread.regs.lr,      // LR
                    9 => thread.regs.ctr,     // CTR
                    _ => {
                        tracing::warn!("Unimplemented mfspr SPR {} at 0x{:08x}", spr, thread.pc());
                        0
                    }
                };
                thread.set_gpr(rt as usize, value);
            }
            // mtspr - Move To Special Purpose Register
            467 => {
                let spr = ((rb as u16) << 5) | (ra as u16);
                let value = thread.gpr(rt as usize);
                match spr {
                    1 => thread.regs.xer = value,    // XER
                    8 => thread.regs.lr = value,     // LR
                    9 => thread.regs.ctr = value,    // CTR
                    _ => {
                        tracing::warn!("Unimplemented mtspr SPR {} at 0x{:08x}", spr, thread.pc());
                    }
                }
            }
            // XO-form arithmetic instructions (dispatched as X-form by decoder)
            // Note: These have a 10-bit XO in the decoder, but only 9-bit in the instruction
            // So we need to mask to 9 bits for matching
            _ if (xo & 0x1FF) == 266 => {
                // add - Add
                let (rt, ra, rb, _, _) = PpuDecoder::x_form(opcode);
                let oe = ((opcode >> 10) & 1) != 0;
                let rc = (opcode & 1) != 0;
                
                let a = thread.gpr(ra as usize);
                let b = thread.gpr(rb as usize);
                let result = a.wrapping_add(b);
                thread.set_gpr(rt as usize, result);
                if oe {
                    let overflow = ((a as i64).overflowing_add(b as i64)).1;
                    thread.set_xer_ov(overflow);
                    if overflow { thread.set_xer_so(true); }
                }
                if rc { self.update_cr0(thread, result); }
            }
            _ if (xo & 0x1FF) == 40 => {
                // subf - Subtract From
                let (rt, ra, rb, _, _) = PpuDecoder::x_form(opcode);
                let oe = ((opcode >> 10) & 1) != 0;
                let rc = (opcode & 1) != 0;
                
                let a = thread.gpr(ra as usize);
                let b = thread.gpr(rb as usize);
                let result = b.wrapping_sub(a);
                thread.set_gpr(rt as usize, result);
                if oe {
                    let overflow = ((b as i64).overflowing_sub(a as i64)).1;
                    thread.set_xer_ov(overflow);
                    if overflow { thread.set_xer_so(true); }
                }
                if rc { self.update_cr0(thread, result); }
            }
            _ => {
                tracing::warn!("Unimplemented X-form xo {} at 0x{:08x}", xo, thread.pc());
            }
        }

        thread.advance_pc();
        Ok(())
    }

    /// Execute XO-form instructions (integer arithmetic)
    fn execute_xo_form(&self, thread: &mut PpuThread, opcode: u32, xo: u16) -> Result<(), PpuError> {
        let (rt, ra, rb, oe, _, rc) = PpuDecoder::xo_form(opcode);

        match xo {
            // add
            266 => {
                let a = thread.gpr(ra as usize);
                let b = thread.gpr(rb as usize);
                let result = a.wrapping_add(b);
                thread.set_gpr(rt as usize, result);
                if oe {
                    let overflow = ((a as i64).overflowing_add(b as i64)).1;
                    thread.set_xer_ov(overflow);
                    if overflow { thread.set_xer_so(true); }
                }
                if rc { self.update_cr0(thread, result); }
            }
            // subf - Subtract From
            40 => {
                let a = thread.gpr(ra as usize);
                let b = thread.gpr(rb as usize);
                let result = b.wrapping_sub(a);
                thread.set_gpr(rt as usize, result);
                if oe {
                    let overflow = ((b as i64).overflowing_sub(a as i64)).1;
                    thread.set_xer_ov(overflow);
                    if overflow { thread.set_xer_so(true); }
                }
                if rc { self.update_cr0(thread, result); }
            }
            // mullw - Multiply Low Word
            235 => {
                let a = thread.gpr(ra as usize) as i32;
                let b = thread.gpr(rb as usize) as i32;
                let result = (a as i64 * b as i64) as u64;
                thread.set_gpr(rt as usize, result);
                if rc { self.update_cr0(thread, result); }
            }
            // divw - Divide Word
            491 => {
                let a = thread.gpr(ra as usize) as i32;
                let b = thread.gpr(rb as usize) as i32;
                if b != 0 && !(a == i32::MIN && b == -1) {
                    let result = (a / b) as i64 as u64;
                    thread.set_gpr(rt as usize, result);
                } else {
                    thread.set_gpr(rt as usize, 0);
                    if oe {
                        thread.set_xer_ov(true);
                        thread.set_xer_so(true);
                    }
                }
                if rc { self.update_cr0(thread, thread.gpr(rt as usize)); }
            }
            _ => {
                tracing::warn!("Unimplemented XO-form xo {} at 0x{:08x}", xo, thread.pc());
            }
        }

        thread.advance_pc();
        Ok(())
    }

    /// Execute XL-form instructions (branch to LR/CTR and CR logical)
    fn execute_xl_form(&self, thread: &mut PpuThread, opcode: u32, xo: u16) -> Result<(), PpuError> {
        let bo = ((opcode >> 21) & 0x1F) as u8;
        let bi = ((opcode >> 16) & 0x1F) as u8;
        let bb = ((opcode >> 11) & 0x1F) as u8;
        let lk = (opcode & 1) != 0;

        match xo {
            // bclr - Branch Conditional to Link Register
            16 => {
                let ctr_ok = if (bo & 0x04) != 0 {
                    true
                } else {
                    thread.regs.ctr = thread.regs.ctr.wrapping_sub(1);
                    ((thread.regs.ctr != 0) as u8) ^ ((bo >> 1) & 1) != 0
                };

                let cond_ok = if (bo & 0x10) != 0 {
                    true
                } else {
                    let cr_bit = (thread.regs.cr >> (31 - bi)) & 1;
                    (cr_bit as u8) == ((bo >> 3) & 1)
                };

                if ctr_ok && cond_ok {
                    let target = thread.regs.lr & !3;
                    if lk {
                        thread.regs.lr = thread.pc() + 4;
                    }
                    thread.set_pc(target);
                } else {
                    thread.advance_pc();
                }
            }
            // bcctr - Branch Conditional to Count Register
            528 => {
                let cond_ok = if (bo & 0x10) != 0 {
                    true
                } else {
                    let cr_bit = (thread.regs.cr >> (31 - bi)) & 1;
                    (cr_bit as u8) == ((bo >> 3) & 1)
                };

                if cond_ok {
                    let target = thread.regs.ctr & !3;
                    if lk {
                        thread.regs.lr = thread.pc() + 4;
                    }
                    thread.set_pc(target);
                } else {
                    thread.advance_pc();
                }
            }
            // mcrf - Move Condition Register Field
            0 => {
                let bf = (bo >> 2) & 7;
                let bfa = (bi >> 2) & 7;
                system::mcrf(thread, bf, bfa);
                thread.advance_pc();
            }
            // crand - Condition Register AND
            257 => {
                system::crand(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // cror - Condition Register OR
            449 => {
                system::cror(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // crxor - Condition Register XOR
            193 => {
                system::crxor(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // crnand - Condition Register NAND
            225 => {
                system::crnand(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // crnor - Condition Register NOR
            33 => {
                system::crnor(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // creqv - Condition Register EQV (XNOR)
            289 => {
                system::creqv(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // crandc - Condition Register AND with Complement
            129 => {
                system::crandc(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // crorc - Condition Register OR with Complement
            417 => {
                system::crorc(thread, bo, bi, bb);
                thread.advance_pc();
            }
            // isync - Instruction Synchronize
            150 => {
                system::isync(thread);
                thread.advance_pc();
            }
            _ => {
                tracing::warn!("Unimplemented XL-form xo {} at 0x{:08x}", xo, thread.pc());
                thread.advance_pc();
            }
        }

        Ok(())
    }

    /// Execute M-form instructions (rotate)
    fn execute_m_form(&self, thread: &mut PpuThread, opcode: u32, op: u8) -> Result<(), PpuError> {
        let (rs, ra, rb_sh, mb, me, rc) = PpuDecoder::m_form(opcode);

        match op {
            // rlwinm - Rotate Left Word Immediate then AND with Mask
            21 => {
                let sh = rb_sh as u32;
                let value = thread.gpr(rs as usize) as u32;
                let rotated = value.rotate_left(sh);
                let mask = Self::generate_mask_32(mb, me);
                let result = (rotated & mask) as u64;
                thread.set_gpr(ra as usize, result);
                if rc { self.update_cr0(thread, result); }
            }
            // rlwimi - Rotate Left Word Immediate then Mask Insert
            20 => {
                let sh = rb_sh as u32;
                let value = thread.gpr(rs as usize) as u32;
                let rotated = value.rotate_left(sh);
                let mask = Self::generate_mask_32(mb, me);
                let result = ((rotated & mask) | (thread.gpr(ra as usize) as u32 & !mask)) as u64;
                thread.set_gpr(ra as usize, result);
                if rc { self.update_cr0(thread, result); }
            }
            // rlwnm - Rotate Left Word then AND with Mask
            23 => {
                let sh = (thread.gpr(rb_sh as usize) & 0x1F) as u32;
                let value = thread.gpr(rs as usize) as u32;
                let rotated = value.rotate_left(sh);
                let mask = Self::generate_mask_32(mb, me);
                let result = (rotated & mask) as u64;
                thread.set_gpr(ra as usize, result);
                if rc { self.update_cr0(thread, result); }
            }
            _ => {
                tracing::warn!("Unimplemented M-form op {} at 0x{:08x}", op, thread.pc());
            }
        }

        thread.advance_pc();
        Ok(())
    }

    /// Execute system call
    fn execute_sc(&self, thread: &mut PpuThread, _opcode: u32) -> Result<(), PpuError> {
        // System call - the syscall number is in R11
        let syscall_num = thread.gpr(11);
        tracing::trace!("System call {} at 0x{:08x}", syscall_num, thread.pc());
        
        // For now, just advance PC. LV2 kernel will handle syscalls.
        thread.advance_pc();
        Ok(())
    }

    /// Update CR0 based on result (for Rc=1 instructions)
    #[inline]
    fn update_cr0(&self, thread: &mut PpuThread, value: u64) {
        let value = value as i64;
        let c = if value < 0 { 0b1000 } else if value > 0 { 0b0100 } else { 0b0010 };
        let c = c | if thread.get_xer_so() { 1 } else { 0 };
        thread.set_cr_field(0, c);
    }

    /// Generate 32-bit mask for rotate instructions
    #[inline]
    fn generate_mask_32(mb: u8, me: u8) -> u32 {
        let mb = mb as u32;
        let me = me as u32;
        if mb <= me {
            (u32::MAX >> mb) & (u32::MAX << (31 - me))
        } else {
            (u32::MAX >> mb) | (u32::MAX << (31 - me))
        }
    }

    /// Execute A-form instructions (floating-point multiply-add)
    fn execute_a_form(&self, thread: &mut PpuThread, opcode: u32, xo: u16) -> Result<(), PpuError> {
        let frt = ((opcode >> 21) & 0x1F) as usize;
        let fra = ((opcode >> 16) & 0x1F) as usize;
        let frb = ((opcode >> 11) & 0x1F) as usize;
        let frc = ((opcode >> 6) & 0x1F) as usize;
        let rc = (opcode & 1) != 0;
        let primary = (opcode >> 26) & 0x3F;

        // Get operand values
        let a = thread.fpr(fra);
        let b = thread.fpr(frb);
        let c = thread.fpr(frc);

        let result = match (primary, xo) {
            // fmadd - Floating Multiply-Add (Double)
            (63, 29) => float::fmadd(a, c, b),
            // fmsub - Floating Multiply-Subtract (Double)
            (63, 28) => float::fmsub(a, c, b),
            // fnmadd - Floating Negative Multiply-Add (Double)
            (63, 31) => float::fnmadd(a, c, b),
            // fnmsub - Floating Negative Multiply-Subtract (Double)
            (63, 30) => float::fnmsub(a, c, b),
            // fmadds - Floating Multiply-Add Single
            (59, 29) => float::frsp(float::fmadd(a, c, b)),
            // fmsubs - Floating Multiply-Subtract Single
            (59, 28) => float::frsp(float::fmsub(a, c, b)),
            // fnmadds - Floating Negative Multiply-Add Single
            (59, 31) => float::frsp(float::fnmadd(a, c, b)),
            // fnmsubs - Floating Negative Multiply-Subtract Single
            (59, 30) => float::frsp(float::fnmsub(a, c, b)),
            // fmul - Floating Multiply
            (63, 25) => a * c,
            // fmuls - Floating Multiply Single
            (59, 25) => float::frsp(a * c),
            // fadd - Floating Add
            (63, 21) => a + b,
            // fadds - Floating Add Single
            (59, 21) => float::frsp(a + b),
            // fsub - Floating Subtract
            (63, 20) => a - b,
            // fsubs - Floating Subtract Single
            (59, 20) => float::frsp(a - b),
            // fdiv - Floating Divide
            (63, 18) => a / b,
            // fdivs - Floating Divide Single
            (59, 18) => float::frsp(a / b),
            // fsel - Floating Select
            (63, 23) => float::fsel(a, b, c),
            _ => {
                tracing::warn!("Unimplemented A-form xo {} at 0x{:08x}", xo, thread.pc());
                0.0
            }
        };

        thread.set_fpr(frt, result);
        float::update_fprf(thread, result);
        
        if rc {
            float::update_cr1(thread);
        }

        thread.advance_pc();
        Ok(())
    }

    /// Execute VA-form instructions (vector three-operand)
    fn execute_va_form(&self, thread: &mut PpuThread, opcode: u32) -> Result<(), PpuError> {
        let vrt = ((opcode >> 21) & 0x1F) as usize;
        let vra = ((opcode >> 16) & 0x1F) as usize;
        let vrb = ((opcode >> 11) & 0x1F) as usize;
        let vrc = ((opcode >> 6) & 0x1F) as usize;
        let xo = (opcode & 0x3F) as u8;

        let a = thread.vr(vra);
        let b = thread.vr(vrb);
        let c = thread.vr(vrc);

        let result = match xo {
            // vperm - Vector Permute
            0x2B => vector::vperm(a, b, c),
            // vmaddfp - Vector Multiply-Add Floating-Point
            0x2E => vector::vmaddfp(a, c, b),
            // vnmsubfp - Vector Negative Multiply-Subtract Floating-Point
            0x2F => vector::vnmsubfp(a, c, b),
            // vsel - Vector Select
            0x2A => vector::vsel(a, b, c),
            _ => {
                tracing::warn!("Unimplemented VA-form xo {} at 0x{:08x}", xo, thread.pc());
                [0u32; 4]
            }
        };

        thread.set_vr(vrt, result);
        thread.advance_pc();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_env() -> (PpuInterpreter, PpuThread) {
        let memory = MemoryManager::new().unwrap();
        let interpreter = PpuInterpreter::new(memory.clone());
        let thread = PpuThread::new(0, memory);
        (interpreter, thread)
    }

    /// Helper to write an instruction to memory and execute it
    fn execute_instruction(interpreter: &PpuInterpreter, thread: &mut PpuThread, opcode: u32) -> Result<(), PpuError> {
        let pc = thread.pc() as u32;
        interpreter.memory.write_be32(pc, opcode).unwrap();
        interpreter.step(thread)
    }

    #[test]
    fn test_interpreter_creation() {
        let (interpreter, thread) = create_test_env();
        assert_eq!(thread.pc(), 0);
        drop(interpreter);
    }

    #[test]
    fn test_mask_generation() {
        assert_eq!(PpuInterpreter::generate_mask_32(0, 31), 0xFFFFFFFF);
        assert_eq!(PpuInterpreter::generate_mask_32(16, 31), 0x0000FFFF);
        assert_eq!(PpuInterpreter::generate_mask_32(0, 15), 0xFFFF0000);
    }

    // ===== ADDI Tests =====
    
    #[test]
    fn test_addi_basic() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // addi r3, r0, 100  (opcode 14, rt=3, ra=0, simm=100)
        // When ra=0, addi loads the immediate directly
        let opcode = 0x38600064u32; // addi r3, r0, 100
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 100);
        assert_eq!(thread.pc(), 0x2000_0004);
    }

    #[test]
    fn test_addi_with_register() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        thread.set_gpr(4, 1000);
        
        // addi r3, r4, 50  (r3 = r4 + 50 = 1050)
        let opcode = 0x38640032u32; // addi r3, r4, 50
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 1050);
    }

    #[test]
    fn test_addi_negative_immediate() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        thread.set_gpr(5, 100);
        
        // addi r3, r5, -50  (r3 = r5 - 50 = 50)
        // -50 in 16-bit signed = 0xFFCE
        let opcode = 0x3865FFCEu32; // addi r3, r5, -50
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 50);
    }

    // ===== LWZ/STW Tests =====
    
    #[test]
    fn test_stw_lwz_basic() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Store value to memory
        thread.set_gpr(3, 0xDEADBEEF);
        thread.set_gpr(4, 0x2000_1000); // Base address
        
        // stw r3, 0(r4)
        let stw_opcode = 0x90640000u32; // stw r3, 0(r4)
        execute_instruction(&interpreter, &mut thread, stw_opcode).unwrap();
        
        // Clear r3 and load back
        thread.set_gpr(3, 0);
        
        // lwz r3, 0(r4)
        let lwz_opcode = 0x80640000u32; // lwz r3, 0(r4)
        execute_instruction(&interpreter, &mut thread, lwz_opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 0xDEADBEEF);
    }

    #[test]
    fn test_lwz_stw_with_displacement() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        thread.set_gpr(3, 0x12345678);
        thread.set_gpr(4, 0x2000_1000);
        
        // stw r3, 16(r4) - store at base + 16
        let stw_opcode = 0x90640010u32; // stw r3, 16(r4)
        execute_instruction(&interpreter, &mut thread, stw_opcode).unwrap();
        
        thread.set_gpr(3, 0);
        
        // lwz r3, 16(r4) - load from base + 16
        let lwz_opcode = 0x80640010u32; // lwz r3, 16(r4)
        execute_instruction(&interpreter, &mut thread, lwz_opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 0x12345678);
    }

    // ===== Branch Tests =====
    
    #[test]
    fn test_branch_unconditional() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // b 0x100 (relative branch forward 0x100 bytes)
        let opcode = 0x48000100u32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.pc(), 0x2000_0100);
    }

    #[test]
    fn test_branch_with_link() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // bl 0x200 (branch and link)
        let opcode = 0x48000201u32; // bl 0x200
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.pc(), 0x2000_0200);
        assert_eq!(thread.regs.lr, 0x2000_0004); // Return address
    }

    #[test]
    fn test_branch_backward() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_1000);
        
        // b -0x100 (branch backward)
        // -0x100 in 26-bit signed, left-shifted by 2
        let opcode = 0x4BFFFF00u32; // b -0x100
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.pc(), 0x2000_0F00);
    }

    // ===== Conditional Branch (bc) Tests =====
    
    #[test]
    fn test_bc_branch_if_equal() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set CR0 EQ bit (bit 2 of CR0, which is bit 30 in CR register)
        thread.set_cr_field(0, 0b0010); // EQ set
        
        // beq 0x40 (branch if CR0 EQ set)
        // BO=01100 (branch if condition true), BI=2 (CR0 EQ)
        let opcode = 0x41820040u32; // beq 0x40
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.pc(), 0x2000_0040);
    }

    #[test]
    fn test_bc_no_branch_if_not_equal() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set CR0 to GT (not equal)
        thread.set_cr_field(0, 0b0100); // GT set, EQ clear
        
        // beq 0x40 (should NOT branch since EQ is not set)
        let opcode = 0x41820040u32; // beq 0x40
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Should just advance PC since condition is false
        assert_eq!(thread.pc(), 0x2000_0004);
    }

    #[test]
    fn test_bc_branch_if_less_than() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set CR0 LT bit
        thread.set_cr_field(0, 0b1000); // LT set
        
        // blt 0x80 (branch if less than)
        // BO=01100 (branch if condition true), BI=0 (CR0 LT)
        let opcode = 0x41800080u32; // blt 0x80
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.pc(), 0x2000_0080);
    }

    // ===== CR Logical Operations Tests =====
    
    #[test]
    fn test_crand() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set bits 0 and 1 of CR (bits 31 and 30 in position terms)
        thread.regs.cr = 0xC000_0000; // bits 0 and 1 set
        
        // crand bt=2, ba=0, bb=1 (CR[2] = CR[0] & CR[1])
        // XL-form: [0:5]=19, [6:10]=bt=2, [11:15]=ba=0, [16:20]=bb=1, [21:30]=xo=257, [31]=0
        // Binary: 010011 00010 00000 00001 0100000001 0
        let opcode = 0x4C40_0A02u32; // crand 2, 0, 1
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Bit 2 should be set (1 & 1 = 1)
        assert!((thread.regs.cr >> 29) & 1 == 1);
    }

    #[test]
    fn test_cror() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set only bit 0
        thread.regs.cr = 0x8000_0000;
        
        // cror bt=2, ba=0, bb=1 (CR[2] = CR[0] | CR[1])
        // XL-form: [0:5]=19, [6:10]=bt=2, [11:15]=ba=0, [16:20]=bb=1, [21:30]=xo=449, [31]=0
        // xo=449 in bits 21-30, shifted: 449 << 1 = 0x382
        // Binary: 010011 00010 00000 00001 0111000001 0
        let opcode = 0x4C40_0B82u32; // cror 2, 0, 1
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Bit 2 should be set (1 | 0 = 1)
        assert!((thread.regs.cr >> 29) & 1 == 1);
    }

    #[test]
    fn test_crxor_clear() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        thread.regs.cr = 0xFFFF_FFFF;
        
        // crxor bt=0, ba=0, bb=0 (CR[0] = CR[0] ^ CR[0] = 0)
        // XL-form: [0:5]=19, [6:10]=bt=0, [11:15]=ba=0, [16:20]=bb=0, [21:30]=xo=193, [31]=0
        // xo=193 = 0b0011000001
        // Binary: 010011 00000 00000 00000 0011000001 0
        let opcode = 0x4C00_0182u32; // crxor 0, 0, 0
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Bit 0 should be cleared
        assert!((thread.regs.cr >> 31) & 1 == 0);
    }

    // ===== FMADD Tests =====
    
    #[test]
    fn test_fmadd_basic() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // fmadd f3, f1, f2, f4 => f3 = (f1 * f2) + f4
        // A-form: fmadd frt, fra, frc, frb  => frt = (fra * frc) + frb
        // So fmadd f3, f1, f2, f4 means f3 = (f1 * f2) + f4
        thread.set_fpr(1, 2.0);  // fra
        thread.set_fpr(2, 3.0);  // frc
        thread.set_fpr(4, 4.0);  // frb
        
        // fmadd f3, f1, f2, f4
        // Primary opcode 63 (0x3F), A-form
        // [0:5]=63, [6:10]=frt=3, [11:15]=fra=1, [16:20]=frb=4, [21:25]=frc=2, [26:30]=xo=29, [31]=rc=0
        // Binary: 111111 00011 00001 00100 00010 11101 0
        let opcode = 0xFC61_20BAu32; // fmadd f3, f1, f2, f4
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Result should be (2.0 * 3.0) + 4.0 = 10.0
        assert_eq!(thread.fpr(3), 10.0);
    }

    #[test]
    fn test_fmsub_basic() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // fmsub f3, f1, f2, f4 => f3 = (f1 * f2) - f4
        thread.set_fpr(1, 5.0);  // fra
        thread.set_fpr(2, 2.0);  // frc
        thread.set_fpr(4, 3.0);  // frb
        
        // fmsub f3, f1, f2, f4
        // [0:5]=63, [6:10]=frt=3, [11:15]=fra=1, [16:20]=frb=4, [21:25]=frc=2, [26:30]=xo=28, [31]=rc=0
        // Binary: 111111 00011 00001 00100 00010 11100 0
        let opcode = 0xFC61_20B8u32; // fmsub f3, f1, f2, f4
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Result = (5.0 * 2.0) - 3.0 = 7.0
        assert_eq!(thread.fpr(3), 7.0);
    }

    // ===== VPERM (Vector Permute) Tests =====
    
    #[test]
    fn test_vperm_identity() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set up source vectors
        thread.set_vr(1, [0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F]);
        thread.set_vr(2, [0x10111213, 0x14151617, 0x18191A1B, 0x1C1D1E1F]);
        
        // Control vector: identity permutation (0,1,2,3,4,5,6,7,8,9,A,B,C,D,E,F)
        thread.set_vr(3, [0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F]);
        
        // vperm v4, v1, v2, v3
        // VA-form: [0:5]=4, [6:10]=vrt=4, [11:15]=vra=1, [16:20]=vrb=2, [21:25]=vrc=3, [26:31]=xo=43
        // Binary: 000100 00100 00001 00010 00011 101011
        let opcode = 0x1081_10EBu32; // vperm v4, v1, v2, v3
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Result should be same as v1 (identity permutation selects first 16 bytes)
        let result = thread.vr(4);
        assert_eq!(result, [0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F]);
    }

    #[test]
    fn test_vperm_swap_halves() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Source vectors
        thread.set_vr(1, [0x00010203, 0x04050607, 0x08090A0B, 0x0C0D0E0F]);
        thread.set_vr(2, [0x10111213, 0x14151617, 0x18191A1B, 0x1C1D1E1F]);
        
        // Control: select bytes 8-15 then 0-7 from first vector
        thread.set_vr(3, [0x08090A0B, 0x0C0D0E0F, 0x00010203, 0x04050607]);
        
        // vperm v4, v1, v2, v3
        let opcode = 0x1081_10EBu32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Result: bytes 8-15 followed by 0-7
        let result = thread.vr(4);
        assert_eq!(result, [0x08090A0B, 0x0C0D0E0F, 0x00010203, 0x04050607]);
    }

    #[test]
    fn test_vmaddfp() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // vmaddfp v4, v1, v2, v3 => v4 = (v1 * v2) + v3
        // VA-form: vmaddfp vrt, vra, vrc, vrb  => (vra * vrc) + vrb
        let a = [2.0f32.to_bits(), 3.0f32.to_bits(), 4.0f32.to_bits(), 5.0f32.to_bits()];
        let c = [1.5f32.to_bits(), 2.0f32.to_bits(), 0.5f32.to_bits(), 1.0f32.to_bits()];
        let b = [1.0f32.to_bits(), 1.0f32.to_bits(), 1.0f32.to_bits(), 1.0f32.to_bits()];
        
        thread.set_vr(1, a);  // vra
        thread.set_vr(2, c);  // vrc
        thread.set_vr(3, b);  // vrb
        
        // vmaddfp v4, v1, v2, v3
        // VA-form: [0:5]=4, [6:10]=vrt=4, [11:15]=vra=1, [16:20]=vrb=3, [21:25]=vrc=2, [26:31]=xo=46
        // Note: vrb is the addend, vrc is the multiplier
        // Binary: 000100 00100 00001 00011 00010 101110
        let opcode = 0x1081_18AEu32; // vmaddfp v4, v1, v2, v3
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        let result = thread.vr(4);
        // v4[0] = 2.0*1.5 + 1.0 = 4.0
        // v4[1] = 3.0*2.0 + 1.0 = 7.0
        // v4[2] = 4.0*0.5 + 1.0 = 3.0
        // v4[3] = 5.0*1.0 + 1.0 = 6.0
        assert_eq!(f32::from_bits(result[0]), 4.0);
        assert_eq!(f32::from_bits(result[1]), 7.0);
        assert_eq!(f32::from_bits(result[2]), 3.0);
        assert_eq!(f32::from_bits(result[3]), 6.0);
    }

    // ===== Edge Case Tests =====

    #[test]
    fn test_addi_overflow() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Test overflow with max i64 value
        thread.set_gpr(4, i64::MAX as u64);
        
        // addi r3, r4, 1 (should wrap around)
        let opcode = 0x38640001u32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // Result should wrap to min value (wrapping add)
        assert_eq!(thread.gpr(3) as i64, i64::MIN);
    }

    #[test]
    fn test_addi_ra_zero_special_case() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set r0 to a value (should be ignored)
        thread.set_gpr(0, 999);
        
        // addi r3, r0, 42 (ra=0 means load immediate, not add to r0)
        let opcode = 0x3860002Au32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        assert_eq!(thread.gpr(3), 42);
    }

    #[test]
    fn test_divw_by_zero() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Setup: divide by zero
        thread.set_gpr(4, 100);
        thread.set_gpr(5, 0);
        
        // divw r3, r4, r5 with OE=0 (no overflow exception)
        // XO-form: op=31, rt=3, ra=4, rb=5, oe=0, xo=491, rc=0
        let opcode = 0x7C64_2BD6u32;
        
        // Write instruction and execute
        interpreter.memory.write_be32(0x2000_0000, opcode).unwrap();
        interpreter.step(&mut thread).unwrap();
        
        // Result should be 0 on divide by zero
        assert_eq!(thread.gpr(3), 0);
    }

    #[test]
    fn test_divw_overflow() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Setup: i32::MIN / -1 causes overflow
        thread.set_gpr(4, i32::MIN as u64);
        thread.set_gpr(5, (-1i32) as u32 as u64);
        
        // divw r3, r4, r5 with OE=0
        let opcode = 0x7C64_2BD6u32;
        interpreter.memory.write_be32(0x2000_0000, opcode).unwrap();
        interpreter.step(&mut thread).unwrap();
        
        // Result should be 0 on overflow
        assert_eq!(thread.gpr(3), 0);
    }

    #[test]
    fn test_branch_boundary() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Branch forward by a reasonable offset (not near 32-bit boundary for test safety)
        let offset = 0x1000i32;
        
        // Create branch instruction
        let li = ((offset >> 2) & 0x00FFFFFF) as u32;
        let opcode = 0x48000000u32 | (li << 2);
        
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        assert_eq!(thread.pc(), 0x2000_0000 + offset as u64);
    }

    #[test]
    fn test_cmp_signed_vs_unsigned() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Test signed comparison with negative value
        thread.set_gpr(4, (-10i64) as u64);
        thread.set_gpr(5, 10u64);
        
        // cmp cr0, 0, r4, r5 (signed comparison, 64-bit)
        // X-form: op=31, bf=0, l=1, ra=4, rb=5, xo=0
        let opcode = 0x7C04_2800u32 | (1 << 21); // l=1 for 64-bit
        interpreter.memory.write_be32(0x2000_0000, opcode).unwrap();
        interpreter.step(&mut thread).unwrap();
        
        // -10 < 10, so LT bit should be set in CR0
        let cr0 = thread.get_cr_field(0);
        assert_eq!(cr0 & 0b1000, 0b1000); // LT bit set
    }

    #[test]
    fn test_cmpl_unsigned() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Test unsigned comparison
        thread.set_gpr(4, (-10i64) as u64); // Large unsigned value
        thread.set_gpr(5, 10u64);
        
        // cmpl cr0, 1, r4, r5 (unsigned comparison, 64-bit)
        // X-form: op=31, bf=0, l=1, ra=4, rb=5, xo=32
        let opcode = 0x7C04_2840u32 | (1 << 21); // l=1 for 64-bit
        interpreter.memory.write_be32(0x2000_0000, opcode).unwrap();
        interpreter.step(&mut thread).unwrap();
        
        // As unsigned, -10 > 10, so GT bit should be set
        let cr0 = thread.get_cr_field(0);
        assert_eq!(cr0 & 0b0100, 0b0100); // GT bit set
    }

    #[test]
    fn test_rotate_mask_edge_cases() {
        // Test mask generation
        // When mb <= me: mask includes bits mb through me
        // When mb > me: mask wraps around
        
        // Test full mask (mb=0, me=31)
        assert_eq!(PpuInterpreter::generate_mask_32(0, 31), 0xFFFFFFFF);
        
        // Test single bit mask at bit 31 (mb=31, me=31)
        assert_eq!(PpuInterpreter::generate_mask_32(31, 31), 0x00000001);
        
        // Test single bit mask at bit 0 (mb=0, me=0)
        assert_eq!(PpuInterpreter::generate_mask_32(0, 0), 0x80000000);
        
        // Test contiguous mask (mb=8, me=15) - bits 8-15
        assert_eq!(PpuInterpreter::generate_mask_32(8, 15), 0x00FF0000);
    }

    #[test]
    fn test_rlwinm_extract_bits() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Test basic rotate and mask (simplified version)
        thread.set_gpr(4, 0xABCD1234);
        
        // rlwinm r3, r4, 8, 24, 31 - rotate left 8 bits and mask bits 24-31
        // This should give us the second byte rotated to the last position
        // M-form: op=21, rs=4, ra=3, sh=8, mb=24, me=31, rc=0
        let opcode = 0x5483_443Eu32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // After rotating 0xABCD1234 left by 8: 0xCD1234AB
        // Mask bits 24-31: 0x000000AB
        assert_eq!(thread.gpr(3) & 0xFF, 0xAB);
    }

    #[test]
    fn test_overflow_flag_propagation() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Simple test for overflow detection
        // Test with i64::MAX + 1 which should overflow
        thread.set_gpr(5, 0x7FFFFFFFFFFFFFFF_u64); // i64::MAX
        thread.set_gpr(6, 1);
        
        // add r4, r5, r6 with OE=1 (enable overflow detection)
        let opcode = 0x7C85_3614u32;
        interpreter.memory.write_be32(0x2000_0000, opcode).unwrap();
        interpreter.step(&mut thread).unwrap();
        
        // Overflow should be detected (i64::MAX + 1 overflows in signed arithmetic)
        assert!(thread.get_xer_ov(), "OV bit should be set on overflow");
        assert!(thread.get_xer_so(), "SO bit should be set on overflow");
    }

    #[test]
    fn test_conditional_branch_ctr() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set CTR to 5
        thread.regs.ctr = 5;
        
        // bdnz 0x40 (branch if --CTR != 0)
        // BO=16 (decrement CTR, branch if CTR != 0)
        let opcode = 0x42000040u32;
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // CTR should be decremented
        assert_eq!(thread.regs.ctr, 4);
        // Should have branched
        assert_eq!(thread.pc(), 0x2000_0040);
    }

    #[test]
    fn test_conditional_branch_no_ctr_decrement() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Set CTR to 5
        thread.regs.ctr = 5;
        
        // Branch with BO bit 2 set (don't modify CTR)
        // BO=20 (ignore CTR)
        let opcode = 0x42800040u32; // bc with BO=20
        execute_instruction(&interpreter, &mut thread, opcode).unwrap();
        
        // CTR should NOT be decremented
        assert_eq!(thread.regs.ctr, 5);
    }

    // ===== Breakpoint Tests =====

    #[test]
    fn test_breakpoint_unconditional() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Add breakpoint at current PC
        interpreter.add_breakpoint(0x2000_0000, BreakpointType::Unconditional);
        
        // Write a simple instruction
        interpreter.memory.write_be32(0x2000_0000, 0x38600064).unwrap();
        
        // Step should hit breakpoint
        let result = interpreter.step(&mut thread);
        assert!(matches!(result, Err(PpuError::Breakpoint { addr: 0x2000_0000 })));
    }

    #[test]
    fn test_breakpoint_conditional_gpr() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Add conditional breakpoint that triggers when r3 == 42
        interpreter.add_breakpoint(
            0x2000_0000,
            BreakpointType::Conditional(BreakpointCondition::GprEquals {
                reg: 3,
                value: 42,
            }),
        );
        
        thread.set_gpr(3, 41);
        interpreter.memory.write_be32(0x2000_0000, 0x38600064).unwrap();
        
        // Should not break (r3 != 42)
        assert!(interpreter.step(&mut thread).is_ok());
        
        // Set r3 to 42
        thread.set_pc(0x2000_0000);
        thread.set_gpr(3, 42);
        
        // Should break now
        let result = interpreter.step(&mut thread);
        assert!(matches!(result, Err(PpuError::Breakpoint { .. })));
    }

    #[test]
    fn test_breakpoint_disable() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Add and then disable breakpoint
        interpreter.add_breakpoint(0x2000_0000, BreakpointType::Unconditional);
        interpreter.disable_breakpoint(0x2000_0000);
        
        interpreter.memory.write_be32(0x2000_0000, 0x38600064).unwrap();
        
        // Should not break (disabled)
        assert!(interpreter.step(&mut thread).is_ok());
    }

    #[test]
    fn test_breakpoint_hit_count() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        interpreter.add_breakpoint(0x2000_0000, BreakpointType::Unconditional);
        interpreter.memory.write_be32(0x2000_0000, 0x38600064).unwrap();
        
        // Hit breakpoint once
        let _ = interpreter.step(&mut thread);
        
        // Check hit count
        let breakpoints = interpreter.get_breakpoints();
        assert_eq!(breakpoints[0].hit_count, 1);
    }

    #[test]
    fn test_instruction_count() {
        let (interpreter, mut thread) = create_test_env();
        thread.set_pc(0x2000_0000);
        
        // Write some instructions
        for i in 0..5 {
            interpreter
                .memory
                .write_be32(0x2000_0000 + i * 4, 0x60000000)
                .unwrap(); // nop
        }
        
        // Execute 3 instructions
        for _ in 0..3 {
            let _ = interpreter.step(&mut thread);
        }
        
        assert_eq!(interpreter.instruction_count(), 3);
        
        interpreter.reset_instruction_count();
        assert_eq!(interpreter.instruction_count(), 0);
    }
}
