//! Mount point management

use std::collections::HashMap;
use std::path::PathBuf;

use oc_core::config::PathConfig;
use parking_lot::RwLock;

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

    /// Mount the common PS3 devices using the provided paths.
    pub fn mount_defaults(&self, paths: &PathConfig) {
        self.mount("/dev_hdd0", paths.dev_hdd0.clone());
        self.mount("/dev_hdd1", paths.dev_hdd1.clone());
        self.mount("/dev_bdvd", paths.games.clone());
        self.mount("/dev_flash", paths.dev_flash.clone());

        // Provide a couple of generic USB mount points rooted under /dev_hdd1
        self.mount("/dev_usb000", paths.dev_hdd1.join("usb000"));
        self.mount("/dev_usb001", paths.dev_hdd1.join("usb001"));
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

    /// Check whether a virtual path has been mounted.
    pub fn is_mounted(&self, virtual_path: &str) -> bool {
        self.mounts.read().contains_key(virtual_path)
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
    fn test_default_mounts() {
        let vfs = VirtualFileSystem::new();
        let paths = PathConfig::default();
        vfs.mount_defaults(&paths);

        assert!(vfs.is_mounted("/dev_hdd0"));
        assert!(vfs.is_mounted("/dev_usb000"));

        let resolved = vfs.resolve("/dev_usb000/demo.bin").unwrap();
        assert!(resolved.to_string_lossy().contains("usb000"));
    }
}
