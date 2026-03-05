# Spirix FPGA Silicon Fmax Report

**Target:** Colorlight 5A-75B v8.0 (Lattice ECP5-25F, speed grade 6)
**Method:** CE-gated single-instance test harness, 64-bit Galois LFSR PRNG,
rotate-XOR accumulator, bit-tap protocol. Gold run (CE=1/256) then test run
(CE=1, full PLL speed). PASS/FAIL on CRT via NTSC framebuffer.
**Seed:** nextpnr --seed 4 (best of tested seeds).
**Date:** 2026-03-05

## Results — Binary32 Equivalent (FRAC=25, EXP=8)

All counts from `yosys synth_ecp5 -nowidelut` (pure LUT4, no wide LUTs).
Silicon Fmax from binary search on real hardware.

### Arithmetic Units

| Module | Stages | LUT4 | FF | DSP | Silicon Fmax | nextpnr est. | Margin | Throughput |
|----------------------|--------|------|-----|-----|-------------|-------------|--------|------------|
| addsub | 1 | 608 | 100 | 0 | 95 MHz | 26 MHz | 3.7x | 95 Mop/s |
| addsub_pipe2 | 2 | 657 | 107 | 0 | 148 MHz | ~65 MHz | 2.3x | 148 Mop/s |
| multiply | 1 | 94 | 99 | 4 | 115 MHz | 43 MHz | 2.7x | 115 Mop/s |
| multiply_pipe2 | 2 | 99 | 91 | 4 | 181 MHz | ~78 MHz | 2.3x | 181 Mop/s |
| divide (comb) | 1 | 7356 | - | 5 | 11 MHz | 3.5 MHz | 3.2x | 11 Mop/s |
| divide_iter | 25 cyc | 377 | 133 | 0 | 153 MHz | ~70 MHz | 2.2x | 6.1 Mop/s |
| divmod_nr (div only) | 6 | 547 | 489 | 20 | 105 MHz | 42 MHz | 2.5x | 105 Mop/s |
| divmod_nr (div+mod) | 8 | 1415 | 802 | 24 | 98 MHz | 39 MHz | 2.5x | 98 Mop/s |
| sqrt_nr | 10 | 921 | 663 | 27 | 125 MHz | 55 MHz | 2.3x | 125 Mop/s |
| fma | 1 | 1130 | - | 4 | 63 MHz | 20 MHz | 3.2x | 63 Mop/s |

### Notes

- **Consistent 2.2-3.5x margin** over nextpnr static timing estimates on
  ECP5-25F speed-6. Routing dominates critical path at high frequencies
  (~77% of delay).

- **divide (comb)** is a fully unrolled 25-bit long divider — 7356 LUT4
  in one combinational blob. Runs at 11 MHz on silicon (3.2x over nextpnr's
  3.5 MHz estimate). A fun single-clock artifact.

- **multiply** is extremely compact: 94 LUT4 + 4 DSP18. The DSP18 blocks
  handle the 25x25 signed multiply; surrounding logic is just normalize +
  round + exponent.

- **divide_iter** has the best Fmax of any divider but is multi-cycle
  (25 clocks per result), giving ~6.1 Mop/s effective throughput.
  **divmod_nr** is pipelined (1 result/clock) at 105 Mop/s — 17x higher
  throughput at the cost of 20 DSP18.

- **sqrt_nr** uses 27 of 28 DSP18 on ECP5-25F (96%). Requires ECP5-45F+
  if combined with divmod_nr (47 DSP total).

- **fma** is a single-stage fused multiply-add: 1130 LUT4 + 4 DSP18.
  Pre-aligned architecture: C alignment runs in parallel with the DSP
  multiply (exponent diff computed from raw a_exp+b_exp). Close threshold
  widened to |raw_diff|<=2, far normalize widened to 0-3 bits to absorb
  the un-normalized product (N1 or N2). 26% faster than post-DSP normalize.

- **Pipeline splitting** was tested on divmod_nr:
  - 6-stage (original): 105 MHz
  - 7-stage (S6 remainder correction split only): 106 MHz (+1%, marginal)
  - 8-stage (S2 cascade split + S6 split): 120 MHz (+14%, both needed)
  - Splits compound — neither helps alone, but together they break two
    independent bottlenecks.

### Spirix vs HardFloat vs FPnew — IEEE 754 End-to-End Comparison

Binary32-equivalent precision. Pure LUT4 (`-nowidelut`). All wrapped in
registered input/output benches. Silicon Fmax from binary search on real
hardware (Colorlight 5A-75B, seed=4).

- **Spirix:** FRAC=25, EXP=8, two's complement, N1-normalized, native IEEE binary32.
- **HardFloat:** expWidth=8, sigWidth=24, recoded format + fNToRecFN/recFNToFN IEEE converters.
- **FPnew (cvfpu):** ETH Zürich PULP platform FPU. Native IEEE 754, unified FMA datapath.
  Converted from SystemVerilog via sv2v for Yosys compatibility.

#### FPGA (with DSP blocks)

| Operation | Spirix LUT4 | DSP | Fmax | HF+IEEE LUT4 | DSP | Fmax | FPnew LUT4 | DSP | Fmax |
|-----------|-------------|-----|----------|---------------|-----|----------|------------|-----|----------|
| Add/Sub | 618 | 0 | **95 MHz** | 1050 | 0 | 88 MHz | 825 | 0 | 74 MHz |
| Multiply | 94 | 4 | **115 MHz** | 786 | 4 | 65 MHz | 574 | 0 | 74 MHz |
| FMA | 1128 | 4 | **63 MHz** | 2057 | 4 | 47 MHz | 1283 | 0 | 25 MHz |

#### ASIC-equivalent (no DSP, `-nodsp -nowidelut`)

| Operation | Spirix LUT4 | Fmax | FPnew LUT4 | Fmax | HF+IEEE LUT4 |
|-----------|-------------|----------|------------|----------|---------------|
| Add/Sub | **618** | **95 MHz** | 825 | 74 MHz | 1050 |
| Multiply | 1956 | - | **574** | 74 MHz | 786 |
| FMA | 3045 | **57 MHz** | **1283** | 25 MHz | 2057 |

Note: Spirix add/sub never uses DSPs, so its no-DSP Fmax is the same 95 MHz.
Spirix FMA no-DSP silicon verified: 57 pass, 58 fail (nextpnr est. ~20 MHz, 2.9x margin).

**Spirix wins silicon Fmax on every operation** in FPGA deployment.

In the ASIC-equivalent (no DSP) comparison, FPnew has the smallest multiply
and FMA gate count — its pure-LUT multiplier was designed for standard-cell
synthesis. Spirix wins add/sub (no multipliers involved). However, FPnew's
area advantage comes at a severe speed cost: its FMA runs at only 25 MHz
(2.5x slower than Spirix, 1.9x slower than HardFloat).

Key observations:

- **Multiply** is Spirix's standout on FPGA: 94 LUT4 + 4 DSP18 vs FPnew's
  574 LUT4 + 0 DSP. On an ASIC (no DSP), Spirix balloons to 1956 LUT4 —
  the DSP18 blocks were doing all the heavy lifting.

- **FPnew add ≈ FPnew mul** at 74 MHz — both go through the same unified
  FMA datapath, so the constant operand (b=1.0 for add, c=0 for mul)
  doesn't meaningfully shorten the critical path.

- **FPnew FMA at 25 MHz** is limited by the pure-LUT 24×24 multiplier.
  This is the full FMA datapath with no DSP assistance — the ASIC use case.

- **Add/Sub** is the closest race: Spirix 95, HardFloat 88, FPnew 74 MHz.
  All three use pure logic (no DSP), so ASIC LUT counts are the same as
  FPGA counts.

Apples-to-apples caveats:
- HardFloat includes full IEEE special-case handling (NaN, Inf, signed zero,
  all rounding modes, subnormal support). FPnew has the same. Spirix has
  none of this — only ambiguous-exponent for zero/underflow and RNE rounding.
- HardFloat's recoded format (33-bit) is designed for chained operations
  without conversion overhead. In a pipeline staying in recoded format,
  the converter cost is amortized. These numbers represent the worst case
  for HardFloat (single-op, IEEE in/out).
- FPnew uses a unified FMA unit for all ops (add via b=1.0, mul via c=0).
  This means add and mul inherit full FMA complexity, penalizing simple ops.
- LUT4 counts from `yosys synth_ecp5 -nowidelut`. Silicon Fmax from binary
  search on Colorlight 5A-75B, seed=4.

### HardFloat Core-Only Reference (no IEEE converters)

For comparison, HardFloat core modules in their native recoded format:

| Operation | HF Core LUT4 | DSP | Fmax (est) |
|-----------|-------------|-----|------------|
| addRecFN | 546 | 0 | 38 MHz |
| mulRecFN | 282 | 4 | 41 MHz |
| divSqrtRecFN (div) | 407 | 0 | 96 MHz |
| divSqrtRecFN (sqrt) | 393 | 0 | 76 MHz |
| mulAddRecFN | 1344 | 4 | 21 MHz |

Note: Spirix divide_iter is multi-cycle (25 clocks); HardFloat divSqrtRecFN is
also multi-cycle (~26 clocks). Both have similar throughput (~6 Mop/s).

### Test Harness

- **PRNG:** 64-bit Galois LFSR, taps x^64+x^63+x^61+x^60 (1 LUT critical path)
- **Protocol:** 10-bit counter, bit[7]=accumulate, bit[9]=done (zero comparisons)
- **Accumulator:** rotate-XOR: `{accum[30:0],accum[31]} ^ result[31:0]`
- **CE:** registered lookahead for clean fanout at high frequencies
- **Harness ceiling:** ~500 MHz (LFSR-only bypass test)
