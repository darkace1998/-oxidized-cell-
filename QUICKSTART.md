# 🚀 Quick Start Guide - Testing Games with Oxidized-Cell

This guide will help you build and run oxidized-cell to test PS3 games. The instructions here are based on the actual codebase.

> ⚠️ **Early Development**: This emulator is under active development. Game compatibility is limited and many features are still being implemented.

---

## Prerequisites

### Required (Rust 1.80+)

```bash
# Install Rust via rustup (https://rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Platform-Specific Dependencies

**Linux (Ubuntu/Debian)**:
```bash
sudo apt update
sudo apt install -y build-essential libasound2-dev libvulkan-dev
```

**Windows**:
1. Install [Rust](https://rustup.rs)
2. Install [Visual Studio 2019+](https://visualstudio.microsoft.com/) with C++ workload
3. Install [Vulkan SDK](https://vulkan.lunarg.com/)

**macOS**:
```bash
brew install llvm cmake
```

---

## Step 1: Build the Emulator

```bash
# Clone the repository
git clone https://github.com/darkace1998/oxidized-cell.git
cd oxidized-cell

# Build in release mode (recommended for performance)
cargo build --release
```

The build takes several minutes on first run. The resulting binary will be at `target/release/oxidized-cell`.

---

## Step 2: Download PS3 Firmware (Required for Encrypted Games)

Most PS3 games use SELF (encrypted ELF) format and require the official PS3 firmware to decrypt. The firmware is free and legal to download from Sony.

### Option A: Download Script (Recommended)

```bash
# Linux/macOS
./scripts/download-firmware.sh

# Windows
scripts\download-firmware.bat
```

### Option B: Manual Download

```bash
# Create firmware directory
mkdir -p firmware

# Download (Linux/macOS with wget)
wget http://dus01.ps3.update.playstation.net/update/ps3/image/us/2025_0305_c179ad173bbc08b55431d30947725a4b/PS3UPDAT.PUP -O firmware/PS3UPDAT.PUP

# Or with curl
curl -L -o firmware/PS3UPDAT.PUP "http://dus01.ps3.update.playstation.net/update/ps3/image/us/2025_0305_c179ad173bbc08b55431d30947725a4b/PS3UPDAT.PUP"
```

**Windows (PowerShell)**:
```powershell
mkdir firmware
Invoke-WebRequest -Uri "http://dus01.ps3.update.playstation.net/update/ps3/image/us/2025_0305_c179ad173bbc08b55431d30947725a4b/PS3UPDAT.PUP" -OutFile "firmware\PS3UPDAT.PUP"
```

The firmware file should be placed in the `firmware/` directory or installed via:
```bash
cargo run --release -- --install-firmware /path/to/PS3UPDAT.PUP
```

---

## Step 3: Run the Emulator

### Launch the UI
```bash
cargo run --release
```

This opens the emulator with a graphical interface where you can:
- **File → Open Game...** to add games to the list
- Double-click a game to launch it
- Access **Settings** to configure CPU, GPU, and audio options

### Launch with a Specific Game
```bash
# Load a game directly from command line
cargo run --release -- /path/to/game.elf

# Or with a folder containing PS3_GAME/USRDIR/EBOOT.BIN
cargo run --release -- /path/to/game_folder/
```

---

## Supported Game Formats

The loader (`oc-loader`) supports these formats:

| Format | Extension | Description |
|--------|-----------|-------------|
| **ELF** | `.elf` | Plain executables (unencrypted) - **best for testing** |
| **SELF** | `.self`, `.bin` | Encrypted PS3 executables (requires firmware) |
| **ISO** | `.iso` | PS3 disc images (searches for EBOOT.BIN inside) |
| **PRX** | `.prx` | PS3 loadable modules |
| **Game Folder** | Directory | Looks for `PS3_GAME/USRDIR/EBOOT.BIN` or `USRDIR/EBOOT.BIN` |

### Game Loading Order
When you provide a path, the loader tries:
1. Direct file (ELF, SELF, ISO)
2. `PS3_GAME/USRDIR/EBOOT.BIN`
3. `USRDIR/EBOOT.BIN`
4. `EBOOT.BIN`
5. Any `.elf` or `.self` file in the directory

---

## Step 4: Understanding the UI

### Main Views
- **Game List**: Shows added games, double-click to launch
- **Emulation**: RSX output area with Start/Pause/Stop controls
- **Debugger**: PPU/SPU state inspection
- **Log Viewer**: Real-time logs (View → Log Viewer)
- **Memory Viewer**: Inspect emulator memory

### Menu Options
- **File → Open Game...**: Add games to the list
- **Emulation → Start/Pause/Stop**: Control emulation
- **Settings → Configuration**: CPU, GPU, audio settings
- **Settings → Install Firmware**: Install PS3 firmware file
- **View → Performance Overlay**: Show FPS and frame stats

---

## Running Tests

Verify the build with the test suite:

```bash
# Run specific crate tests (these are most reliable)
cargo test -p oc-memory    # Memory management tests (128+ tests)
cargo test -p oc-ppu       # PPU interpreter tests (75+ tests)
cargo test -p oc-spu       # SPU interpreter tests (14+ tests)
cargo test -p oc-rsx       # RSX graphics tests (36+ tests)

# Run with verbose output
cargo test -p oc-memory -- --nocapture
```

---

## Keyboard Controls

Default keyboard mapping (from `oc-input` crate):

| PS3 Button | Keyboard Key |
|------------|--------------|
| Cross (✕) | Z |
| Circle (○) | X |
| Square (□) | C |
| Triangle (△) | V |
| D-Pad Up/Down/Left/Right | Arrow Keys |
| L1 | Q |
| L2 | A |
| R1 | E |
| R2 | D |
| Start | Enter |
| Select | Backspace |

---

## Troubleshooting

### Build Fails with `alsa-sys` Error
```bash
# Install ALSA development libraries (Linux)
sudo apt install libasound2-dev
```

### Build Fails with Vulkan Error
```bash
# Install Vulkan development libraries (Linux)
sudo apt install libvulkan-dev

# Verify Vulkan is working
vulkaninfo
```

### "Encrypted PS3 Executable" Error
This means the game is SELF format and needs firmware:
1. Download PS3 firmware (see Step 2)
2. Place `PS3UPDAT.PUP` in the `firmware/` folder
3. Alternatively, use decrypted ELF files for testing

### Game Won't Start / Crashes
- Check View → Log Viewer for error messages
- The emulator is in early development - many games won't work yet
- Try simpler homebrew ELF files for initial testing

### No Graphics Output
- Vulkan backend is required
- Ensure your GPU supports Vulkan 1.2+
- Check that Vulkan is properly installed

---

## Configuration

Configuration is stored in `config.toml`. Key settings from `oc-core`:

```toml
[cpu]
ppu_decoder = "interpreter"  # "interpreter" or "jit" (jit incomplete)
spu_decoder = "interpreter"  # "interpreter" or "jit"

[graphics]
backend = "vulkan"
resolution_scale = 1

[audio]
backend = "cpal"
volume = 100

[debug]
log_level = "info"  # off, error, warn, info, debug, trace
```

---

## Project Architecture

The emulator is organized into crates under `crates/`:

| Crate | Purpose |
|-------|---------|
| `oc-core` | Configuration, logging, scheduling |
| `oc-memory` | 4GB virtual memory, page management |
| `oc-ppu` | PowerPC interpreter (2700+ lines) |
| `oc-spu` | SPU interpreter with 128-bit registers |
| `oc-rsx` | RSX graphics, Vulkan backend |
| `oc-lv2` | LV2 kernel syscalls |
| `oc-hle` | High-level emulation modules |
| `oc-loader` | ELF/SELF/PRX/ISO loading |
| `oc-integration` | EmulatorRunner tying everything together |
| `oc-ui` | egui-based user interface |

---

## Current Limitations

From `todo.md` and code analysis:
- JIT compilation is incomplete (interpreter mode is more compatible)
- RSX graphics backend is work-in-progress
- Some HLE modules are stubs
- Limited game compatibility
- No full firmware decryption (partial support)

---

## Next Steps

1. **Test with homebrew**: Unencrypted ELF files work best
2. **Check todo.md**: See current development priorities
3. **Read the code**: `crates/oc-integration/src/runner.rs` shows how games are loaded and executed
4. **Contribute**: See Contributing section in README.md

---

## Getting Help

- **GitHub Issues**: https://github.com/darkace1998/oxidized-cell/issues
- **Documentation**: Check the `docs/` folder
- **User Manual**: See `docs/USER_MANUAL.md` for detailed configuration
