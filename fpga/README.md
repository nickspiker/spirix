# Spirix FPGA Implementation

Hardware implementation of Spirix floating-point arithmetic for ECP5 FPGAs.

## Directory Structure

```
fpga/
├── bench/            # Benchmark modules (single-width, binary32-equivalent)
│   ├── spirix_addsub.v          # Add/Sub (combinational, 842 LUT4, 95 MHz)
│   ├── spirix_addsub_pipe2.v    # Add/Sub (2-stage, 147 MHz)
│   ├── spirix_multiply.v        # Multiply (combinational, 227 LUT4/4 DSP, 115 MHz)
│   ├── spirix_multiply_pipe2.v  # Multiply (2-stage, 181 MHz)
│   ├── spirix_fma.v             # FMA (combinational, 1472 LUT4/3 DSP, 63 MHz)
│   ├── spirix_divide.v          # Divide (combinational, 11 MHz — artifact)
│   ├── spirix_divide_iter.v     # Divide (iterative, 535 LUT4, 234 MHz)
│   ├── spirix_divmod_nr.v       # Div/Mod Newton-Raphson (8-stage, 20 DSP, 120 MHz)
│   ├── spirix_sqrt.v            # Sqrt (combinational)
│   ├── spirix_sqrt_iter.v       # Sqrt (iterative, 101 LUT4, 400+ MHz)
│   ├── spirix_nr_divsqrt.v      # Sqrt Newton-Raphson (10-stage, 27 DSP, 125 MHz)
│   ├── top_ntsc.v               # Self-test harness + NTSC CRT display
│   ├── top_trng_alu.v           # TRNG OLED demo (button → random scalar)
│   ├── ssd1306_oled.v           # SH1106 OLED controller (128x64, I2C)
│   └── ssd1306_i2c.v            # I2C bit-bang driver
├── cores/ops/        # Multi-width ALU modules (see cores/ops/README.md)
│   ├── spirix_alu_basic.v       # NEG, ABS, SIGN, SHL, SHR (231 MHz)
│   ├── spirix_alu_minmax.v      # MIN, MAX (208 MHz)
│   ├── spirix_alu_addbit.v      # ADD, SUB, AND, OR, XOR (108 MHz)
│   ├── spirix_alu_addbit_pipe.v # ADD, SUB, AND, OR, XOR (201 MHz, 2-stage)
│   ├── spirix_alu_round.v       # FLOOR, CEIL, ROUND, FRAC (>=500 MHz)
│   ├── spirix_alu_multiply.v    # MUL (~95 MHz, no DSP)
│   ├── spirix_alu_multiply_pipe.v # MUL (170 MHz, 2-stage, no DSP)
│   ├── spirix_alu_divmodsqrt.v  # DIV, SQRT, MOD (188 MHz, iterative)
│   ├── spirix_alu_random.v      # RANDOM (>=800 MHz, 72-RO TRNG)
│   ├── spirix_neg.v             # Negate primitive
│   ├── spirix_cmp.v             # Compare primitive
│   ├── spirix_abs.v             # Absolute value primitive
│   └── spirix_floor.v           # Floor primitive
├── rtl/              # Legacy single-width modules
├── constraints/      # Pin constraints (Colorlight 5A-75B v8.0)
├── scripts/          # Build scripts
│   └── build_ntsc.sh           # Synthesize + place&route + program
├── tools/            # Font/bitmap generators
└── build/            # Build artifacts
```

## Quick Start

### Prerequisites

```bash
# ECP5 toolchain (Yosys + nextpnr + prjtrellis)
sudo dnf install yosys nextpnr trellis   # Fedora
# or build from source: https://github.com/YosysHQ/oss-cad-suite-build

# Simulation
sudo dnf install iverilog
```

### Build & Flash

```bash
# Build at 25 MHz (no PLL) and program
DUT=spirix_addsub SEED=4 bash fpga/scripts/build_ntsc.sh 25 --program

# Build at target frequency (PLL) and program
DUT=spirix_addsub SEED=4 bash fpga/scripts/build_ntsc.sh 95 --program

# TRNG OLED demo
DUT=spirix_random SEED=4 bash fpga/scripts/build_ntsc.sh 25 --program
```

### Available DUTs

**Bench modules** (single-width, FRAC=25/EXP=8, binary32-equivalent):
`spirix_addsub`, `spirix_addsub_pipe2`, `spirix_mul`, `spirix_mul_pipe2`,
`spirix_fma`, `spirix_div_iter`, `spirix_divmod_nr`, `spirix_sqrt_nr`,
`spirix_sqrt_iter`

**Ops modules** (multi-width, runtime-selectable 8/16/32/64-bit):
`spirix_basic`, `spirix_minmax`, `spirix_addbit`, `spirix_addbit_pipe`,
`spirix_round`, `spirix_mul_ops`, `spirix_mul_ops_pipe`, `spirix_divmodsqrt`, `spirix_random`

**Competitors** (IEEE 754 binary32):
`hf_add`, `hf_mul`, `hf_fma`, `hf_div`, `hf_sqrt` (HardFloat),
`fpn_add`, `fpn_mul`, `fpn_fma`, `fpn_div`, `fpn_sqrt` (FPnew/cvfpu)

## Silicon Results (ECP5-25F speed-6, Colorlight 5A-75B v8.0)

All Fmax values are real silicon measurements via CE-gated self-test,
not static timing estimates. Consistent ~2-2.5x margin over nextpnr estimates.

### Spirix Ops (Multi-Width ALU, 21 ops total)

| Module | Ops | LUT4 | DSP | Silicon Fmax | Latency |
|--------|-----|------|-----|-------------|---------|
| basic | NEG/ABS/SIGN/SHL/SHR | 3,900 | 0 | 231 MHz | 1 clk |
| minmax | MIN/MAX | 2,127 | 0 | 208 MHz | 1 clk |
| addbit | ADD/SUB/AND/OR/XOR | 6,341 | 0 | 108 MHz | 1 clk |
| addbit_pipe | ADD/SUB/AND/OR/XOR | ~6,700 | 0 | 201 MHz | 2 clk |
| round | FLOOR/CEIL/ROUND/FRAC | 4,076 | 0 | >=500 MHz* | 1 clk |
| multiply | MUL | ~2,131 | 0 | ~95 MHz | 1 clk |
| multiply_pipe | MUL | ~2,666 | 0 | 170 MHz | 2 clk |
| divmodsqrt | DIV/SQRT/MOD | 5,433 | 0 | 188 MHz | ~F+2 clk |
| random | RANDOM (TRNG) | ~487 | 0 | >=800 MHz | 2 clk |

*Harness-limited (passes at harness ceiling).

### Spirix Bench (Binary32-Equivalent, Single-Width)

| Op | LUT4 | DSP | Silicon Fmax |
|----------|------|-----|-------------|
| Add/Sub | 842 | 0 | 95 MHz |
| Add/Sub 2-stage | — | 0 | 147 MHz |
| Multiply | 227 | 4 | 115 MHz |
| Multiply 2-stage | — | 4 | 181 MHz |
| FMA | 1,472 | 3 | 63 MHz |
| Divide (iter) | 535 | 0 | 234 MHz |
| Div NR (8-stage) | — | 20 | 120 MHz |
| Sqrt (iter) | 101 | 0 | 400+ MHz |
| Sqrt NR (10-stage) | — | 27 | 125 MHz |

### vs HardFloat (IEEE 754, with DSP)

| Op | Spirix LUT4 | Spirix Fmax | HF LUT4 | HF Fmax |
|----------|-------------|-------------|----------|---------|
| Add/Sub | 842 | 95 MHz | 1,050 | 88 MHz |
| Multiply | 227 | 115 MHz | 786 | 65 MHz |
| FMA | 1,472 | 63 MHz | 2,057 | 47 MHz |

### vs FPnew (IEEE 754, no DSP)

| Op | Spirix LUT4 | Spirix Fmax | FPnew LUT4 | FPnew Fmax |
|----------|-------------|-------------|------------|------------|
| Add/Sub | 842 | 95 MHz | 825 | 74 MHz |
| Multiply | 2,131 | 95 MHz | 2,850 | 74 MHz |
| FMA | 3,004 | 53 MHz | 2,850 | 25 MHz |

## Hardware TRNG (`spirix_alu_random`)

True Random Number Generator using ring oscillator jitter. First-class ALU operation
producing N1-normalized scalars in (-1, 1).

**Architecture:**
- 72 ring oscillators with frequency diversity (24x A-pin ~1.4 GHz, 24x B-pin ~1.25 GHz, 24x C-pin ~1.1 GHz)
- Temporal XOR: sample all 72 ROs, XOR against previous sample to extract jitter
- BLAKE3-inspired rotation mix: `mixed[i] = jitter[i] ^ jitter[(i+9)%72] ^ jitter[(i+31)%72]`
- Sign-aware CLZ normalize on full 72 bits, truncate to 64-bit output
- ROs run continuously after first activation; entropy stays fresh between operations

**Timing:** 2-clock latency (warm), 4-clock cold start. ~487 LUT4, 0 DSP. >=800 MHz silicon.

## Hardware Self-Test

Single-instance CE-gated test: run DUT twice (gold at CE=1/256, test at full PLL speed),
compare 32-bit rotate-XOR accumulators. 64-bit Galois LFSR for PRNG inputs.

Results displayed on NTSC CRT (320x240 @ 1bpp) and LED
(RUN=50% blink, PASS=7/8 duty, FAIL=1/8 duty).

```bash
# Find silicon Fmax by binary search
SEED=4 bash fpga/scripts/build_ntsc.sh 100 --program  # start
SEED=4 bash fpga/scripts/build_ntsc.sh 200 --program  # binary search up
```

## Design Notes

- **Two's complement fractions** — real subtraction required, but comparison/zero-check is simpler than IEEE sign-magnitude
- **Karatsuba multiply** — 3 sub-multiplies vs 4 naive. With DSP: 3 MULT18X18D. No DSP: 2131 LUT4.
- **Edge cases** — all modules handle zero/infinity/exploded/vanished/undefined per Spirix spec