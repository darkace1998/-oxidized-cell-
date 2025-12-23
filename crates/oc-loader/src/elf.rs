//! ELF file parser

use std::io::{Read, Seek, SeekFrom};
use oc_core::error::LoaderError;

/// ELF file header (64-bit)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// ELF program header (64-bit)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// ELF section header (64-bit)
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

/// ELF magic bytes
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// Program header types
pub mod pt {
    pub const NULL: u32 = 0;
    pub const LOAD: u32 = 1;
    pub const DYNAMIC: u32 = 2;
    pub const INTERP: u32 = 3;
    pub const NOTE: u32 = 4;
    pub const TLS: u32 = 7;
}

/// ELF loader
pub struct ElfLoader;

impl ElfLoader {
    /// Parse ELF header from reader
    pub fn parse_header<R: Read + Seek>(reader: &mut R) -> Result<Elf64Header, LoaderError> {
        reader.seek(SeekFrom::Start(0)).map_err(|e| LoaderError::InvalidElf(e.to_string()))?;

        let mut header = Elf64Header::default();
        
        // Read ident
        reader.read_exact(&mut header.e_ident).map_err(|e| LoaderError::InvalidElf(e.to_string()))?;
        
        // Verify magic
        if header.e_ident[0..4] != ELF_MAGIC {
            return Err(LoaderError::InvalidElf("Invalid ELF magic".to_string()));
        }
        
        // Check 64-bit
        if header.e_ident[4] != 2 {
            return Err(LoaderError::InvalidElf("Not a 64-bit ELF".to_string()));
        }
        
        // Check big-endian (PS3 is big-endian)
        if header.e_ident[5] != 2 {
            return Err(LoaderError::InvalidElf("Not big-endian ELF".to_string()));
        }
        
        // Read rest of header
        let mut buf = [0u8; 48];
        reader.read_exact(&mut buf).map_err(|e| LoaderError::InvalidElf(e.to_string()))?;
        
        header.e_type = u16::from_be_bytes([buf[0], buf[1]]);
        header.e_machine = u16::from_be_bytes([buf[2], buf[3]]);
        header.e_version = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        header.e_entry = u64::from_be_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]);
        header.e_phoff = u64::from_be_bytes([buf[16], buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23]]);
        header.e_shoff = u64::from_be_bytes([buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31]]);
        header.e_flags = u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]);
        header.e_ehsize = u16::from_be_bytes([buf[36], buf[37]]);
        header.e_phentsize = u16::from_be_bytes([buf[38], buf[39]]);
        header.e_phnum = u16::from_be_bytes([buf[40], buf[41]]);
        header.e_shentsize = u16::from_be_bytes([buf[42], buf[43]]);
        header.e_shnum = u16::from_be_bytes([buf[44], buf[45]]);
        header.e_shstrndx = u16::from_be_bytes([buf[46], buf[47]]);
        
        Ok(header)
    }
    
    /// Parse program headers
    pub fn parse_phdrs<R: Read + Seek>(reader: &mut R, header: &Elf64Header) -> Result<Vec<Elf64Phdr>, LoaderError> {
        let mut phdrs = Vec::with_capacity(header.e_phnum as usize);
        
        for i in 0..header.e_phnum {
            let offset = header.e_phoff + (i as u64 * header.e_phentsize as u64);
            reader.seek(SeekFrom::Start(offset)).map_err(|e| LoaderError::InvalidElf(e.to_string()))?;
            
            let mut buf = [0u8; 56];
            reader.read_exact(&mut buf).map_err(|e| LoaderError::InvalidElf(e.to_string()))?;
            
            let phdr = Elf64Phdr {
                p_type: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
                p_flags: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
                p_offset: u64::from_be_bytes([buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]]),
                p_vaddr: u64::from_be_bytes([buf[16], buf[17], buf[18], buf[19], buf[20], buf[21], buf[22], buf[23]]),
                p_paddr: u64::from_be_bytes([buf[24], buf[25], buf[26], buf[27], buf[28], buf[29], buf[30], buf[31]]),
                p_filesz: u64::from_be_bytes([buf[32], buf[33], buf[34], buf[35], buf[36], buf[37], buf[38], buf[39]]),
                p_memsz: u64::from_be_bytes([buf[40], buf[41], buf[42], buf[43], buf[44], buf[45], buf[46], buf[47]]),
                p_align: u64::from_be_bytes([buf[48], buf[49], buf[50], buf[51], buf[52], buf[53], buf[54], buf[55]]),
            };
            
            phdrs.push(phdr);
        }
        
        Ok(phdrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_magic() {
        assert_eq!(ELF_MAGIC, [0x7F, b'E', b'L', b'F']);
    }
}
