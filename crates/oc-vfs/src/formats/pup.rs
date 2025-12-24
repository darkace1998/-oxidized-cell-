//! PS3 Update Package (PUP) file format parser
//!
//! PUP files contain PS3 firmware updates and consist of:
//! - Header with magic "SCEUF"
//! - File entry table
//! - File data

use std::io::{self, Read, Seek, SeekFrom};
use thiserror::Error;

/// PUP magic signature
pub const PUP_MAGIC: &[u8; 5] = b"SCEUF";

/// PUP file format version
pub const PUP_VERSION: u64 = 1;

/// PUP file header
#[derive(Debug, Clone)]
pub struct PupHeader {
    /// Magic signature (should be "SCEUF")
    pub magic: [u8; 5],
    /// Package version
    pub package_version: u64,
    /// Image version
    pub image_version: u64,
    /// Number of files in the package
    pub file_count: u64,
    /// Header size
    pub header_size: u64,
    /// Data size
    pub data_size: u64,
}

/// PUP file entry
#[derive(Debug, Clone)]
pub struct PupEntry {
    /// Entry ID (type identifier)
    pub entry_id: u64,
    /// Offset to file data
    pub offset: u64,
    /// File size
    pub size: u64,
    /// Reserved field
    pub reserved: u64,
}

/// Known PUP entry types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum PupEntryType {
    /// Version information
    Version = 0x100,
    /// License file
    License = 0x101,
    /// PRX files
    Prx = 0x103,
    /// CoreOS
    CoreOs = 0x200,
    /// CoreOS extra
    CoreOsExtra = 0x201,
    /// CoreOS second loader
    CoreOsLoader = 0x202,
    /// Kernel
    Kernel = 0x300,
    /// SPU modules
    SpuModule = 0x501,
    /// SPU kernel
    SpuKernel = 0x601,
    /// Unknown entry type
    Unknown = 0xFFFFFFFF,
}

impl From<u64> for PupEntryType {
    fn from(value: u64) -> Self {
        match value {
            0x100 => Self::Version,
            0x101 => Self::License,
            0x103 => Self::Prx,
            0x200 => Self::CoreOs,
            0x201 => Self::CoreOsExtra,
            0x202 => Self::CoreOsLoader,
            0x300 => Self::Kernel,
            0x501 => Self::SpuModule,
            0x601 => Self::SpuKernel,
            _ => Self::Unknown,
        }
    }
}

impl PupEntryType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Version => "Version Info",
            Self::License => "License",
            Self::Prx => "PRX Module",
            Self::CoreOs => "CoreOS",
            Self::CoreOsExtra => "CoreOS Extra",
            Self::CoreOsLoader => "CoreOS Loader",
            Self::Kernel => "Kernel",
            Self::SpuModule => "SPU Module",
            Self::SpuKernel => "SPU Kernel",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Error, Debug)]
pub enum PupError {
    #[error("Invalid PUP magic signature")]
    InvalidMagic,
    #[error("Unsupported PUP version: {0}")]
    UnsupportedVersion(u64),
    #[error("Invalid file entry")]
    InvalidEntry,
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Entry not found: {0}")]
    EntryNotFound(u64),
}

pub type Result<T> = std::result::Result<T, PupError>;

/// PUP file parser
pub struct PupFile {
    pub header: PupHeader,
    pub entries: Vec<PupEntry>,
}

impl PupFile {
    /// Parse a PUP file from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        // Read header
        let header = Self::read_header(reader)?;
        
        // Validate header
        if &header.magic != PUP_MAGIC {
            return Err(PupError::InvalidMagic);
        }

        // Read file entries
        let entries = Self::read_entries(reader, header.file_count)?;

        Ok(Self { header, entries })
    }

    /// Read PUP header
    fn read_header<R: Read>(reader: &mut R) -> Result<PupHeader> {
        let mut magic = [0u8; 5];
        reader.read_exact(&mut magic)?;

        let mut buf = [0u8; 8];
        
        // Skip 3 bytes of padding
        reader.read_exact(&mut [0u8; 3])?;
        
        // Read package version (u64 big-endian at offset 0x08)
        reader.read_exact(&mut buf)?;
        let package_version = u64::from_be_bytes(buf);
        
        // Read image version (u64 big-endian at offset 0x10)
        reader.read_exact(&mut buf)?;
        let image_version = u64::from_be_bytes(buf);
        
        // Read file count (u64 big-endian at offset 0x18)
        reader.read_exact(&mut buf)?;
        let file_count = u64::from_be_bytes(buf);
        
        // Read header size (u64 big-endian at offset 0x20)
        reader.read_exact(&mut buf)?;
        let header_size = u64::from_be_bytes(buf);
        
        // Read data size (u64 big-endian at offset 0x28)
        reader.read_exact(&mut buf)?;
        let data_size = u64::from_be_bytes(buf);

        Ok(PupHeader {
            magic,
            package_version,
            image_version,
            file_count,
            header_size,
            data_size,
        })
    }

    /// Read file entries
    fn read_entries<R: Read>(reader: &mut R, count: u64) -> Result<Vec<PupEntry>> {
        let mut entries = Vec::with_capacity(count as usize);
        let mut buf = [0u8; 8];

        for _ in 0..count {
            // Read entry ID
            reader.read_exact(&mut buf)?;
            let entry_id = u64::from_be_bytes(buf);

            // Read offset
            reader.read_exact(&mut buf)?;
            let offset = u64::from_be_bytes(buf);

            // Read size
            reader.read_exact(&mut buf)?;
            let size = u64::from_be_bytes(buf);

            // Read reserved
            reader.read_exact(&mut buf)?;
            let reserved = u64::from_be_bytes(buf);

            entries.push(PupEntry {
                entry_id,
                offset,
                size,
                reserved,
            });
        }

        Ok(entries)
    }

    /// Extract an entry from the PUP file
    pub fn extract_entry<R: Read + Seek>(
        &self,
        reader: &mut R,
        entry_id: u64,
    ) -> Result<Vec<u8>> {
        let entry = self
            .entries
            .iter()
            .find(|e| e.entry_id == entry_id)
            .ok_or(PupError::EntryNotFound(entry_id))?;

        reader.seek(SeekFrom::Start(entry.offset))?;
        let mut data = vec![0u8; entry.size as usize];
        reader.read_exact(&mut data)?;

        Ok(data)
    }

    /// Get entry by ID
    pub fn get_entry(&self, entry_id: u64) -> Option<&PupEntry> {
        self.entries.iter().find(|e| e.entry_id == entry_id)
    }

    /// Get all entries of a specific type
    pub fn get_entries_by_type(&self, entry_type: PupEntryType) -> Vec<&PupEntry> {
        self.entries
            .iter()
            .filter(|e| PupEntryType::from(e.entry_id) == entry_type)
            .collect()
    }

    /// Print PUP information
    pub fn print_info(&self) {
        println!("=== PS3 Update Package (PUP) Information ===");
        println!("Package Version: 0x{:016X}", self.header.package_version);
        println!("Image Version: 0x{:016X}", self.header.image_version);
        println!("File Count: {}", self.header.file_count);
        println!("Header Size: 0x{:X} bytes", self.header.header_size);
        println!("Data Size: 0x{:X} bytes", self.header.data_size);
        println!("\n=== File Entries ===");
        
        for (i, entry) in self.entries.iter().enumerate() {
            let entry_type = PupEntryType::from(entry.entry_id);
            println!(
                "Entry {:2}: ID=0x{:03X} Type={:20} Offset=0x{:08X} Size=0x{:08X} ({} bytes)",
                i,
                entry.entry_id,
                entry_type.name(),
                entry.offset,
                entry.size,
                entry.size
            );
        }
    }

    /// Validate the PUP file structure
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check for overlapping entries
        let mut sorted_entries = self.entries.clone();
        sorted_entries.sort_by_key(|e| e.offset);

        for i in 0..sorted_entries.len() - 1 {
            let current = &sorted_entries[i];
            let next = &sorted_entries[i + 1];
            
            let current_end = current.offset + current.size;
            if current_end > next.offset {
                issues.push(format!(
                    "Overlapping entries: Entry 0x{:03X} ends at 0x{:X} but Entry 0x{:03X} starts at 0x{:X}",
                    current.entry_id, current_end, next.entry_id, next.offset
                ));
            }
        }

        // Check for entries outside data bounds
        let total_size = self.header.header_size + self.header.data_size;
        for entry in &self.entries {
            let entry_end = entry.offset + entry.size;
            if entry_end > total_size {
                issues.push(format!(
                    "Entry 0x{:03X} extends beyond file bounds: ends at 0x{:X}, file size is 0x{:X}",
                    entry.entry_id, entry_end, total_size
                ));
            }
        }

        // Check for zero-size entries
        for entry in &self.entries {
            if entry.size == 0 {
                issues.push(format!(
                    "Entry 0x{:03X} ({}) has zero size",
                    entry.entry_id,
                    PupEntryType::from(entry.entry_id).name()
                ));
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pup_magic() {
        assert_eq!(PUP_MAGIC, b"SCEUF");
    }

    #[test]
    fn test_entry_type_names() {
        assert_eq!(PupEntryType::Version.name(), "Version Info");
        assert_eq!(PupEntryType::CoreOs.name(), "CoreOS");
        assert_eq!(PupEntryType::Kernel.name(), "Kernel");
    }

    #[test]
    fn test_entry_type_conversion() {
        assert_eq!(PupEntryType::from(0x100), PupEntryType::Version);
        assert_eq!(PupEntryType::from(0x200), PupEntryType::CoreOs);
        assert_eq!(PupEntryType::from(0x999), PupEntryType::Unknown);
    }
}
