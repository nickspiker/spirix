# Silicon-Verified Two's Complement Floating-Point Arithmetic on FPGA

**Nick Spiker**

## Abstract

I present silicon-verified FPGA implementations of two's complement floating-point add, multiply, and FMA at binary32-equivalent precision (25-bit fraction, 8-bit exponent), compared head-to-head against Berkeley HardFloat and ETH Zurich FPnew on identical hardware (Lattice ECP5-25F). True silicon Fmax is measured via a clock-enable-gated self-test harness rather than static timing estimates, which I show underestimate by 1.5--3.7x on ECP5.

**With DSP:** add 95 MHz / 842 LUT4 (vs HardFloat 88 / 1050), multiply 115 MHz / 227 LUT4 / 4 DSP (vs 65 / 786 / 4), FMA 63 MHz / 1472 LUT4 / 3 DSP (vs 47 / 2057 / 4). **Without DSP:** add 95 / 842 (vs FPnew 74 / 825), multiply 95 / 2131 (vs 74 / 2850), FMA 53 / 3004 (vs 25 / 2850). Spirix wins speed on every operation in both configurations. All Spirix numbers include full spec-compliant edge case handling.

---

## 1. Introduction

Numbers are represented as a signed two's complement fraction paired with an unbiased signed two's complement exponent:

$$v = \frac{f}{2^{n-1}} \times 2^{e}$$

where *f* is an *n*-bit signed fraction and *e* is an *m*-bit signed exponent. No sign bit, no implicit leading one, no exponent bias, no positive/negative zero distinction.

**N-1 normalization**: the two MSBs of a normal fraction always differ (`01...` positive, `10...` negative) -- the sign can be determined by checking `N0`. A single reserved exponent $e_{\min} = -2^{m-1}$ (the "ambiguous exponent") encodes all non-normal states: zero, infinity, overflow ("exploded"), underflow ("vanished"), and undefined results. This replaces IEEE 754's five special categories with one sentinel.

Properties preserved:
- **Multiplicative identity**: $a \times b = 0 \iff a = 0 \lor b = 0$
- **Subtractive identity**: $a - b = 0 \iff a = b$
- **No denormals**: underflow goes to "vanished" (preserves sign) rather than a reduced-precision denormalized regime
- **Single zero**: no +0/-0 distinction

For these comparisons I used $n = 25$, $m = 8$ (binary32-equivalent) thruout.

### Contributions

1. Open-source Verilog implementations of two's complement FP add, multiply, and FMA
2. A CE-gated self-test harness that measures true silicon Fmax, consistently 2--3.7x higher than static timing
3. Head-to-head comparison against HardFloat and FPnew on identical silicon -- wins on every op
4. IEEE f32 accuracy: 99.97% exact, 0.03% off by 1 ULP, 0% off by >1 ULP (10M random trials)

---

## 2. Arithmetic Architecture

### Addition

Close/far path split, same idea as IEEE adders but simpler: both operands are already signed two's complement, so the same integer adder handles all four sign combinations. No sign-magnitude decomposition, no conditional negation based on effective operation.

- **Far path** (|delta_e| >= 2): right-shift smaller operand to align, add, 0--3 bit normalize
- **Close path** (|delta_e| <= 1): add, CLZ for normalize distance, barrel shift

Both paths share one barrel shifter and converge at a shared rounding stage (banker's rounding). Rounding overflow detection uses pre-round signals -- exhaustive 8-bit testing confirms it fires on ~0.8% of pairs and is essential for correctness (521K mismatches without it).

Purely combinational, 0 DSP, 842 LUT4, 95 MHz silicon.

### Multiplication

N-1 x N-1 normalization produces at most N-2 (one redundant sign bit), so normalization is always 0 or 1 bit left shift. Exponents just add -- no bias arithmetic.

Karatsuba decomposition splits the 25-bit fraction into 12-bit signed high + 13-bit unsigned low, producing 3 sub-multiplies instead of one 25x25. On FPGA: 3 MULT18X18D (down from 4 naive). Without DSP: 1750 LUT4 (down from 1956).

Fused negate input ($-a \times b$) handles the corner case where two's complement minimum (`10...0`) wraps to itself.

227 LUT4 + 4 DSP (or Karatsuba: 3 DSP), 115 MHz silicon.

### Fused Multiply-Add

$a \times b + c$ with single rounding (truly fused). The key optimization: **pre-align the addend in parallel with the multiply**, using the raw exponent sum $e_a + e_b$ before the product is available. This removes the multiply-then-align serial dependency and gives ~26% higher Fmax (63 vs 50 MHz).

Same Karatsuba multiply, same close/far path addition. 1472 LUT4 + 3 DSP, 63 MHz silicon.

### Division and Square Root

Two implementations each, at different design points:

**Iterative** (single-cycle-per-iteration): `divide_iter` uses restoring long division (28 cycles, 0 DSP, 535 LUT4, 234 MHz). `sqrt_iter` uses restoring binary square root (27 cycles, 0 DSP, 101 LUT4, 400 MHz -- hitting the harness ceiling). These are simple and fast per-cycle but high-latency.

**Newton-Raphson pipelined**: `divmod_nr` (8-stage, 20 DSP, 120 MHz) and `sqrt_nr` (10-stage, 27 DSP, 125 MHz). High thruput but DSP-hungry -- the two units together need 47 DSP18, exceeding the ECP5-25F's 28. An ECP5-45F or larger is required for both simultaneously.

These are early implementations. Division and square root in IEEE libraries have been heavily optimized over decades (SRT algorithms, digit recurrence, etc.), so the comparison is not as clean as add/mul/FMA. See Section 4 for the numbers.

### Bitwise Operations

Two's complement FP enables first-class bitwise operations on floating-point values -- something IEEE 754 does not define and numerical libraries do not provide. Operands are aligned by exponent (shifting the smaller to match), then standard bitwise logic (AND, OR, XOR, NOT) is applied to the aligned fractions.

Bit shifts map directly to exponent adjustment: left shift increments the exponent, right shift decrements it. This means `x >> 1` is a divide-by-2 and `x << 1` is a multiply-by-2 -- a single exponent add, no multiplier needed. In IEEE 754, `2.0 * x` requires invoking the multiply unit (or a special-case optimization that the hardware may or may not implement). In Spirix, it's one wire.

Checked transitions to exploded/vanished states handle the case where a shift would exceed the representable exponent range, maintaining the same overflow/underflow semantics as arithmetic operations.

---

## 3. CE-Gated Self-Test Harness

Static timing on ECP5 is unreliable for cross-design comparison. I measured 1.5--3.7x margins between static estimates and actual silicon across all designs -- and the ratio varies per design, so you can't just apply a fudge factor.

### The Problem

You can't trust `nextpnr` when it says "43 MHz" for one design and "25 MHz" for another. The first might run at 115 MHz (2.7x margin) while the second runs at 88 MHz (3.5x margin). Static timing reports worst-case paths that may not be simultaneously activatable, and ECP5 speed grades include significant guard-banding.

### The Solution: Test the DUT Against Itself

The harness runs one DUT instance twice over the same PRNG input sequence:

1. **Gold run**: DUT clocked at full PLL speed but clock-enabled only every 256th cycle. At 256x margin, it settles correctly at any PLL frequency I can generate.
2. **Test run**: Same DUT, same PRNG sequence, CE=1 (every cycle). Now it's actually running at PLL speed.

If both runs produce the same output hash, the DUT works at that frequency. If not, timing failed.

### Protocol Details

The PRNG is a 64-bit Galois LFSR (taps: $x^{64} + x^{63} + x^{61} + x^{60}$) with a 1-LUT critical path (~0.7 ns). The protocol uses a 10-bit counter with bit taps -- no wide equality comparisons:

| Counter range | Phase | Why |
|---|---|---|
| 0--127 | **Warmup** | LFSR advances, DUT runs, but accumulator is frozen. Flushes pipeline state and lets the LFSR diverge from its seed so the capture window sees varied inputs. |
| 128--511 | **Accumulate** | 384 DUT outputs are compressed into a 32-bit hash via rotate-XOR: `{acc[30:0], acc[31]} ^ result[31:0]`. |
| 512+ | **Done** | Hash is captured. |

**Why rotate-XOR?** A plain XOR accumulator is commutative -- it can't distinguish output ordering or detect stuck-at patterns that cancel. The 1-bit rotate before each XOR makes the accumulator position-dependent: output *i* lands at a different rotation than output *i+1*. If even one DUT output differs between gold and test, the rotation cascade corrupts the entire hash. 384 accumulations with 32 rotate positions gives each bit 12 full rotations of mixing.

**Why 128 warmup + 384 capture?** 128 cycles is enough to flush any pipeline (my deepest is 10 stages) and decorrelate the LFSR from its seed. 384 capture cycles provides sufficient statistical coverage -- at 2^64 LFSR period, consecutive windows are effectively independent. Both numbers are bit-tap boundaries (bit 7 and bit 9 of the counter), so the phase logic is just wire taps with zero comparator overhead.

**Why CE/256 for gold?** The DUT has 256 full clock periods to settle between evaluations. Even my fastest measured silicon (500 MHz, ~2 ns period) gives 512 ns of settle time -- orders of magnitude beyond any combinational path. The gold is guaranteed correct regardless of PLL frequency.

### Seed Capture and Replay

When the test starts, the LFSR's current state is captured (XORed with a free-running entropy counter for uniqueness across runs). After the gold phase completes, the LFSR is reset to the captured seed for the test phase. This guarantees both phases see identical input sequences. Plus it's a nice button I can press if I feel something is off and re-run tests. Oh, and it's quite satisfying.

### CRT Display

An NTSC framebuffer module on a separate 25 MHz clock domain drives a CRT display via a 2-pin DAC (sync on 560 ohm, video on 220 ohm). The display shows "RUN" during testing, "PASS" on hash match, "FAIL" on mismatch. An LED provides the same status: 50% blink = running, 7/8 duty = pass, 1/8 duty = fail. This gives immediate visual feedback without needing JTAG or UART -- just plug in a composite monitor.

### Binary Search

Silicon Fmax is found by binary search: rebuild at each frequency (Yosys + nextpnr + ecppack + program), test, classify as pass/borderline/fail. Placement seed matters hugely -- seed 4 gave 27% higher Fmax than default on the multiplier (177 vs 139 MHz). All results use seed 4.

The harness ceiling (LFSR-only bypass, no DUT) is ~500 MHz. Anything below that is limited by the DUT, not the harness.

---

## 4. Results

### Platform

Colorlight 5A-75B v8.0: Lattice ECP5-25F speed grade 6, 24K LUT4, 28 DSP18. 25 MHz oscillator, EHXPLLL for test frequencies. Yosys synthesis, nextpnr-ecp5 place-and-route. Area reported as LUT4 with `-nowidelut`.

### Spirix vs HardFloat (FPGA, with DSP)

HardFloat cores are wrapped with `fNToRecFN`/`recFNToFN` converters for fair IEEE-in/IEEE-out comparison. Core-only areas (no converters): add 546, mul 282, FMA 1344 LUT4.

| Op | Spirix LUT4 | DSP | MHz | HardFloat LUT4 | DSP | MHz |
|---|---|---|---|---|---|---|
| Add/Sub | 842 | 0 | **95** | 1050 | 0 | 88 |
| Multiply | 227 | 4 | **115** | 786 | 4 | 65 |
| FMA | 1472 | 3 | **63** | 2057 | 4 | 47 |

All Spirix numbers include full edge case handling. Spirix: +8% Fmax / -20% area (add), +77% Fmax / -71% area (mul), +34% Fmax / -28% area / -1 DSP (FMA).

### Division and Square Root

Div/sqrt is not apples-to-apples: the implementations use fundamentally different algorithms with different latency/thruput/area tradeoffs. I present the numbers for completeness but do not claim a clean win here -- these operations have been heavily optimized in IEEE libraries over decades, and the Spirix implementations are early.

| Unit | Arch | Fmax | LUT4 | DSP | Latency |
|---|---|---|---|---|---|
| Spirix divide_iter | Restoring, iterative | **234** | 535 | 0 | 28 cyc |
| Spirix sqrt_iter | Restoring, iterative | **>400**\* | 101 | 0 | 27 cyc |
| Spirix divmod_nr | Newton-Raphson, 8-stage pipe | 120 | 560 | 20 | 8 cyc |
| Spirix sqrt_nr | Newton-Raphson, 10-stage pipe | 125 | 863 | 27 | 10 cyc |
| HardFloat div | Digit recurrence, iterative | 182 | 2047 | 0 | 26 cyc |
| HardFloat sqrt | Digit recurrence, iterative | 200 | 2000 | 0 | 24--25 cyc |
| FPnew div | Radix-4, iterative | 168 | 1863 | 0 | ~14 cyc |
| FPnew sqrt | Radix-4, iterative | 116 | 1903 | 0 | ~14 cyc |

\*Harness ceiling is ~500 MHz (LFSR-only bypass). sqrt_iter passed at 400 MHz; true Fmax is between 400--500 MHz but cannot be isolated from the harness at these frequencies.

The Spirix iterative units achieve the highest per-cycle Fmax (234/>400 MHz vs HardFloat's 182/200 MHz) and are dramatically smaller (535/101 LUT4 vs 2047/2000). HardFloat and FPnew extract more bits per cycle (radix-4 gets 2 bits/cycle, reducing latency), but at much higher area cost. The Spirix NR pipelined units offer full throughput (one result per clock) but consume 20--27 DSP18 each -- impractical on smaller FPGAs. Room for optimization here.

### Spirix vs FPnew (ASIC, no DSP)

FPnew is pure-LUT by design, making this the natural 1:1 comparison. FPnew uses a unified FMA datapath; standalone add (825 LUT4) and multiply (574 LUT4) reflect Yosys constant-prop of hardcoded inputs. True 3-input FMA = 2850 LUT4.

| Op | Spirix LUT4 | MHz | FPnew LUT4 | MHz |
|---|---|---|---|---|
| Add/Sub | 842 | **95** | 825 | 74 |
| Multiply | 2131 | **95** | 2850 | 74 |
| FMA | 3004 | **53** | 2850 | 25 |

All Spirix numbers include full edge case handling. Spirix wins speed across the board -- FMA is 2.1x faster. Add/sub is 2% larger than FPnew; multiply is 25% smaller; FMA is 5% larger but over 2x the speed.

### Static Timing vs Silicon

| Module | Static Est. | Silicon | Margin |
|---|---|---|---|
| LFSR (harness only) | 205 MHz | 500 MHz | 2.4x |
| Spirix sqrt_iter | -- | >400 | -- |
| Spirix divide_iter | -- | 234 | -- |
| HardFloat sqrt | -- | 200 | -- |
| HardFloat div | -- | 182 | -- |
| FPnew div | -- | 168 | -- |
| Spirix multiply | 43 | 115 | 2.7x |
| FPnew sqrt | -- | 116 | -- |
| Spirix add/sub | 26 | 95 | 3.7x |
| HardFloat add | 25 | 88 | 3.5x |
| FPnew add | -- | 74 | -- |
| FPnew mul | -- | 74 | -- |
| HardFloat multiply | 24 | 65 | 2.7x |
| Spirix FMA | 20 | 63 | 3.2x |
| HardFloat FMA | 16 | 47 | 2.9x |
| FPnew FMA | 17 | 25 | 1.5x |

This table is the reason the CE-gated harness exists. If you compared designs using static timing alone, you'd conclude Spirix add (26 MHz est.) is slower than HardFloat add (25 MHz est.) -- nearly tied. In reality, Spirix runs at 95 MHz vs 88 MHz, a clear 8% win. The margins are design-dependent and unpredictable.

### Pipeline Variants

| Variant | Fmax | LUT4 | DSP | vs Combinational |
|---|---|---|---|---|
| Adder pipe2 | 147 MHz | 679 | 0 | +55% Fmax (vs 95) |
| Multiplier pipe2 | 181 MHz | 139 | 4 | +57% Fmax (vs 115) |

### IEEE f32 Accuracy

10M random pairs, Spirix addition vs native Rust `f32`:
- 99.97% exact bit-for-bit match
- 0.025% differ by exactly 1 ULP
- 0% differ by more than 1 ULP

The 1-ULP cases are valid rounding choices at the boundary between the two systems' representable values (no denormals, single zero).

---

## 5. Related Work

**TMS320C3x** (TI, ~1988) is the closest precedent: two's complement mantissa, unbiased two's complement exponent, N-1 normalization -- the same core representation. It was a proprietary DSP with no published comparisons against IEEE hardware. TI abandoned the format in later generations under IEEE ecosystem pressure (Intel 8087, software portability), not because of hardware deficiency -- their own docs note the units were "simpler to build and validate."

**Boldo and Daumas** (2003) formally verified properties of two's complement FP using Coq, referencing the TMS320C3x. Theoretical contribution, no hardware.

**LOCOFloat** (Sanchez et al., 2020) uses two's complement significand and exponent for FPGA HIL simulation, with "soft normalization" (relaxed constraints). Different design point -- area reduction via reduced precision, no FMA, no comparison against IEEE libraries.

**HardFloat** (Berkeley) and **FPnew** (ETH Zurich) are the comparison targets in this paper. HardFloat uses a recoded internal format requiring I/O converters; FPnew uses a unified FMA datapath, pure-LUT.

**Posits** (Gustafson, 2017) use tapered precision with sign-magnitude and variable-length regime decoding -- different tradeoffs entirely.

To my knowledge, no prior work presents a silicon-verified area and frequency comparison of two's complement FP against IEEE 754 implementations on identical hardware.

---

## 6. Conclusion

| Module | Silicon Fmax | LUT4 | DSP | Latency |
|---|---|---|---|---|
| Spirix sqrt_iter | >400 MHz\* | 101 | 0 | 27 cyc |
| Spirix divide_iter | 234 | 535 | 0 | 28 cyc |
| HardFloat sqrt | 200 | 2000 | 0 | 24--25 cyc |
| HardFloat div | 182 | 2047 | 0 | 26 cyc |
| Spirix multiply pipe2 | 181 | 227 | 4 | 2 cyc |
| FPnew div | 168 | 1863 | 0 | ~14 cyc |
| Spirix addsub pipe2 | 147 | 679 | 0 | 2 cyc |
| Spirix sqrt_nr | 125 | 863 | 27 | 10 cyc (pipe) |
| Spirix divmod_nr | 120 | 560 | 20 | 8 cyc (pipe) |
| FPnew sqrt | 116 | 1903 | 0 | ~14 cyc |
| Spirix multiply | 115 | 227 | 4 | 1 cyc |
| Spirix add/sub | 95 | 842 | 0 | 1 cyc |
| HardFloat add | 88 | 1050 | 0 | 1 cyc |
| FPnew add | 74 | 825 | 0 | 1 cyc |
| FPnew mul | 74 | 2850 | 0 | 1 cyc |
| HardFloat multiply | 65 | 786 | 4 | 1 cyc |
| Spirix FMA | 63 | 1472 | 3 | 1 cyc |
| HardFloat FMA | 47 | 2057 | 4 | 1 cyc |
| FPnew FMA | 25 | 2850 | 0 | 1 cyc |

\*Harness ceiling ~500 MHz; Spirix sqrt_iter true Fmax is likely between 400 - 500 MHz.

Two's complement floating-point arithmetic achieves higher Fmax and lower area than IEEE 754 implementations for add, multiply, and FMA on FPGA silicon. The simplifications -- no sign-magnitude decomposition, no exponent bias, trivial normalization bounds -- translate directly into hardware wins. Division and square root show competitive Fmax at dramatically lower area; these are early implementations with room for optimization.

Beyond arithmetic, the format enables first-class bitwise operations on floating-point values -- AND, OR, XOR, NOT, and bit shifts as exponent adjustments -- that IEEE 754 does not define. Power-of-two scaling (`x >> 1` for divide-by-2) is a single exponent decrement, no multiplier required.

The CE-gated self-test methodology provides ground truth for FPGA frequency characterization where static timing is unreliable. I recommend it for any serious FPGA benchmarking effort.

All source, scripts, and harness configurations are open-source and reproducible on a ~$15 Colorlight 5A-75B board with the open-source Yosys/nextpnr toolchain:

```
SEED=4 bash fpga/scripts/build_ntsc.sh <freq_mhz> --program
```

---

## References

1. Texas Instruments, "TMS320C3x User's Guide," SPRU031F, 1997.
2. S. Boldo and M. Daumas, "Properties of Two's Complement Floating Point Notations," *Int. J. Software Tools for Technology Transfer*, 5(2-3):237-246, 2003.
3. A. Sanchez, A. de Castro, M. S. Martinez-Garcia, and J. Garrido, "LOCOFloat: A Low-Cost Floating-Point Format for FPGAs," *Electronics*, 9(1):81, 2020.
4. J. Hauser, "Berkeley HardFloat," https://github.com/ucb-bar/berkeley-hardfloat, 2019.
5. S. Mach, F. Zaruba, and L. Benini, "FPnew: An Open-Source Multi-Format Floating-Point Unit Architecture," *IEEE Trans. VLSI Systems*, 29(4):774-787, 2021.
6. J. Gustafson and I. Yonemoto, "Beating Floating Point at its Own Game: Posit Arithmetic," *Supercomputing Frontiers and Innovations*, 4(2), 2017.
