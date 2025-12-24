# Encryption Keys Guide

This document explains how to add encryption keys to the oxidized-cell PS3 emulator for decrypting SELF files.

## ⚠️ Legal Notice

**Important:** This emulator does NOT include any encryption keys for legal reasons. You must provide your own keys obtained through legal means. Keys should only be extracted from hardware that you own.

## Key File Format

Keys are stored in JSON format. The key file should be named `keys.json` and placed in the configuration directory:

- **Linux/macOS**: `~/.config/oxidized-cell/keys.json`
- **Windows**: `%APPDATA%\oxidized-cell\keys.json`

### JSON Structure

```json
{
  "version": "1.0",
  "keys": [
    {
      "type": "debug",
      "key": "00112233445566778899aabbccddeeff",
      "iv": "00000000000000000000000000000000",
      "description": "Debug key for development SELF files"
    },
    {
      "type": "retail",
      "key": "00112233445566778899aabbccddeeff",
      "iv": "00000000000000000000000000000000",
      "description": "Retail key for production SELF files"
    },
    {
      "type": "app",
      "key": "00112233445566778899aabbccddeeff",
      "description": "Application-specific key"
    }
  ]
}
```

### Key Types

The emulator supports the following key types:

- **`retail`** - Keys for retail/production PS3 systems
- **`debug`** - Keys for debug/development PS3 systems
- **`app`** - Application-specific keys
- **`iso_spu`** - Isolated SPU keys
- **`lv1`** - LV1 (hypervisor) keys
- **`lv2`** - LV2 (kernel) keys

### Fields

- **`type`** (required): One of the key types listed above
- **`key`** (required): The encryption key in hexadecimal format (32 characters for AES-128)
- **`iv`** (optional): The initialization vector in hexadecimal format (32 characters)
- **`description`** (optional): Human-readable description of the key

## Key Format

Keys should be provided as hexadecimal strings without any separators:
- **Correct**: `00112233445566778899aabbccddeeff`
- **Also works**: `00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff` (colons/spaces are stripped)
- **Incorrect**: `0x00112233...` (no 0x prefix)

### AES-128 Keys
- Key length: 16 bytes (32 hex characters)
- IV length: 16 bytes (32 hex characters)

## Example Key File

```json
{
  "version": "1.0",
  "keys": [
    {
      "type": "debug",
      "key": "00000000000000000000000000000000",
      "iv": "00000000000000000000000000000000",
      "description": "Placeholder debug key - REPLACE WITH REAL KEY"
    }
  ]
}
```

## Loading Keys

Keys are automatically loaded when the emulator starts if a `keys.json` file exists in the configuration directory.

You can also load keys programmatically:

```rust
use oc_loader::CryptoEngine;

let mut engine = CryptoEngine::new();
engine.load_keys_from_file("/path/to/keys.json")?;
```

## Verifying Keys

After loading keys, you can verify they were loaded correctly:

```rust
let stats = engine.get_stats();
println!("Loaded {} retail keys", stats.retail_keys);
println!("Loaded {} debug keys", stats.debug_keys);
println!("Total keys: {}", stats.total());
```

## Security Considerations

1. **Keep keys secure**: Never share or distribute your keys
2. **Use appropriate permissions**: On Unix systems, consider setting `chmod 600 keys.json`
3. **Backup keys**: Keep a secure backup of your key file
4. **Source verification**: Only use keys extracted from hardware you own

## Obtaining Keys

This emulator does not provide guidance on obtaining keys. Keys must be:
- Extracted from hardware you legally own
- Obtained through legal means
- Not redistributed or shared

## Troubleshooting

### Keys Not Loading

If keys aren't loading, check:
1. The file is named exactly `keys.json`
2. The file is in the correct directory
3. The JSON format is valid (use a JSON validator)
4. Hex strings are the correct length (32 characters for AES-128)

### Decryption Failures

If decryption fails:
1. Verify you're using the correct key type for the file
2. Check that the key and IV are correct
3. Ensure the SELF file is not corrupted
4. Verify the key was extracted correctly

## API Reference

### CryptoEngine Methods

```rust
// Load keys from file
engine.load_keys_from_file(path: &str) -> Result<(), LoaderError>

// Save keys to file
engine.save_keys_to_file(path: &str) -> Result<(), LoaderError>

// Add a key manually
engine.add_key(entry: KeyEntry)

// Check if a key type is available
engine.has_key(key_type: KeyType) -> bool

// Get key statistics
engine.get_stats() -> KeyStats

// Decrypt data with AES-CBC
engine.decrypt_aes(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, LoaderError>

// Verify SHA-1 hash
engine.verify_sha1(data: &[u8], expected: &[u8; 20]) -> bool

// Compute SHA-1 hash
engine.compute_sha1(data: &[u8]) -> [u8; 20]
```

## Additional Resources

- [PS3 Developer Wiki - Encryption](https://www.psdevwiki.com/ps3/Encryption)
- [PS3 Developer Wiki - SELF](https://www.psdevwiki.com/ps3/SELF_File_Format)
- Project README for more information about the emulator

---

**Remember**: This emulator is for educational and preservation purposes. Always respect copyright and intellectual property laws.
