//! Disassembler for PPU and SPU instructions

use oc_ppu::decoder::PpuDecoder;
use oc_spu::decoder::SpuDecoder;

/// Disassembled instruction
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    /// Address of the instruction
    pub address: u64,
    /// Raw opcode bytes
    pub opcode: u32,
    /// Mnemonic (instruction name)
    pub mnemonic: String,
    /// Operands as a string
    pub operands: String,
    /// Comment (if any)
    pub comment: Option<String>,
}

impl DisassembledInstruction {
    /// Format the instruction as a string
    pub fn to_string(&self) -> String {
        if self.operands.is_empty() {
            self.mnemonic.clone()
        } else {
            format!("{:8} {}", self.mnemonic, self.operands)
        }
    }

    /// Get opcode as hex string
    pub fn opcode_hex(&self) -> String {
        format!("{:08X}", self.opcode)
    }
}

/// PPU instruction disassembler
pub struct PpuDisassembler;

impl PpuDisassembler {
    /// Disassemble a single PPU instruction
    pub fn disassemble(address: u64, opcode: u32) -> DisassembledInstruction {
        let decoded = PpuDecoder::decode(opcode);
        let op = decoded.op;
        
        let (mnemonic, operands) = match op {
            // Branch instructions (I-Form)
            18 => {
                let (li, aa, lk) = PpuDecoder::i_form(opcode);
                let target = if aa { li as u64 } else { address.wrapping_add(li as u64) };
                let mnem = match (aa, lk) {
                    (false, false) => "b",
                    (false, true) => "bl",
                    (true, false) => "ba",
                    (true, true) => "bla",
                };
                (mnem.to_string(), format!("0x{:X}", target))
            }
            
            // Conditional branch (B-Form)
            16 => {
                let (bo, bi, bd, aa, lk) = PpuDecoder::b_form(opcode);
                let target = if aa { bd as u64 } else { address.wrapping_add(bd as i64 as u64) };
                let mnem = match (aa, lk) {
                    (false, false) => "bc",
                    (false, true) => "bcl",
                    (true, false) => "bca",
                    (true, true) => "bcla",
                };
                (mnem.to_string(), format!("{}, {}, 0x{:X}", bo, bi, target))
            }
            
            // System call
            17 => ("sc".to_string(), String::new()),
            
            // addi
            14 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                if ra == 0 {
                    ("li".to_string(), format!("r{}, {}", rt, d))
                } else {
                    ("addi".to_string(), format!("r{}, r{}, {}", rt, ra, d))
                }
            }
            
            // addis
            15 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                if ra == 0 {
                    ("lis".to_string(), format!("r{}, 0x{:X}", rt, d as u16))
                } else {
                    ("addis".to_string(), format!("r{}, r{}, 0x{:X}", rt, ra, d as u16))
                }
            }
            
            // ori
            24 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                if rs == 0 && ra == 0 && imm == 0 {
                    ("nop".to_string(), String::new())
                } else {
                    ("ori".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
                }
            }
            
            // oris
            25 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                ("oris".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
            }
            
            // xori
            26 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                ("xori".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
            }
            
            // xoris
            27 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                ("xoris".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
            }
            
            // andi.
            28 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                ("andi.".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
            }
            
            // andis.
            29 => {
                let (rs, ra, imm) = Self::d_form_logical(opcode);
                ("andis.".to_string(), format!("r{}, r{}, 0x{:X}", ra, rs, imm))
            }
            
            // Load/store word
            32 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lwz".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            33 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lwzu".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            34 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lbz".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            35 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lbzu".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            36 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("stw".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            37 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("stwu".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            38 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("stb".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            39 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("stbu".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            
            // Load/store halfword
            40 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lhz".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            41 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lhzu".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            42 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lha".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            43 => {
                let (rt, ra, d) = PpuDecoder::d_form(opcode);
                ("lhau".to_string(), format!("r{}, {}(r{})", rt, d, ra))
            }
            44 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("sth".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            45 => {
                let (rs, ra, d) = PpuDecoder::d_form(opcode);
                ("sthu".to_string(), format!("r{}, {}(r{})", rs, d, ra))
            }
            
            // Floating point load/store
            48 => {
                let (frt, ra, d) = PpuDecoder::d_form(opcode);
                ("lfs".to_string(), format!("f{}, {}(r{})", frt, d, ra))
            }
            50 => {
                let (frt, ra, d) = PpuDecoder::d_form(opcode);
                ("lfd".to_string(), format!("f{}, {}(r{})", frt, d, ra))
            }
            52 => {
                let (frs, ra, d) = PpuDecoder::d_form(opcode);
                ("stfs".to_string(), format!("f{}, {}(r{})", frs, d, ra))
            }
            54 => {
                let (frs, ra, d) = PpuDecoder::d_form(opcode);
                ("stfd".to_string(), format!("f{}, {}(r{})", frs, d, ra))
            }
            
            // Extended opcodes (op=19, XL-Form)
            19 => {
                let xo = decoded.xo;
                Self::disassemble_op19(opcode, xo)
            }
            
            // Extended opcodes (op=31, X-Form)
            31 => {
                let xo = decoded.xo;
                Self::disassemble_op31(opcode, xo)
            }
            
            // Extended opcodes (op=63, floating point)
            63 => {
                let xo = decoded.xo;
                Self::disassemble_op63(opcode, xo)
            }
            
            // Compare immediate
            10 => {
                let (bf, ra, si) = Self::d_form_cmp(opcode);
                ("cmpwi".to_string(), format!("cr{}, r{}, {}", bf, ra, si))
            }
            11 => {
                let (bf, ra, ui) = Self::d_form_cmpu(opcode);
                ("cmplwi".to_string(), format!("cr{}, r{}, {}", bf, ra, ui))
            }
            
            // mulli
            7 => {
                let (rt, ra, si) = PpuDecoder::d_form(opcode);
                ("mulli".to_string(), format!("r{}, r{}, {}", rt, ra, si))
            }
            
            // subfic
            8 => {
                let (rt, ra, si) = PpuDecoder::d_form(opcode);
                ("subfic".to_string(), format!("r{}, r{}, {}", rt, ra, si))
            }
            
            // Rotate instructions (M-Form)
            20 => {
                let (rs, ra, sh, mb, me, rc) = PpuDecoder::m_form(opcode);
                let mnem = if rc { "rlwimi." } else { "rlwimi" };
                (mnem.to_string(), format!("r{}, r{}, {}, {}, {}", ra, rs, sh, mb, me))
            }
            21 => {
                let (rs, ra, sh, mb, me, rc) = PpuDecoder::m_form(opcode);
                let mnem = if rc { "rlwinm." } else { "rlwinm" };
                (mnem.to_string(), format!("r{}, r{}, {}, {}, {}", ra, rs, sh, mb, me))
            }
            23 => {
                let (rs, ra, rb, mb, me, rc) = PpuDecoder::m_form(opcode);
                let mnem = if rc { "rlwnm." } else { "rlwnm" };
                (mnem.to_string(), format!("r{}, r{}, r{}, {}, {}", ra, rs, rb, mb, me))
            }
            
            _ => {
                ("???".to_string(), format!("op={}", op))
            }
        };
        
        DisassembledInstruction {
            address,
            opcode,
            mnemonic,
            operands,
            comment: None,
        }
    }

    /// Disassemble op=19 (XL-Form) instructions
    fn disassemble_op19(opcode: u32, xo: u16) -> (String, String) {
        let bo = ((opcode >> 21) & 0x1F) as u8;
        let bi = ((opcode >> 16) & 0x1F) as u8;
        let lk = (opcode & 1) != 0;
        
        match xo {
            16 => {
                // bclr
                let mnem = if lk { "bclrl" } else { "bclr" };
                // Simplified mnemonics
                if bo == 20 && bi == 0 {
                    if lk { ("blrl".to_string(), String::new()) }
                    else { ("blr".to_string(), String::new()) }
                } else {
                    (mnem.to_string(), format!("{}, {}", bo, bi))
                }
            }
            528 => {
                // bcctr
                let mnem = if lk { "bcctrl" } else { "bcctr" };
                if bo == 20 && bi == 0 {
                    if lk { ("bctrl".to_string(), String::new()) }
                    else { ("bctr".to_string(), String::new()) }
                } else {
                    (mnem.to_string(), format!("{}, {}", bo, bi))
                }
            }
            150 => ("isync".to_string(), String::new()),
            0 => {
                let bf = ((opcode >> 23) & 0x7) as u8;
                let bfa = ((opcode >> 18) & 0x7) as u8;
                ("mcrf".to_string(), format!("cr{}, cr{}", bf, bfa))
            }
            33 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crnor".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            129 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crandc".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            193 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crxor".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            225 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crnand".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            257 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crand".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            289 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("creqv".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            417 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("crorc".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            449 => {
                let bt = ((opcode >> 21) & 0x1F) as u8;
                let ba = ((opcode >> 16) & 0x1F) as u8;
                let bb = ((opcode >> 11) & 0x1F) as u8;
                ("cror".to_string(), format!("{}, {}, {}", bt, ba, bb))
            }
            _ => ("???".to_string(), format!("op19 xo={}", xo))
        }
    }

    /// Disassemble op=31 (X-Form) instructions
    fn disassemble_op31(opcode: u32, xo: u16) -> (String, String) {
        let (rt, ra, rb, _, rc) = PpuDecoder::x_form(opcode);
        
        match xo {
            // Arithmetic
            266 => {
                let mnem = if rc { "add." } else { "add" };
                (mnem.to_string(), format!("r{}, r{}, r{}", rt, ra, rb))
            }
            40 => {
                let mnem = if rc { "subf." } else { "subf" };
                (mnem.to_string(), format!("r{}, r{}, r{}", rt, ra, rb))
            }
            235 => {
                let mnem = if rc { "mullw." } else { "mullw" };
                (mnem.to_string(), format!("r{}, r{}, r{}", rt, ra, rb))
            }
            491 => {
                let mnem = if rc { "divw." } else { "divw" };
                (mnem.to_string(), format!("r{}, r{}, r{}", rt, ra, rb))
            }
            459 => {
                let mnem = if rc { "divwu." } else { "divwu" };
                (mnem.to_string(), format!("r{}, r{}, r{}", rt, ra, rb))
            }
            
            // Logical
            28 => {
                let mnem = if rc { "and." } else { "and" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            60 => {
                let mnem = if rc { "andc." } else { "andc" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            444 => {
                let mnem = if rc { "or." } else { "or" };
                if rt == rb {
                    ("mr".to_string(), format!("r{}, r{}", ra, rt))
                } else {
                    (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
                }
            }
            412 => {
                let mnem = if rc { "orc." } else { "orc" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            316 => {
                let mnem = if rc { "xor." } else { "xor" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            476 => {
                let mnem = if rc { "nand." } else { "nand" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            124 => {
                let mnem = if rc { "nor." } else { "nor" };
                if rt == rb {
                    ("not".to_string(), format!("r{}, r{}", ra, rt))
                } else {
                    (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
                }
            }
            284 => {
                let mnem = if rc { "eqv." } else { "eqv" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            
            // Shift
            24 => {
                let mnem = if rc { "slw." } else { "slw" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            536 => {
                let mnem = if rc { "srw." } else { "srw" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            792 => {
                let mnem = if rc { "sraw." } else { "sraw" };
                (mnem.to_string(), format!("r{}, r{}, r{}", ra, rt, rb))
            }
            824 => {
                let sh = rb;
                let mnem = if rc { "srawi." } else { "srawi" };
                (mnem.to_string(), format!("r{}, r{}, {}", ra, rt, sh))
            }
            
            // Compare
            0 => {
                let bf = (rt >> 2) as u8;
                ("cmp".to_string(), format!("cr{}, r{}, r{}", bf, ra, rb))
            }
            32 => {
                let bf = (rt >> 2) as u8;
                ("cmpl".to_string(), format!("cr{}, r{}, r{}", bf, ra, rb))
            }
            
            // Load/store indexed
            23 => ("lwzx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            55 => ("lwzux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            87 => ("lbzx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            119 => ("lbzux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            151 => ("stwx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            183 => ("stwux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            215 => ("stbx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            247 => ("stbux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            279 => ("lhzx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            311 => ("lhzux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            343 => ("lhax".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            375 => ("lhaux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            407 => ("sthx".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            439 => ("sthux".to_string(), format!("r{}, r{}, r{}", rt, ra, rb)),
            
            // Special registers
            339 => {
                let spr = ((rb as u16) << 5) | (ra as u16);
                let spr_name = Self::spr_name(spr);
                ("mfspr".to_string(), format!("r{}, {}", rt, spr_name))
            }
            467 => {
                let spr = ((rb as u16) << 5) | (ra as u16);
                let spr_name = Self::spr_name(spr);
                ("mtspr".to_string(), format!("{}, r{}", spr_name, rt))
            }
            19 => ("mfcr".to_string(), format!("r{}", rt)),
            144 => {
                let fxm = ((opcode >> 12) & 0xFF) as u8;
                ("mtcrf".to_string(), format!("0x{:02X}, r{}", fxm, rt))
            }
            
            // Extend
            26 => {
                let mnem = if rc { "cntlzw." } else { "cntlzw" };
                (mnem.to_string(), format!("r{}, r{}", ra, rt))
            }
            954 => {
                let mnem = if rc { "extsb." } else { "extsb" };
                (mnem.to_string(), format!("r{}, r{}", ra, rt))
            }
            922 => {
                let mnem = if rc { "extsh." } else { "extsh" };
                (mnem.to_string(), format!("r{}, r{}", ra, rt))
            }
            986 => {
                let mnem = if rc { "extsw." } else { "extsw" };
                (mnem.to_string(), format!("r{}, r{}", ra, rt))
            }
            
            // Sync
            598 => ("sync".to_string(), String::new()),
            854 => ("eieio".to_string(), String::new()),
            
            _ => ("???".to_string(), format!("op31 xo={}", xo))
        }
    }

    /// Disassemble op=63 (floating point) instructions
    fn disassemble_op63(opcode: u32, xo: u16) -> (String, String) {
        let frt = ((opcode >> 21) & 0x1F) as u8;
        let fra = ((opcode >> 16) & 0x1F) as u8;
        let frb = ((opcode >> 11) & 0x1F) as u8;
        let frc = ((opcode >> 6) & 0x1F) as u8;
        let rc = (opcode & 1) != 0;
        
        // A-Form (5-bit xo)
        let xo5 = (xo & 0x1F) as u8;
        
        match xo5 {
            18 => {
                let mnem = if rc { "fdiv." } else { "fdiv" };
                (mnem.to_string(), format!("f{}, f{}, f{}", frt, fra, frb))
            }
            20 => {
                let mnem = if rc { "fsub." } else { "fsub" };
                (mnem.to_string(), format!("f{}, f{}, f{}", frt, fra, frb))
            }
            21 => {
                let mnem = if rc { "fadd." } else { "fadd" };
                (mnem.to_string(), format!("f{}, f{}, f{}", frt, fra, frb))
            }
            25 => {
                let mnem = if rc { "fmul." } else { "fmul" };
                (mnem.to_string(), format!("f{}, f{}, f{}", frt, fra, frc))
            }
            28 => {
                let mnem = if rc { "fmsub." } else { "fmsub" };
                (mnem.to_string(), format!("f{}, f{}, f{}, f{}", frt, fra, frc, frb))
            }
            29 => {
                let mnem = if rc { "fmadd." } else { "fmadd" };
                (mnem.to_string(), format!("f{}, f{}, f{}, f{}", frt, fra, frc, frb))
            }
            30 => {
                let mnem = if rc { "fnmsub." } else { "fnmsub" };
                (mnem.to_string(), format!("f{}, f{}, f{}, f{}", frt, fra, frc, frb))
            }
            31 => {
                let mnem = if rc { "fnmadd." } else { "fnmadd" };
                (mnem.to_string(), format!("f{}, f{}, f{}, f{}", frt, fra, frc, frb))
            }
            _ => {
                // X-Form (10-bit xo)
                let xo10 = ((opcode >> 1) & 0x3FF) as u16;
                match xo10 {
                    0 => {
                        let bf = (frt >> 2) as u8;
                        ("fcmpu".to_string(), format!("cr{}, f{}, f{}", bf, fra, frb))
                    }
                    12 => {
                        let mnem = if rc { "frsp." } else { "frsp" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    14 => {
                        let mnem = if rc { "fctiw." } else { "fctiw" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    15 => {
                        let mnem = if rc { "fctiwz." } else { "fctiwz" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    32 => {
                        let bf = (frt >> 2) as u8;
                        ("fcmpo".to_string(), format!("cr{}, f{}, f{}", bf, fra, frb))
                    }
                    40 => {
                        let mnem = if rc { "fneg." } else { "fneg" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    72 => {
                        let mnem = if rc { "fmr." } else { "fmr" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    136 => {
                        let mnem = if rc { "fnabs." } else { "fnabs" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    264 => {
                        let mnem = if rc { "fabs." } else { "fabs" };
                        (mnem.to_string(), format!("f{}, f{}", frt, frb))
                    }
                    583 => ("mffs".to_string(), format!("f{}", frt)),
                    711 => {
                        let fm = ((opcode >> 17) & 0xFF) as u8;
                        ("mtfsf".to_string(), format!("0x{:02X}, f{}", fm, frb))
                    }
                    _ => ("???".to_string(), format!("op63 xo10={}", xo10))
                }
            }
        }
    }

    /// Extract D-form fields for logical instructions (different field order)
    fn d_form_logical(opcode: u32) -> (u8, u8, u16) {
        let rs = ((opcode >> 21) & 0x1F) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let imm = (opcode & 0xFFFF) as u16;
        (rs, ra, imm)
    }

    /// Extract D-form fields for compare instructions
    fn d_form_cmp(opcode: u32) -> (u8, u8, i16) {
        let bf = ((opcode >> 23) & 0x7) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let si = (opcode & 0xFFFF) as i16;
        (bf, ra, si)
    }

    /// Extract D-form fields for compare unsigned
    fn d_form_cmpu(opcode: u32) -> (u8, u8, u16) {
        let bf = ((opcode >> 23) & 0x7) as u8;
        let ra = ((opcode >> 16) & 0x1F) as u8;
        let ui = (opcode & 0xFFFF) as u16;
        (bf, ra, ui)
    }

    /// Get SPR name from number
    fn spr_name(spr: u16) -> String {
        match spr {
            1 => "XER".to_string(),
            8 => "LR".to_string(),
            9 => "CTR".to_string(),
            18 => "DSISR".to_string(),
            19 => "DAR".to_string(),
            22 => "DEC".to_string(),
            25 => "SDR1".to_string(),
            26 => "SRR0".to_string(),
            27 => "SRR1".to_string(),
            256 => "VRSAVE".to_string(),
            272 => "SPRG0".to_string(),
            273 => "SPRG1".to_string(),
            274 => "SPRG2".to_string(),
            275 => "SPRG3".to_string(),
            287 => "PVR".to_string(),
            _ => format!("SPR{}", spr),
        }
    }

    /// Disassemble a range of instructions from memory
    pub fn disassemble_range(memory: &[u8], base_address: u64, count: usize) -> Vec<DisassembledInstruction> {
        let mut result = Vec::with_capacity(count);
        let mut offset = 0;
        
        for _ in 0..count {
            if offset + 4 > memory.len() {
                break;
            }
            
            let opcode = u32::from_be_bytes([
                memory[offset],
                memory[offset + 1],
                memory[offset + 2],
                memory[offset + 3],
            ]);
            
            let addr = base_address + offset as u64;
            result.push(Self::disassemble(addr, opcode));
            offset += 4;
        }
        
        result
    }
}

/// SPU instruction disassembler
pub struct SpuDisassembler;

impl SpuDisassembler {
    /// Disassemble a single SPU instruction
    pub fn disassemble(address: u32, opcode: u32) -> DisassembledInstruction {
        let op4 = SpuDecoder::op4(opcode);
        let op7 = SpuDecoder::op7(opcode);
        let op8 = SpuDecoder::op8(opcode);
        let op9 = SpuDecoder::op9(opcode);
        let op10 = SpuDecoder::op10(opcode);
        let op11 = SpuDecoder::op11(opcode);

        let (mnemonic, operands) = Self::decode_instruction(opcode, op4, op7, op8, op9, op10, op11, address);
        
        DisassembledInstruction {
            address: address as u64,
            opcode,
            mnemonic,
            operands,
            comment: None,
        }
    }

    fn decode_instruction(
        opcode: u32,
        _op4: u8,
        op7: u8,
        op8: u8,
        _op9: u16,
        op10: u16,
        op11: u16,
        address: u32,
    ) -> (String, String) {
        // Check more specific opcodes first (longer opcodes have priority)
        
        // RR-type with op11 (most specific, 11-bit opcode)
        match op11 {
            0b00110101000 => {
                let rt = (opcode & 0x7F) as u8;
                return ("bi".to_string(), format!("${}", rt));
            }
            0b00110101001 => {
                let rt = (opcode & 0x7F) as u8;
                return ("bisl".to_string(), format!("$lr, ${}", rt));
            }
            0b00000000001 => {
                return ("nop".to_string(), String::new());
            }
            0b00000000000 => {
                return ("stop".to_string(), String::new());
            }
            0b00000001100 => {
                return ("dsync".to_string(), String::new());
            }
            0b00000000010 => {
                return ("lnop".to_string(), String::new());
            }
            0b00001111111 => {
                return ("sync".to_string(), String::new());
            }
            _ => {}
        }

        // RR-type (10-bit opcode)
        match op10 {
            0b0011000000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("a".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011001000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("ah".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001000000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("sf".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001001000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("sfh".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001100001 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("and".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001000001 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("or".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b1001000001 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("xor".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001100011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("andc".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001000011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("orc".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b1001100001 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("nand".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0001100000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("nor".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b1001100011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("eqv".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0111100000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("ceq".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0111101000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("ceqh".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0100100000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("cgt".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0100101000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("cgth".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0111000100 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("mpy".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0111001100 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("mpyu".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0101100000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("fa".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0101100001 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("fs".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0101100010 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("fm".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011010011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("shl".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011010000 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("roth".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011000011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("rotqby".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011011011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("shlqby".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011111011 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("rotqbi".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            0b0011111111 => {
                let (rb, ra, rt) = SpuDecoder::rr_form(opcode);
                return ("shlqbi".to_string(), format!("${}, ${}, ${}", rt, ra, rb));
            }
            _ => {}
        }

        // RI10-type (8-bit opcode)
        match op8 {
            0b00011100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("ai".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00011101 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("ahi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00010100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("sfi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00010101 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("sfhi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00010110 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("andi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00000100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("ori".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b01000100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("xori".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b01111100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("ceqi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b01111101 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("ceqhi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b01001110 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("cgti".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b01001111 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("cgthi".to_string(), format!("${}, ${}, {}", rt, ra, i10));
            }
            0b00100100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("stqd".to_string(), format!("${}, {}(${})", rt, i10 << 4, ra));
            }
            0b00110100 => {
                let (i10, ra, rt) = SpuDecoder::ri10_form(opcode);
                return ("lqd".to_string(), format!("${}, {}(${})", rt, i10 << 4, ra));
            }
            _ => {}
        }

        // RI16-type (7-bit opcode) - check before RI18 branches
        match op7 {
            0b0100001 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                return ("il".to_string(), format!("${}, {}", rt, i16_val));
            }
            0b0100010 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                return ("ilh".to_string(), format!("${}, 0x{:X}", rt, i16_val as u16));
            }
            0b0100011 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                return ("ilhu".to_string(), format!("${}, 0x{:X}", rt, i16_val as u16));
            }
            0b0110000 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brnz".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            0b0110001 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brz".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            0b0110010 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brhnz".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            0b0110011 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brhz".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            0b0100000 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                return ("iohl".to_string(), format!("${}, 0x{:X}", rt, i16_val as u16));
            }
            _ => {}
        }
        
        // RI18-type (4-bit opcode) - checked after more specific opcodes
        // br: bits 28-31 = 0b0010 (not 0b0100)
        // According to SPU ISA, br is 0b001000100 (9 bits) at bits 23-31
        // Let's check the specific branch opcodes
        let op9_branch = (opcode >> 23) & 0x1FF;
        match op9_branch {
            0b001100100 => {
                let (i16_val, _rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("br".to_string(), format!("0x{:X}", target));
            }
            0b001100000 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brsl".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            0b001000010 => {
                let (i16_val, _rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("bra".to_string(), format!("0x{:X}", target));
            }
            0b001100010 => {
                let (i16_val, rt) = SpuDecoder::ri16_form(opcode);
                let target = (address as i32).wrapping_add((i16_val as i32) << 2) as u32;
                return ("brasl".to_string(), format!("${}, 0x{:X}", rt, target));
            }
            _ => {}
        }

        // RRR-type (4-operand)
        let (rc, rb, ra, rt) = SpuDecoder::rrr_form(opcode);
        let op4_rrr = (opcode >> 28) & 0xF;
        match op4_rrr {
            0b1000 => return ("selb".to_string(), format!("${}, ${}, ${}, ${}", rt, ra, rb, rc)),
            0b1011 => return ("shufb".to_string(), format!("${}, ${}, ${}, ${}", rt, ra, rb, rc)),
            0b1100 => return ("mpya".to_string(), format!("${}, ${}, ${}, ${}", rt, ra, rb, rc)),
            0b1110 => return ("fma".to_string(), format!("${}, ${}, ${}, ${}", rt, ra, rb, rc)),
            0b1111 => return ("fms".to_string(), format!("${}, ${}, ${}, ${}", rt, ra, rb, rc)),
            _ => {}
        }

        // Unknown instruction
        ("???".to_string(), format!("0x{:08X}", opcode))
    }
    /// Disassemble a range of instructions from memory
    pub fn disassemble_range(memory: &[u8], base_address: u32, count: usize) -> Vec<DisassembledInstruction> {
        let mut result = Vec::with_capacity(count);
        let mut offset = 0;
        
        for _ in 0..count {
            if offset + 4 > memory.len() {
                break;
            }
            
            let opcode = u32::from_be_bytes([
                memory[offset],
                memory[offset + 1],
                memory[offset + 2],
                memory[offset + 3],
            ]);
            
            let addr = base_address + offset as u32;
            result.push(Self::disassemble(addr, opcode));
            offset += 4;
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppu_disassemble_nop() {
        // ori r0, r0, 0 = nop
        let dis = PpuDisassembler::disassemble(0, 0x60000000);
        assert_eq!(dis.mnemonic, "nop");
    }

    #[test]
    fn test_ppu_disassemble_li() {
        // li r3, 100
        let dis = PpuDisassembler::disassemble(0, 0x38600064);
        assert_eq!(dis.mnemonic, "li");
        assert!(dis.operands.contains("r3"));
        assert!(dis.operands.contains("100"));
    }

    #[test]
    fn test_ppu_disassemble_blr() {
        // blr
        let dis = PpuDisassembler::disassemble(0, 0x4E800020);
        assert_eq!(dis.mnemonic, "blr");
    }

    #[test]
    fn test_ppu_disassemble_branch() {
        // b 0x100
        let dis = PpuDisassembler::disassemble(0, 0x48000100);
        assert_eq!(dis.mnemonic, "b");
        assert!(dis.operands.contains("0x100"));
    }

    #[test]
    fn test_spu_disassemble_nop() {
        // nop - op11 = 0x001 (1)
        // Opcode: 0x00200000 is lnop, 0x40200000 looks like br
        // Actually SPU nop is 0x00200000 >> 21 gives op11 = 0x001
        let dis = SpuDisassembler::disassemble(0, 0x00200000);
        assert_eq!(dis.mnemonic, "nop");
    }

    #[test]
    fn test_spu_disassemble_il() {
        // il $3, 100 - op7 = 0b0100001 (33), i16 = 100, rt = 3
        // op7 is bits 25-31, so 33 << 25 = 0x42000000
        // i16 is bits 7-22, so 100 << 7 = 0x3200
        // rt is bits 0-6, so 3 = 0x03
        // Combined: 0x42000000 | (100 << 7) | 3 = 0x42003203
        let dis = SpuDisassembler::disassemble(0, 0x42003203);
        assert_eq!(dis.mnemonic, "il");
    }
}
