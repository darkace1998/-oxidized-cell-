//! Mount point management

use std::collections::HashMap;
use std::path::PathBuf;
use parking_lot::RwLock;

/// Common PS3 device mount points
pub mod devices {
    /// Internal hard disk
    pub const DEV_HDD0: &str = "/dev_hdd0";
    /// Removable hard disk
    pub const DEV_HDD1: &str = "/dev_hdd1";
    /// Blu-ray disc drive
    pub const DEV_BDVD: &str = "/dev_bdvd";
    /// USB device 0
    pub const DEV_USB000: &str = "/dev_usb000";
    /// USB device 1
    pub const DEV_USB001: &str = "/dev_usb001";
    /// Flash memory
    pub const DEV_FLASH: &str = "/dev_flash";
    /// Flash memory 2
    pub const DEV_FLASH2: &str = "/dev_flash2";
    /// Flash memory 3
    pub const DEV_FLASH3: &str = "/dev_flash3";
    /// Host root (special for development)
    pub const HOST_ROOT: &str = "/host_root";
    /// Application home
    pub const APP_HOME: &str = "/app_home";
}

/// Virtual file system
pub struct VirtualFileSystem {
    /// Mount points (virtual path -> host path)
    mounts: RwLock<HashMap<String, PathBuf>>,
}

impl VirtualFileSystem {
    /// Create a new VFS
    pub fn new() -> Self {
        Self {
            mounts: RwLock::new(HashMap::new()),
        }
    }

    /// Mount a device
    pub fn mount(&self, virtual_path: &str, host_path: PathBuf) {
        let mut mounts = self.mounts.write();
        mounts.insert(virtual_path.to_string(), host_path);
    }

    /// Unmount a device
    pub fn unmount(&self, virtual_path: &str) {
        let mut mounts = self.mounts.write();
        mounts.remove(virtual_path);
    }

    /// Resolve a virtual path to a host path
    pub fn resolve(&self, virtual_path: &str) -> Option<PathBuf> {
        let mounts = self.mounts.read();
        
        for (mount_point, host_path) in mounts.iter() {
            if virtual_path.starts_with(mount_point) {
                let relative = virtual_path.strip_prefix(mount_point).unwrap_or("");
                let relative = relative.trim_start_matches('/');
                return Some(host_path.join(relative));
            }
        }
        
        None
    }

    /// List mount points
    pub fn list_mounts(&self) -> Vec<(String, PathBuf)> {
        let mounts = self.mounts.read();
        mounts.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Check if a virtual path is mounted
    pub fn is_mounted(&self, virtual_path: &str) -> bool {
        let mounts = self.mounts.read();
        mounts.iter().any(|(mount_point, _)| virtual_path.starts_with(mount_point))
    }

    /// Get the mount point for a virtual path
    pub fn get_mount_point(&self, virtual_path: &str) -> Option<String> {
        let mounts = self.mounts.read();
        for (mount_point, _) in mounts.iter() {
            if virtual_path.starts_with(mount_point) {
                return Some(mount_point.clone());
            }
        }
        None
    }

    /// Initialize common PS3 directories on a mounted device
    pub fn init_device_directories(&self, mount_point: &str) -> std::io::Result<()> {
        let mounts = self.mounts.read();
        if let Some(host_path) = mounts.get(mount_point) {
            // Create common directories based on device type
            match mount_point {
                devices::DEV_HDD0 => {
                    let dirs = ["game", "savedata", "photo", "music", "video", "tmp", "vsh"];
                    for dir in &dirs {
                        let path = host_path.join(dir);
                        if !path.exists() {
                            std::fs::create_dir_all(&path)?;
                            tracing::debug!("Created directory: {:?}", path);
                        }
                    }
                }
                devices::DEV_FLASH | devices::DEV_FLASH2 | devices::DEV_FLASH3 => {
                    let dirs = ["sys", "vsh"];
                    for dir in &dirs {
                        let path = host_path.join(dir);
                        if !path.exists() {
                            std::fs::create_dir_all(&path)?;
                            tracing::debug!("Created directory: {:?}", path);
                        }
                    }
                }
                _ => {
                    // For other devices, just ensure the mount point exists
                    if !host_path.exists() {
                        std::fs::create_dir_all(host_path)?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfs_mount() {
        let vfs = VirtualFileSystem::new();
        
        vfs.mount("/dev_hdd0", PathBuf::from("/tmp/dev_hdd0"));
        
        let resolved = vfs.resolve("/dev_hdd0/game/test.elf");
        assert!(resolved.is_some());
        assert!(resolved.unwrap().to_string_lossy().contains("test.elf"));
    }

    #[test]
    fn test_vfs_device_check() {
        let vfs = VirtualFileSystem::new();
        
        // Not mounted yet
        assert!(!vfs.is_mounted("/dev_hdd0/game/test.elf"));
        
        // Mount device
        vfs.mount(devices::DEV_HDD0, PathBuf::from("/tmp/dev_hdd0"));
        
        // Now it's mounted
        assert!(vfs.is_mounted("/dev_hdd0/game/test.elf"));
        assert_eq!(vfs.get_mount_point("/dev_hdd0/game/test.elf"), Some(devices::DEV_HDD0.to_string()));
    }

    #[test]
    fn test_vfs_multiple_devices() {
        let vfs = VirtualFileSystem::new();
        
        vfs.mount(devices::DEV_HDD0, PathBuf::from("/tmp/dev_hdd0"));
        vfs.mount(devices::DEV_BDVD, PathBuf::from("/tmp/dev_bdvd"));
        vfs.mount(devices::DEV_USB000, PathBuf::from("/tmp/dev_usb000"));
        
        let mounts = vfs.list_mounts();
        assert_eq!(mounts.len(), 3);
        
        // Test resolution for each device
        assert!(vfs.resolve("/dev_hdd0/game/test.elf").is_some());
        assert!(vfs.resolve("/dev_bdvd/PS3_GAME/USRDIR/EBOOT.BIN").is_some());
        assert!(vfs.resolve("/dev_usb000/data.bin").is_some());
    }

    #[test]
    fn test_vfs_unmount() {
        let vfs = VirtualFileSystem::new();
        
        vfs.mount(devices::DEV_HDD0, PathBuf::from("/tmp/dev_hdd0"));
        assert!(vfs.is_mounted("/dev_hdd0/game/test.elf"));
        
        vfs.unmount(devices::DEV_HDD0);
        assert!(!vfs.is_mounted("/dev_hdd0/game/test.elf"));
    }
}
