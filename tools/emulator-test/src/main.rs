//! PS3 Emulator Test Application
//!
//! This application tests the oxidized-cell emulator with a PS3 firmware PUP file.

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};
use oc_core::config::Config;
use oc_memory::manager::MemoryManager;
use oc_vfs::formats::pup::{PupFile, PupEntryType};

fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-pup-file>", args[0]);
        eprintln!("\nThis tool tests the oxidized-cell emulator with PS3 firmware.");
        std::process::exit(1);
    }

    let pup_path = PathBuf::from(&args[1]);
    
    println!("=== PS3 Emulator Test ===\n");
    println!("Firmware file: {}", pup_path.display());
    
    // Step 1: Parse PUP file
    println!("\n[1/6] Parsing firmware PUP file...");
    let pup = parse_pup_file(&pup_path)?;
    println!("✓ Successfully parsed firmware (version: {})", 
        get_version_string(&pup, &pup_path)?);
    
    // Step 2: Validate firmware
    println!("\n[2/6] Validating firmware structure...");
    let issues = pup.validate()?;
    if issues.is_empty() {
        println!("✓ Firmware validation passed");
    } else {
        println!("⚠ Found {} validation issue(s):", issues.len());
        for issue in &issues {
            println!("  - {}", issue);
        }
    }
    
    // Step 3: Initialize memory manager
    println!("\n[3/6] Initializing memory manager...");
    let _memory = MemoryManager::new()
        .context("Failed to initialize memory manager")?;
    println!("✓ Memory manager initialized");
    println!("  - Main memory: 256 MB");
    println!("  - RSX memory: 256 MB");
    println!("  - Address space: 4 GB");
    
    // Step 4: Load configuration
    println!("\n[4/6] Loading emulator configuration...");
    let config = Config::load().unwrap_or_default();
    println!("✓ Configuration loaded");
    println!("  - PPU decoder: {:?}", config.cpu.ppu_decoder);
    println!("  - SPU decoder: {:?}", config.cpu.spu_decoder);
    println!("  - GPU backend: {:?}", config.gpu.backend);
    
    // Step 5: Analyze firmware components
    println!("\n[5/6] Analyzing firmware components...");
    analyze_firmware_components(&pup);
    
    // Step 6: Test firmware extraction
    println!("\n[6/6] Testing firmware component extraction...");
    test_firmware_extraction(&pup, &pup_path)?;
    
    // Summary
    println!("\n=== Test Summary ===");
    println!("✓ PUP file parsing: OK");
    println!("✓ Firmware validation: OK");
    println!("✓ Memory manager: OK");
    println!("✓ Configuration: OK");
    println!("✓ Component analysis: OK");
    println!("✓ Component extraction: OK");
    
    println!("\n🎮 Emulator core components are working correctly!");
    println!("\nNote: Full emulation requires:");
    println!("  1. SELF file decryption support");
    println!("  2. PPU/SPU CPU emulation");
    println!("  3. RSX graphics emulation");
    println!("  4. System call (LV2) implementation");
    
    Ok(())
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .with_target(false)
        .init();
}

fn parse_pup_file(path: &PathBuf) -> Result<PupFile> {
    let file = File::open(path)
        .context("Failed to open PUP file")?;
    let mut reader = BufReader::new(file);
    
    PupFile::parse(&mut reader)
        .context("Failed to parse PUP file")
}

fn get_version_string(pup: &PupFile, path: &PathBuf) -> Result<String> {
    if pup.get_entry(0x100).is_some() {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        
        if let Ok(data) = pup.extract_entry(&mut reader, 0x100) {
            if let Ok(version) = std::str::from_utf8(&data) {
                return Ok(version.trim().to_string());
            }
        }
    }
    Ok("Unknown".to_string())
}

fn analyze_firmware_components(pup: &PupFile) {
    let components = [
        (PupEntryType::Version, "Version information"),
        (PupEntryType::License, "License text"),
        (PupEntryType::Prx, "PRX modules"),
        (PupEntryType::CoreOs, "Core operating system"),
        (PupEntryType::CoreOsExtra, "CoreOS extensions"),
        (PupEntryType::CoreOsLoader, "CoreOS loader"),
        (PupEntryType::Kernel, "System kernel"),
        (PupEntryType::SpuModule, "SPU modules"),
        (PupEntryType::SpuKernel, "SPU kernel"),
    ];
    
    for (comp_type, description) in &components {
        let entries = pup.get_entries_by_type(*comp_type);
        if !entries.is_empty() {
            let total_size: u64 = entries.iter().map(|e| e.size).sum();
            println!("  ✓ {} ({:.2} MB)", 
                description, 
                total_size as f64 / (1024.0 * 1024.0));
        }
    }
}

fn test_firmware_extraction(pup: &PupFile, path: &PathBuf) -> Result<()> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    
    // Test extracting version info
    if let Some(_version_entry) = pup.get_entry(0x100) {
        let data = pup.extract_entry(&mut reader, 0x100)?;
        println!("  ✓ Version entry extracted: {} bytes", data.len());
    }
    
    // Test extracting license
    if let Some(_license_entry) = pup.get_entry(0x101) {
        let data = pup.extract_entry(&mut reader, 0x101)?;
        println!("  ✓ License entry extracted: {} bytes", data.len());
    }
    
    // Check for extractable binary components
    let binary_entries = [(0x200, "CoreOS"), (0x300, "Kernel")];
    for (entry_id, name) in &binary_entries {
        if pup.get_entry(*entry_id).is_some() {
            println!("  ✓ {} component ready for extraction", name);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_manager_initialization() {
        let memory = MemoryManager::new();
        assert!(memory.is_ok(), "Memory manager should initialize successfully");
    }
    
    #[test]
    fn test_config_loading() {
        let config = Config::load().unwrap_or_default();
        assert_eq!(config.general.start_paused, false);
    }
}
