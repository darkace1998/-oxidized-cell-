//! SELF file loader

use oc_core::error::LoaderError;

/// SELF file magic
pub const SELF_MAGIC: [u8; 4] = [0x53, 0x43, 0x45, 0x00]; // "SCE\0"

/// SELF loader
pub struct SelfLoader;

impl SelfLoader {
    /// Check if data is a SELF file
    pub fn is_self(data: &[u8]) -> bool {
        data.len() >= 4 && data[0..4] == SELF_MAGIC
    }

    /// Decrypt and extract ELF from SELF
    pub fn decrypt(_data: &[u8]) -> Result<Vec<u8>, LoaderError> {
        // SELF decryption requires encryption keys
        // This is a placeholder - actual implementation would need keys
        Err(LoaderError::DecryptionFailed("SELF decryption not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_magic() {
        assert_eq!(SELF_MAGIC, [0x53, 0x43, 0x45, 0x00]);
    }

    #[test]
    fn test_is_self() {
        let self_data = [0x53, 0x43, 0x45, 0x00, 0x00, 0x00];
        assert!(SelfLoader::is_self(&self_data));

        let elf_data = [0x7F, b'E', b'L', b'F', 0x00, 0x00];
        assert!(!SelfLoader::is_self(&elf_data));
    }
}
