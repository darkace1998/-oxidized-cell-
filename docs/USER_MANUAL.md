# 📖 Oxidized-Cell User Manual

**Version 0.1.0**

Welcome to **oxidized-cell**, a PlayStation 3 emulator written in Rust and C++. This manual will guide you through installation, configuration, and usage of the emulator.

---

## Table of Contents

1. [Introduction](#introduction)
2. [System Requirements](#system-requirements)
3. [Installation](#installation)
   - [Linux](#linux)
   - [Windows](#windows)
   - [macOS](#macos)
4. [Getting Started](#getting-started)
5. [User Interface](#user-interface)
   - [Main Window](#main-window)
   - [Menu Bar](#menu-bar)
   - [Game List](#game-list)
6. [Loading Games](#loading-games)
7. [Controls](#controls)
   - [Default Keyboard Mapping](#default-keyboard-mapping)
   - [Customizing Controls](#customizing-controls)
8. [Configuration](#configuration)
   - [General Settings](#general-settings)
   - [CPU Settings](#cpu-settings)
   - [GPU Settings](#gpu-settings)
   - [Audio Settings](#audio-settings)
   - [Input Settings](#input-settings)
   - [Path Settings](#path-settings)
   - [Debug Settings](#debug-settings)
9. [Debugging Tools](#debugging-tools)
   - [Log Viewer](#log-viewer)
   - [Memory Viewer](#memory-viewer)
   - [Shader Debugger](#shader-debugger)
10. [Troubleshooting](#troubleshooting)
11. [Keyboard Shortcuts](#keyboard-shortcuts)
12. [Frequently Asked Questions](#frequently-asked-questions)
13. [Legal Notice](#legal-notice)

---

## Introduction

**oxidized-cell** is an open-source PlayStation 3 emulator that aims to accurately emulate the Cell Broadband Engine and RSX graphics processor. The project is built using a hybrid architecture:

- **Rust (70%)**: Core emulation logic, memory safety, and system services
- **C++ (30%)**: High-performance JIT compilation using LLVM

> ⚠️ **Note**: This emulator is under active development. Game compatibility is limited and many features are still being implemented.

---

## System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 10 64-bit, Linux (Ubuntu 20.04+), macOS 11+ |
| **CPU** | x86-64 processor with SSE4.2 support |
| **RAM** | 8 GB |
| **GPU** | Vulkan 1.2 compatible graphics card |
| **Storage** | 500 MB for emulator + game storage |

### Recommended Requirements

| Component | Recommendation |
|-----------|----------------|
| **CPU** | Intel Core i7 / AMD Ryzen 5 or better |
| **RAM** | 16 GB or more |
| **GPU** | NVIDIA GTX 1060 / AMD RX 580 or better |
| **Storage** | SSD with adequate space for games |

---

## Installation

### Linux

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y build-essential cmake llvm-dev libvulkan-dev libasound2-dev

# Clone and build
git clone https://github.com/darkace1998/oxidized-cell.git
cd oxidized-cell
cargo build --release

# Run the emulator
./target/release/oxidized-cell
```

### Windows

1. **Install Prerequisites**:
   - Install [Rust](https://rustup.rs)
   - Install [Visual Studio 2019+](https://visualstudio.microsoft.com/) with C++ workload
   - Install [Vulkan SDK](https://vulkan.lunarg.com/)

2. **Build the Emulator**:
   ```powershell
   git clone https://github.com/darkace1998/oxidized-cell.git
   cd oxidized-cell
   cargo build --release
   ```

3. **Run**:
   ```powershell
   .\target\release\oxidized-cell.exe
   ```

### macOS

```bash
# Install dependencies
brew install llvm cmake

# Clone and build
git clone https://github.com/darkace1998/oxidized-cell.git
cd oxidized-cell
cargo build --release

# Run the emulator
./target/release/oxidized-cell
```

---

## Getting Started

1. **Launch the Emulator**: Run `oxidized-cell` (or `oxidized-cell.exe` on Windows)
2. **Configure Paths**: Go to **Settings → Paths** and set your game directory
3. **Load a Game**: Select a game from the game list or use **File → Open**
4. **Play**: The emulator will start once the game is loaded

---

## User Interface

### Main Window

The main window consists of several areas:

```
┌─────────────────────────────────────────────────────────────┐
│  Menu Bar                                                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                                                              │
│                    Game List / Emulation                     │
│                         View Area                            │
│                                                              │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  Status Bar (FPS, Frame Time, Emulation State)              │
└─────────────────────────────────────────────────────────────┘
```

### Menu Bar

| Menu | Description |
|------|-------------|
| **File** | Open games, manage recent files, exit |
| **Emulation** | Start, pause, stop, reset emulation |
| **View** | Toggle windows (Log Viewer, Memory Viewer, etc.) |
| **Settings** | Configure emulator settings |
| **Debug** | Access debugging tools |
| **Help** | About, documentation |

### Game List

The game list displays all detected games with:
- **Title**: Game name
- **ID**: Game serial number (e.g., BLUS00001, BLES00002)
- **Region**: US, EU, JP, etc.
- **Category**: Game genre
- **Version**: Game version
- **Last Played**: Date of last play session

Double-click a game to launch it.

---

## Loading Games

### Supported Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| **ELF** | `.elf` | Executable and Linkable Format |
| **SELF** | `.self` | Sony Encrypted ELF |
| **PRX** | `.prx` | PlayStation Relocatable Executable |
| **ISO** | `.iso` | Disc image |
| **PKG** | `.pkg` | PlayStation Package |

### How to Load a Game

1. **Via Game List**:
   - Add your games folder in **Settings → Paths → Games Directory**
   - Games will appear in the game list
   - Double-click to launch

2. **Via File Menu**:
   - Go to **File → Open**
   - Navigate to your game file (.elf, .self, etc.)
   - Click **Open**

3. **Via Command Line**:
   ```bash
   oxidized-cell /path/to/game.elf
   ```

---

## Controls

### Default Keyboard Mapping

The default keyboard controls map to a DualShock 3 controller:

| PS3 Button | Keyboard Key |
|------------|--------------|
| **Cross (✕)** | Z |
| **Circle (○)** | X |
| **Square (□)** | C |
| **Triangle (△)** | V |
| **D-Pad Up** | ↑ (Arrow Up) |
| **D-Pad Down** | ↓ (Arrow Down) |
| **D-Pad Left** | ← (Arrow Left) |
| **D-Pad Right** | → (Arrow Right) |
| **L1** | Q |
| **L2** | A |
| **L3** | (Left Stick Click) |
| **R1** | E |
| **R2** | D |
| **R3** | (Right Stick Click) |
| **Start** | Enter |
| **Select** | Backspace |

### Customizing Controls

1. Go to **Settings → Input**
2. Click on the button you want to remap
3. Press the new key on your keyboard
4. Click **Save** to apply changes

You can also edit the configuration file directly:

```toml
[input.keyboard_mapping]
cross = "Z"
circle = "X"
square = "C"
triangle = "V"
l1 = "Q"
l2 = "A"
r1 = "E"
r2 = "D"
start = "Return"
select = "BackSpace"
dpad_up = "Up"
dpad_down = "Down"
dpad_left = "Left"
dpad_right = "Right"
```

---

## Configuration

Configuration is stored in `config.toml` in the emulator directory. You can edit settings through the UI or by modifying this file directly.

### General Settings

| Setting | Default | Description |
|---------|---------|-------------|
| **Start Paused** | `false` | Begin emulation in paused state |
| **Confirm Exit** | `true` | Show confirmation when closing |
| **Auto Save State** | `false` | Save state automatically on exit |

### CPU Settings

| Setting | Default | Description |
|---------|---------|-------------|
| **PPU Decoder** | `Recompiler` | `Interpreter` or `Recompiler` (JIT) |
| **SPU Decoder** | `Recompiler` | `Interpreter` or `Recompiler` (JIT) |
| **PPU Threads** | `1` | Number of PPU threads (1-8) |
| **SPU Threads** | `1` | Number of SPU threads (1-6) |
| **Accurate DFMA** | `false` | Use accurate decimal FMA operations |
| **Accurate RSX Reservation** | `false` | Accurate RSX memory reservation |
| **SPU Loop Detection** | `true` | Optimize detected SPU loops |
| **Cycle Accurate Timing** | `false` | Enable precise timing simulation |
| **Pipeline Simulation** | `false` | Simulate CPU pipeline |

**Decoder Recommendations**:
- **Interpreter**: Slower but more compatible. Use for debugging or if games don't work with JIT.
- **Recompiler (JIT)**: Faster performance. Recommended for most games.

### GPU Settings

| Setting | Default | Description |
|---------|---------|-------------|
| **Backend** | `Vulkan` | Graphics backend (`Vulkan` or `Null`) |
| **Resolution Scale** | `1` | Internal resolution multiplier (1-4) |
| **Anisotropic Filter** | `1` | Anisotropic filtering level (1-16) |
| **VSync** | `true` | Enable vertical sync |
| **Frame Limit** | `60` | Maximum frames per second |
| **Shader Cache** | `true` | Cache compiled shaders |
| **Write Color Buffers** | `false` | Write color buffers to CPU |
| **Write Depth Buffer** | `false` | Write depth buffer to CPU |

### Audio Settings

| Setting | Default | Description |
|---------|---------|-------------|
| **Backend** | `Auto` | Audio backend (`Auto` or `Null`) |
| **Enable** | `true` | Enable audio output |
| **Volume** | `1.0` | Volume level (0.0 - 1.0) |
| **Buffer Duration** | `32` | Audio buffer size in milliseconds |
| **Time Stretching** | `true` | Stretch audio to match emulation speed |

### Input Settings

| Setting | Description |
|---------|-------------|
| **Controller** | Configure up to 4 controllers (Player 1-4) |
| **Keyboard Mapping** | Customize keyboard to PS3 button mapping |

### Path Settings

| Path | Description |
|------|-------------|
| **Games** | Directory containing your PS3 games |
| **dev_hdd0** | Virtual HDD0 (internal storage) |
| **dev_hdd1** | Virtual HDD1 (game data) |
| **dev_flash** | Virtual flash storage |
| **Save Data** | Save game directory |
| **Shader Cache** | Compiled shader cache directory |

### Debug Settings

| Setting | Default | Description |
|---------|---------|-------------|
| **Log Level** | `Info` | Logging verbosity: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace` |
| **Log to File** | `false` | Write logs to file |
| **Log Path** | `./logs` | Log file directory |
| **Dump Shaders** | `false` | Dump compiled shaders to disk |
| **Trace PPU** | `false` | Enable PPU instruction tracing |
| **Trace SPU** | `false` | Enable SPU instruction tracing |
| **Trace RSX** | `false` | Enable RSX command tracing |

---

## Debugging Tools

oxidized-cell includes several debugging tools for developers and advanced users.

### Log Viewer

Access via **View → Log Viewer** or by clicking the log viewer panel.

Features:
- Real-time log display with color coding by severity
- Filter logs by level (Error, Warn, Info, Debug, Trace)
- Filter by module/component
- Search functionality
- Export logs to file

Log Levels:
- 🔴 **Error**: Critical issues
- 🟡 **Warn**: Potential problems
- 🔵 **Info**: General information
- ⚪ **Debug**: Detailed debugging info
- 🟣 **Trace**: Very detailed execution trace

### Memory Viewer

Access via **View → Memory Viewer**

Features:
- View PS3 memory in real-time
- Hex and ASCII display
- Jump to address
- Memory search
- Watch specific memory regions
- Track memory changes

### Shader Debugger

Access via **View → Shader Debugger**

Features:
- View compiled RSX shaders
- SPIR-V disassembly
- Shader performance metrics
- Shader source inspection

### Debugger View

Access via **View → Debugger**

Features:
- PPU register inspection
- SPU register inspection
- Breakpoint management
- Step-by-step execution
- Disassembly view

---

## Troubleshooting

### Common Issues

#### Game Won't Start

1. **Check file format**: Ensure the game file is a supported format (.elf, .self, etc.)
2. **Check logs**: View the log viewer for error messages
3. **Try Interpreter mode**: Switch PPU/SPU decoder to Interpreter in CPU settings
4. **Verify Vulkan**: Ensure Vulkan drivers are installed and up to date

#### Poor Performance

1. **Use Recompiler**: Set PPU and SPU decoder to Recompiler
2. **Lower resolution**: Reduce Resolution Scale in GPU settings
3. **Disable accuracy options**: Turn off Accurate DFMA and Accurate RSX Reservation
4. **Enable SPU Loop Detection**: This can improve performance in many games

#### Graphics Issues

1. **Update GPU drivers**: Install the latest drivers for your graphics card
2. **Enable Write Color Buffers**: May fix some rendering issues
3. **Try different Resolution Scale**: Some games work better at specific scales
4. **Disable Shader Cache**: If shaders seem corrupted, clear and rebuild cache

#### Audio Issues

1. **Check Volume**: Ensure audio is enabled and volume is not 0
2. **Adjust Buffer Duration**: Try increasing to 64ms for stability
3. **Disable Time Stretching**: May help with audio crackling

#### Crash on Startup

1. **Check Vulkan installation**: Run `vulkaninfo` to verify Vulkan is working
2. **Delete config**: Remove `config.toml` to reset to defaults
3. **Check logs**: Look for error messages in the terminal or log file

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **F5** | Start/Resume Emulation |
| **F6** | Pause Emulation |
| **F7** | Stop Emulation |
| **F9** | Toggle Fullscreen |
| **F11** | Take Screenshot |
| **F12** | Toggle Performance Overlay |
| **Ctrl+O** | Open Game |
| **Ctrl+S** | Save Configuration |
| **Ctrl+Q** | Quit |
| **Ctrl+L** | Toggle Log Viewer |
| **Ctrl+M** | Toggle Memory Viewer |
| **Ctrl+,** | Open Settings |
| **Escape** | Exit Fullscreen / Return to Game List |

---

## Frequently Asked Questions

### Do I need a PS3 to use this emulator?

No, you do not need a PS3 console. However, you do need legally obtained game files.

### Where can I get games?

You must dump games from discs you own or purchase digital games. We do not provide games or support piracy.

### Why is my game running slowly?

PS3 emulation is computationally intensive. Ensure you:
- Meet the recommended system requirements
- Use Recompiler mode for PPU and SPU
- Have up-to-date GPU drivers

### Can I use a real PS3 controller?

Support for native DualShock 3 controllers is planned but not yet implemented. Currently, keyboard and generic gamepads are supported.

### Where are my save files stored?

Save files are stored in the path configured under **Settings → Paths → Save Data**. By default, this is in the `save_data` directory within the emulator folder.

### How do I report a bug?

Please open an issue on our [GitHub repository](https://github.com/darkace1998/oxidized-cell/issues) with:
- Your system specifications
- Steps to reproduce the issue
- Relevant log output
- Game information (title, ID, region)

### Is this emulator legal?

Yes, emulation is legal. However, downloading copyrighted games is not. Only use games you legally own.

---

## Legal Notice

- **oxidized-cell** is an open-source project licensed under the GPL-3.0 License.
- PlayStation 3, PS3, Cell, and related trademarks are property of Sony Interactive Entertainment.
- This project is not affiliated with or endorsed by Sony.
- Only use legally obtained games and system files.

---

## Support & Resources

- **GitHub Repository**: [https://github.com/darkace1998/oxidized-cell](https://github.com/darkace1998/oxidized-cell)
- **Issue Tracker**: [https://github.com/darkace1998/oxidized-cell/issues](https://github.com/darkace1998/oxidized-cell/issues)
- **Documentation**: See the `docs/` folder for technical documentation

---

**Thank you for using oxidized-cell!**

*This manual is for version 0.1.0. Features and options may change in future releases.*
