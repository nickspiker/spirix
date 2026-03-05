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

### Test Harness

- **PRNG:** 64-bit Galois LFSR, taps x^64+x^63+x^61+x^60 (1 LUT critical path)
- **Protocol:** 10-bit counter, bit[7]=accumulate, bit[9]=done (zero comparisons)
- **Accumulator:** rotate-XOR: `{accum[30:0],accum[31]} ^ result[31:0]`
- **CE:** registered lookahead for clean fanout at high frequencies
- **Harness ceiling:** ~500 MHz (LFSR-only bypass test)
