# PPU (PowerPC Processing Unit) Instruction Set Implementation

## Overview

The PPU emulation in oxidized-cell implements the PowerPC 64-bit architecture used in the PlayStation 3's Cell Broadband Engine. This document describes the implemented instruction set, architecture details, and usage.

## Architecture

### Register Set

#### General Purpose Registers (GPRs)
- **32 × 64-bit registers** (r0-r31)
- Follow PowerPC 64-bit calling conventions
- r0 can be used as a normal register (unlike some RISC ISAs where r0 is hardwired to zero)
- r1 is typically used as the stack pointer by convention

#### Floating-Point Registers (FPRs)
- **32 × 64-bit registers** (f0-f31)
- IEEE 754 double-precision format
- Supports single-precision operations (promoted to double internally)

#### Vector Registers (VRs) - AltiVec/VMX
- **32 × 128-bit registers** (v0-v31)
- Used for SIMD operations
- Each register contains 4 × 32-bit elements
- Supports byte, halfword, word, and floating-point operations

### Special Purpose Registers

| Register | Description | Access |
|----------|-------------|--------|
| **PC** (Program Counter) | Current instruction address | Read/Write |
| **LR** (Link Register) | Function call return address | Read/Write |
| **CTR** (Count Register) | Loop counter | Read/Write |
| **XER** | Fixed-point exception register (CA, OV, SO) | Read/Write |
| **CR** | Condition register (8 × 4-bit fields) | Read/Write |
| **FPSCR** | Floating-point status and control | Read/Write |
| **VSCR** | Vector status and control | Read/Write |

### Condition Register (CR)

The CR consists of 8 4-bit fields (CR0-CR7), each containing:
- **LT** (bit 0): Less Than
- **GT** (bit 1): Greater Than
- **EQ** (bit 2): Equal
- **SO** (bit 3): Summary Overflow

### XER Register

- **SO** (bit 0): Summary Overflow - sticky overflow bit
- **OV** (bit 1): Overflow - set on signed overflow
- **CA** (bit 2): Carry - set on unsigned carry/borrow

## Instruction Formats

The PPU uses 32-bit big-endian instructions with multiple formats:

- **I-Form**: Branch instructions with 24-bit immediate
- **B-Form**: Conditional branches with 14-bit immediate
- **D-Form**: Load/store and immediate arithmetic (16-bit immediate)
- **DS-Form**: 64-bit load/store with 14-bit aligned displacement
- **X-Form**: Register-register operations
- **XL-Form**: Branch to LR/CTR and CR logical operations
- **XFX-Form**: Move to/from special registers
- **XO-Form**: Integer arithmetic with overflow detection
- **A-Form**: Floating-point multiply-add
- **M-Form**: Rotate and mask operations
- **VA-Form**: Vector three-operand instructions
- **VX-Form**: Vector two-operand instructions

## Implemented Instructions

### 1. Branch Instructions

#### Unconditional Branch
- **`b`** - Branch (relative or absolute)
- **`ba`** - Branch absolute
- **`bl`** - Branch and link (function call)
- **`bla`** - Branch and link absolute

```rust
// Example: b 0x100 - Branch forward 0x100 bytes
let opcode = 0x48000100u32;
```

#### Conditional Branch
- **`bc`** - Branch conditional
- **`bclr`** - Branch to Link Register (function return)
- **`bcctr`** - Branch to Count Register

**Branch conditions (BO field):**
- Decrement CTR and branch if CTR != 0
- Decrement CTR and branch if CTR == 0
- Branch if CR condition is true
- Branch if CR condition is false

```rust
// Example: beq 0x40 - Branch if equal (CR0 EQ bit set)
let opcode = 0x41820040u32;
```

### 2. Integer Instructions

#### Arithmetic
- **`add`**, **`addo`** - Add with optional overflow detection
- **`addc`**, **`addco`** - Add carrying
- **`adde`**, **`addeo`** - Add extended (with carry)
- **`addi`** - Add immediate
- **`addis`** - Add immediate shifted (load high 16 bits)
- **`addic`**, **`addic.`** - Add immediate carrying
- **`subf`**, **`subfo`** - Subtract from
- **`subfc`**, **`subfco`** - Subtract from carrying
- **`subfe`**, **`subfeo`** - Subtract from extended
- **`subfic`** - Subtract from immediate carrying
- **`neg`**, **`nego`** - Negate
- **`mulli`** - Multiply low immediate
- **`mullw`**, **`mullwo`** - Multiply low word
- **`mulhw`**, **`mulhwu`** - Multiply high word (signed/unsigned)
- **`divw`**, **`divwo`** - Divide word
- **`divwu`**, **`divwuo`** - Divide word unsigned

**Special cases:**
- Division by zero returns 0 (no exception in user mode)
- `i32::MIN / -1` overflow returns 0 and sets OV flag

#### Logical
- **`and`**, **`andi.`**, **`andis.`** - AND operations
- **`andc`** - AND with complement
- **`or`**, **`ori`**, **`oris`** - OR operations
- **`orc`** - OR with complement
- **`xor`**, **`xori`**, **`xoris`** - XOR operations
- **`nand`** - NAND
- **`nor`** - NOR
- **`eqv`** - Equivalent (XNOR)

#### Shift and Rotate
- **`slw`**, **`sld`** - Shift left word/doubleword
- **`srw`**, **`srd`** - Shift right word/doubleword
- **`sraw`**, **`srad`** - Shift right algebraic (sign-extending)
- **`srawi`**, **`sradi`** - Shift right algebraic immediate
- **`rlwinm`** - Rotate left word immediate then AND with mask
- **`rlwimi`** - Rotate left word immediate then mask insert
- **`rlwnm`** - Rotate left word then AND with mask

#### Comparison
- **`cmp`**, **`cmpi`** - Signed compare
- **`cmpl`**, **`cmpli`** - Unsigned compare

#### Count and Bit Operations
- **`cntlzw`**, **`cntlzd`** - Count leading zeros
- **`popcntw`**, **`popcntd`** - Population count (count set bits)
- **`extsb`**, **`extsh`**, **`extsw`** - Sign extend byte/halfword/word

### 3. Load/Store Instructions

#### Basic Load (Zero-Extended)
- **`lbz`**, **`lbzx`**, **`lbzu`**, **`lbzux`** - Load byte
- **`lhz`**, **`lhzx`**, **`lhzu`**, **`lhzux`** - Load halfword
- **`lwz`**, **`lwzx`**, **`lwzu`**, **`lwzux`** - Load word
- **`ld`**, **`ldx`**, **`ldu`**, **`ldux`** - Load doubleword

#### Load Sign-Extended
- **`lha`**, **`lhax`**, **`lhau`**, **`lhaux`** - Load halfword algebraic
- **`lwa`**, **`lwax`**, **`lwau`**, **`lwaux`** - Load word algebraic

#### Store
- **`stb`**, **`stbx`**, **`stbu`**, **`stbux`** - Store byte
- **`sth`**, **`sthx`**, **`sthu`**, **`sthux`** - Store halfword
- **`stw`**, **`stwx`**, **`stwu`**, **`stwux`** - Store word
- **`std`**, **`stdx`**, **`stdu`**, **`stdux`** - Store doubleword

#### Multiple Word Operations
- **`lmw`** - Load multiple word (load r[rt..31])
- **`stmw`** - Store multiple word (store r[rt..31])

#### Atomic Operations
Critical for multi-threaded PS3 applications:

- **`lwarx`** - Load word and reserve indexed
  - Acquires a reservation on a memory location
  - Used for atomic read-modify-write sequences
  
- **`ldarx`** - Load doubleword and reserve indexed
  
- **`stwcx.`** - Store word conditional indexed
  - Stores only if reservation is still valid
  - Sets CR0 EQ bit on success, clears on failure
  
- **`stdcx.`** - Store doubleword conditional indexed

**Atomic sequence example:**
```rust
// Atomic increment using lwarx/stwcx.
// lwarx  r4, 0, r3      // Load and reserve
// addi   r4, r4, 1      // Increment
// stwcx. r4, 0, r3      // Store conditional
// bne    retry          // Retry if failed
```

#### Floating-Point Load/Store
- **`lfs`**, **`lfsx`**, **`lfsu`**, **`lfsux`** - Load single
- **`lfd`**, **`lfdx`**, **`lfdu`**, **`lfdux`** - Load double
- **`stfs`**, **`stfsx`**, **`stfsu`**, **`stfsux`** - Store single
- **`stfd`**, **`stfdx`**, **`stfdu`**, **`stfdux`** - Store double

### 4. Floating-Point Instructions

#### Arithmetic
- **`fadd`**, **`fadds`** - Add (double/single)
- **`fsub`**, **`fsubs`** - Subtract
- **`fmul`**, **`fmuls`** - Multiply
- **`fdiv`**, **`fdivs`** - Divide
- **`fsqrt`**, **`fsqrts`** - Square root

#### Fused Multiply-Add (FMA)
High-performance operations crucial for graphics and physics:

- **`fmadd`**, **`fmadds`** - `(a × c) + b` with single rounding
- **`fmsub`**, **`fmsubs`** - `(a × c) - b`
- **`fnmadd`**, **`fnmadds`** - `-((a × c) + b)`
- **`fnmsub`**, **`fnmsubs`** - `-((a × c) - b)`

#### Conversion
- **`fcfid`** - Convert signed integer doubleword to double
- **`fctid`** - Convert double to signed integer doubleword
- **`fctidz`** - Convert to integer with round toward zero
- **`fctiw`** - Convert double to signed integer word
- **`fctiwz`** - Convert to integer word with round toward zero
- **`frsp`** - Round to single precision

#### Comparison
- **`fcmpu`** - Compare unordered (no exception on NaN)
- **`fcmpo`** - Compare ordered (exception on NaN)

#### Special Operations
- **`fsel`** - Floating select (conditional move without branches)
- **`fabs`** - Absolute value
- **`fneg`** - Negate
- **`fnabs`** - Negative absolute value
- **`fmr`** - Move register
- **`fres`** - Reciprocal estimate
- **`frsqrte`** - Reciprocal square root estimate

### 5. Vector/SIMD Instructions (AltiVec/VMX)

#### Integer Arithmetic
- **`vaddubm`**, **`vadduhm`**, **`vadduwm`** - Add (byte/half/word)
- **`vaddubs`**, **`vadduhs`**, **`vadduws`** - Add unsigned saturate
- **`vaddsbs`**, **`vaddshs`**, **`vaddsws`** - Add signed saturate
- **`vsububm`**, **`vsubuhm`**, **`vsubuwm`** - Subtract
- **`vsububs`**, **`vsubuhs`**, **`vsubuws`** - Subtract unsigned saturate
- **`vsubsbs`**, **`vsubshs`**, **`vsubsws`** - Subtract signed saturate
- **`vmuluwm`** - Multiply unsigned word
- **`vmulesb`**, **`vmulesh`**, **`vmulesw`** - Multiply even signed
- **`vmuloub`**, **`vmulouh`**, **`vmulouw`** - Multiply odd unsigned

#### Floating-Point Arithmetic
- **`vaddfp`** - Add single-precision (4-way SIMD)
- **`vsubfp`** - Subtract single-precision
- **`vmaddfp`** - Multiply-add: `(a × c) + b`
- **`vnmsubfp`** - Negative multiply-subtract: `-((a × c) - b)`
- **`vmaxfp`**, **`vminfp`** - Maximum/minimum
- **`vrefp`** - Reciprocal estimate
- **`vrsqrtefp`** - Reciprocal square root estimate

#### Logical Operations
- **`vand`** - AND
- **`vandc`** - AND with complement
- **`vor`** - OR
- **`vnor`** - NOR
- **`vxor`** - XOR

#### Comparison
- **`vcmpequb`**, **`vcmpequh`**, **`vcmpequw`** - Compare equal
- **`vcmpgtsb`**, **`vcmpgtsh`**, **`vcmpgtsw`** - Compare greater than signed
- **`vcmpgtub`**, **`vcmpgtuh`**, **`vcmpgtuw`** - Compare greater than unsigned
- **`vcmpeqfp`**, **`vcmpgtfp`** - Compare equal/greater than floating-point

#### Shift and Rotate
- **`vslb`**, **`vslh`**, **`vslw`** - Shift left
- **`vsrb`**, **`vsrh`**, **`vsrw`** - Shift right
- **`vsrab`**, **`vsrah`**, **`vsraw`** - Shift right algebraic
- **`vrlb`**, **`vrlh`**, **`vrlw`** - Rotate left

#### Permute and Pack
- **`vperm`** - Vector permute (arbitrary byte shuffle)
- **`vsel`** - Vector select (bitwise multiplex)
- **`vmrghb`**, **`vmrghh`**, **`vmrghw`** - Merge high
- **`vmrglb`**, **`vmrglh`**, **`vmrglw`** - Merge low
- **`vpkuhum`**, **`vpkuwum`** - Pack (narrow with truncation)
- **`vpkuhus`**, **`vpkuwus`** - Pack unsigned saturate

#### Splat Operations
- **`vspltb`**, **`vsplth`**, **`vspltw`** - Splat element
- **`vspltisb`**, **`vspltish`**, **`vspltisw`** - Splat immediate

#### Conversion
- **`vcfsx`**, **`vcfux`** - Convert from signed/unsigned integer
- **`vctsxs`**, **`vctuxs`** - Convert to signed/unsigned integer with saturation

#### Load/Store
- **`lvx`** - Load vector indexed (16-byte aligned)
- **`stvx`** - Store vector indexed (16-byte aligned)
- **`lvsl`**, **`lvsr`** - Load vector for shift left/right (permute control)

### 6. System Instructions

#### Special Purpose Register Access
- **`mfspr`** - Move from SPR
- **`mtspr`** - Move to SPR

**Common SPRs:**
- SPR 1: XER (Fixed-Point Exception Register)
- SPR 8: LR (Link Register)
- SPR 9: CTR (Count Register)
- SPR 268/269: TB/TBU (Time Base)
- SPR 287: PVR (Processor Version Register) - returns `0x00700100` for Cell BE

#### Condition Register Operations
- **`mfcr`** - Move from CR (entire 32-bit CR)
- **`mfocrf`** - Move from one CR field
- **`mtcrf`** - Move to CR fields (selective)
- **`mtocrf`** - Move to one CR field
- **`mcrf`** - Move CR field to CR field

#### CR Logical Operations
- **`crand`** - CR AND
- **`cror`** - CR OR (also used as `crmove`)
- **`crxor`** - CR XOR (also used as `crclr` when ba==bb==bt)
- **`crnand`** - CR NAND
- **`crnor`** - CR NOR (also used as `crnot`)
- **`creqv`** - CR equivalent (XNOR)
- **`crandc`** - CR AND with complement
- **`crorc`** - CR OR with complement

#### Synchronization
- **`sync`** - Synchronize (full memory barrier)
- **`lwsync`** - Lightweight sync (acquire/release semantics)
- **`eieio`** - Enforce in-order execution of I/O
- **`isync`** - Instruction synchronize (context sync)

#### Cache Control (no-op in emulator)
- **`dcbt`** - Data cache block touch (prefetch hint)
- **`dcbst`** - Data cache block store
- **`dcbf`** - Data cache block flush
- **`icbi`** - Instruction cache block invalidate

#### System Call
- **`sc`** - System call
  - Transfers control to LV2 kernel
  - Syscall number in r11
  - Arguments in r3-r10
  - Return value in r3

#### FPSCR Operations
- **`mffs`** - Move from FPSCR
- **`mtfsf`** - Move to FPSCR fields
- **`mtfsfi`** - Move to FPSCR field immediate
- **`mtfsb0`**, **`mtfsb1`** - Move to FPSCR bit

#### Trap Instructions
- **`tw`** - Trap word (conditional trap)
- **`twi`** - Trap word immediate
- **`td`** - Trap doubleword
- **`tdi`** - Trap doubleword immediate

## Memory Model

### Endianness
The PPU operates in **big-endian** mode for PS3 compatibility. All memory accesses are big-endian:
- Multi-byte loads/stores use `read_be16`, `read_be32`, `read_be64`
- Instructions are fetched as big-endian 32-bit words

### Address Calculation
Load/store effective address (EA) calculation:
- **D-form**: `EA = (RA|0) + SIMM` (RA=0 means EA=SIMM)
- **X-form**: `EA = (RA|0) + RB`
- **DS-form**: `EA = (RA|0) + (SIMM & ~3)` (aligned to 8 bytes)

### Alignment
- Byte operations: No alignment required
- Halfword (2-byte): Should be 2-byte aligned (but emulator is lenient)
- Word (4-byte): Should be 4-byte aligned
- Doubleword (8-byte): Should be 8-byte aligned (required for ld/std)
- Quadword (16-byte): Must be 16-byte aligned (vector ops)

### Reservation System
The atomic operations (`lwarx`/`stwcx.`, `ldarx`/`stdcx.`) use a reservation mechanism:
1. `lwarx`/`ldarx` acquires a reservation on a cache line
2. Reservation is lost if:
   - Another processor writes to the reserved address
   - This processor writes to any address (except via `stwcx.`/`stdcx.`)
   - Context switch occurs
3. `stwcx.`/`stdcx.` succeeds only if reservation is still valid

The emulator implements this using the memory manager's reservation system with proper synchronization.

## Interpreter Features

### Execution Loop
The interpreter follows a standard fetch-decode-execute cycle:

```rust
loop {
    // 1. Check for breakpoints
    if should_break(thread) { return Breakpoint; }
    
    // 2. Fetch instruction (32-bit big-endian)
    let opcode = memory.read_be32(thread.pc())?;
    
    // 3. Decode instruction
    let decoded = PpuDecoder::decode(opcode);
    
    // 4. Execute instruction
    execute(thread, opcode, decoded)?;
    
    // 5. PC is updated by instruction (branch) or advanced by 4
}
```

### Performance
The interpreter includes several optimizations:
- **Hot path optimization**: D-form instructions (most common) have dedicated fast path
- **Inline execution**: Critical instructions are inlined in the interpreter
- **Instruction counting**: Tracks executed instructions for profiling
- **Expected performance**: >10 MIPS (Million Instructions Per Second) in interpreter mode

### Cycle Counting
The interpreter increments an instruction counter on each step, useful for:
- Performance profiling
- Conditional breakpoints based on instruction count
- Timing-sensitive debugging

### Condition Register Updates
Many instructions have a "Record" bit (Rc) that updates CR0 based on the result:
- **LT** (bit 0): Set if result is negative (bit 63 == 1)
- **GT** (bit 1): Set if result is positive (bit 63 == 0 and not zero)
- **EQ** (bit 2): Set if result is zero
- **SO** (bit 3): Copied from XER[SO]

## Debugging Support

### Breakpoints

The interpreter supports powerful breakpoint functionality:

#### Unconditional Breakpoints
```rust
interpreter.add_breakpoint(0x10000, BreakpointType::Unconditional);
```

#### Conditional Breakpoints
```rust
// Break when r3 == 42
interpreter.add_breakpoint(
    0x10000,
    BreakpointType::Conditional(
        BreakpointCondition::GprEquals { reg: 3, value: 42 }
    )
);

// Break after 1000 instructions
interpreter.add_breakpoint(
    0x10000,
    BreakpointType::Conditional(
        BreakpointCondition::InstructionCount { count: 1000 }
    )
);
```

### Single-Stepping
```rust
// Execute one instruction
interpreter.step(&mut thread)?;
```

### Register Inspection
```rust
// GPRs
let value = thread.gpr(3);
thread.set_gpr(3, new_value);

// FPRs
let value = thread.fpr(1);
thread.set_fpr(1, 3.14159);

// VRs
let value = thread.vr(0);
thread.set_vr(0, [0x12345678, 0x9ABCDEF0, 0x11111111, 0x22222222]);

// Special registers
let lr = thread.regs.lr;
let cr = thread.regs.cr;
let cr0 = thread.get_cr_field(0);
```

### Error Handling
The interpreter provides detailed error information:
- **InvalidInstruction**: Unknown or malformed instruction
- **MemoryError**: Access violation or invalid address
- **Breakpoint**: Execution stopped at breakpoint
- **SyscallFailed**: System call returned error

## Testing

The PPU implementation includes a comprehensive test suite with 75+ tests covering:

### Instruction Tests
- **D-form**: `addi`, `addis`, `lwz`, `stw`, `lbz`, `stb`, `ori`, `andi`, `cmpi`, `cmpli`
- **I-form**: `b`, `bl`, absolute and relative branches
- **B-form**: `bc`, `beq`, `blt`, `bgt`, conditional branches with CTR
- **X-form**: `and`, `or`, `xor`, `cmp`, `cmpl`, `lwzx`, `stwx`, `lwarx`, `stwcx.`
- **XO-form**: `add`, `subf`, `mullw`, `divw`, overflow detection
- **XL-form**: `bclr`, `bcctr`, CR logical operations
- **M-form**: `rlwinm`, `rlwimi`, `rlwnm`
- **A-form**: `fmadd`, `fmsub`, `fnmadd`, `fnmsub`
- **VA-form**: `vperm`, `vsel`, `vmaddfp`
- **VX-form**: `vaddsws`, `vand`, `vcmpequw`

### Edge Case Tests
- Integer overflow and wrapping
- Division by zero
- Signed vs unsigned comparisons
- Rotate mask generation
- Atomic operation failure/success
- Breakpoint conditions

### Integration Tests
- Atomic increment sequence
- Function call/return (bl/blr)
- Loop with CTR (bdnz)
- Vector permutation patterns

### Running Tests
```bash
# Run all PPU tests
cargo test --package oc-ppu --lib

# Run specific test
cargo test --package oc-ppu --lib test_addi_basic

# Run with verbose output
cargo test --package oc-ppu --lib -- --nocapture
```

## Integration with Other Components

### Memory Manager Integration
```rust
use oc_memory::MemoryManager;
use oc_ppu::{PpuThread, PpuInterpreter};

let memory = Arc::new(MemoryManager::new()?);
let mut thread = PpuThread::new(0, memory.clone());
let interpreter = PpuInterpreter::new(memory);

// Execute code
while thread.is_running() {
    interpreter.step(&mut thread)?;
}
```

### LV2 Kernel Integration
System calls are intercepted by checking for the `sc` instruction:
```rust
// In the interpreter, sc instruction:
let syscall_num = thread.gpr(11);
// Forward to LV2 kernel for handling
```

### Multi-Threading
Each PPU thread has its own `PpuThread` instance:
- Independent register state
- Shared memory via `Arc<MemoryManager>`
- Atomic operations synchronized via reservation system

## Implementation Notes

### Design Decisions

1. **Interpreter-First Approach**: 
   - Focus on correctness over speed
   - JIT compilation deferred to Phase 10
   - Easier debugging and validation

2. **Error Handling**:
   - Invalid instructions logged but don't crash
   - Graceful handling of edge cases (divide by zero, overflow)
   - Detailed error context for debugging

3. **Big-Endian Everywhere**:
   - All memory operations use big-endian
   - Matches PS3 hardware behavior
   - Simplifies integration with game code

4. **Atomic Operations**:
   - Full implementation of lwarx/stwcx reservation system
   - Critical for multi-threaded games
   - Integrated with memory manager Phase 2

### Known Limitations

1. **No Privileged Instructions**: 
   - Many supervisor-level instructions are no-ops or return fixed values
   - Sufficient for game emulation (runs in user mode)

2. **Simplified Exception Model**:
   - No floating-point exceptions by default
   - Traps not fully implemented
   - Syscalls used instead of interrupts

3. **Cache Hints Ignored**:
   - `dcbt`, `dcbf`, `icbi` are no-ops
   - Host caching used instead
   - Doesn't affect correctness

4. **Time Base Approximation**:
   - TB/TBU use system time, not cycle-accurate
   - Sufficient for most timing needs
   - May cause issues with precise timing code

### Future Enhancements

1. **JIT Compilation** (Phase 10):
   - Translate hot instruction sequences to native code
   - Target 100+ MIPS performance
   - Maintain interpreter for rarely-executed code

2. **Cycle-Accurate Timing**:
   - Track instruction cycles
   - Emulate pipeline stalls
   - Better timing for CPU-intensive code

3. **Advanced Breakpoints**:
   - Memory watchpoints
   - Conditional expressions
   - Call stack inspection

4. **Trace Recording**:
   - Record instruction traces
   - Replay for debugging
   - Performance analysis

## References

### Documentation
- **PowerISA v2.06**: Official PowerPC instruction set architecture
- **Cell BE Programming Handbook**: IBM's guide to Cell programming
- **PS3 SDK Documentation**: Official PlayStation 3 development documentation

### Related Code
- **RPCS3**: Reference implementation (GPL-3.0 compatible)
- **oc-memory**: Memory manager with reservation system (Phase 2)
- **oc-lv2**: LV2 kernel syscall handling
- **oc-loader**: ELF/SELF loader for executable files

### External Resources
- [PowerPC Architecture Book](https://wiki.osdev.org/PowerPC)
- [AltiVec Programming Guide](https://developer.apple.com/documentation/accelerate/simd)
- [PS3 Architecture](https://www.psdevwiki.com/)

## Conclusion

The PPU implementation provides a complete, tested PowerPC 64-bit interpreter suitable for PS3 emulation. With 75+ passing tests, comprehensive instruction coverage, and integration with the memory manager, it forms a solid foundation for running PS3 games.

The implementation prioritizes correctness and completeness, with performance optimizations planned for the JIT phase. The extensive debugging support makes it easy to track down issues in game code and emulator implementation.

For questions or contributions, see the main repository README.md.
