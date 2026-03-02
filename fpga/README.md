# Spirix FPGA Implementation

Hardware implementation of Spirix floating-point format for FPGAs.

## Directory Structure

```
fpga/
├── rtl/              # RTL (Verilog) modules
│   └── spirix_subtract_f4e4.v    # ScalarF4E4 subtraction (combinational)
├── testbench/        # Simulation testbenches
│   └── test_spirix_subtract_f4e4.v
├── sim/              # Simulation scripts and waveforms
└── doc/              # Documentation
```

## Quick Start

### Prerequisites

Install Icarus Verilog (free, open-source):
```bash
# Ubuntu/Debian
sudo apt-get install iverilog gtkwave

# macOS
brew install icarus-verilog gtkwave

# Fedora
sudo dnf install iverilog gtkwave
```

### Running the Testbench

```bash
cd /mnt/Octopus/Code/spirix/fpga

# Compile and run testbench
iverilog -o sim/test_subtract \
    rtl/spirix_subtract_f4e4.v \
    testbench/test_spirix_subtract_f4e4.v

# Run simulation
vvp sim/test_subtract

# View waveforms (if you add $dumpfile in testbench)
gtkwave sim/test_subtract.vcd
```

## Current Status

### Implemented Modules

- ✅ **spirix_subtract_f4e4.v** - ScalarF4E4 subtraction (combinational)
  - Direct translation from Rust implementation
  - Handles exponent alignment
  - Two's complement subtraction
  - Automatic normalization
  - Underflow detection

### TODO

- ⏳ **spirix_add_f4e4.v** - Addition (trivial: subtraction with negated operand)
- ⏳ **spirix_multiply_f4e4.v** - Multiplication
- ⏳ **spirix_divide_f4e4.v** - Division (Newton-Raphson with LUT)
- ⏳ **spirix_sqrt_f4e4.v** - Square root
- ⏳ Pipelined versions for higher throughput
- ⏳ Other bit widths (F3E3, F5E5, F6E6, F7E7)

## Module: spirix_subtract_f4e4

### Interface

```verilog
module spirix_subtract_f4e4 #(
    parameter ROUNDING_MODE = 0  // 0=floor (free), 1=ceiling, 2=nearest
) (
    input  wire signed [15:0] a_frac,      // Operand A fraction
    input  wire signed [15:0] a_exp,       // Operand A exponent
    input  wire signed [15:0] b_frac,      // Operand B fraction
    input  wire signed [15:0] b_exp,       // Operand B exponent
    output reg  signed [15:0] result_frac, // Result fraction
    output reg  signed [15:0] result_exp   // Result exponent
);
```

### Rounding Modes

**ROUNDING_MODE = 0: Floor (toward -∞)** - Default
- **Cost**: FREE (arithmetic right shift naturally floors)
- **Behavior**: Always rounds down on the number line
- **Use case**: Default, fastest, no extra hardware, matches integer behavior on most platforms

**ROUNDING_MODE = 1: Ceiling (toward +∞)**
- **Cost**: +1 incrementer (~22 LUTs, 2-3ns)
- **Behavior**: Always rounds up on the number line
- **Use case**: Interval arithmetic upper bounds

**ROUNDING_MODE = 2: Nearest (ties to even)**
- **Cost**: +1 incrementer (~22 LUTs, 2-3ns)
- **Behavior**: Round to nearest
- **Use case**: Scientific computing, statistics

**ROUNDING_MODE = 3: Stochastic (probabilistic, unbiased)**
- **Cost**: +1 comparator + 16-bit Galois LFSR (~40 LUTs, 16 FFs)
- **Behavior**: Rounds up with probability equal to fractional remainder
- **Convergence**: Unbiased - expected value matches true value after many operations
- **Use case**: Neural network training, gradient descent, low-precision ML
- **Note**: Requires clock and reset (sequential, not combinational)

### Features

- **Modes 0-2: Combinational** (single-cycle, no clock needed)
- **Mode 3: Sequential** (clocked, requires clk and rst inputs)
- **17-bit internal arithmetic** (fraction + 1 carry bit)
- **Automatic normalization** using leading zero count
- **Underflow detection** and vanished state handling
- **Optimized shortcuts** for large exponent differences

### Timing

- **Combinational delay**: ~10-15ns on typical FPGA
- **For pipelined version**: See spirix_subtract_f4e4_pipe.v (TODO)

### Resource Usage (Estimated)

On Xilinx 7-series FPGA:
- **LUTs**: ~200-300 (floor) / ~220-320 (ceiling/nearest) / ~240-340 (stochastic)
- **FFs**: 0 (modes 0-2) / 16 (mode 3 - LFSR state)
- **DSPs**: 0
- **Block RAM**: 0

### Instantiation Examples

```verilog
// Default floor rounding (free, fastest)
spirix_subtract_f4e4 subtract_floor (
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .result_frac(result_frac), .result_exp(result_exp)
);

// Ceiling rounding
spirix_subtract_f4e4 #(.ROUNDING_MODE(1)) subtract_ceiling (
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .result_frac(result_frac), .result_exp(result_exp)
);

// Nearest rounding (best accuracy)
spirix_subtract_f4e4 #(.ROUNDING_MODE(2)) subtract_nearest (
    .clk(clk), .rst(rst),  // Can connect but ignored in modes 0-2
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .result_frac(result_frac), .result_exp(result_exp)
);

// Stochastic rounding (unbiased, for ML training)
spirix_subtract_f4e4 #(
    .ROUNDING_MODE(3),
    .LFSR_SEED(16'hACE1)  // Non-zero seed (default is fine)
) subtract_stochastic (
    .clk(clk), .rst(rst),  // Required for stochastic mode!
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .result_frac(result_frac), .result_exp(result_exp)
);
```

## Design Notes

### Why Only 17 Bits?

Unlike IEEE-754 which needs double-width intermediates (48 bits for float32), Spirix only needs **fraction_width + 1** for carry:

```
IEEE float32:    24 × 24 = 48-bit intermediate
Spirix F4E4:     16-bit fraction + 1 carry = 17 bits
```

This is a **huge hardware advantage**:
- Smaller adders (17-bit vs 48-bit)
- Faster critical path
- Lower power consumption
- Easier to pipeline

### Two's Complement vs Sign-Magnitude

Spirix uses two's complement fractions, so we need **real subtraction**:
```verilog
sub_result = big - small;  // Actual subtraction, not just sign flip
```

IEEE uses sign-magnitude, so subtraction is just addition with sign flip:
```verilog
effSignB = subOp ? !signB : signB;  // Just flip a bit!
```

Trade-off: Spirix subtraction is more complex, but comparison/zero-check is simpler.

### Rounding Simplicity

Spirix's uniform two's complement number line makes rounding **much simpler than IEEE**:

**Spirix**: Single incrementer for all modes, no sign checks
```verilog
needs_increment = (mode == 1) ? |truncated_bits :              // Ceiling
                  (mode == 2) ? truncated_bits[15] :           // Nearest
                  (mode == 3) ? (lfsr < truncated_bits) :      // Stochastic
                  1'b0;                                         // Floor (free)
result_frac = base_frac + needs_increment;  // One adder, uniform behavior
```

**IEEE**: Asymmetric with guard/round/sticky bits
```verilog
// Different logic for positive vs negative
// Multiple special cases for denormals
// Guard, round, and sticky bit tracking
// Tie-breaking for round-to-even
// Much more complex!
```

**Why Spirix is simpler**:
- ✅ Arithmetic right shift naturally floors (free)
- ✅ Same increment logic for positive and negative
- ✅ No guard/round/sticky bit tracking needed
- ✅ 32-bit headroom prevents overflow during rounding
- ✅ ~22 LUTs for ceiling/nearest (vs IEEE's 100+ LUTs)

### Stochastic Rounding Convergence

**How stochastic rounding achieves unbiased convergence:**

When we truncate bits, the lower 16 bits (`truncated_bits`) represent the fractional remainder as a value from 0-65535:
```
Probability(round up) = truncated_bits / 65536
```

**Example**: If we truncate 0.7 (truncated_bits ≈ 45875):
- 70% of operations: round up (+1 ULP error)
- 30% of operations: round down (0 ULP error)
- **Expected error**: 0.7 × (+1) + 0.3 × (0) = **+0.7 ULP** ← matches true fractional value!

**Implementation:**
```verilog
needs_increment = (lfsr_state < truncated_bits);
```

Since `lfsr_state` is uniformly distributed [0, 65535], this comparison naturally creates probability proportional to the fractional remainder. **No manual bias needed!**

**Why this matters for ML:**
- Deterministic rounding (floor/ceiling) accumulates systematic bias over millions of gradient updates
- Stochastic rounding is unbiased: errors cancel out to zero over many operations
- Critical for training neural networks in low-precision (F4E4, F7E7) where rounding errors dominate
- Veritas neural network training benefits significantly from this property

**Galois LFSR properties:**
- Maximal period: 2^16 - 1 (all non-zero states)
- Uniform distribution over full cycle
- Fast: 1 XOR gate in critical path
- Small: ~20 LUTs for 16-bit version
- Standard polynomial: x^16 + x^14 + x^13 + x^11 + 1

### Normalization

The leading zero/one counter is the critical path. For better performance:
1. Use FPGA primitives if available (e.g., Xilinx LUT6)
2. Pipeline this stage
3. Or use a tree structure for faster counting

### Comparison to GPU Implementation

This Verilog is a **direct translation** of the GPU HIP kernel:
- Same algorithm (exponent align → subtract → normalize)
- Same bit widths (17-bit intermediate)
- Same special case handling

**Differences**:
- GPU uses `__clz()` intrinsic, Verilog uses combinational function
- GPU has explicit thread handling, Verilog is pure dataflow
- GPU is pipelined by nature, this version is combinational

## Next Steps

### 1. Test the Module (Now)

```bash
cd /mnt/Octopus/Code/spirix/fpga
iverilog -o sim/test_subtract rtl/spirix_subtract_f4e4.v testbench/test_spirix_subtract_f4e4.v
vvp sim/test_subtract
```

### 2. Add More Test Cases

Expand the testbench with your comprehensive division tests:
- All 170 test cases from `veritas/examples/comprehensive_division_tests.rs`
- Generate Verilog testbench from Rust test suite

### 3. Synthesize for Your FPGA

Once you have your FPGA board:
```bash
# For Xilinx (Vivado)
vivado -mode tcl -source synth_spirix.tcl

# For Lattice (open-source)
yosys -p "synth_ecp5 -top spirix_subtract_f4e4 -json spirix.json" rtl/spirix_subtract_f4e4.v
nextpnr-ecp5 --json spirix.json --lpf pins.lpf --textcfg spirix.config
```

### 4. Create Pipelined Version

For high throughput (1 operation per clock):
- Stage 1: Exponent comparison
- Stage 2: Fraction alignment
- Stage 3: Subtraction
- Stage 4: Normalization

## References

- **Rust Implementation**: `spirix/src/implementations/subtraction/scalar_scalar.rs`
- **GPU Implementation**: `spirix/gpu/hip/kernels/scalar_ops.hip`
- **HardFloat**: `HardFloat/source/addRecFN.v` (for reference, but IEEE-specific)

## License

Same as Spirix library (see root LICENSE file)
