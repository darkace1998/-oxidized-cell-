//! Game disc image management
//!
//! Handles mounting and managing PS3 game disc images (ISO files)

use crate::devices::bdvd::BdvdDevice;
use crate::formats::iso::{IsoReader, IsoVolume};
use crate::VirtualFileSystem;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

/// Disc image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscFormat {
    /// ISO 9660 image
    Iso,
    /// Folder structure (extracted disc)
    Folder,
}

/// Disc image manager
pub struct DiscManager {
    /// BDVD device
    bdvd: Arc<RwLock<BdvdDevice>>,
    /// Currently mounted disc
    current_disc: RwLock<Option<DiscInfo>>,
}

/// Information about a mounted disc
#[derive(Debug, Clone)]
pub struct DiscInfo {
    /// Path to the disc image or folder
    pub path: PathBuf,
    /// Disc format
    pub format: DiscFormat,
    /// Volume information (for ISO)
    pub volume: Option<IsoVolume>,
    /// Game title (from PARAM.SFO if available)
    pub title: Option<String>,
    /// Game ID (from directory structure)
    pub game_id: Option<String>,
}

impl DiscManager {
    /// Create a new disc manager
    pub fn new() -> Self {
        Self {
            bdvd: Arc::new(RwLock::new(BdvdDevice::new())),
            current_disc: RwLock::new(None),
        }
    }

    /// Mount a disc image
    pub fn mount_disc(&self, vfs: &VirtualFileSystem, disc_path: PathBuf) -> Result<(), String> {
        // Determine disc format
        let format = if disc_path.is_dir() {
            DiscFormat::Folder
        } else if disc_path.extension().and_then(|s| s.to_str()) == Some("iso") {
            DiscFormat::Iso
        } else {
            return Err(format!("Unsupported disc format: {:?}", disc_path));
        };

        // Parse ISO if needed
        let volume = if format == DiscFormat::Iso {
            let mut iso_reader = IsoReader::new(disc_path.clone());
            match iso_reader.open() {
                Ok(_) => iso_reader.volume().cloned(),
                Err(e) => {
                    tracing::warn!("Failed to parse ISO volume: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Extract game information
        let (title, game_id) = self.extract_game_info(&disc_path, format);

        // Mount the disc to BDVD device
        let mut bdvd = self.bdvd.write();
        bdvd.mount(disc_path.clone())?;

        // Update VFS mount
        vfs.mount("/dev_bdvd", disc_path.clone());

        // Store disc info
        let disc_info = DiscInfo {
            path: disc_path,
            format,
            volume,
            title,
            game_id,
        };

        *self.current_disc.write() = Some(disc_info.clone());

        tracing::info!(
            "Mounted disc: format={:?}, title={:?}, game_id={:?}",
            disc_info.format,
            disc_info.title,
            disc_info.game_id
        );

        Ok(())
    }

    /// Unmount the current disc
    pub fn unmount_disc(&self, vfs: &VirtualFileSystem) {
        let mut bdvd = self.bdvd.write();
        bdvd.unmount();

        vfs.unmount("/dev_bdvd");
        *self.current_disc.write() = None;

        tracing::info!("Disc unmounted");
    }

    /// Check if a disc is mounted
    pub fn is_disc_mounted(&self) -> bool {
        self.current_disc.read().is_some()
    }

    /// Get information about the current disc
    pub fn disc_info(&self) -> Option<DiscInfo> {
        self.current_disc.read().clone()
    }

    /// Extract game information from disc structure
    fn extract_game_info(&self, disc_path: &PathBuf, format: DiscFormat) -> (Option<String>, Option<String>) {
        // For folder format, look for PS3_GAME directory
        if format == DiscFormat::Folder {
            let ps3_game_path = disc_path.join("PS3_GAME");
            if ps3_game_path.exists() {
                // Try to find PARAM.SFO
                let param_sfo_path = ps3_game_path.join("PARAM.SFO");
                if param_sfo_path.exists() {
                    // TODO: Parse PARAM.SFO for title and game ID
                    // For now, just extract game ID from path if available
                    let game_id = disc_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string());
                    return (None, game_id);
                }
            }
        }

        // For ISO format, we would need to parse the ISO to find PARAM.SFO
        // This is more complex and would require full ISO directory parsing

        (None, None)
    }

    /// Verify disc integrity (basic check)
    pub fn verify_disc(&self) -> Result<bool, String> {
        let disc_info = self.current_disc.read();
        let disc_info = disc_info.as_ref().ok_or("No disc mounted")?;

        match disc_info.format {
            DiscFormat::Folder => {
                // Check for PS3_GAME directory
                let ps3_game_path = disc_info.path.join("PS3_GAME");
                Ok(ps3_game_path.exists())
            }
            DiscFormat::Iso => {
                // Check if ISO file exists and has valid volume
                Ok(disc_info.path.exists() && disc_info.volume.is_some())
            }
        }
    }
}

impl Default for DiscManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disc_manager_creation() {
        let manager = DiscManager::new();
        assert!(!manager.is_disc_mounted());
        assert!(manager.disc_info().is_none());
    }

    #[test]
    fn test_disc_format_detection() {
        // This would require actual test files, so we just test the structure
        let manager = DiscManager::new();
        assert!(!manager.is_disc_mounted());
    }
}
