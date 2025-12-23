//! PARAM.SFO file format

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// SFO file entry
#[derive(Debug, Clone)]
pub enum SfoValue {
    Utf8(String),
    Utf8S(String),
    Integer(u32),
}

/// PARAM.SFO parser
pub struct Sfo {
    entries: HashMap<String, SfoValue>,
}

impl Sfo {
    /// Parse SFO from reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self, std::io::Error> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if &magic != b"\x00PSF" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid SFO magic",
            ));
        }

        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;

        let _version = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let key_table_start = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let data_table_start = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let entries_count = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);

        let mut entries = HashMap::new();

        for i in 0..entries_count {
            let entry_offset = 20 + i * 16;
            reader.seek(SeekFrom::Start(entry_offset as u64))?;

            let mut entry_data = [0u8; 16];
            reader.read_exact(&mut entry_data)?;

            let key_offset = u16::from_le_bytes([entry_data[0], entry_data[1]]);
            let data_fmt = u16::from_le_bytes([entry_data[2], entry_data[3]]);
            let data_len = u32::from_le_bytes([entry_data[4], entry_data[5], entry_data[6], entry_data[7]]);
            let _data_max_len = u32::from_le_bytes([entry_data[8], entry_data[9], entry_data[10], entry_data[11]]);
            let data_offset = u32::from_le_bytes([entry_data[12], entry_data[13], entry_data[14], entry_data[15]]);

            // Read key
            reader.seek(SeekFrom::Start((key_table_start + key_offset as u32) as u64))?;
            let mut key = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                reader.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                key.push(byte[0]);
            }
            let key = String::from_utf8_lossy(&key).to_string();

            // Read value
            reader.seek(SeekFrom::Start((data_table_start + data_offset) as u64))?;
            let value = match data_fmt {
                0x0404 => {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf)?;
                    SfoValue::Integer(u32::from_le_bytes(buf))
                }
                0x0004 | 0x0204 => {
                    let mut buf = vec![0u8; data_len as usize];
                    reader.read_exact(&mut buf)?;
                    // Remove null terminator if present
                    while buf.last() == Some(&0) {
                        buf.pop();
                    }
                    let s = String::from_utf8_lossy(&buf).to_string();
                    if data_fmt == 0x0004 {
                        SfoValue::Utf8S(s)
                    } else {
                        SfoValue::Utf8(s)
                    }
                }
                _ => continue,
            };

            entries.insert(key, value);
        }

        Ok(Self { entries })
    }

    /// Get a string value
    pub fn get_string(&self, key: &str) -> Option<&str> {
        match self.entries.get(key)? {
            SfoValue::Utf8(s) | SfoValue::Utf8S(s) => Some(s),
            _ => None,
        }
    }

    /// Get an integer value
    pub fn get_integer(&self, key: &str) -> Option<u32> {
        match self.entries.get(key)? {
            SfoValue::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// Get title
    pub fn title(&self) -> Option<&str> {
        self.get_string("TITLE")
    }

    /// Get title ID
    pub fn title_id(&self) -> Option<&str> {
        self.get_string("TITLE_ID")
    }

    /// Get version
    pub fn version(&self) -> Option<&str> {
        self.get_string("VERSION")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sfo_struct() {
        // Test would require actual SFO data
    }
}
