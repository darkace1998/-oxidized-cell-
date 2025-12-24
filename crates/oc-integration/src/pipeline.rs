//! Game Loading Pipeline
//!
//! This module provides game discovery, scanning, and initialization
//! functionality for the oxidized-cell PS3 emulator.

use oc_core::error::{EmulatorError, LoaderError};
use oc_core::Result;
use oc_hle::ModuleRegistry;
use oc_memory::MemoryManager;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Game information extracted from PARAM.SFO
#[derive(Debug, Clone, Default)]
pub struct GameInfo {
    /// Game title
    pub title: String,
    /// Title ID (e.g., "BLUS00001")
    pub title_id: String,
    /// Game version
    pub version: String,
    /// Path to the game directory or EBOOT.BIN
    pub path: PathBuf,
    /// Category (e.g., "DG" for disc game, "HG" for HDD game)
    pub category: String,
    /// Parental level
    pub parental_level: u32,
    /// Resolution (e.g., 1=480p, 2=720p, 4=1080p)
    pub resolution: u32,
    /// Sound format
    pub sound_format: u32,
}

/// Game scanner for discovering PS3 games
pub struct GameScanner {
    /// Search directories
    search_dirs: Vec<PathBuf>,
    /// Discovered games
    games: HashMap<String, GameInfo>,
}

impl GameScanner {
    /// Create a new game scanner
    pub fn new() -> Self {
        Self {
            search_dirs: Vec::new(),
            games: HashMap::new(),
        }
    }

    /// Add a directory to search for games
    pub fn add_search_directory<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref().to_path_buf();
        if !self.search_dirs.contains(&path) {
            self.search_dirs.push(path);
        }
    }

    /// Scan all search directories for games
    pub fn scan(&mut self) -> Result<Vec<GameInfo>> {
        info!("Scanning {} directories for games", self.search_dirs.len());
        self.games.clear();

        for dir in self.search_dirs.clone() {
            if let Err(e) = self.scan_directory(&dir) {
                warn!("Failed to scan directory {:?}: {}", dir, e);
            }
        }

        info!("Found {} games", self.games.len());
        Ok(self.games.values().cloned().collect())
    }

    /// Scan a single directory for games
    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        debug!("Scanning directory: {:?}", dir);

        // Check if this directory is a PS3 game directory itself
        if self.is_game_directory(dir) {
            if let Some(game_info) = self.extract_game_info(dir)? {
                self.games.insert(game_info.title_id.clone(), game_info);
            }
            return Ok(());
        }

        // Scan subdirectories
        let entries = fs::read_dir(dir).map_err(|e| {
            EmulatorError::Loader(LoaderError::InvalidElf(format!(
                "Failed to read directory: {}",
                e
            )))
        })?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if self.is_game_directory(&path) {
                        if let Some(game_info) = self.extract_game_info(&path)? {
                            self.games.insert(game_info.title_id.clone(), game_info);
                        }
                    } else {
                        // Recursively scan subdirectories (one level deep)
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            for sub_entry in sub_entries.flatten() {
                                let sub_path = sub_entry.path();
                                if sub_path.is_dir() && self.is_game_directory(&sub_path) {
                                    if let Some(game_info) = self.extract_game_info(&sub_path)? {
                                        self.games.insert(game_info.title_id.clone(), game_info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a directory is a PS3 game directory
    fn is_game_directory(&self, dir: &Path) -> bool {
        // Check for PS3_GAME/PARAM.SFO structure (disc games)
        let param_sfo = dir.join("PS3_GAME").join("PARAM.SFO");
        if param_sfo.exists() {
            return true;
        }

        // Check for PARAM.SFO in current directory (HDD games)
        let param_sfo = dir.join("PARAM.SFO");
        if param_sfo.exists() {
            return true;
        }

        // Check for EBOOT.BIN (minimal check)
        let eboot = dir.join("PS3_GAME").join("USRDIR").join("EBOOT.BIN");
        if eboot.exists() {
            return true;
        }

        let eboot = dir.join("USRDIR").join("EBOOT.BIN");
        eboot.exists()
    }

    /// Extract game info from a game directory
    fn extract_game_info(&self, dir: &Path) -> Result<Option<GameInfo>> {
        // Find PARAM.SFO
        let param_sfo_path = if dir.join("PS3_GAME").join("PARAM.SFO").exists() {
            dir.join("PS3_GAME").join("PARAM.SFO")
        } else if dir.join("PARAM.SFO").exists() {
            dir.join("PARAM.SFO")
        } else {
            debug!("No PARAM.SFO found in {:?}", dir);
            return Ok(None);
        };

        // Parse PARAM.SFO
        let game_info = self.parse_param_sfo(&param_sfo_path, dir)?;
        Ok(Some(game_info))
    }

    /// Parse PARAM.SFO file and extract game information
    fn parse_param_sfo(&self, sfo_path: &Path, game_dir: &Path) -> Result<GameInfo> {
        debug!("Parsing PARAM.SFO: {:?}", sfo_path);

        let file = File::open(sfo_path).map_err(|e| {
            EmulatorError::Loader(LoaderError::InvalidElf(format!(
                "Failed to open PARAM.SFO: {}",
                e
            )))
        })?;
        let mut reader = BufReader::new(file);

        // Parse SFO header
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(|e| {
            EmulatorError::Loader(LoaderError::InvalidElf(format!(
                "Failed to read SFO magic: {}",
                e
            )))
        })?;

        if &magic != b"\x00PSF" {
            return Err(EmulatorError::Loader(LoaderError::InvalidElf(
                "Invalid SFO magic".to_string(),
            )));
        }

        let mut header = [0u8; 16];
        reader.read_exact(&mut header).map_err(|e| {
            EmulatorError::Loader(LoaderError::InvalidElf(format!(
                "Failed to read SFO header: {}",
                e
            )))
        })?;

        let _version = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let key_table_start = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let data_table_start = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let entries_count = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);

        let mut entries: HashMap<String, SfoValue> = HashMap::new();

        // Parse entries
        for i in 0..entries_count {
            let entry_offset = 20 + i * 16;
            reader
                .seek(std::io::SeekFrom::Start(entry_offset as u64))
                .map_err(|e| {
                    EmulatorError::Loader(LoaderError::InvalidElf(format!(
                        "Failed to seek to entry: {}",
                        e
                    )))
                })?;

            let mut entry_data = [0u8; 16];
            reader.read_exact(&mut entry_data).map_err(|e| {
                EmulatorError::Loader(LoaderError::InvalidElf(format!(
                    "Failed to read SFO entry: {}",
                    e
                )))
            })?;

            let key_offset = u16::from_le_bytes([entry_data[0], entry_data[1]]);
            let data_fmt = u16::from_le_bytes([entry_data[2], entry_data[3]]);
            let data_len =
                u32::from_le_bytes([entry_data[4], entry_data[5], entry_data[6], entry_data[7]]);
            let data_offset = u32::from_le_bytes([
                entry_data[12],
                entry_data[13],
                entry_data[14],
                entry_data[15],
            ]);

            // Read key
            reader
                .seek(std::io::SeekFrom::Start(
                    (key_table_start + key_offset as u32) as u64,
                ))
                .map_err(|e| {
                    EmulatorError::Loader(LoaderError::InvalidElf(format!(
                        "Failed to seek to key: {}",
                        e
                    )))
                })?;

            let mut key = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                if reader.read_exact(&mut byte).is_err() {
                    break;
                }
                if byte[0] == 0 {
                    break;
                }
                key.push(byte[0]);
            }
            let key = String::from_utf8_lossy(&key).to_string();

            // Read value
            reader
                .seek(std::io::SeekFrom::Start(
                    (data_table_start + data_offset) as u64,
                ))
                .map_err(|e| {
                    EmulatorError::Loader(LoaderError::InvalidElf(format!(
                        "Failed to seek to value: {}",
                        e
                    )))
                })?;

            let value = match data_fmt {
                0x0404 => {
                    let mut buf = [0u8; 4];
                    reader.read_exact(&mut buf).map_err(|e| {
                        EmulatorError::Loader(LoaderError::InvalidElf(format!(
                            "Failed to read integer value: {}",
                            e
                        )))
                    })?;
                    SfoValue::Integer(u32::from_le_bytes(buf))
                }
                0x0004 | 0x0204 => {
                    let mut buf = vec![0u8; data_len as usize];
                    reader.read_exact(&mut buf).map_err(|e| {
                        EmulatorError::Loader(LoaderError::InvalidElf(format!(
                            "Failed to read string value: {}",
                            e
                        )))
                    })?;
                    // Remove null terminator
                    while buf.last() == Some(&0) {
                        buf.pop();
                    }
                    SfoValue::String(String::from_utf8_lossy(&buf).to_string())
                }
                _ => continue,
            };

            entries.insert(key, value);
        }

        // Build GameInfo from parsed entries
        let title = entries
            .get("TITLE")
            .and_then(|v| v.as_string())
            .unwrap_or("Unknown Title")
            .to_string();
        let title_id = entries
            .get("TITLE_ID")
            .and_then(|v| v.as_string())
            .unwrap_or("UNKNOWN")
            .to_string();
        let version = entries
            .get("VERSION")
            .and_then(|v| v.as_string())
            .unwrap_or("01.00")
            .to_string();
        let category = entries
            .get("CATEGORY")
            .and_then(|v| v.as_string())
            .unwrap_or("DG")
            .to_string();
        let parental_level = entries
            .get("PARENTAL_LEVEL")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);
        let resolution = entries
            .get("RESOLUTION")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);
        let sound_format = entries
            .get("SOUND_FORMAT")
            .and_then(|v| v.as_integer())
            .unwrap_or(0);

        info!(
            "Parsed game: {} ({}) v{}",
            title, title_id, version
        );

        Ok(GameInfo {
            title,
            title_id,
            version,
            path: game_dir.to_path_buf(),
            category,
            parental_level,
            resolution,
            sound_format,
        })
    }

    /// Get discovered games
    pub fn games(&self) -> &HashMap<String, GameInfo> {
        &self.games
    }

    /// Get a game by title ID
    pub fn get_game(&self, title_id: &str) -> Option<&GameInfo> {
        self.games.get(title_id)
    }
}

impl Default for GameScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// SFO value type for internal parsing
#[derive(Debug, Clone)]
enum SfoValue {
    String(String),
    Integer(u32),
}

impl SfoValue {
    fn as_string(&self) -> Option<&str> {
        match self {
            SfoValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_integer(&self) -> Option<u32> {
        match self {
            SfoValue::Integer(v) => Some(*v),
            _ => None,
        }
    }
}

/// System module initialization state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Module is not loaded
    Unloaded,
    /// Module is loaded but not started
    Loaded,
    /// Module is started and running
    Running,
    /// Module is stopped
    Stopped,
}

/// System module information
#[derive(Debug, Clone)]
pub struct SystemModule {
    /// Module name
    pub name: String,
    /// Module state
    pub state: ModuleState,
    /// Module ID
    pub id: u32,
}

/// Game loading pipeline that coordinates all aspects of game loading
pub struct GamePipeline {
    /// HLE module registry
    module_registry: ModuleRegistry,
    /// Loaded system modules
    system_modules: HashMap<String, SystemModule>,
    /// Memory manager reference
    memory: Arc<MemoryManager>,
    /// Game scanner
    scanner: GameScanner,
    /// Next module ID
    next_module_id: u32,
}

impl GamePipeline {
    /// Create a new game pipeline
    pub fn new(memory: Arc<MemoryManager>) -> Self {
        Self {
            module_registry: ModuleRegistry::new(),
            system_modules: HashMap::new(),
            memory,
            scanner: GameScanner::new(),
            next_module_id: 1,
        }
    }

    /// Add a directory to scan for games
    pub fn add_game_directory<P: AsRef<Path>>(&mut self, path: P) {
        self.scanner.add_search_directory(path);
    }

    /// Scan for games in all configured directories
    pub fn scan_games(&mut self) -> Result<Vec<GameInfo>> {
        self.scanner.scan()
    }

    /// Get the game scanner
    pub fn scanner(&self) -> &GameScanner {
        &self.scanner
    }

    /// Get mutable reference to game scanner
    pub fn scanner_mut(&mut self) -> &mut GameScanner {
        &mut self.scanner
    }

    /// Initialize all required system modules before game start
    pub fn initialize_system_modules(&mut self) -> Result<()> {
        info!("Initializing system modules");

        // List of core modules that need to be initialized for most games
        let core_modules = [
            "cellSysutil",
            "cellGcmSys",
            "cellFs",
            "cellPad",
            "cellAudio",
            "cellSpurs",
            "cellGame",
        ];

        for module_name in &core_modules {
            self.load_module(module_name)?;
        }

        info!(
            "Initialized {} system modules",
            self.system_modules.len()
        );
        Ok(())
    }

    /// Load a specific system module
    pub fn load_module(&mut self, name: &str) -> Result<u32> {
        // Check if already loaded
        if let Some(module) = self.system_modules.get(name) {
            debug!("Module {} already loaded with ID {}", name, module.id);
            return Ok(module.id);
        }

        // Check if module exists in registry
        if self.module_registry.get_module(name).is_none() {
            return Err(EmulatorError::Loader(LoaderError::MissingModule(
                name.to_string(),
            )));
        }

        // Assign module ID and create entry
        let module_id = self.next_module_id;
        self.next_module_id += 1;

        let module = SystemModule {
            name: name.to_string(),
            state: ModuleState::Loaded,
            id: module_id,
        };

        self.system_modules.insert(name.to_string(), module);
        info!("Loaded system module: {} (ID: {})", name, module_id);

        Ok(module_id)
    }

    /// Start a loaded module
    pub fn start_module(&mut self, name: &str) -> Result<()> {
        if let Some(module) = self.system_modules.get_mut(name) {
            if module.state == ModuleState::Loaded {
                module.state = ModuleState::Running;
                debug!("Started module: {}", name);
                Ok(())
            } else {
                Err(EmulatorError::Loader(LoaderError::InvalidElf(format!(
                    "Module {} is not in loaded state",
                    name
                ))))
            }
        } else {
            Err(EmulatorError::Loader(LoaderError::MissingModule(
                name.to_string(),
            )))
        }
    }

    /// Stop a running module
    pub fn stop_module(&mut self, name: &str) -> Result<()> {
        if let Some(module) = self.system_modules.get_mut(name) {
            if module.state == ModuleState::Running {
                module.state = ModuleState::Stopped;
                debug!("Stopped module: {}", name);
                Ok(())
            } else {
                Err(EmulatorError::Loader(LoaderError::InvalidElf(format!(
                    "Module {} is not running",
                    name
                ))))
            }
        } else {
            Err(EmulatorError::Loader(LoaderError::MissingModule(
                name.to_string(),
            )))
        }
    }

    /// Unload a module
    pub fn unload_module(&mut self, name: &str) -> Result<()> {
        if let Some(module) = self.system_modules.remove(name) {
            debug!("Unloaded module: {} (ID: {})", name, module.id);
            Ok(())
        } else {
            Err(EmulatorError::Loader(LoaderError::MissingModule(
                name.to_string(),
            )))
        }
    }

    /// Start all loaded modules
    pub fn start_all_modules(&mut self) -> Result<()> {
        let module_names: Vec<String> = self.system_modules.keys().cloned().collect();
        for name in module_names {
            if let Some(module) = self.system_modules.get(&name) {
                if module.state == ModuleState::Loaded {
                    self.start_module(&name)?;
                }
            }
        }
        Ok(())
    }

    /// Set up proper memory layout for games
    ///
    /// This initializes the PS3 memory layout with proper regions:
    /// - Main memory (256 MB at 0x00000000)
    /// - User memory (256 MB at 0x20000000)
    /// - RSX mapped memory (256 MB at 0x30000000)
    /// - RSX I/O registers (1 MB at 0x40000000)
    /// - RSX local memory (256 MB at 0xC0000000)
    /// - Stack area (256 MB at 0xD0000000)
    /// - SPU local storage (at 0xE0000000)
    pub fn setup_memory_layout(&self) -> Result<MemoryLayoutInfo> {
        info!("Setting up PS3 memory layout");

        // The memory manager already initializes these regions in its constructor
        // We just need to verify and return the layout info

        let layout = MemoryLayoutInfo {
            main_memory_base: 0x0000_0000,
            main_memory_size: 0x1000_0000, // 256 MB
            user_memory_base: 0x2000_0000,
            user_memory_size: 0x1000_0000, // 256 MB
            rsx_map_base: 0x3000_0000,
            rsx_map_size: 0x1000_0000, // 256 MB
            rsx_io_base: 0x4000_0000,
            rsx_io_size: 0x0010_0000, // 1 MB
            rsx_mem_base: 0xC000_0000,
            rsx_mem_size: 0x1000_0000, // 256 MB
            stack_base: 0xD000_0000,
            stack_size: 0x1000_0000, // 256 MB
            spu_base: 0xE000_0000,
            spu_ls_size: 0x0004_0000, // 256 KB per SPU
        };

        // Initialize process data area (PDA) at a known location within main memory
        // This is where PS3 system information is stored
        let pda_addr = 0x0001_0000u32; // Early in main memory, after null page
        
        // Write some initial PDA values
        // Note: These are stub values - real values would come from the system
        self.memory.write_be32(pda_addr, 0)?; // Process ID placeholder
        self.memory.write_be32(pda_addr + 4, 0)?; // Thread ID placeholder

        debug!(
            "Memory layout configured: main=0x{:08x}-0x{:08x}, user=0x{:08x}-0x{:08x}",
            layout.main_memory_base,
            layout.main_memory_base + layout.main_memory_size,
            layout.user_memory_base,
            layout.user_memory_base + layout.user_memory_size
        );

        Ok(layout)
    }

    /// Get the HLE module registry
    pub fn module_registry(&self) -> &ModuleRegistry {
        &self.module_registry
    }

    /// Get mutable reference to HLE module registry
    pub fn module_registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.module_registry
    }

    /// Get loaded system modules
    pub fn system_modules(&self) -> &HashMap<String, SystemModule> {
        &self.system_modules
    }

    /// Get a system module by name
    pub fn get_system_module(&self, name: &str) -> Option<&SystemModule> {
        self.system_modules.get(name)
    }

    /// Call an HLE function by module name and NID
    pub fn call_hle_function(&self, module: &str, nid: u32, args: &[u64]) -> Result<i64> {
        if let Some(func) = self.module_registry.find_function(module, nid) {
            Ok(func(args))
        } else {
            warn!(
                "HLE function not found: module={}, nid=0x{:08x}",
                module, nid
            );
            Err(EmulatorError::Loader(LoaderError::MissingModule(format!(
                "{}:0x{:08x}",
                module, nid
            ))))
        }
    }
}

/// Memory layout information
#[derive(Debug, Clone)]
pub struct MemoryLayoutInfo {
    /// Main memory base address
    pub main_memory_base: u32,
    /// Main memory size
    pub main_memory_size: u32,
    /// User memory base address
    pub user_memory_base: u32,
    /// User memory size
    pub user_memory_size: u32,
    /// RSX mapped memory base
    pub rsx_map_base: u32,
    /// RSX mapped memory size
    pub rsx_map_size: u32,
    /// RSX I/O base
    pub rsx_io_base: u32,
    /// RSX I/O size
    pub rsx_io_size: u32,
    /// RSX local memory base
    pub rsx_mem_base: u32,
    /// RSX local memory size
    pub rsx_mem_size: u32,
    /// Stack base
    pub stack_base: u32,
    /// Stack size
    pub stack_size: u32,
    /// SPU base
    pub spu_base: u32,
    /// SPU local storage size
    pub spu_ls_size: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_scanner_creation() {
        let scanner = GameScanner::new();
        assert!(scanner.games().is_empty());
    }

    #[test]
    fn test_game_scanner_add_directory() {
        let mut scanner = GameScanner::new();
        scanner.add_search_directory("/tmp/games");
        scanner.add_search_directory("/tmp/games"); // Duplicate should be ignored
        assert_eq!(scanner.search_dirs.len(), 1);
    }

    #[test]
    fn test_game_info_default() {
        let info = GameInfo::default();
        assert!(info.title.is_empty());
        assert!(info.title_id.is_empty());
    }

    #[test]
    fn test_sfo_value() {
        let string_val = SfoValue::String("test".to_string());
        assert_eq!(string_val.as_string(), Some("test"));
        assert_eq!(string_val.as_integer(), None);

        let int_val = SfoValue::Integer(42);
        assert_eq!(int_val.as_string(), None);
        assert_eq!(int_val.as_integer(), Some(42));
    }

    #[test]
    fn test_memory_layout_info() {
        let layout = MemoryLayoutInfo {
            main_memory_base: 0x0000_0000,
            main_memory_size: 0x1000_0000,
            user_memory_base: 0x2000_0000,
            user_memory_size: 0x1000_0000,
            rsx_map_base: 0x3000_0000,
            rsx_map_size: 0x1000_0000,
            rsx_io_base: 0x4000_0000,
            rsx_io_size: 0x0010_0000,
            rsx_mem_base: 0xC000_0000,
            rsx_mem_size: 0x1000_0000,
            stack_base: 0xD000_0000,
            stack_size: 0x1000_0000,
            spu_base: 0xE000_0000,
            spu_ls_size: 0x0004_0000,
        };

        assert_eq!(layout.main_memory_size, 256 * 1024 * 1024);
        assert_eq!(layout.user_memory_size, 256 * 1024 * 1024);
    }

    #[test]
    fn test_module_state() {
        let module = SystemModule {
            name: "test".to_string(),
            state: ModuleState::Loaded,
            id: 1,
        };
        assert_eq!(module.state, ModuleState::Loaded);
    }

    #[test]
    fn test_game_pipeline_creation() {
        let memory = MemoryManager::new().unwrap();
        let pipeline = GamePipeline::new(memory);
        assert!(pipeline.system_modules().is_empty());
    }

    #[test]
    fn test_game_pipeline_load_module() {
        let memory = MemoryManager::new().unwrap();
        let mut pipeline = GamePipeline::new(memory);

        let id = pipeline.load_module("cellSysutil").unwrap();
        assert_eq!(id, 1);

        // Loading same module again should return same ID
        let id2 = pipeline.load_module("cellSysutil").unwrap();
        assert_eq!(id2, 1);
    }

    #[test]
    fn test_game_pipeline_start_module() {
        let memory = MemoryManager::new().unwrap();
        let mut pipeline = GamePipeline::new(memory);

        pipeline.load_module("cellSysutil").unwrap();
        pipeline.start_module("cellSysutil").unwrap();

        let module = pipeline.get_system_module("cellSysutil").unwrap();
        assert_eq!(module.state, ModuleState::Running);
    }

    #[test]
    fn test_game_pipeline_stop_module() {
        let memory = MemoryManager::new().unwrap();
        let mut pipeline = GamePipeline::new(memory);

        pipeline.load_module("cellSysutil").unwrap();
        pipeline.start_module("cellSysutil").unwrap();
        pipeline.stop_module("cellSysutil").unwrap();

        let module = pipeline.get_system_module("cellSysutil").unwrap();
        assert_eq!(module.state, ModuleState::Stopped);
    }

    #[test]
    fn test_game_pipeline_initialize_system_modules() {
        let memory = MemoryManager::new().unwrap();
        let mut pipeline = GamePipeline::new(memory);

        pipeline.initialize_system_modules().unwrap();
        assert!(!pipeline.system_modules().is_empty());

        // Check core modules are loaded
        assert!(pipeline.get_system_module("cellSysutil").is_some());
        assert!(pipeline.get_system_module("cellGcmSys").is_some());
        assert!(pipeline.get_system_module("cellFs").is_some());
    }

    #[test]
    fn test_game_pipeline_setup_memory_layout() {
        let memory = MemoryManager::new().unwrap();
        let pipeline = GamePipeline::new(memory);

        let layout = pipeline.setup_memory_layout().unwrap();
        assert_eq!(layout.main_memory_base, 0x0000_0000);
        assert_eq!(layout.user_memory_base, 0x2000_0000);
        assert_eq!(layout.stack_base, 0xD000_0000);
    }

    #[test]
    fn test_game_pipeline_call_hle_function() {
        let memory = MemoryManager::new().unwrap();
        let pipeline = GamePipeline::new(memory);

        // Test calling a registered function
        let result = pipeline.call_hle_function("cellGcmSys", 0x21AC3697, &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
