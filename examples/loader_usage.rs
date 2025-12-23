//! Example usage of the ELF/Game Loader
//!
//! This example demonstrates how to use the oc-loader crate to:
//! 1. Load an ELF executable
//! 2. Parse symbols and apply relocations
//! 3. Load PRX modules
//! 4. Handle SELF decryption

use oc_loader::{ElfLoader, SelfLoader, PrxLoader, CryptoEngine, KeyType, KeyEntry};
use oc_memory::MemoryManager;
use std::sync::Arc;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== ELF/Game Loader Example ===\n");

    // Example 1: Load ELF file
    example_load_elf()?;

    // Example 2: Check and decrypt SELF file
    example_check_self()?;

    // Example 3: Load PRX module
    example_load_prx()?;

    // Example 4: Crypto engine usage
    example_crypto_engine()?;

    Ok(())
}

/// Example: Load an ELF file and parse its contents
fn example_load_elf() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 1: Loading ELF file");
    println!("----------------------------");

    // Create a minimal ELF header for demonstration
    // In real usage, you would open an actual ELF file
    let elf_data = create_minimal_elf();
    let mut reader = Cursor::new(elf_data);

    // Parse ELF file
    match ElfLoader::new(&mut reader) {
        Ok(mut elf) => {
            println!("✓ ELF loaded successfully");
            println!("  Entry point: 0x{:x}", elf.entry_point);
            println!("  Program headers: {}", elf.phdrs.len());
            println!("  Section headers: {}", elf.shdrs.len());

            // Initialize memory manager
            let memory = Arc::new(MemoryManager::new()?);

            // Load segments into memory at base address
            let base_addr = 0x10000;
            elf.load_segments(&mut reader, &memory, base_addr)?;
            println!("  Segments loaded at base 0x{:08x}", base_addr);

            // Parse symbols
            elf.parse_symbols(&mut reader)?;
            println!("  Symbols parsed: {}", elf.symbols.len());

            // Example: Find a specific symbol
            if let Some(symbol) = elf.resolve_symbol("main") {
                println!("  Found symbol 'main' at 0x{:x}", symbol.value);
            }

            // Process relocations
            elf.process_relocations(&mut reader, &memory, base_addr)?;
            println!("  Relocations applied");
        }
        Err(e) => {
            println!("✗ Failed to load ELF: {}", e);
        }
    }

    println!();
    Ok(())
}

/// Example: Check if file is SELF and decrypt it
fn example_check_self() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 2: SELF File Handling");
    println!("------------------------------");

    // Create sample SELF data
    let self_data = vec![0x53, 0x43, 0x45, 0x00]; // "SCE\0" magic

    // Check if file is SELF
    if SelfLoader::is_self(&self_data) {
        println!("✓ File identified as SELF");

        // Create SELF loader with crypto support
        let loader = SelfLoader::new();

        // Attempt to decrypt (will fail without real keys, but shows usage)
        match loader.decrypt(&self_data) {
            Ok(elf_data) => {
                println!("✓ SELF decrypted successfully");
                println!("  Extracted ELF size: {} bytes", elf_data.len());
            }
            Err(e) => {
                println!("✗ Decryption failed: {}", e);
                println!("  (This is expected without real encryption keys)");
            }
        }
    } else {
        println!("✗ File is not SELF format");
    }

    println!();
    Ok(())
}

/// Example: Load a PRX module and resolve symbols
fn example_load_prx() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 3: PRX Module Loading");
    println!("------------------------------");

    // Create PRX loader
    let mut prx_loader = PrxLoader::new();
    println!("✓ PRX loader initialized");

    // In real usage, you would load an actual PRX file
    // Here we demonstrate the API usage

    // List loaded modules (should be empty initially)
    let modules = prx_loader.list_modules();
    println!("  Currently loaded modules: {}", modules.len());

    // Example: Calculate NID for a function name
    // (NID is a hash used for symbol resolution in PS3)
    let test_symbol = "cellGcmGetFlipStatus";
    println!("\n  Symbol resolution example:");
    println!("    Looking up symbol: {}", test_symbol);

    // Attempt to resolve (will be None until modules are loaded)
    match prx_loader.resolve_symbol_by_name(test_symbol) {
        Some(addr) => {
            println!("    ✓ Resolved to address: 0x{:x}", addr);
        }
        None => {
            println!("    Symbol not found (no modules loaded)");
        }
    }

    println!();
    Ok(())
}

/// Example: Using the crypto engine for key management
fn example_crypto_engine() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 4: Crypto Engine");
    println!("-------------------------");

    // Create crypto engine
    let mut crypto = CryptoEngine::new();
    println!("✓ Crypto engine initialized");

    // Check available keys
    println!("\n  Available key types:");
    println!("    Debug keys: {}", crypto.has_key(KeyType::Debug));
    println!("    Retail keys: {}", crypto.has_key(KeyType::Retail));

    // Get key statistics
    let stats = crypto.get_stats();
    println!("\n  Key database statistics:");
    println!("    Debug keys: {}", stats.debug_keys);
    println!("    Retail keys: {}", stats.retail_keys);
    println!("    App keys: {}", stats.app_keys);
    println!("    Total keys: {}", 
             stats.debug_keys + stats.retail_keys + stats.app_keys +
             stats.iso_spu_keys + stats.lv1_keys + stats.lv2_keys);

    // Example: Add a custom key
    let custom_key = KeyEntry {
        key_type: KeyType::App,
        key: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                  0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10],
        iv: Some(vec![0u8; 16]),
        description: "Example application key".to_string(),
    };

    crypto.add_key(custom_key);
    println!("\n  ✓ Custom key added");

    // Demonstrate AES validation
    println!("\n  AES encryption validation:");
    let test_data = vec![0u8; 16];
    let test_key = vec![0u8; 16];
    let test_iv = vec![0u8; 16];

    match crypto.encrypt_aes(&test_data, &test_key, &test_iv) {
        Ok(encrypted) => {
            println!("    ✓ Encryption validation passed");
            println!("    Output size: {} bytes", encrypted.len());
        }
        Err(e) => {
            println!("    ✗ Encryption failed: {}", e);
        }
    }

    println!();
    Ok(())
}

/// Create a minimal valid ELF header for demonstration
fn create_minimal_elf() -> Vec<u8> {
    let mut elf = Vec::new();

    // ELF header (64 bytes)
    elf.extend_from_slice(&[0x7F, b'E', b'L', b'F']); // Magic
    elf.push(2); // 64-bit
    elf.push(2); // Big-endian
    elf.push(1); // ELF version
    elf.extend_from_slice(&[0u8; 9]); // Padding

    // e_type, e_machine, e_version
    elf.extend_from_slice(&[0, 2]); // ET_EXEC
    elf.extend_from_slice(&[0, 0x15]); // EM_PPC64
    elf.extend_from_slice(&[0, 0, 0, 1]); // version 1

    // e_entry (8 bytes) - entry point at 0x10000
    elf.extend_from_slice(&[0, 0, 0, 0, 0, 1, 0, 0]);

    // e_phoff (8 bytes) - program header at offset 64
    elf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 64]);

    // e_shoff (8 bytes) - no section headers
    elf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0]);

    // e_flags, e_ehsize, e_phentsize, e_phnum
    elf.extend_from_slice(&[0, 0, 0, 0]); // flags
    elf.extend_from_slice(&[0, 64]); // ehsize
    elf.extend_from_slice(&[0, 56]); // phentsize
    elf.extend_from_slice(&[0, 0]); // phnum

    // e_shentsize, e_shnum, e_shstrndx
    elf.extend_from_slice(&[0, 64]); // shentsize
    elf.extend_from_slice(&[0, 0]); // shnum
    elf.extend_from_slice(&[0, 0]); // shstrndx

    elf
}
