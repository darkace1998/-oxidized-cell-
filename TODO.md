# 📋 Oxidized-Cell Development TODO

This document outlines the complete development roadmap for oxidized-cell, a PlayStation 3 emulator written in Rust and C++.

---

## 📊 Project Status Summary

| Component | Status | Completion | Priority |
|-----------|--------|------------|----------|
| Memory Management | ✅ Complete | 100% | - |
| PPU Interpreter | ✅ Complete | 95% | Low |
| SPU Interpreter | ✅ Complete | 95% | Low |
| RSX/Vulkan Backend | ✅ Complete | 95% | Medium |
| JIT Compilation | ✅ Complete | 100% | Low |
| LV2 Kernel | ✅ Complete | 95% | Low |
| Audio System | ✅ Complete | 85% | Medium |
| Input System | ✅ Complete | 80% | Medium |
| VFS | ✅ Complete | 80% | Medium |
| ELF/Game Loader | ✅ Complete | 90% | Low |
| HLE Modules | 🚧 In Progress | 40% | **HIGH** |
| User Interface | 🚧 In Progress | 15% | Medium |
| Game Loading Pipeline | ❌ Not Started | 0% | **HIGH** |
| Debugging Tools | 🔨 Mostly Complete | 70% | Low |

---

## 🎯 High Priority Tasks

### 1. HLE Module Implementation (Critical for Game Execution)

The HLE (High-Level Emulation) modules are essential for running PS3 games. Currently at ~40% completion.

#### HLE Infrastructure
- [x] **Global HLE Context** - Centralized manager instances
  - [x] Create HleContext to hold all manager instances
  - [x] Implement thread-safe access via RwLock
  - [x] Provide get_hle_context() and get_hle_context_mut() accessors
  - [x] Add reset_hle_context() for testing/cleanup

#### Graphics Modules
- [ ] **cellGcmSys** - RSX Graphics Command Management (Skeleton exists)
  - [ ] Integrate with actual RSX backend
  - [ ] Implement command buffer submission
  - [ ] Add texture management functions
  - [ ] Implement render target configuration
  - [ ] Add synchronization primitives (flip, finish, wait)

- [ ] **cellResc** - Resolution Scaler
  - [ ] Implement resolution conversion
  - [ ] Add aspect ratio handling
  - [ ] Support upscaling/downscaling

#### System Modules
- [x] **cellSysutil** - System Utilities (Connected to global context)
  - [x] Implement system callbacks
  - [x] Implement check_callback through global manager
  - [x] Get/set system parameters (int/string)
  - [ ] Add dialog support (game data, save data, etc.)
  - [ ] Implement PSID/account handling
  - [ ] Add disc detection functions

- [ ] **cellGame** - Game Data Management (Skeleton exists)
  - [ ] Implement game boot sequence
  - [ ] Add PARAM.SFO reading/writing
  - [ ] Support game data installation
  - [ ] Handle game updates

- [ ] **cellSaveData** - Save Data Management (Skeleton exists)
  - [ ] Implement save data listing
  - [ ] Add save/load operations
  - [ ] Support auto-save
  - [ ] Handle save data encryption

#### SPU/Threading Modules
- [ ] **cellSpurs** - SPU Runtime System (Skeleton exists)
  - [ ] Implement task queue management
  - [ ] Add workload scheduling
  - [ ] Support job chains
  - [ ] Implement taskset operations
  - [ ] Add event flags and barriers

- [ ] **cellSpursJq** - SPURS Job Queue
  - [ ] Implement job submission
  - [ ] Add job completion callbacks
  - [ ] Support job priorities

#### Input Modules
- [ ] **cellPad** - Controller Input (Skeleton exists)
  - [ ] Connect to oc-input backend
  - [ ] Implement pad data structures
  - [ ] Add rumble/vibration support
  - [ ] Support multiple controllers

- [ ] **cellKb** - Keyboard Input
  - [ ] Implement keyboard mapping
  - [ ] Support multiple keyboard layouts

- [ ] **cellMouse** - Mouse Input
  - [ ] Implement mouse position tracking
  - [ ] Add button state handling

#### Audio Modules
- [x] **cellAudio** - Audio Output (Connected to global context)
  - [x] Implement init/quit through global manager
  - [x] Implement port open/close through global manager
  - [x] Implement port start/stop through global manager
  - [ ] Connect to oc-audio backend
  - [ ] Add mixing support

- [ ] **cellMic** - Microphone Input
  - [ ] Implement audio capture
  - [ ] Add device enumeration

#### File System Modules
- [ ] **cellFs** - File System (Skeleton exists)
  - [ ] Connect to oc-vfs backend
  - [ ] Implement file operations
  - [ ] Add directory operations
  - [ ] Support asynchronous I/O

#### Media Decoding Modules
- [x] **cellVdec** - Video Decoder (Connected to global context)
  - [x] Implement open/close through global manager
  - [x] Implement start/end sequence through global manager
  - [x] Implement decode_au through global manager
  - [ ] Implement H.264/AVC decoding backend
  - [ ] Add MPEG-2 support
  - [ ] Support various profiles

- [x] **cellAdec** - Audio Decoder (Connected to global context)
  - [x] Implement open/close through global manager
  - [x] Implement start/end sequence through global manager
  - [x] Implement decode_au through global manager
  - [ ] Implement AAC decoding backend
  - [ ] Add MP3 support
  - [ ] Support ATRAC3+

- [x] **cellDmux** - Demultiplexer (Connected to global context)
  - [x] Implement open/close through global manager
  - [x] Implement set_stream/reset_stream through global manager
  - [x] Implement enable_es/disable_es through global manager
  - [ ] Implement container parsing backend
  - [ ] Add stream separation

- [ ] **cellVpost** - Video Post-Processing (Skeleton exists)
  - [ ] Implement color conversion
  - [ ] Add scaling support

#### Image Decoding Modules
- [ ] **cellPngDec** - PNG Decoder (Skeleton exists)
  - [ ] Implement full PNG decoding
  - [ ] Support various color formats

- [ ] **cellJpgDec** - JPEG Decoder (Skeleton exists)
  - [ ] Implement JPEG decoding
  - [ ] Add progressive JPEG support

- [ ] **cellGifDec** - GIF Decoder (Skeleton exists)
  - [ ] Implement GIF decoding
  - [ ] Support animations

#### Network Modules
- [x] **cellNetCtl** - Network Control (Connected to global context)
  - [x] Implement init/term through global manager
  - [x] Implement get_state through global manager
  - [x] Implement add/remove handler through global manager
  - [x] Implement start/unload dialog through global manager
  - [ ] Connect to actual network backend
  - [ ] Support network configuration

- [x] **cellHttp** - HTTP Client (Connected to global context)
  - [x] Implement init/end through global manager
  - [x] Implement create/destroy client through global manager
  - [ ] Implement HTTP requests
  - [ ] Add HTTPS support

- [x] **cellSsl** - SSL/TLS (Connected to global context)
  - [x] Implement init/end through global manager
  - [x] Implement certificate loader through global manager
  - [ ] Implement TLS connections
  - [ ] Add certificate handling

#### Font Modules
- [ ] **cellFont** - Font Library (Skeleton exists)
  - [ ] Implement font rendering
  - [ ] Support various font formats

- [ ] **cellFontFT** - FreeType Font Library
  - [ ] Integrate with FreeType

---

### 2. Game Loading Pipeline (Critical)

The game loading pipeline connects all components to enable game execution.

- [ ] **Game Discovery**
  - [ ] Implement game directory scanning (partially done in GameScanner)
  - [ ] Parse PARAM.SFO metadata
  - [ ] Extract game icons and backgrounds
  - [ ] Cache game database

- [ ] **EBOOT.BIN Loading**
  - [ ] Parse EBOOT.BIN format
  - [ ] Handle encrypted executables
  - [ ] Load PRX dependencies

- [ ] **PRX Module Loading**
  - [ ] Implement dynamic PRX loading
  - [ ] Resolve module imports/exports
  - [ ] Handle NID (Native ID) resolution
  - [ ] Support stub libraries

- [ ] **Memory Layout**
  - [ ] Initialize PS3 memory regions (done)
  - [ ] Set up stack for main thread
  - [ ] Configure TLS areas
  - [ ] Initialize kernel objects

- [ ] **Main Thread Creation**
  - [ ] Create initial PPU thread
  - [ ] Set up register state
  - [ ] Initialize thread local storage
  - [ ] Start execution

---

## 🔨 Medium Priority Tasks

### 3. LV2 Kernel Enhancements (95% Complete)

#### Thread Management
- [x] Implement thread priorities properly
- [x] Add thread affinity support
- [ ] Improve context switching
- [x] Support thread-local storage

#### Synchronization Primitives
- [x] Improve mutex implementation
- [x] Add event flag support
- [x] Implement reader-writer locks properly
- [x] Add barrier support

#### Memory Syscalls
- [x] Implement mmap/munmap properly
- [x] Add memory attribute handling
- [ ] Support large pages

#### Time Management
- [x] Improve timer accuracy
- [x] Add high-resolution timers
- [x] Implement usleep properly

### 4. User Interface Improvements (15% → 75%)

#### Main Window
- [ ] Implement game grid view with icons
- [ ] Add game search/filter
- [ ] Support game categories
- [ ] Add recent games list

#### Emulation View
- [ ] Connect RSX output to display
- [ ] Add fullscreen support
- [ ] Implement frame rate limiting
- [ ] Add frame skipping option

#### Settings
- [ ] CPU settings (interpreter/JIT, threads)
- [ ] GPU settings (resolution, scaling)
- [ ] Audio settings (backend, volume)
- [ ] Input settings (controller mapping)
- [ ] Path settings (game directories)

#### Debugger View
- [ ] PPU register display
- [ ] SPU register display
- [ ] Memory hex editor
- [ ] Disassembly view
- [ ] Breakpoint management
- [ ] Call stack view

### 5. RSX/Graphics Improvements (95% → 100%)

- [ ] Implement missing NV4097 methods
- [ ] Add shader caching
- [ ] Improve texture sampling accuracy
- [ ] Fix depth buffer handling
- [ ] Add anti-aliasing support
- [ ] Implement vertex processing optimizations
- [ ] Add asynchronous texture loading

### 6. Audio System Improvements (85% → 100%)

- [ ] Implement proper sample rate conversion
- [ ] Add audio mixing improvements
- [ ] Support all audio formats
- [ ] Improve latency
- [ ] Add audio streaming support

### 7. Input System Improvements (80% → 100%)

- [ ] Add pressure-sensitive button support
- [ ] Implement motion controls
- [ ] Add touchpad support (for dualshock 4)
- [ ] Support multiple controller types
- [ ] Improve input latency

### 8. VFS Improvements (80% → 100%)

- [ ] Implement remaining disc formats
- [ ] Add PKG installation support
- [ ] Improve ISO performance
- [ ] Add network path support

---

## 📌 Low Priority Tasks

### 9. PPU Interpreter Improvements (95% → 100%)

- [ ] Implement remaining privileged instructions
- [ ] Add accurate exception handling
- [ ] Improve cycle counting accuracy
- [ ] Add trace logging for debugging

### 10. SPU Interpreter Improvements (95% → 100%)

- [ ] Implement remaining floating-point instructions
- [ ] Add all permute/shuffle variants
- [ ] Improve timing accuracy
- [ ] Add hint instruction support

### 11. JIT Optimizations

- [ ] Add more PPU instructions to JIT
- [ ] Optimize hot code paths
- [ ] Add block linking
- [ ] Implement profiling-guided optimization

### 12. Debugging Tools

- [ ] Add memory watchpoints
- [ ] Implement trace recording/replay
- [ ] Add RSX command buffer debugging
- [ ] Implement SPU debugger improvements
- [ ] Add performance profiler

---

## 🧪 Testing Tasks

### Unit Tests
- [ ] Add more PPU instruction tests
- [ ] Add more SPU instruction tests
- [ ] Add RSX method tests
- [ ] Add HLE module tests

### Integration Tests
- [ ] Test game loading pipeline
- [ ] Test multi-threaded scenarios
- [ ] Test SPU-PPU communication
- [ ] Test memory mapping

### Compatibility Testing
- [ ] Test with PS3 homebrew
- [ ] Create game compatibility database
- [ ] Document known issues per game

---

## 📚 Documentation Tasks

- [ ] Document PPU instruction implementation details
- [ ] Document SPU instruction implementation details
- [ ] Create RSX method reference
- [ ] Write HLE module documentation
- [ ] Create contributing guidelines
- [ ] Add code style guide
- [ ] Write architecture overview

---

## 🔧 Build & Infrastructure

- [ ] Set up CI/CD pipeline
- [ ] Add Windows build support
- [ ] Add macOS build support
- [ ] Create release packaging
- [ ] Add automated testing in CI
- [ ] Set up code coverage reporting
- [ ] Add benchmarking infrastructure

---

## 📁 Crate-Specific TODOs

### oc-core
- [ ] Improve configuration validation
- [ ] Add runtime configuration reloading
- [ ] Implement proper logging levels

### oc-memory
- [x] Memory manager implementation ✅
- [x] Page table management ✅
- [x] Reservation system ✅
- [ ] Add memory statistics/profiling

### oc-ppu
- [x] Full interpreter implementation ✅
- [x] VMX/AltiVec support ✅
- [x] Breakpoint support ✅
- [ ] Improve performance

### oc-spu
- [x] Full interpreter implementation ✅
- [x] MFC DMA engine ✅
- [x] Channel communication ✅
- [ ] Add isolation mode

### oc-rsx
- [x] Vulkan backend ✅
- [x] NV4097 method handlers ✅
- [x] Texture management ✅
- [ ] Shader cache persistence

### oc-lv2
- [x] Process management ✅
- [x] Thread management ✅
- [x] Synchronization primitives ✅
- [x] Event flags ✅
- [x] Barriers ✅
- [x] High-resolution timers ✅
- [x] Thread affinity ✅
- [x] Thread-local storage ✅
- [ ] Complete all syscalls

### oc-audio
- [x] cpal backend ✅
- [x] Multi-channel support ✅
- [ ] Improve mixing quality

### oc-input
- [x] Keyboard mapping ✅
- [x] Controller support ✅
- [ ] Add more controller types

### oc-vfs
- [x] ISO 9660 support ✅
- [x] PKG support ✅
- [x] PARAM.SFO parsing ✅
- [ ] Add more disc formats

### oc-hle
- [ ] Complete cellGcmSys
- [ ] Complete cellSpurs
- [ ] Complete cellSysutil
- [ ] Implement all priority modules

### oc-loader
- [x] ELF parsing ✅
- [x] SELF parsing ✅
- [x] PRX loading ✅
- [ ] Handle encrypted content

### oc-ffi
- [x] PPU JIT bindings ✅
- [x] SPU JIT bindings ✅
- [ ] Add more JIT instructions

### oc-ui
- [x] Basic UI framework ✅
- [x] Game list view ✅
- [ ] Complete all views

### oc-integration
- [x] EmulatorRunner ✅
- [x] GameLoader ✅
- [x] GamePipeline ✅
- [ ] Complete game execution

### oc-debug
- [x] PPU debugger ✅
- [x] SPU debugger ✅
- [x] RSX debugger ✅
- [ ] Add profiler

---

## 📅 Development Phases

### Phase 1: Foundation ✅ (Complete)
- Memory management
- CPU interpreters (PPU/SPU)
- Basic RSX implementation

### Phase 2: System Emulation 🔨 (In Progress)
- LV2 kernel syscalls
- File system
- Audio/Input systems

### Phase 3: HLE Modules 🚧 (Current Focus)
- Implement priority HLE modules
- Game loading pipeline
- PRX module support

### Phase 4: Game Compatibility
- Test with homebrew
- Fix bugs per-game
- Optimize performance

### Phase 5: Polish
- UI improvements
- Documentation
- Release preparation

---

## 🏁 Immediate Next Steps

1. **Implement cellGcmSys fully** - Connect graphics HLE to RSX backend
2. **Implement cellSpurs** - Required by most games for SPU task management
3. **Complete game loading pipeline** - Enable EBOOT.BIN execution
4. **Add PRX loading** - Most games require system PRX modules
5. **Test with homebrew** - Validate implementation with simple apps

---

## 📞 Contributing

See the [Contributing section in README.md](README.md#contributing) for guidelines on how to contribute to this project.

### Quick Start for Contributors

1. Pick an unchecked item from this TODO
2. Create a branch: `git checkout -b feature/your-feature`
3. Implement the feature with tests
4. Submit a pull request

### Priority Labels
- **HIGH** - Critical for running any games
- **Medium** - Improves compatibility/usability
- **Low** - Nice to have, optimizations

---

*Last updated: December 2024*
