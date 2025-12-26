//! Test ISO loading functionality
//!
//! This example tests loading an ISO file to debug the emulator's ISO parsing.

use oc_vfs::{IsoReader, IsoDirectoryEntry};
use std::path::PathBuf;

fn main() {
    // Set up logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Path to the ISO file
    let iso_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("target/Adventure Time - Explore the Dungeon Because I Don't Know! (Europe) (En,Fr,De,Es,It).dec.iso")
        });

    println!("========================================");
    println!("Testing ISO Loading");
    println!("========================================");
    println!("ISO Path: {:?}", iso_path);
    println!();

    // Check if file exists
    if !iso_path.exists() {
        eprintln!("ERROR: ISO file not found: {:?}", iso_path);
        std::process::exit(1);
    }

    // Get file size
    if let Ok(metadata) = std::fs::metadata(&iso_path) {
        println!("File size: {} bytes ({:.2} GB)", 
            metadata.len(),
            metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0));
    }
    println!();

    // Open the ISO
    println!("Opening ISO...");
    let mut iso_reader = IsoReader::new(iso_path.clone());
    
    match iso_reader.open() {
        Ok(()) => {
            println!("✓ ISO opened successfully");
            
            // Print volume info
            if let Some(volume) = iso_reader.volume() {
                println!();
                println!("Volume Information:");
                println!("  Volume ID:     '{}'", volume.volume_id);
                println!("  System ID:     '{}'", volume.system_id);
                println!("  Volume Size:   {} blocks", volume.volume_size);
                println!("  Block Size:    {} bytes", volume.block_size);
                println!("  Total Size:    {} bytes ({:.2} GB)", 
                    volume.volume_size_bytes(),
                    volume.volume_size_bytes() as f64 / (1024.0 * 1024.0 * 1024.0));
                println!("  Root Dir LBA:  {}", volume.root_dir_lba);
                println!("  Root Dir Size: {} bytes", volume.root_dir_size);
            }
            println!();

            // List root directory
            println!("Root Directory Contents:");
            list_directory(&iso_reader, "/", 0);
            println!();

            // Try to find PS3_GAME
            println!("Looking for PS3_GAME directory...");
            match iso_reader.list_directory("/PS3_GAME") {
                Ok(entries) => {
                    println!("✓ Found /PS3_GAME:");
                    for entry in &entries {
                        let entry_type = if entry.is_directory { "DIR " } else { "FILE" };
                        println!("    [{entry_type}] {} ({} bytes)", entry.name, entry.size);
                    }
                    println!();

                    // Try to find USRDIR
                    println!("Looking for USRDIR...");
                    match iso_reader.list_directory("/PS3_GAME/USRDIR") {
                        Ok(usrdir_entries) => {
                            println!("✓ Found /PS3_GAME/USRDIR:");
                            for entry in usrdir_entries.iter().take(20) {
                                let entry_type = if entry.is_directory { "DIR " } else { "FILE" };
                                println!("    [{entry_type}] {} ({} bytes)", entry.name, entry.size);
                            }
                            if usrdir_entries.len() > 20 {
                                println!("    ... and {} more files", usrdir_entries.len() - 20);
                            }
                            println!();

                            // Try to read EBOOT.BIN
                            println!("Attempting to read EBOOT.BIN...");
                            match iso_reader.read_file("/PS3_GAME/USRDIR/EBOOT.BIN") {
                                Ok(data) => {
                                    println!("✓ Successfully read EBOOT.BIN ({} bytes)", data.len());
                                    
                                    // Show first bytes to identify format
                                    if data.len() >= 4 {
                                        let magic_hex: String = data[0..16.min(data.len())]
                                            .iter()
                                            .map(|b| format!("{:02X}", b))
                                            .collect::<Vec<_>>()
                                            .join(" ");
                                        println!("  Magic bytes: {}", magic_hex);
                                        
                                        if data[0..4] == [0x53, 0x43, 0x45, 0x00] {
                                            println!("  → This is a SELF file (encrypted PS3 executable)");
                                        } else if data[0..4] == [0x7F, b'E', b'L', b'F'] {
                                            println!("  → This is a plain ELF file");
                                        } else {
                                            println!("  → Unknown format");
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Failed to read EBOOT.BIN: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ USRDIR not found: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("✗ PS3_GAME not found: {}", e);
                    println!("  This may not be a PS3 game disc.");
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to open ISO: {}", e);
            std::process::exit(1);
        }
    }

    println!();
    println!("========================================");
    println!("ISO Test Complete");
    println!("========================================");
}

fn list_directory(reader: &IsoReader, path: &str, depth: usize) {
    let indent = "  ".repeat(depth + 1);
    match reader.list_directory(path) {
        Ok(entries) => {
            for entry in entries.iter().take(15) {
                let entry_type = if entry.is_directory { "DIR " } else { "FILE" };
                println!("{indent}[{entry_type}] {} ({} bytes)", entry.name, entry.size);
            }
            if entries.len() > 15 {
                println!("{indent}... and {} more entries", entries.len() - 15);
            }
        }
        Err(e) => {
            println!("{indent}Error listing: {}", e);
        }
    }
}
