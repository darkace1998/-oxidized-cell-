# JIT Compilation in oxidized-cell

## Overview

The oxidized-cell PS3 emulator implements Just-In-Time (JIT) compilation for both PPU (PowerPC) and SPU (Synergistic Processing Unit) code using LLVM. This document describes the JIT compilation system, its usage, and configuration options.

## Architecture

### Components

1. **PPU JIT Compiler** (`cpp/src/ppu_jit.cpp`)
   - Compiles PowerPC 64-bit instructions to native code
   - Supports common integer, floating-point, and load/store operations
   - Uses LLVM IR for platform-independent code generation

2. **SPU JIT Compiler** (`cpp/src/spu_jit.cpp`)
   - Compiles SPU SIMD instructions to native code
   - Handles 128-bit vector operations
   - Optimized for SIMD performance

3. **FFI Layer** (`crates/oc-ffi/src/jit.rs`)
   - Provides safe Rust wrappers for C++ JIT compilers
   - Manages code cache and breakpoint integration
   - Thread-safe operation

## Features

### Basic Block Compilation

The JIT compiler uses basic block compilation strategy:
- **Fast Compilation**: Minimal compilation overhead
- **Efficient Cache**: O(1) lookup for compiled blocks
- **Easy Invalidation**: Simple to invalidate when code changes

### LLVM IR Generation

#### PPU Instructions

The PPU JIT generates LLVM IR for:

**Integer Operations:**
- `addi`, `addis` - Add immediate
- `add`, `subf` - Add/subtract register
- `mullw` - Multiply low word
- `and`, `or`, `xor` - Logical operations
- `ori`, `andi` - Logical immediate operations

**Load/Store Operations:**
- `lwz` - Load word and zero
- `stw` - Store word
- `lfs` - Load floating-point single
- `lfd` - Load floating-point double

**Floating-Point Operations:**
- `fadd`, `fsub`, `fmul`, `fdiv` - Basic arithmetic
- All operations with FPSCR flag tracking

#### SPU Instructions

The SPU JIT generates LLVM IR for:

**Vector Integer Operations:**
- `ai`, `andi` - Immediate arithmetic/logical
- `a`, `sf` - Add/subtract word
- `and`, `or`, `xor` - Logical operations

**Vector Floating-Point Operations:**
- `fa`, `fs`, `fm` - Float add, subtract, multiply
- All operations on 4x32-bit float vectors

### Optimization Passes

The JIT applies LLVM's standard O2 optimization pipeline:

1. **Function Inlining** - Inline small helper functions
2. **Dead Code Elimination** - Remove unused code
3. **Constant Propagation** - Evaluate constants at compile time
4. **Loop Optimizations** - Unroll and optimize loops
5. **Instruction Combining** - Combine multiple instructions
6. **SIMD Optimization** - Vectorize operations (SPU)

## Usage

### Rust API

```rust
use oc_ffi::jit::{PpuJitCompiler, SpuJitCompiler};

// Create PPU JIT compiler
let mut ppu_jit = PpuJitCompiler::new().expect("Failed to create PPU JIT");

// Compile PPU code
let ppu_code = [0x60, 0x00, 0x00, 0x00]; // nop instruction
ppu_jit.compile(0x1000, &ppu_code)?;

// Get compiled code
if let Some(compiled_ptr) = ppu_jit.get_compiled(0x1000) {
    // Execute compiled code (requires proper calling convention)
    // In practice, this is handled by the emulator core
}

// Create SPU JIT compiler
let mut spu_jit = SpuJitCompiler::new().expect("Failed to create SPU JIT");

// Compile SPU code
let spu_code = [0x40, 0x20, 0x00, 0x00]; // nop instruction
spu_jit.compile(0x0, &spu_code)?;
```

### Configuration

JIT compilation can be configured in `config.toml`:

```toml
[cpu]
ppu_decoder = "Recompiler"  # Use JIT for PPU
spu_decoder = "Recompiler"  # Use JIT for SPU

# Or use interpreter for debugging
# ppu_decoder = "Interpreter"
# spu_decoder = "Interpreter"
```

### Debugging Support

The JIT supports breakpoints for debugging:

```rust
// Add breakpoint
ppu_jit.add_breakpoint(0x1000);

// Check if breakpoint exists
if ppu_jit.has_breakpoint(0x1000) {
    // Handle breakpoint
}

// Remove breakpoint
ppu_jit.remove_breakpoint(0x1000);
```

### Cache Management

```rust
// Invalidate specific address
ppu_jit.invalidate(0x1000);

// Clear entire cache
ppu_jit.clear_cache();
```

## Performance Considerations

### When to Use JIT

**Use JIT for:**
- Long-running game code
- Frequently executed loops
- Compute-intensive operations

**Use Interpreter for:**
- One-time initialization code
- Self-modifying code
- Debugging sessions

### Memory Usage

- **Code Cache Size**: 64 MB per JIT (configurable)
- **Typical Block Size**: 4-16 KB per compiled block
- **Overhead**: ~10% memory overhead for metadata

### Compilation Time

- **First Compilation**: 1-10 ms per basic block
- **Cache Hit**: < 100 ns (O(1) lookup)
- **Optimization**: Adds ~20-50% to compilation time

## Building with LLVM

### Requirements

- **LLVM 15+** (recommended: LLVM 17)
- **CMake 3.20+**
- **C++20 compiler**

### Build Steps

```bash
# Install LLVM (Ubuntu/Debian)
sudo apt-get install llvm-17-dev

# Or via Homebrew (macOS)
brew install llvm@17

# Build oxidized-cell
mkdir build && cd build
cmake ..
make

# Build Rust crates
cargo build --release
```

### Without LLVM

The project can build without LLVM:

```bash
# LLVM will not be detected, JIT will use fallback mode
cmake ..
make
cargo build --release
```

Fallback mode provides:
- Basic block identification
- Cache management
- Breakpoint support
- But no actual compilation (returns placeholder code)

## Limitations

### Current Limitations

1. **Incomplete Instruction Coverage**
   - Not all PPU/SPU instructions generate LLVM IR yet
   - Unhandled instructions emit nop operations
   - Interpreter fallback for missing instructions

2. **No Cross-Block Optimization**
   - Each basic block compiled independently
   - No interprocedural optimization

3. **Limited Branch Handling**
   - Basic blocks end at branches
   - No branch prediction or speculation

4. **SPU Backend**
   - Uses native backend (x86/ARM) instead of SPU-specific
   - Some SPU-specific optimizations not available

### Future Enhancements

1. **Extended Instruction Coverage**
   - Complete all PowerPC and SPU instructions
   - Add VMX/AltiVec full support
   - Implement all floating-point edge cases

2. **Advanced Optimizations**
   - Cross-block optimization
   - Profile-guided optimization
   - Adaptive compilation levels

3. **Custom Backends**
   - PowerPC64 backend for better code quality
   - Custom SPU backend for dual-issue pipeline

4. **Runtime Profiling**
   - Hot path detection
   - Adaptive optimization
   - Performance counters

## Troubleshooting

### LLVM Not Found

```bash
# Set LLVM_DIR environment variable
export LLVM_DIR=/usr/lib/llvm-17/lib/cmake/llvm
cmake ..
```

### Compilation Errors

If JIT compilation fails:
1. Check LLVM version (15+ required)
2. Verify C++20 support
3. Enable fallback mode (builds without LLVM)

### Performance Issues

If JIT is slower than interpreter:
1. Check code cache size (may need increase)
2. Verify optimization level (should be O2)
3. Profile with `perf` or `vtune`

### Debugging

Enable JIT debug output:

```rust
// Set log level to trace for JIT operations
RUST_LOG=oc_ffi::jit=trace cargo run
```

## References

- [LLVM Documentation](https://llvm.org/docs/)
- [PowerPC ISA](https://openpowerfoundation.org/)
- [Cell BE Programming Handbook](https://www.ibm.com/support/pages/cell-be-programming-handbook)
- [PS3 Developer Wiki](https://www.psdevwiki.com/)

## Contributing

Contributions to the JIT compiler are welcome:

1. **Instruction Implementation**
   - Add LLVM IR generation for missing instructions
   - Test with real game code

2. **Optimization**
   - Implement new optimization passes
   - Profile and improve performance

3. **Testing**
   - Add unit tests for new instructions
   - Create benchmark suites

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
