//! Cryptographic operations for SELF decryption
//!
//! Note: Actual decryption keys are not included for legal reasons.
//! This module provides the infrastructure for decryption when keys are available.

use oc_core::error::LoaderError;
use std::collections::HashMap;
use tracing::{debug, warn};
use aes::Aes128;
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha1::{Sha1, Digest};

/// Key types for PS3 encryption
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyType {
    /// Retail (production) keys
    Retail,
    /// Debug keys
    Debug,
    /// Application-specific keys
    App,
    /// Isolated SPU keys
    IsoSpu,
    /// LV1 (hypervisor) keys
    Lv1,
    /// LV2 (kernel) keys
    Lv2,
}

/// Encryption algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    /// AES-128 CBC
    Aes128Cbc,
    /// AES-256 CBC
    Aes256Cbc,
    /// No encryption
    None,
}

/// Key database entry
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub key_type: KeyType,
    pub key: Vec<u8>,
    pub iv: Option<Vec<u8>>,
    pub description: String,
}

/// AES key size constants
const AES_128_KEY_SIZE: usize = 16;
const AES_256_KEY_SIZE: usize = 32;
const AES_IV_SIZE: usize = 16;
const AES_BLOCK_SIZE: usize = 16;

/// Crypto engine for SELF decryption
pub struct CryptoEngine {
    keys: HashMap<KeyType, Vec<KeyEntry>>,
}

impl CryptoEngine {
    /// Create a new crypto engine
    pub fn new() -> Self {
        let mut engine = Self {
            keys: HashMap::new(),
        };

        // Initialize with placeholder keys
        // Real implementation would load actual keys from a secure key database
        engine.init_placeholder_keys();
        
        engine
    }

    /// Initialize placeholder keys for testing
    fn init_placeholder_keys(&mut self) {
        warn!("Using placeholder encryption keys - decryption will not work with real SELF files");

        // Add placeholder entries
        self.add_key(KeyEntry {
            key_type: KeyType::Debug,
            key: vec![0u8; AES_128_KEY_SIZE],
            iv: Some(vec![0u8; AES_IV_SIZE]),
            description: "Placeholder debug key".to_string(),
        });

        self.add_key(KeyEntry {
            key_type: KeyType::Retail,
            key: vec![0u8; AES_128_KEY_SIZE],
            iv: Some(vec![0u8; AES_IV_SIZE]),
            description: "Placeholder retail key".to_string(),
        });
    }

    /// Add a key to the database
    pub fn add_key(&mut self, entry: KeyEntry) {
        debug!("Adding key: {}", entry.description);
        self.keys
            .entry(entry.key_type)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get a key by type
    pub fn get_key(&self, key_type: KeyType) -> Option<&[u8]> {
        self.keys
            .get(&key_type)
            .and_then(|entries| entries.first())
            .map(|entry| entry.key.as_slice())
    }

    /// Get all keys of a specific type
    pub fn get_keys(&self, key_type: KeyType) -> Vec<&KeyEntry> {
        self.keys
            .get(&key_type)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    /// Decrypt data using AES
    pub fn decrypt_aes(
        &self,
        encrypted_data: &[u8],
        key: &[u8],
        iv: &[u8],
    ) -> Result<Vec<u8>, LoaderError> {
        debug!(
            "AES decryption: data_len={}, key_len={}, iv_len={}",
            encrypted_data.len(),
            key.len(),
            iv.len()
        );

        // Validate inputs
        if key.len() != AES_128_KEY_SIZE && key.len() != AES_256_KEY_SIZE {
            return Err(LoaderError::DecryptionFailed(
                format!("Invalid key length (must be {} or {} bytes)", AES_128_KEY_SIZE, AES_256_KEY_SIZE),
            ));
        }

        if iv.len() != AES_IV_SIZE {
            return Err(LoaderError::DecryptionFailed(
                format!("Invalid IV length (must be {} bytes)", AES_IV_SIZE),
            ));
        }

        if encrypted_data.len() % AES_BLOCK_SIZE != 0 {
            return Err(LoaderError::DecryptionFailed(
                "Encrypted data length must be multiple of 16".to_string(),
            ));
        }

        // Perform AES-128-CBC decryption
        if key.len() == AES_128_KEY_SIZE {
            let mut buffer = encrypted_data.to_vec();
            
            type Aes128CbcDec = Decryptor<Aes128>;
            let decryptor = Aes128CbcDec::new_from_slices(key, iv)
                .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create decryptor: {}", e)))?;
            
            let decrypted = decryptor.decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buffer)
                .map_err(|e| LoaderError::DecryptionFailed(format!("Decryption failed: {}", e)))?;
            
            Ok(decrypted.to_vec())
        } else {
            // For AES-256, we'd need to use Aes256 instead
            // For now, return an error as PS3 primarily uses AES-128
            Err(LoaderError::DecryptionFailed(
                "AES-256 not yet implemented".to_string(),
            ))
        }
    }

    /// Encrypt data using AES
    pub fn encrypt_aes(
        &self,
        plaintext: &[u8],
        key: &[u8],
        iv: &[u8],
    ) -> Result<Vec<u8>, LoaderError> {
        debug!(
            "AES encryption: data_len={}, key_len={}, iv_len={}",
            plaintext.len(),
            key.len(),
            iv.len()
        );

        // Validate inputs
        if key.len() != AES_128_KEY_SIZE && key.len() != AES_256_KEY_SIZE {
            return Err(LoaderError::DecryptionFailed(
                format!("Invalid key length (must be {} or {} bytes)", AES_128_KEY_SIZE, AES_256_KEY_SIZE),
            ));
        }

        if iv.len() != AES_IV_SIZE {
            return Err(LoaderError::DecryptionFailed(
                format!("Invalid IV length (must be {} bytes)", AES_IV_SIZE),
            ));
        }

        // Perform AES-128-CBC encryption
        if key.len() == AES_128_KEY_SIZE {
            type Aes128CbcEnc = Encryptor<Aes128>;
            let encryptor = Aes128CbcEnc::new_from_slices(key, iv)
                .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create encryptor: {}", e)))?;
            
            let ciphertext = encryptor.encrypt_padded_vec_mut::<cbc::cipher::block_padding::Pkcs7>(plaintext);
            Ok(ciphertext)
        } else {
            // For AES-256, we'd need to use Aes256 instead
            Err(LoaderError::DecryptionFailed(
                "AES-256 not yet implemented".to_string(),
            ))
        }
    }

    /// Decrypt metadata using MetaLV2 keys
    pub fn decrypt_metadata_lv2(
        &self,
        encrypted_metadata: &[u8],
        key_type: KeyType,
    ) -> Result<Vec<u8>, LoaderError> {
        debug!("Decrypting MetaLV2 metadata with key type: {:?}", key_type);

        let key = self.get_key(key_type)
            .ok_or_else(|| LoaderError::DecryptionFailed("Key not found".to_string()))?;

        // MetaLV2 uses specific IV (typically all zeros)
        let iv = vec![0u8; AES_IV_SIZE];

        self.decrypt_aes(encrypted_metadata, key, &iv)
    }

    /// Verify SHA-1 hash
    pub fn verify_sha1(&self, data: &[u8], expected_hash: &[u8; 20]) -> bool {
        debug!("SHA-1 verification: data_len={}", data.len());
        
        // Compute SHA-1 hash of the data
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        // Compare with expected hash
        let matches = result.as_slice() == expected_hash;
        
        if !matches {
            debug!("SHA-1 mismatch - Expected: {:02x?}, Got: {:02x?}", 
                expected_hash, result.as_slice());
        }
        
        matches
    }

    /// Compute SHA-1 hash
    pub fn compute_sha1(&self, data: &[u8]) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 20];
        hash.copy_from_slice(result.as_slice());
        hash
    }

    /// Load keys from a file
    pub fn load_keys_from_file(&mut self, path: &str) -> Result<(), LoaderError> {
        use std::fs::File;
        use std::io::Read;
        
        debug!("Loading keys from file: {}", path);
        
        // Try to open the file
        let mut file = File::open(path).map_err(|e| {
            LoaderError::DecryptionFailed(format!("Failed to open key file: {}", e))
        })?;
        
        // Read the file contents
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| {
            LoaderError::DecryptionFailed(format!("Failed to read key file: {}", e))
        })?;
        
        // Parse as JSON
        let key_data: serde_json::Value = serde_json::from_str(&contents).map_err(|e| {
            LoaderError::DecryptionFailed(format!("Failed to parse key file: {}", e))
        })?;
        
        // Process keys array
        if let Some(keys_array) = key_data["keys"].as_array() {
            for key_obj in keys_array {
                self.load_key_from_json(key_obj)?;
            }
        }
        
        debug!("Successfully loaded keys from file");
        Ok(())
    }
    
    /// Load a single key from JSON object
    fn load_key_from_json(&mut self, key_obj: &serde_json::Value) -> Result<(), LoaderError> {
        // Extract key type
        let key_type_str = key_obj["type"].as_str()
            .ok_or_else(|| LoaderError::DecryptionFailed("Missing key type".to_string()))?;
        
        let key_type = match key_type_str {
            "retail" => KeyType::Retail,
            "debug" => KeyType::Debug,
            "app" => KeyType::App,
            "iso_spu" => KeyType::IsoSpu,
            "lv1" => KeyType::Lv1,
            "lv2" => KeyType::Lv2,
            _ => return Err(LoaderError::DecryptionFailed(
                format!("Unknown key type: {}", key_type_str)
            )),
        };
        
        // Extract key hex string
        let key_hex = key_obj["key"].as_str()
            .ok_or_else(|| LoaderError::DecryptionFailed("Missing key data".to_string()))?;
        
        // Decode hex string
        let key = Self::decode_hex(key_hex)?;
        
        // Extract optional IV
        let iv = if let Some(iv_hex) = key_obj["iv"].as_str() {
            Some(Self::decode_hex(iv_hex)?)
        } else {
            None
        };
        
        // Extract description
        let description = key_obj["description"].as_str()
            .unwrap_or("No description")
            .to_string();
        
        // Add the key
        self.add_key(KeyEntry {
            key_type,
            key,
            iv,
            description,
        });
        
        Ok(())
    }
    
    /// Decode a hex string to bytes
    fn decode_hex(hex: &str) -> Result<Vec<u8>, LoaderError> {
        // Remove any whitespace or colons
        let cleaned: String = hex.chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();
        
        if cleaned.len() % 2 != 0 {
            return Err(LoaderError::DecryptionFailed(
                "Hex string must have even length".to_string()
            ));
        }
        
        let mut result = Vec::new();
        for i in (0..cleaned.len()).step_by(2) {
            let byte_str = &cleaned[i..i+2];
            let byte = u8::from_str_radix(byte_str, 16)
                .map_err(|e| LoaderError::DecryptionFailed(format!("Invalid hex: {}", e)))?;
            result.push(byte);
        }
        
        Ok(result)
    }
    
    /// Save keys to a file
    pub fn save_keys_to_file(&self, path: &str) -> Result<(), LoaderError> {
        use std::fs::File;
        use std::io::Write;
        
        debug!("Saving keys to file: {}", path);
        
        let mut keys_array = Vec::new();
        
        for (key_type, entries) in &self.keys {
            for entry in entries {
                let key_type_str = match key_type {
                    KeyType::Retail => "retail",
                    KeyType::Debug => "debug",
                    KeyType::App => "app",
                    KeyType::IsoSpu => "iso_spu",
                    KeyType::Lv1 => "lv1",
                    KeyType::Lv2 => "lv2",
                };
                
                let mut key_obj = serde_json::json!({
                    "type": key_type_str,
                    "key": Self::encode_hex(&entry.key),
                    "description": entry.description,
                });
                
                if let Some(iv) = &entry.iv {
                    key_obj["iv"] = serde_json::json!(Self::encode_hex(iv));
                }
                
                keys_array.push(key_obj);
            }
        }
        
        let output = serde_json::json!({
            "version": "1.0",
            "keys": keys_array,
        });
        
        let json_str = serde_json::to_string_pretty(&output)
            .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to serialize keys: {}", e)))?;
        
        let mut file = File::create(path)
            .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create key file: {}", e)))?;
        
        file.write_all(json_str.as_bytes())
            .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to write key file: {}", e)))?;
        
        debug!("Successfully saved keys to file");
        Ok(())
    }
    
    /// Encode bytes to hex string
    fn encode_hex(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    /// Check if a key type is available
    pub fn has_key(&self, key_type: KeyType) -> bool {
        self.keys.contains_key(&key_type)
    }

    /// Get key database statistics
    pub fn get_stats(&self) -> KeyStats {
        let mut stats = KeyStats::default();
        
        for (key_type, entries) in &self.keys {
            let count = entries.len();
            match key_type {
                KeyType::Retail => stats.retail_keys = count,
                KeyType::Debug => stats.debug_keys = count,
                KeyType::App => stats.app_keys = count,
                KeyType::IsoSpu => stats.iso_spu_keys = count,
                KeyType::Lv1 => stats.lv1_keys = count,
                KeyType::Lv2 => stats.lv2_keys = count,
            }
        }

        stats
    }
}

/// Key database statistics
#[derive(Debug, Default)]
pub struct KeyStats {
    pub retail_keys: usize,
    pub debug_keys: usize,
    pub app_keys: usize,
    pub iso_spu_keys: usize,
    pub lv1_keys: usize,
    pub lv2_keys: usize,
}

impl KeyStats {
    /// Get total number of keys across all types
    pub fn total(&self) -> usize {
        self.retail_keys + self.debug_keys + self.app_keys +
        self.iso_spu_keys + self.lv1_keys + self.lv2_keys
    }
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_engine_creation() {
        let engine = CryptoEngine::new();
        assert!(engine.has_key(KeyType::Debug));
        assert!(engine.has_key(KeyType::Retail));
    }

    #[test]
    fn test_key_addition() {
        let mut engine = CryptoEngine::new();
        
        let key_entry = KeyEntry {
            key_type: KeyType::App,
            key: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            iv: Some(vec![0u8; 16]),
            description: "Test key".to_string(),
        };

        engine.add_key(key_entry);
        assert!(engine.has_key(KeyType::App));
    }

    #[test]
    fn test_key_retrieval() {
        let engine = CryptoEngine::new();
        let key = engine.get_key(KeyType::Debug);
        assert!(key.is_some());
        assert_eq!(key.unwrap().len(), 16);
    }

    #[test]
    fn test_aes_validation() {
        let engine = CryptoEngine::new();
        
        // Test with invalid key length
        let result = engine.decrypt_aes(&[0u8; 16], &[0u8; 8], &[0u8; 16]);
        assert!(result.is_err());

        // Test with invalid IV length
        let result = engine.decrypt_aes(&[0u8; 16], &[0u8; 16], &[0u8; 8]);
        assert!(result.is_err());

        // Test with non-block-aligned data
        let result = engine.decrypt_aes(&[0u8; 15], &[0u8; 16], &[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_stats() {
        let engine = CryptoEngine::new();
        let stats = engine.get_stats();
        
        assert_eq!(stats.debug_keys, 1);
        assert_eq!(stats.retail_keys, 1);
    }

    #[test]
    fn test_key_types() {
        assert_ne!(KeyType::Retail, KeyType::Debug);
        assert_ne!(KeyType::App, KeyType::Lv1);
    }
    
    #[test]
    fn test_aes_encryption_decryption() {
        let engine = CryptoEngine::new();
        
        let key = vec![0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                       0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let iv = vec![0u8; 16];
        let plaintext = b"Hello, World! This is a test message.";
        
        // Encrypt
        let encrypted = engine.encrypt_aes(plaintext, &key, &iv).unwrap();
        assert_ne!(encrypted.as_slice(), plaintext);
        assert_eq!(encrypted.len() % 16, 0); // Should be padded to block size
        
        // Decrypt
        let decrypted = engine.decrypt_aes(&encrypted, &key, &iv).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
    
    #[test]
    fn test_sha1_verification() {
        let engine = CryptoEngine::new();
        
        let data = b"The quick brown fox jumps over the lazy dog";
        let expected_hash = [
            0x2f, 0xd4, 0xe1, 0xc6, 0x7a, 0x2d, 0x28, 0xfc,
            0xed, 0x84, 0x9e, 0xe1, 0xbb, 0x76, 0xe7, 0x39,
            0x1b, 0x93, 0xeb, 0x12,
        ];
        
        assert!(engine.verify_sha1(data, &expected_hash));
        
        // Test with wrong hash
        let wrong_hash = [0u8; 20];
        assert!(!engine.verify_sha1(data, &wrong_hash));
    }
    
    #[test]
    fn test_sha1_computation() {
        let engine = CryptoEngine::new();
        
        let data = b"The quick brown fox jumps over the lazy dog";
        let hash = engine.compute_sha1(data);
        
        let expected_hash = [
            0x2f, 0xd4, 0xe1, 0xc6, 0x7a, 0x2d, 0x28, 0xfc,
            0xed, 0x84, 0x9e, 0xe1, 0xbb, 0x76, 0xe7, 0x39,
            0x1b, 0x93, 0xeb, 0x12,
        ];
        
        assert_eq!(hash, expected_hash);
    }
    
    #[test]
    fn test_hex_encode_decode() {
        let engine = CryptoEngine::new();
        
        let original = vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0];
        let hex = CryptoEngine::encode_hex(&original);
        assert_eq!(hex, "123456789abcdef0");
        
        let decoded = CryptoEngine::decode_hex(&hex).unwrap();
        assert_eq!(decoded, original);
    }
    
    #[test]
    fn test_hex_decode_with_formatting() {
        // Should handle various hex formats
        let hex1 = "12:34:56:78";
        let hex2 = "12 34 56 78";
        let hex3 = "12345678";
        
        let result1 = CryptoEngine::decode_hex(hex1).unwrap();
        let result2 = CryptoEngine::decode_hex(hex2).unwrap();
        let result3 = CryptoEngine::decode_hex(hex3).unwrap();
        
        assert_eq!(result1, vec![0x12, 0x34, 0x56, 0x78]);
        assert_eq!(result2, vec![0x12, 0x34, 0x56, 0x78]);
        assert_eq!(result3, vec![0x12, 0x34, 0x56, 0x78]);
    }
}
