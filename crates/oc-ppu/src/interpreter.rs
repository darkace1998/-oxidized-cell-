//! PPU interpreter implementation

use std::sync::Arc;
use oc_memory::MemoryManager;
use oc_core::error::PpuError;
use crate::decoder::{PpuDecoder, InstructionForm};
use crate::thread::PpuThread;

/// PPU interpreter for instruction execution
pub struct PpuInterpreter {
    /// Memory manager
    memory: Arc<MemoryManager>,
}

impl PpuInterpreter {
    /// Create a new PPU interpreter
    pub fn new(memory: Arc<MemoryManager>) -> Self {
        Self { memory }
    }

    /// Execute a single instruction
    pub fn step(&self, thread: &mut PpuThread) -> Result<(), PpuError> {
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

    /// Execute a decoded instruction
    fn execute(&self, thread: &mut PpuThread, opcode: u32, decoded: crate::decoder::DecodedInstruction) -> Result<(), PpuError> {
        match decoded.form {
            InstructionForm::D => self.execute_d_form(thread, opcode, decoded.op),
            InstructionForm::I => self.execute_i_form(thread, opcode),
            InstructionForm::B => self.execute_b_form(thread, opcode),
            InstructionForm::X => self.execute_x_form(thread, opcode, decoded.xo),
            InstructionForm::XO => self.execute_xo_form(thread, opcode, decoded.xo),
            InstructionForm::XL => self.execute_xl_form(thread, opcode, decoded.xo),
            InstructionForm::M => self.execute_m_form(thread, opcode, decoded.op),
            InstructionForm::SC => self.execute_sc(thread, opcode),
            _ => {
                tracing::warn!("Unimplemented instruction form: {:?} at 0x{:08x}", decoded.form, thread.pc());
                thread.advance_pc();
                Ok(())
            }
        }
    }

    /// Execute D-form instructions
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

    /// Execute XL-form instructions (branch to LR/CTR)
    fn execute_xl_form(&self, thread: &mut PpuThread, opcode: u32, xo: u16) -> Result<(), PpuError> {
        let bo = ((opcode >> 21) & 0x1F) as u8;
        let bi = ((opcode >> 16) & 0x1F) as u8;
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
    fn update_cr0(&self, thread: &mut PpuThread, value: u64) {
        let value = value as i64;
        let c = if value < 0 { 0b1000 } else if value > 0 { 0b0100 } else { 0b0010 };
        let c = c | if thread.get_xer_so() { 1 } else { 0 };
        thread.set_cr_field(0, c);
    }

    /// Generate 32-bit mask for rotate instructions
    fn generate_mask_32(mb: u8, me: u8) -> u32 {
        let mb = mb as u32;
        let me = me as u32;
        if mb <= me {
            (u32::MAX >> mb) & (u32::MAX << (31 - me))
        } else {
            (u32::MAX >> mb) | (u32::MAX << (31 - me))
        }
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
}
