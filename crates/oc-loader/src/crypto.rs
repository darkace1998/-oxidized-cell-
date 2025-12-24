//! Cryptographic operations for SELF decryption
//!
//! Note: Actual decryption keys are not included for legal reasons.
//! This module provides the infrastructure for decryption when keys are available.

use oc_core::error::LoaderError;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, warn, info};
use aes::Aes128;
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use sha1::{Sha1, Digest};

type Aes128CbcEnc = cbc::Encryptor<Aes128>;
type Aes128CbcDec = cbc::Decryptor<Aes128>;

use aes::Aes256;
type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

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

        // Perform actual AES-CBC decryption
        match key.len() {
            AES_128_KEY_SIZE => {
                let cipher = Aes128CbcDec::new_from_slices(key, iv)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create AES-128 cipher: {}", e)))?;
                
                let mut buffer = encrypted_data.to_vec();
                let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("AES-128 decryption failed: {}", e)))?;
                
                Ok(decrypted.to_vec())
            }
            AES_256_KEY_SIZE => {
                let cipher = Aes256CbcDec::new_from_slices(key, iv)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create AES-256 cipher: {}", e)))?;
                
                let mut buffer = encrypted_data.to_vec();
                let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("AES-256 decryption failed: {}", e)))?;
                
                Ok(decrypted.to_vec())
            }
            _ => unreachable!("Key length already validated"),
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
                format!("IV length (must be {} bytes)", AES_IV_SIZE),
            ));
        }

        // Perform actual AES-CBC encryption (with automatic PKCS7 padding)
        match key.len() {
            AES_128_KEY_SIZE => {
                let cipher = Aes128CbcEnc::new_from_slices(key, iv)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create AES-128 cipher: {}", e)))?;
                
                let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
                Ok(ciphertext)
            }
            AES_256_KEY_SIZE => {
                let cipher = Aes256CbcEnc::new_from_slices(key, iv)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to create AES-256 cipher: {}", e)))?;
                
                let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
                Ok(ciphertext)
            }
            _ => unreachable!("Key length already validated"),
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
        
        // Compute SHA-1 hash
        let mut hasher = Sha1::new();
        hasher.update(data);
        let computed_hash = hasher.finalize();
        
        // Compare with expected hash
        let matches = computed_hash.as_slice() == expected_hash;
        
        if matches {
            debug!("SHA-1 hash verification successful");
        } else {
            warn!("SHA-1 hash verification failed");
            debug!("Expected: {:02x?}", expected_hash);
            debug!("Computed: {:02x?}", computed_hash.as_slice());
        }
        
        matches
    }

    /// Load keys from a file
    pub fn load_keys_from_file(&mut self, path: &str) -> Result<(), LoaderError> {
        info!("Loading keys from file: {}", path);
        
        let path = Path::new(path);
        
        if !path.exists() {
            return Err(LoaderError::DecryptionFailed(
                format!("Key file not found: {}", path.display())
            ));
        }
        
        let contents = std::fs::read_to_string(path)
            .map_err(|e| LoaderError::DecryptionFailed(format!("Failed to read key file: {}", e)))?;
        
        // Try to parse as JSON first, then TOML
        if let Ok(keys) = serde_json::from_str::<KeyFile>(&contents) {
            self.load_keys_from_data(keys)?;
        } else {
            return Err(LoaderError::DecryptionFailed(
                "Failed to parse key file (expected JSON format)".to_string()
            ));
        }
        
        info!("Successfully loaded keys from file");
        Ok(())
    }
    
    /// Load keys from parsed key file data
    fn load_keys_from_data(&mut self, key_file: KeyFile) -> Result<(), LoaderError> {
        for key_data in key_file.keys {
            // Parse key type
            let key_type = match key_data.key_type.to_lowercase().as_str() {
                "retail" => KeyType::Retail,
                "debug" => KeyType::Debug,
                "app" => KeyType::App,
                "iso_spu" | "isospu" => KeyType::IsoSpu,
                "lv1" => KeyType::Lv1,
                "lv2" => KeyType::Lv2,
                _ => {
                    warn!("Unknown key type: {}, skipping", key_data.key_type);
                    continue;
                }
            };
            
            // Decode hex key
            let key = hex_decode(&key_data.key)
                .map_err(|e| LoaderError::DecryptionFailed(format!("Invalid key hex: {}", e)))?;
            
            // Validate key length
            if key.len() != AES_128_KEY_SIZE && key.len() != AES_256_KEY_SIZE {
                warn!("Invalid key length for {}: {} bytes, skipping", key_data.description, key.len());
                continue;
            }
            
            // Decode IV if present
            let iv = if let Some(iv_hex) = key_data.iv {
                let decoded = hex_decode(&iv_hex)
                    .map_err(|e| LoaderError::DecryptionFailed(format!("Invalid IV hex: {}", e)))?;
                if decoded.len() != AES_IV_SIZE {
                    warn!("Invalid IV length for {}: {} bytes, skipping", key_data.description, decoded.len());
                    continue;
                }
                Some(decoded)
            } else {
                None
            };
            
            let entry = KeyEntry {
                key_type,
                key,
                iv,
                description: key_data.description.clone(),
            };
            
            self.add_key(entry);
        }
        
        Ok(())
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

/// Key file structure for JSON format
#[derive(Debug, serde::Deserialize)]
struct KeyFile {
    keys: Vec<KeyData>,
}

/// Individual key data in key file
#[derive(Debug, serde::Deserialize)]
struct KeyData {
    key_type: String,
    key: String,
    iv: Option<String>,
    description: String,
}

/// Decode hex string to bytes
fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    // Remove any whitespace and common prefixes
    let hex = hex.trim().trim_start_matches("0x").trim_start_matches("0X");
    
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {}: {}", i, e))
        })
        .collect()
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
    fn test_aes_128_encryption_decryption() {
        let engine = CryptoEngine::new();
        let key = vec![0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                       0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let iv = vec![0u8; 16];
        let plaintext = b"Hello, PS3 World!";
        
        // Encrypt
        let ciphertext = engine.encrypt_aes(plaintext, &key, &iv).unwrap();
        assert_ne!(ciphertext.as_slice(), plaintext);
        assert!(ciphertext.len() >= plaintext.len()); // Should be padded
        
        // Decrypt
        let decrypted = engine.decrypt_aes(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
    
    #[test]
    fn test_aes_256_encryption_decryption() {
        let engine = CryptoEngine::new();
        let key = vec![
            0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe,
            0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81,
            0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7,
            0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
        ];
        let iv = vec![0u8; 16];
        let plaintext = b"Testing AES-256!";
        
        // Encrypt
        let ciphertext = engine.encrypt_aes(plaintext, &key, &iv).unwrap();
        assert_ne!(ciphertext.as_slice(), plaintext);
        
        // Decrypt
        let decrypted = engine.decrypt_aes(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
    
    #[test]
    fn test_aes_with_different_iv() {
        let engine = CryptoEngine::new();
        let key = vec![0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                       0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let iv1 = vec![0u8; 16];
        let iv2 = vec![1u8; 16];
        let plaintext = b"Same plaintext!";
        
        let ciphertext1 = engine.encrypt_aes(plaintext, &key, &iv1).unwrap();
        let ciphertext2 = engine.encrypt_aes(plaintext, &key, &iv2).unwrap();
        
        // Different IVs should produce different ciphertexts
        assert_ne!(ciphertext1, ciphertext2);
    }
    
    #[test]
    fn test_sha1_verification() {
        let engine = CryptoEngine::new();
        let data = b"Hello, World!";
        
        // Known SHA-1 hash for "Hello, World!"
        let expected_hash: [u8; 20] = [
            0x0a, 0x0a, 0x9f, 0x2a, 0x67, 0x72, 0x94, 0x25,
            0x57, 0xab, 0x53, 0x55, 0xd7, 0x6a, 0xf4, 0x42,
            0xf8, 0xf6, 0x5e, 0x01,
        ];
        
        assert!(engine.verify_sha1(data, &expected_hash));
        
        // Wrong hash should fail
        let wrong_hash = [0u8; 20];
        assert!(!engine.verify_sha1(data, &wrong_hash));
    }
    
    #[test]
    fn test_hex_decode() {
        // Valid hex
        assert_eq!(hex_decode("48656c6c6f").unwrap(), b"Hello");
        assert_eq!(hex_decode("0x48656c6c6f").unwrap(), b"Hello");
        assert_eq!(hex_decode("  48656c6c6f  ").unwrap(), b"Hello");
        
        // Invalid hex
        assert!(hex_decode("4").is_err()); // Odd length
        assert!(hex_decode("4g").is_err()); // Invalid character
    }
    
    #[test]
    fn test_key_file_loading_invalid_path() {
        let mut engine = CryptoEngine::new();
        let result = engine.load_keys_from_file("/nonexistent/path/keys.json");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_aes_block_padding() {
        let engine = CryptoEngine::new();
        let key = vec![0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                       0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
        let iv = vec![0u8; 16];
        
        // Test with data that needs padding
        let plaintext = b"Short";
        let ciphertext = engine.encrypt_aes(plaintext, &key, &iv).unwrap();
        
        // Ciphertext should be padded to block size
        assert_eq!(ciphertext.len() % 16, 0);
        assert!(ciphertext.len() >= 16);
        
        // Decrypt should remove padding
        let decrypted = engine.decrypt_aes(&ciphertext, &key, &iv).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }
}
