//! PKG file format

use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

/// Minimal PKG header representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PkgHeader {
    pub pkg_type: u32,
    pub pkg_size: u64,
}

impl PkgHeader {
    /// Parse a PKG header. Real PKG files start with the magic 0x7F 'P' 'K' 'G'.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self, std::io::Error> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"\x7FPKG" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid PKG magic",
            ));
        }

        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        let pkg_type = u32::from_be_bytes(buf);

        let mut size_buf = [0u8; 8];
        reader.read_exact(&mut size_buf)?;
        let pkg_size = u64::from_be_bytes(size_buf);

        Ok(Self { pkg_type, pkg_size })
    }
}

/// Quick helper to validate whether a file looks like a PKG.
pub fn is_pkg(path: &Path) -> bool {
    File::open(path)
        .and_then(|mut f| {
            let mut magic = [0u8; 4];
            f.read_exact(&mut magic)?;
            Ok(&magic == b"\x7FPKG")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_pkg_header() {
        let data: Vec<u8> = vec![
            0x7F, b'P', b'K', b'G', // magic
            0x00, 0x00, 0x00, 0x02, // type
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // size = 4096
        ];
        let mut cursor = Cursor::new(data);
        let header = PkgHeader::parse(&mut cursor).unwrap();
        assert_eq!(header.pkg_type, 2);
        assert_eq!(header.pkg_size, 4096);
    }
}
