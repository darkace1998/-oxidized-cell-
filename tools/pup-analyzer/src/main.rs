//! PS3 Firmware PUP file analyzer
//!
//! This tool analyzes PS3 firmware update (PUP) files and reports any issues found.

use std::env;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use oc_vfs::formats::pup::{PupFile, PupEntryType};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-pup-file> [output-report.md]", args[0]);
        std::process::exit(1);
    }

    let pup_path = &args[1];
    let report_path = args.get(2).map(|s| s.as_str()).unwrap_or("FIRMWARE_TEST_REPORT.md");

    println!("Analyzing PS3 firmware file: {}", pup_path);
    
    // Open the PUP file
    let file = match File::open(pup_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    let mut reader = BufReader::new(file);

    // Parse the PUP file
    let pup = match PupFile::parse(&mut reader) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing PUP file: {}", e);
            std::process::exit(1);
        }
    };

    // Print info to console
    pup.print_info();

    // Validate the PUP file
    println!("\n=== Validation Results ===");
    let issues = match pup.validate() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Error during validation: {}", e);
            std::process::exit(1);
        }
    };

    if issues.is_empty() {
        println!("✓ No structural issues found!");
    } else {
        println!("✗ Found {} issue(s):", issues.len());
        for issue in &issues {
            println!("  - {}", issue);
        }
    }

    // Perform deeper content analysis
    println!("\n=== Content Analysis ===");
    let content_issues = analyze_content(&pup, &mut reader);
    
    for issue in &content_issues {
        println!("  - {}", issue);
    }

    if content_issues.is_empty() {
        println!("✓ No content issues found!");
    }

    // Generate report
    let all_issues: Vec<String> = issues.into_iter().chain(content_issues.clone()).collect();
    let report = generate_report(&pup, pup_path, &all_issues, &content_issues);
    
    match std::fs::write(report_path, report) {
        Ok(_) => println!("\nReport written to: {}", report_path),
        Err(e) => eprintln!("Error writing report: {}", e),
    }
}

fn analyze_content(pup: &PupFile, reader: &mut BufReader<File>) -> Vec<String> {
    let mut issues = Vec::new();

    // Check version info entry
    if let Some(version_entry) = pup.get_entry(0x100) {
        if version_entry.size < 4 {
            issues.push(format!(
                "Version entry has unusually small size: {} bytes (expected at least 4)",
                version_entry.size
            ));
        }
        
        // Try to read version data
        match reader.seek(SeekFrom::Start(version_entry.offset)) {
            Ok(_) => {
                let mut buf = vec![0u8; version_entry.size as usize];
                if reader.read_exact(&mut buf).is_ok() {
                    // Check if version string is ASCII/UTF-8
                    if let Ok(version_str) = std::str::from_utf8(&buf) {
                        println!("  Version string: {:?}", version_str);
                    } else {
                        issues.push("Version entry contains non-UTF-8 data".to_string());
                    }
                }
            }
            Err(e) => issues.push(format!("Failed to seek to version entry: {}", e)),
        }
    }

    // Check for suspiciously small critical entries
    for entry in &pup.entries {
        let entry_type = PupEntryType::from(entry.entry_id);
        match entry_type {
            PupEntryType::CoreOs | PupEntryType::Kernel => {
                if entry.size < 1024 * 1024 {
                    issues.push(format!(
                        "{} entry is suspiciously small: {} bytes (expected > 1MB)",
                        entry_type.name(),
                        entry.size
                    ));
                }
            }
            PupEntryType::CoreOsLoader => {
                if entry.size < 256 {
                    issues.push(format!(
                        "{} entry is suspiciously small: {} bytes (expected > 256 bytes)",
                        entry_type.name(),
                        entry.size
                    ));
                }
            }
            _ => {}
        }
    }

    // Check for SELF/ELF headers in binary entries
    let binary_entries = vec![0x200, 0x300, 0x501, 0x601]; // CoreOS, Kernel, SPU Module, SPU Kernel
    for &entry_id in &binary_entries {
        if let Some(entry) = pup.get_entry(entry_id) {
            if entry.size >= 4 {
                match reader.seek(SeekFrom::Start(entry.offset)) {
                    Ok(_) => {
                        let mut magic = [0u8; 4];
                        if reader.read_exact(&mut magic).is_ok() {
                            let entry_type = PupEntryType::from(entry_id);
                            // Check for common file signatures
                            if &magic == b"SCE\0" {
                                println!("  {} has SELF signature (SCE)", entry_type.name());
                            } else if &magic == b"\x7fELF" {
                                println!("  {} has ELF signature", entry_type.name());
                            } else {
                                println!(
                                    "  {} has unknown signature: {:02X} {:02X} {:02X} {:02X}",
                                    entry_type.name(),
                                    magic[0], magic[1], magic[2], magic[3]
                                );
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }

    issues
}

fn generate_report(pup: &PupFile, file_path: &str, all_issues: &[String], content_issues: &[String]) -> String {
    let mut report = String::new();
    
    report.push_str("# PS3 Firmware Analysis Report\n\n");
    report.push_str(&format!("**File:** `{}`\n\n", file_path));
    report.push_str(&format!("**Analysis Date:** {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    report.push_str("## File Information\n\n");
    report.push_str(&format!("- **Package Version:** `0x{:016X}`\n", pup.header.package_version));
    report.push_str(&format!("- **Image Version:** `0x{:016X}`\n", pup.header.image_version));
    report.push_str(&format!("- **File Count:** {}\n", pup.header.file_count));
    report.push_str(&format!("- **Header Size:** {} bytes (0x{:X})\n", pup.header.header_size, pup.header.header_size));
    report.push_str(&format!("- **Data Size:** {} bytes (0x{:X})\n", pup.header.data_size, pup.header.data_size));
    report.push_str(&format!("- **Total Size:** {} bytes ({:.2} MB)\n\n", 
        pup.header.header_size + pup.header.data_size,
        (pup.header.header_size + pup.header.data_size) as f64 / (1024.0 * 1024.0)
    ));

    report.push_str("## File Entries\n\n");
    report.push_str("| # | Entry ID | Type | Offset | Size | Size (MB) |\n");
    report.push_str("|---|----------|------|--------|------|----------|\n");
    
    for (i, entry) in pup.entries.iter().enumerate() {
        let entry_type = PupEntryType::from(entry.entry_id);
        report.push_str(&format!(
            "| {} | 0x{:03X} | {} | 0x{:08X} | {} | {:.2} |\n",
            i,
            entry.entry_id,
            entry_type.name(),
            entry.offset,
            entry.size,
            entry.size as f64 / (1024.0 * 1024.0)
        ));
    }
    
    report.push_str("\n## Entry Type Summary\n\n");
    
    // Count entry types
    let mut type_counts = std::collections::HashMap::new();
    let mut type_sizes = std::collections::HashMap::new();
    
    for entry in &pup.entries {
        let entry_type = PupEntryType::from(entry.entry_id);
        let name = entry_type.name().to_string();
        *type_counts.entry(name.clone()).or_insert(0) += 1;
        *type_sizes.entry(name).or_insert(0u64) += entry.size;
    }
    
    report.push_str("| Type | Count | Total Size (MB) |\n");
    report.push_str("|------|-------|-----------------|\n");
    
    for (type_name, count) in &type_counts {
        let size = type_sizes.get(type_name).unwrap_or(&0);
        report.push_str(&format!(
            "| {} | {} | {:.2} |\n",
            type_name,
            count,
            *size as f64 / (1024.0 * 1024.0)
        ));
    }

    report.push_str("\n## Validation Results\n\n");
    
    let structural_issues: Vec<&String> = all_issues.iter()
        .filter(|i| !content_issues.contains(i))
        .collect();
    
    if structural_issues.is_empty() {
        report.push_str("### Structural Validation\n\n");
        report.push_str("✅ **No structural issues found!**\n\n");
        report.push_str("The firmware file structure appears valid:\n");
        report.push_str("- All entries have valid offsets\n");
        report.push_str("- No overlapping entries detected\n");
        report.push_str("- All entries are within file bounds\n");
        report.push_str("- All entries have non-zero size\n\n");
    } else {
        report.push_str("### Structural Validation\n\n");
        report.push_str(&format!("⚠️ **Found {} structural issue(s):**\n\n", structural_issues.len()));
        for issue in structural_issues {
            report.push_str(&format!("- {}\n", issue));
        }
        report.push_str("\n");
    }

    if !content_issues.is_empty() {
        report.push_str("### Content Validation\n\n");
        report.push_str(&format!("⚠️ **Found {} content issue(s):**\n\n", content_issues.len()));
        for issue in content_issues {
            report.push_str(&format!("- {}\n", issue));
        }
        report.push_str("\n");
    } else {
        report.push_str("### Content Validation\n\n");
        report.push_str("✅ **No content issues found!**\n\n");
    }

    report.push_str("## Analysis Summary\n\n");
    
    // Check for expected components
    let has_coreos = pup.entries.iter().any(|e| PupEntryType::from(e.entry_id) == PupEntryType::CoreOs);
    let has_kernel = pup.entries.iter().any(|e| PupEntryType::from(e.entry_id) == PupEntryType::Kernel);
    let has_version = pup.entries.iter().any(|e| PupEntryType::from(e.entry_id) == PupEntryType::Version);
    
    report.push_str("### Component Checklist\n\n");
    report.push_str(&format!("- [{}] CoreOS present\n", if has_coreos { "x" } else { " " }));
    report.push_str(&format!("- [{}] Kernel present\n", if has_kernel { "x" } else { " " }));
    report.push_str(&format!("- [{}] Version info present\n", if has_version { "x" } else { " " }));
    
    if !has_coreos {
        report.push_str("\n⚠️ **WARNING:** CoreOS component not found. This is unusual for a PS3 firmware.\n");
    }
    if !has_kernel {
        report.push_str("\n⚠️ **WARNING:** Kernel component not found. This is unusual for a PS3 firmware.\n");
    }

    report.push_str("\n## Conclusion\n\n");
    
    if all_issues.is_empty() && has_coreos && has_kernel && has_version {
        report.push_str("✅ The PS3 firmware file appears to be **valid and complete**. ");
        report.push_str("No major structural or content issues were detected.\n");
    } else if !all_issues.is_empty() {
        let critical_issues = content_issues.iter()
            .filter(|i| i.contains("suspiciously small"))
            .count();
        
        if critical_issues > 0 {
            report.push_str("⚠️ The firmware file has **potential issues** that should be reviewed. ");
            report.push_str("Some critical components appear to have unusual sizes.\n");
        } else {
            report.push_str("⚠️ The firmware file has **minor issues** that were detected. ");
            report.push_str("These may not prevent proper extraction or installation.\n");
        }
    } else {
        report.push_str("⚠️ The firmware file structure is valid but **missing expected components**. ");
        report.push_str("This may indicate an incomplete or specialized firmware package.\n");
    }

    report
}
