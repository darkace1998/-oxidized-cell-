//! ISO 9660 file system

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Minimal ISO 9660 parser used for quick validation of disc images.
pub struct IsoImage {
    valid: bool,
    volume_id: Option<String>,
}

impl IsoImage {
    /// Open and validate an ISO file by checking for the primary volume
    /// descriptor ("CD001") at the expected location.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let mut file = File::open(path)?;

        // Primary Volume Descriptor is located at offset 0x8000; the "CD001"
        // magic begins one byte later.
        file.seek(SeekFrom::Start(0x8001))?;
        let mut magic = [0u8; 5];
        file.read_exact(&mut magic)?;
        let valid = &magic == b"CD001";

        let mut volume_id = None;
        if valid {
            // Volume ID is located at offset 0x8028 and is 32 bytes long.
            file.seek(SeekFrom::Start(0x8028))?;
            let mut buf = [0u8; 32];
            file.read_exact(&mut buf)?;
            let id = String::from_utf8_lossy(&buf)
                .trim_end_matches('\u{0}')
                .trim()
                .to_string();
            if !id.is_empty() {
                volume_id = Some(id);
            }
        }

        Ok(Self { valid, volume_id })
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn volume_id(&self) -> Option<&str> {
        self.volume_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_iso_magic() {
        let tmp_path = std::env::temp_dir().join("oc_iso_test.iso");
        let mut file = File::create(&tmp_path).unwrap();
        file.write_all(&vec![0u8; 0x8001]).unwrap();
        file.write_all(b"CD001").unwrap();
        // pad until volume id location
        let current_len = file.metadata().unwrap().len();
        if current_len < 0x8028 {
            let pad = (0x8028 - current_len) as usize;
            file.write_all(&vec![0u8; pad]).unwrap();
        }
        let mut volume = [0u8; 32];
        let id = b"TEST_VOLUME_ID";
        volume[..id.len()].copy_from_slice(id);
        file.write_all(&volume).unwrap();
        drop(file);

        let iso = IsoImage::open(&tmp_path).unwrap();
        assert!(iso.is_valid());
        assert!(iso.volume_id().is_some());
    }
}
