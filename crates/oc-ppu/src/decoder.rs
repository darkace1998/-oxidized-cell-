//! PPU instruction decoder

/// Decoded PPU instruction
#[derive(Debug, Clone, Copy)]
pub struct DecodedInstruction {
    /// Raw opcode
    pub opcode: u32,
    /// Primary opcode (bits 0-5)
    pub op: u8,
    /// Extended opcode (various positions depending on instruction form)
    pub xo: u16,
    /// Instruction form
    pub form: InstructionForm,
}

/// PPU instruction forms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionForm {
    /// I-Form: Branch instructions
    I,
    /// B-Form: Conditional branch
    B,
    /// SC-Form: System call
    SC,
    /// D-Form: Load/store with displacement
    D,
    /// DS-Form: Load/store double with displacement
    DS,
    /// X-Form: Indexed load/store, misc
    X,
    /// XL-Form: Branch conditional to LR/CTR
    XL,
    /// XFX-Form: Move to/from special registers
    XFX,
    /// XFL-Form: Move to FPSCR
    XFL,
    /// XS-Form: Shift double
    XS,
    /// XO-Form: Integer arithmetic
    XO,
    /// A-Form: Floating-point multiply-add
    A,
    /// M-Form: Rotate and mask
    M,
    /// MD-Form: Rotate and mask (64-bit)
    MD,
    /// MDS-Form: Rotate and mask shift (64-bit)
    MDS,
    /// VA-Form: Vector three-operand
    VA,
    /// VX-Form: Vector two-operand
    VX,
    /// VXR-Form: Vector compare
    VXR,
    /// Unknown form
    Unknown,
}

/// PPU instruction decoder
pub struct PpuDecoder;

impl PpuDecoder {
    /// Decode a 32-bit PPU instruction
    pub fn decode(opcode: u32) -> DecodedInstruction {
        let op = ((opcode >> 26) & 0x3F) as u8;
        
        let (form, xo) = match op {
            // I-Form branches
            18 => (InstructionForm::I, 0),
            
            // B-Form conditional branches
            16 => (InstructionForm::B, 0),
            
            // SC-Form system call
            17 => (InstructionForm::SC, 0),
            
            // D-Form load/store
            14 | 15 | // addi, addis
            32..=39 | // lwz, lwzu, lbz, lbzu, stw, stwu, stb, stbu
            40..=47 | // lhz, lhzu, lha, lhau, sth, sthu, lmw, stmw
            48..=55 | // lfs, lfsu, lfd, lfdu, stfs, stfsu, stfd, stfdu
            8..=13 | // subfic, cmpli, cmpi, addic, addic., mulli
            24..=29 | // ori, oris, xori, xoris, andi., andis.
            2 | 3 | // tdi, twi
            7 | // mulli
            58 => (InstructionForm::D, 0),
            
            // DS-Form
            62 => (InstructionForm::DS, 0),
            
            // Special opcode groups
            19 => {
                let xo = ((opcode >> 1) & 0x3FF) as u16;
                (InstructionForm::XL, xo)
            }
            
            31 => {
                let xo = ((opcode >> 1) & 0x3FF) as u16;
                // Could be X, XO, XS, XFX, or other forms
                (InstructionForm::X, xo)
            }
            
            30 => {
                // MD/MDS-Form rotate
                let xo = ((opcode >> 2) & 0x7) as u16;
                (InstructionForm::MD, xo)
            }
            
            // M-Form rotate
            20..=23 => (InstructionForm::M, 0),
            
            // A-Form floating-point
            59 | 63 => {
                let xo = ((opcode >> 1) & 0x1F) as u16;
                (InstructionForm::A, xo)
            }
            
            // Vector instructions
            4 => {
                let xo = (opcode & 0x3F) as u16;
                (InstructionForm::VA, xo)
            }
            
            _ => (InstructionForm::Unknown, 0),
        };
        
        DecodedInstruction {
            opcode,
            op,
            xo,
            form,
        }
    }
    
    /// Extract D-form fields
    #[inline]
    pub fn d_form(opcode: u32) -> (u8, u8, i16) {
        let rt = ((opcode >> 21) & 0x1F) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let d = (opcode & 0xFFFF) as i16;
        (rt, ra, d)
    }
    
    /// Extract X-form fields
    #[inline]
    pub fn x_form(opcode: u32) -> (u8, u8, u8, u16, bool) {
        let rt = ((opcode >> 21) & 0x1F) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let rb = ((opcode >> 11) & 0x1F) as u8;
        let xo = ((opcode >> 1) & 0x3FF) as u16;
        let rc = (opcode & 1) != 0;
        (rt, ra, rb, xo, rc)
    }
    
    /// Extract XO-form fields (integer arithmetic)
    #[inline]
    pub fn xo_form(opcode: u32) -> (u8, u8, u8, bool, u16, bool) {
        let rt = ((opcode >> 21) & 0x1F) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let rb = ((opcode >> 11) & 0x1F) as u8;
        let oe = ((opcode >> 10) & 1) != 0;
        let xo = ((opcode >> 1) & 0x1FF) as u16;
        let rc = (opcode & 1) != 0;
        (rt, ra, rb, oe, xo, rc)
    }
    
    /// Extract I-form fields (branch)
    #[inline]
    pub fn i_form(opcode: u32) -> (i32, bool, bool) {
        let li = ((opcode >> 2) & 0xFFFFFF) as i32;
        // Sign extend from 24 bits
        let li = if li & 0x800000 != 0 {
            li | !0xFFFFFF
        } else {
            li
        } << 2;
        let aa = ((opcode >> 1) & 1) != 0;
        let lk = (opcode & 1) != 0;
        (li, aa, lk)
    }
    
    /// Extract B-form fields (conditional branch)
    #[inline]
    pub fn b_form(opcode: u32) -> (u8, u8, i16, bool, bool) {
        let bo = ((opcode >> 21) & 0x1F) as u8;
        let bi = ((opcode >> 16) & 0x1F) as u8;
        let bd = ((opcode >> 2) & 0x3FFF) as i16;
        // Sign extend from 14 bits
        let bd = if bd & 0x2000 != 0 {
            bd | !0x3FFF
        } else {
            bd
        } << 2;
        let aa = ((opcode >> 1) & 1) != 0;
        let lk = (opcode & 1) != 0;
        (bo, bi, bd, aa, lk)
    }
    
    /// Extract M-form fields (rotate)
    #[inline]
    pub fn m_form(opcode: u32) -> (u8, u8, u8, u8, u8, bool) {
        let rs = ((opcode >> 21) & 0x1F) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let rb = ((opcode >> 11) & 0x1F) as u8;
        let mb = ((opcode >> 6) & 0x1F) as u8;
        let me = ((opcode >> 1) & 0x1F) as u8;
        let rc = (opcode & 1) != 0;
        (rs, ra, rb, mb, me, rc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_addi() {
        // addi r3, r0, 100
        let opcode = 0x38600064u32;
        let decoded = PpuDecoder::decode(opcode);
        assert_eq!(decoded.op, 14);
        assert_eq!(decoded.form, InstructionForm::D);
    }

    #[test]
    fn test_d_form_extract() {
        // addi r3, r1, 8
        let opcode = 0x38610008u32;
        let (rt, ra, d) = PpuDecoder::d_form(opcode);
        assert_eq!(rt, 3);
        assert_eq!(ra, 1);
        assert_eq!(d, 8);
    }

    #[test]
    fn test_i_form_branch() {
        // b 0x100
        let opcode = 0x48000100u32;
        let (li, aa, lk) = PpuDecoder::i_form(opcode);
        assert_eq!(li, 0x100);
        assert!(!aa);
        assert!(!lk);
    }
}
