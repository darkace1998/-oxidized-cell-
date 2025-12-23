//! HLE module registry

use std::collections::HashMap;

/// HLE function signature
pub type HleFunction = fn(args: &[u64]) -> i64;

/// HLE module
pub struct HleModule {
    /// Module name
    pub name: String,
    /// Exported functions (NID -> function)
    pub functions: HashMap<u32, HleFunction>,
}

impl HleModule {
    /// Create a new HLE module
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register(&mut self, nid: u32, func: HleFunction) {
        self.functions.insert(nid, func);
    }

    /// Get a function by NID
    pub fn get_function(&self, nid: u32) -> Option<&HleFunction> {
        self.functions.get(&nid)
    }
}

/// Module registry
pub struct ModuleRegistry {
    modules: HashMap<String, HleModule>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        registry.register_default_modules();
        registry
    }

    /// Register default HLE modules
    fn register_default_modules(&mut self) {
        // cellGcmSys
        let mut gcm = HleModule::new("cellGcmSys");
        gcm.register(0x055BD74D, |_| 0); // cellGcmGetTiledPitchSize
        gcm.register(0x21AC3697, |_| 0); // cellGcmInit
        self.modules.insert("cellGcmSys".to_string(), gcm);

        // cellSysutil
        let mut sysutil = HleModule::new("cellSysutil");
        sysutil.register(0x0BAE8772, |_| 0); // cellSysutilCheckCallback
        sysutil.register(0x40E34A7A, |_| 0); // cellSysutilRegisterCallback
        self.modules.insert("cellSysutil".to_string(), sysutil);

        // cellPad
        let mut pad = HleModule::new("cellPad");
        pad.register(0x578E3C98, |_| 0); // cellPadInit
        pad.register(0x3733EA3C, |_| 0); // cellPadEnd
        pad.register(0x1CF98800, |_| 0); // cellPadGetData
        self.modules.insert("cellPad".to_string(), pad);
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&HleModule> {
        self.modules.get(name)
    }

    /// Register a module
    pub fn register_module(&mut self, module: HleModule) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Get a function from any module
    pub fn find_function(&self, module: &str, nid: u32) -> Option<&HleFunction> {
        self.modules.get(module)?.get_function(nid)
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry() {
        let registry = ModuleRegistry::new();
        
        let gcm = registry.get_module("cellGcmSys");
        assert!(gcm.is_some());
        
        let func = registry.find_function("cellGcmSys", 0x21AC3697);
        assert!(func.is_some());
    }
}
