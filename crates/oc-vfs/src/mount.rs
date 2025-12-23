//! Mount point management

use std::collections::HashMap;
use std::path::PathBuf;
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
}
