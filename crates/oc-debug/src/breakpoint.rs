//! Breakpoint management for debugging

use std::collections::HashMap;

/// Breakpoint type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    /// Execution breakpoint (break when PC reaches address)
    Execution,
    /// Read breakpoint (break when memory is read)
    Read,
    /// Write breakpoint (break when memory is written)
    Write,
    /// Access breakpoint (break on any memory access)
    Access,
}

/// A single breakpoint
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Unique breakpoint ID
    pub id: u32,
    /// Address of the breakpoint
    pub address: u64,
    /// Breakpoint type
    pub bp_type: BreakpointType,
    /// Whether the breakpoint is enabled
    pub enabled: bool,
    /// Hit count (number of times breakpoint was triggered)
    pub hit_count: u64,
    /// Optional condition expression
    pub condition: Option<String>,
    /// Optional description/label
    pub label: Option<String>,
}

impl Breakpoint {
    /// Create a new execution breakpoint
    pub fn new_execution(id: u32, address: u64) -> Self {
        Self {
            id,
            address,
            bp_type: BreakpointType::Execution,
            enabled: true,
            hit_count: 0,
            condition: None,
            label: None,
        }
    }

    /// Create a new memory read breakpoint
    pub fn new_read(id: u32, address: u64) -> Self {
        Self {
            id,
            address,
            bp_type: BreakpointType::Read,
            enabled: true,
            hit_count: 0,
            condition: None,
            label: None,
        }
    }

    /// Create a new memory write breakpoint
    pub fn new_write(id: u32, address: u64) -> Self {
        Self {
            id,
            address,
            bp_type: BreakpointType::Write,
            enabled: true,
            hit_count: 0,
            condition: None,
            label: None,
        }
    }

    /// Set the breakpoint label
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Record a hit on this breakpoint
    pub fn record_hit(&mut self) {
        self.hit_count += 1;
    }
}

/// Breakpoint manager
#[derive(Debug, Default)]
pub struct BreakpointManager {
    /// All breakpoints indexed by ID
    breakpoints: HashMap<u32, Breakpoint>,
    /// Execution breakpoints indexed by address for fast lookup
    execution_bp: HashMap<u64, u32>,
    /// Read breakpoints indexed by address
    read_bp: HashMap<u64, u32>,
    /// Write breakpoints indexed by address
    write_bp: HashMap<u64, u32>,
    /// Next breakpoint ID
    next_id: u32,
}

impl BreakpointManager {
    /// Create a new breakpoint manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an execution breakpoint at the given address
    pub fn add_execution_breakpoint(&mut self, address: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let bp = Breakpoint::new_execution(id, address);
        self.execution_bp.insert(address, id);
        self.breakpoints.insert(id, bp);
        
        tracing::debug!("Added execution breakpoint {} at 0x{:016x}", id, address);
        id
    }

    /// Add a read breakpoint at the given address
    pub fn add_read_breakpoint(&mut self, address: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let bp = Breakpoint::new_read(id, address);
        self.read_bp.insert(address, id);
        self.breakpoints.insert(id, bp);
        
        tracing::debug!("Added read breakpoint {} at 0x{:016x}", id, address);
        id
    }

    /// Add a write breakpoint at the given address
    pub fn add_write_breakpoint(&mut self, address: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let bp = Breakpoint::new_write(id, address);
        self.write_bp.insert(address, id);
        self.breakpoints.insert(id, bp);
        
        tracing::debug!("Added write breakpoint {} at 0x{:016x}", id, address);
        id
    }

    /// Remove a breakpoint by ID
    pub fn remove_breakpoint(&mut self, id: u32) -> Option<Breakpoint> {
        if let Some(bp) = self.breakpoints.remove(&id) {
            match bp.bp_type {
                BreakpointType::Execution => {
                    self.execution_bp.remove(&bp.address);
                }
                BreakpointType::Read => {
                    self.read_bp.remove(&bp.address);
                }
                BreakpointType::Write | BreakpointType::Access => {
                    self.write_bp.remove(&bp.address);
                }
            }
            tracing::debug!("Removed breakpoint {} at 0x{:016x}", id, bp.address);
            Some(bp)
        } else {
            None
        }
    }

    /// Enable a breakpoint
    pub fn enable_breakpoint(&mut self, id: u32) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a breakpoint
    pub fn disable_breakpoint(&mut self, id: u32) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = false;
            true
        } else {
            false
        }
    }

    /// Check if there's an execution breakpoint at the given address
    pub fn check_execution(&mut self, address: u64) -> Option<&mut Breakpoint> {
        if let Some(&id) = self.execution_bp.get(&address) {
            if let Some(bp) = self.breakpoints.get_mut(&id) {
                if bp.enabled {
                    bp.record_hit();
                    return Some(bp);
                }
            }
        }
        None
    }

    /// Check if there's a read breakpoint at the given address
    pub fn check_read(&mut self, address: u64) -> Option<&mut Breakpoint> {
        if let Some(&id) = self.read_bp.get(&address) {
            if let Some(bp) = self.breakpoints.get_mut(&id) {
                if bp.enabled {
                    bp.record_hit();
                    return Some(bp);
                }
            }
        }
        None
    }

    /// Check if there's a write breakpoint at the given address
    pub fn check_write(&mut self, address: u64) -> Option<&mut Breakpoint> {
        if let Some(&id) = self.write_bp.get(&address) {
            if let Some(bp) = self.breakpoints.get_mut(&id) {
                if bp.enabled {
                    bp.record_hit();
                    return Some(bp);
                }
            }
        }
        None
    }

    /// Get all breakpoints
    pub fn get_all(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Get a breakpoint by ID
    pub fn get(&self, id: u32) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Get the total number of breakpoints
    pub fn count(&self) -> usize {
        self.breakpoints.len()
    }

    /// Clear all breakpoints
    pub fn clear(&mut self) {
        self.breakpoints.clear();
        self.execution_bp.clear();
        self.read_bp.clear();
        self.write_bp.clear();
        tracing::debug!("Cleared all breakpoints");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let bp = Breakpoint::new_execution(0, 0x10000);
        assert_eq!(bp.id, 0);
        assert_eq!(bp.address, 0x10000);
        assert_eq!(bp.bp_type, BreakpointType::Execution);
        assert!(bp.enabled);
        assert_eq!(bp.hit_count, 0);
    }

    #[test]
    fn test_breakpoint_manager_add_remove() {
        let mut mgr = BreakpointManager::new();
        
        let id1 = mgr.add_execution_breakpoint(0x10000);
        let id2 = mgr.add_execution_breakpoint(0x10004);
        
        assert_eq!(mgr.count(), 2);
        
        mgr.remove_breakpoint(id1);
        assert_eq!(mgr.count(), 1);
        
        assert!(mgr.get(id2).is_some());
        assert!(mgr.get(id1).is_none());
    }

    #[test]
    fn test_breakpoint_check() {
        let mut mgr = BreakpointManager::new();
        
        mgr.add_execution_breakpoint(0x10000);
        
        // Should hit
        assert!(mgr.check_execution(0x10000).is_some());
        
        // Should not hit
        assert!(mgr.check_execution(0x10004).is_none());
    }

    #[test]
    fn test_breakpoint_enable_disable() {
        let mut mgr = BreakpointManager::new();
        
        let id = mgr.add_execution_breakpoint(0x10000);
        
        // Should hit when enabled
        assert!(mgr.check_execution(0x10000).is_some());
        
        // Disable
        mgr.disable_breakpoint(id);
        
        // Should not hit when disabled
        assert!(mgr.check_execution(0x10000).is_none());
        
        // Enable again
        mgr.enable_breakpoint(id);
        
        // Should hit again
        assert!(mgr.check_execution(0x10000).is_some());
    }
}
