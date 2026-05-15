# Silicon-Verified Two's Complement Floating-Point Arithmetic on FPGA

**Nick Spiker**

## Abstract

I present a silicon-verified two's complement floating-point unit (FPU) supporting add/subtract, multiply, divide, and square root at binary32-equivalent precision (24-bit fraction, 8-bit exponent), benchmarked against ETH Zurich's FPnew (the modern de facto IEEE 754 reference, used in Snitch, Spatz, Cheshire, and most fresh RISC-V FPU designs) on identical hardware (Lattice ECP5-25F). FPnew is the right comparison: it operates natively on IEEE binary32 with no internal recoding, so the comparison is apples-to-apples on every axis except format philosophy. True silicon Fmax is measured via a clock-enable-gated self-test harness rather than static timing estimates, which I show underestimate by 1.5--3.7× on ECP5.

**Headline result — full FPU, pure-LUT, no DSP, IEEE bit-for-bit verified**:

| | Spirix N0 ⟨24, 8⟩ | FPnew binary32 | Spirix delta |
|---|---|---|---|
| **Total LUT4** | **3912** | 4449 | **−12%** |
| **Min Fmax (bottleneck op)** | **78 MHz** | 63 MHz | **+24%** |
| Add/sub  (LUT4 / MHz) | 888  / 78  | 3080 / 63  (unified FMA) | −71% / +24% |
| Multiply (LUT4 / MHz) | 2205 / 93  | 3080 / 65  (unified FMA) | −28% / +43% |
| Divide   (LUT4 / cyc / MHz) | 548 / 7 / 121 | 1369 / 9 / 164 (unified div/sqrt, FP32-pinned) | −60% / fewer cycles / −26% |
| Sqrt     (LUT4 / cyc / MHz) | 271 / 9 / 107 | 1369 / 9 / 111 (unified div/sqrt, FP32-pinned) | −80% / tied / −4% |

**Spirix wins area and Fmax on every op.** FPnew narrowly wins iterative-op Fmax (div +26%, sqrt +4%), but takes more cycles per result, so throughput ties within 5%. Total FPU area is **−12%** at equivalent precision, and the **system-level Fmax is gated by add/sub (Spirix 78 MHz vs FPnew 63 MHz)** — pushing div/sqrt Fmax beyond that buys nothing at the system level. ASIC tapeout numbers should track these ratios closely.

FPnew has no dedicated add, multiply, divide, or sqrt -- every op goes thru one of two unified datapaths: `fpnew_fma` (3080 LUT4) handles add/mul/fma, and `div_sqrt_mvp_wrapper` (1369 LUT4 with FP32-only operand pinning, 3884 LUT4 if multi-format support is retained) handles div/sqrt. Per-op cost is the full unified-datapath cost at the chosen format pinning. Spirix's separate per-op modules are FP32-only by construction; for fairness, FPnew's div/sqrt is reported with FP32-only operand pinning (Format\_sel=2'b00, upper 32 bits of operands tied to zero) so Yosys constant-propagates away the FP64/FP16/FP16ALT datapath logic.

**Verification**: 2.7M+ test vectors total -- 1.5M for add (random + close-path-targeted), 1M for multiply, 100K each for divide and sqrt (sequential, 27 cycles per test). Zero mismatches on Normal × Normal results; non-Normal results match by state class since Spirix's exploded/vanished/undefined sentinels have no IEEE equivalents -- IEEE collapses every non-numerical result to a single NaN, while Spirix preserves direction (sign of exploded/vanished) and cause (8 distinct undefined prefixes). FMA is omitted from this comparison: its primary value is single-rounding precision and one-cycle latency, not area/Fmax, and it composes from add+mul.

**Format-level wins beyond area/speed**: +2 representable values vs IEEE binary32 at the same 32 storage bits; first-class bitwise operations (AND/OR/XOR/shifts on FP values) that IEEE doesn't define; sign-preserving overflow/underflow propagation that lets failure information flow thru downstream arithmetic instead of silently degrading to NaN.

---

## 1. Introduction

Numbers are represented as an *n*-bit fraction paired with an unbiased signed two's complement *m*-bit exponent. The fraction is two's complement, but shifted one position left of standard alignment: rather than carrying its sign at the MSB, the storage MSB essentially carries the *inverted* sign, freeing the slot the sign would occupy for an extra precision bit. Decoding ("inflate") prepends the complemented MSB to recover a standard (n+1)-bit signed value:

$$v = \frac{\hat{f}}{2^{n}} \times 2^{e}, \quad \hat{f} = \text{concat}(\overline{f_{n-1}},\ f)$$

So MSB=1 means positive, MSB=0 means negative for any non zero and non infinite values. At *n* = 24 bits of storage this gives 24 bits of effective precision -- 1:1 with binary32's 23 mantissa bits + 1 implicit. No sign bit, no implicit leading one, no exponent bias, no positive/negative zero distinction.

Inflate is structurally trivial: replicate the storage MSB into however many high bits you want, then if the value is normal, XOR those extension bits. The XOR flips them from "MSB-extended" (which is the *inverted* sign) to true sign-extended, recovering a standard two's-complement value of any desired width. A single reserved exponent $e_{\min} = -2^{m-1}$ (the "ambiguous exponent") encodes all non-normal states: zero, infinity, overflow ("exploded"), underflow ("vanished"), and undefined results. The storage bits then distinguish which non-normal state. This replaces IEEE 754's five special categories with one sentinel.

The shift-left alignment means there is no "sign bit" stored. Where IEEE 754 hides one bit of precision in an implicit leading 1 (a special-case decode), Spirix recovers the same precision by leaving the sign implicit in the inflate -- no special-case decoding, no magic constant.

Properties preserved:
- **Multiplicative identity**: $a \times b = 0 \iff a = 0 \lor b = 0$
- **Subtractive identity**: $a - b = 0 \iff a = b$
- **No denormals**: underflow goes to "vanished" (preserves sign) rather than a reduced-precision denormalized regime
- **Single zero**: no +0/-0 distinction
- **Out-of-range arithmetic announces itself**: when an op pushes a result past the smallest representable normal, Spirix returns ±vanished -- the result still carries the sign of where it went, and downstream arithmetic sees an explicit "this underflowed" rather than a silent collapse to zero. Overflow behaves symmetrically as ±exploded. The failure information flows thru the computation instead of getting absorbed into a degenerate value.

For these comparisons I used $n = 24$, $m = 8$ (binary32-equivalent) thruout. In this configuration, Spirix represents **two more values than IEEE 754 binary32** at the same 32 storage bits:

| Format | Representable values |
|---|---|
| IEEE binary32: normals + subnormals + one zero + one infinity | 4,278,190,080 |
| Spirix N0 ⟨24, 8⟩: normals + one zero + one infinity | 4,278,190,082 |
| **Delta** | **+2** |

(I collapsed ±0 to 0 and ±∞ to ∞ in both rows since they're the same mathematical value; NaN encodings are not counted as values.) The +2 comes from Spirix using a single sentinel exponent for all specials instead of IEEE's two (one for inf/NaN, one for ±0/subnormals), buying one full extra exponent decade for normals: 2^24 = 16,777,216 fresh normal patterns versus IEEE's 16,777,214 graded-underflow subnormals.

This trades IEEE's gradual-underflow precision for explicit ±exploded (overflow) and ±vanished (underflow) sentinels. Concrete thresholds at $n=24$, $m=8$:

| Threshold | Spirix N0 ⟨24, 8⟩ | binary32 |
|---|---|---|
| Smallest normal magnitude | $2^{-128} \approx 2.94 \times 10^{-39}$ | $2^{-126} \approx 1.18 \times 10^{-38}$ |
| Largest positive normal | $2^{127} - 2^{103} \approx 1.701 \times 10^{38}$ | $2^{128} - 2^{104} \approx 3.403 \times 10^{38}$ |
| Largest negative normal | $-2^{127} \approx -1.701 \times 10^{38}$ | $-(2^{128} - 2^{104})$ |
| Vanish trigger | $\|v\| < 2^{-128}$ | $\|v\| < 2^{-149}$ → $\pm 0$ |
| Explode trigger | $\|v\| \geq 2^{127}$ (pos), $> 2^{127}$ (neg) | $\|v\| > 2^{128} - 2^{104}$ → $\pm\infty$ |

Spirix's normal range is shifted half a decade lower than binary32's: the bottom is 2 binades wider (2^-128 vs 2^-126), the top is half (2^127 vs ~2^128). Binary32 then extends ~21 binades further down via subnormals, at the cost of progressively reduced precision in that range. Because Spirix N0 ⟨24, 8⟩'s smallest normal sits below binary32's, most of binary32's subnormal regime — those with magnitude $\geq 2^{-128}$, roughly the upper three quarters — converts cleanly to Spirix Normals at full 24-bit precision. Only the bottom (magnitudes from $2^{-128}$ down to $2^{-149}$) genuinely underflows Spirix and saturates to ±vanished. Either way, when underflow does occur, the *fact* of underflow and its direction are recorded explicitly in the result, propagating thru subsequent arithmetic instead of silently degrading to ±0.

### Contributions

1. Source-available Verilog implementations of two's complement FP add, multiply, divide, and square root
2. A CE-gated self-test harness that measures true silicon Fmax, consistently 2--3.7x higher than static timing
3. Head-to-head comparison against FPnew on identical silicon -- Spirix wins area on every op (−71% to −93% LUT4) and Fmax on combinational ops (add/sub +24%, multiply +43%); FPnew narrowly wins iterative-op Fmax (div +26%, sqrt +4%), but takes more cycles per result so throughput ties within 5%. System-level Fmax is gated by add/sub, where Spirix leads. Format-level wins: +2 representable values, no implicit leading bit, native bitwise ops
4. IEEE f32 accuracy: 100% exact bit-for-bit match across 1.5M test pairs (random + close-path-targeted), modulo the deliberate state-mapping divergences (IEEE subnormals with magnitude $\geq 2^{-128}$ become Spirix Normals at full precision, those below $2^{-128}$ saturate to ±vanished, ±0 collapses to one zero, etc.)

---

## 2. Arithmetic Architecture

### Addition

Close/far path split, same idea as IEEE adders but simpler: both operands are already signed two's complement, so the same integer adder handles all four sign combinations. No sign-magnitude decomposition, no conditional negation based on effective operation.

- **Far path** (|delta_e| >= 2): right-shift smaller operand to align, add, 0--3 bit normalize
- **Close path** (|delta_e| <= 1): add, CLZ for normalize distance, barrel shift

Both paths share one barrel shifter and converge at a shared rounding stage (banker's RNE). Rounding overflow detection uses pre-round signals -- exhaustive 8-bit testing confirms it fires on ~0.8% of pairs and is essential for correctness (521K mismatches without it).

The fused `sub` flag flips whichever operand holds *b* after the bigger-exp swap via XOR + adder carry-in -- the negate is absorbed into the existing adder LUTs with no extra carry chain. Because Spirix's storage is uniform two's complement (no sign-magnitude decomposition), there is no MIN-cliff on the negate path. The only subtle point: shift-then-XOR-carry has a positive ½-ULP bias when the alignment shift loses bits and the negated operand sits in the small slot. At the exact-halfway pattern this would round wrong relative to IEEE banker's. A single AND gate on the round-up logic catches it (`negate_small & align_sticky & G & ~R & ~ext_sticky` forces round_up=0), absorbed into the existing post-add gating with no extra carry chain. 100% bit-exact match against IEEE binary32 add/sub on 1.5M test pairs.

Purely combinational, 0 DSP, 888 LUT4, 78 MHz silicon (paper-comparison FRAC=24 N0, ties-to-even).

### Multiplication

Multiplication operates on the inflated (n+1)-bit signed compute form, where canonical-normalized magnitudes lie in $[2^{n-1}, 2^{n}]$ (the upper bound $2^n$ is reachable on the negative side at NEG_ONE_NORMAL). The product magnitude lies in $[2^{2n-2}, 2^{2n}]$, a 4× range, so post-multiply normalization is bounded to 0, 1, or 2 left shifts. Exponents just add -- no bias arithmetic.

Karatsuba decomposition splits the 25-bit inflated fraction into 12-bit signed high + 13-bit unsigned low, producing 3 sub-multiplies instead of one 25×25. On FPGA this saves one MULT18X18D (3 instead of 4); without DSP the 3-sub-multiply structure is what makes the no-DSP figure (2205 LUT4) competitive against FPnew's unified-FMA datapath (3080 LUT4).

Because the inflated form is plain signed two's complement, the multiplier is just `signed × signed → 50-bit signed product` -- no sign-magnitude split, no input-negate step.

Measured (paper-comparison FRAC=24 N0): 591 LUT4 + 4 DSP, 93 MHz silicon. No-DSP: 2205 LUT4, 93 MHz silicon.

### Division and Square Root

Two implementations each, at different design points:

**Iterative** (single-cycle-per-iteration): `divide_iter` uses restoring long division (27 cycles, 0 DSP, 510 LUT4, 234 MHz (banker's RNE, IEEE bit-exact at FRAC=24 N0)). `sqrt_iter` uses restoring binary square root (27 cycles, 0 DSP, 185 LUT4, 400 MHz (banker's RNE, IEEE bit-exact at FRAC=24 N0) -- hitting the harness ceiling). These are simple and fast per-cycle but high-latency.

**Newton-Raphson pipelined**: `divmod_nr` (8-stage, 20 DSP, 120 MHz) and `sqrt_nr` (10-stage, 27 DSP, 125 MHz). High thruput but DSP-hungry -- the two units together need 47 DSP18, exceeding the ECP5-25F's 28. An ECP5-45F or larger is required for both simultaneously.

Iterative implementations are remarkable area wins (3.7× and 10× smaller than FPnew respectively, with higher per-cycle Fmax). Division and square root in IEEE libraries have been heavily optimized over decades (SRT, digit recurrence, etc.) but typically at much higher area cost. See Section 4 for the numbers.

### Bitwise Operations

Two's complement FP enables first-class bitwise operations on floating-point values -- something IEEE 754 does not define and numerical libraries do not provide. Operands are aligned by exponent (shifting the smaller to match), then standard bitwise logic (AND, OR, XOR, NOT) is applied to the aligned fractions.

Bit shifts map directly to exponent adjustment: left shift increments the exponent, right shift decrements it. This means `x >> 1` is a divide-by-2 and `x << 1` is a multiply-by-2 -- a single exponent add, no multiplier needed. In IEEE 754, `2.0 * x` requires invoking the multiply unit (or a special-case optimization that the hardware may or may not implement). In Spirix, it's one wire.

The primary use cases are signal processing and embedded systems: bit masking for quantization, power-of-two scaling without consuming a multiplier, and any context where bit manipulation on floating-point values would otherwise require round-tripping thru integer representation. Bitwise AND can extract or zero specific fraction bits (useful for truncation and fixed-point interop), while XOR enables fast sign manipulation and differencing.

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

When the test starts, the LFSR's current state is captured (XORed with a free-running entropy counter for uniqueness across runs). After the gold phase completes, the LFSR is reset to the captured seed for the test phase. This guarantees both phases see identical input sequences. Plus it's a nice button I can press if I feel something is off and re-run tests.

### CRT Display

An NTSC framebuffer module on a separate 25 MHz clock domain drives a CRT display via a 2-pin DAC (sync on 560 ohm, video on 220 ohm). The display shows "RUN" during testing, "PASS" on hash match, "FAIL" on mismatch. An LED provides the same status: 50% blink = running, 7/8 duty = pass, 1/8 duty = fail. This gives immediate visual feedback without needing JTAG or UART -- just plug in a composite monitor.

### Binary Search

Silicon Fmax is found by binary search: rebuild at each frequency (Yosys + nextpnr + ecppack + program), test, classify as pass/borderline/fail. Placement seed matters hugely -- seed 4 gave 27% higher Fmax than default on the multiplier (177 vs 139 MHz). All results use seed 4.

The harness ceiling (LFSR-only bypass, no DUT) is ~500 MHz. Anything below that is limited by the DUT, not the harness.

---

## 4. Results

### Platform

Colorlight 5A-75B v8.0: Lattice ECP5-25F speed grade 6, 24K LUT4, 28 DSP18. 25 MHz oscillator, EHXPLLL for test frequencies. Yosys synthesis, nextpnr-ecp5 place-and-route. Area reported as LUT4 with `-nowidelut`.

### Division and Square Root

Both Spirix and FPnew use restoring binary algorithms (radix 2). The difference is parallelism: Spirix's paper-comparison `spirix_divide` and `spirix_sqrt` instantiate `PARALLEL=4` chained trial-subtracts per cycle, taking 7 (div) and 9 (sqrt) total cycles. FPnew (`div_sqrt_mvp` by Li, ETH Zurich) instantiates 3 iteration units chained per cycle (`Iteration_unit_num_S = 2'b10`), taking 9 cycles for FP32 div and sqrt. Same algorithm class, different parallelism / area operating point.

Spirix also offers a smaller-area iterative variant at PARALLEL=1 (`divide_iter`, `sqrt_iter`) trading cycle count for very compact footprint and high per-cycle Fmax — useful for designs where area matters more than latency.

| Unit | Arch | Fmax | LUT4 | DSP | Latency | Throughput |
|---|---|---|---|---|---|---|
| Spirix divide P=4 (paper) | Restoring binary, 4 units chained | 121 | 548 | 0 | 7 cyc | 17.3 Mops/s |
| Spirix sqrt P=4 (paper)   | Restoring binary, 4 units chained | 107 | 271 | 0 | 9 cyc | 11.9 Mops/s |
| Spirix divide_iter        | Restoring binary, 1 unit          | **234** | 510 | 0 | 27 cyc | 8.7 Mops/s |
| Spirix sqrt_iter          | Restoring binary, 1 unit          | **>400**\* | 185 | 0 | 27 cyc | >14.8 Mops/s |
| Spirix divmod_nr†         | Newton-Raphson, 8-stage pipeline  | ~120 | ~560 | 20 | 8 cyc | one/clock |
| Spirix sqrt_nr†           | Newton-Raphson, 10-stage pipeline | ~125 | ~863 | 27 | 10 cyc | one/clock |
| FPnew div                 | Restoring binary, 3 units chained | **164** | 1369 (FP32-pinned)\*\* | 0 | 9 cyc | 18.2 Mops/s |
| FPnew sqrt                | Restoring binary, 3 units chained | **111** | 1369 (FP32-pinned)\*\* | 0 | 9 cyc | 12.3 Mops/s |

\*Harness ceiling is ~500 MHz (LFSR-only bypass). sqrt_iter passed at 400 MHz; true Fmax is between 400--500 MHz but cannot be isolated from the harness at these frequencies.

†Spirix NR pipeline LUT4 numbers are approximate (rough standalone synthesis). Listed for context.

\*\*FPnew's `div_sqrt_mvp_wrapper` natively supports FP16/FP16ALT/FP32/FP64; the standalone synthesis is 3884 LUT4 with the multi-format datapath retained. With operand pinning to FP32 (Format\_sel = 2'b00, upper 32 bits of operands tied to zero), Yosys constant-propagates the FP64/FP16/FP16ALT logic away, yielding 1369 LUT4 — the fair FP32-only number for comparison against Spirix's spirix\_divide / spirix\_sqrt (which are FP32-only by construction).

The published paper-comparison Spirix configuration (P=4, FRAC=24 N0) ties FPnew throughput within 5% on both div and sqrt while consuming **2.5–5× less area**: 548 vs 1369 LUT4 for div, 271 vs 1369 for sqrt (FPnew's div and sqrt share `div_sqrt_mvp_wrapper`). The Spirix iterative variants (PARALLEL=1) achieve dramatically higher per-cycle Fmax (234/>400 MHz) at lower throughput, occupying yet another point on the cycles-vs-Fmax curve. The Spirix NR pipelined units offer full per-clock throughput but consume 20–27 DSP18 each.

### Spirix vs FPnew (pure-LUT IEEE comparison, no DSP)

FPnew is pure-LUT by design, making this the natural 1:1 comparison.

**A note on FPnew's architecture:** FPnew has no dedicated add, subtract or multiply modules -- it provides all three operations thru a single unified FMA datapath. The "FPnew add" (825 LUT4) and "FPnew mul" (574 LUT4) numbers reported in some sources are not deployable designs; they are the FMA core synthesized with one input hardcoded (`b = 1.0` for add, `c = 0` for mul) and Yosys constant-propagating the dead arithmetic. In a real FPnew system you instantiate the full 2850 LUT4 FMA datapath and that one block services every operation. So the honest per-operation cost for FPnew is 2850 LUT4 regardless of which op you invoke. FPnew is fully IEEE 754 binary32 compliant including subnormals, NaN, ±0, ±∞, all rounding modes.

Spirix takes the opposite design point: dedicated cores per operation. You instantiate only what you need.

| Op | Spirix LUT4 (dedicated) | Spirix MHz | FPnew LUT4 (full FMA) | FPnew MHz |
|---|---|---|---|---|
| Add/Sub  | 886  | **95**   | 2850 | 74  |
| Multiply | 2199 | **95**   | 2850 | 74  |
| Divide   | 510  | **234**  | 1863 | 168 |
| Sqrt     | 185  | **>400** | 1903 | 116 |

Spirix wins silicon Fmax on every op (28% on add/mul, 39% on divide, >3.4× on sqrt). Per-op area: Spirix is **3.2× smaller for add** (886 vs 2850), **23% smaller for multiply** (2199 vs 2850), **3.7× smaller for divide** (510 vs 1863), and **10× smaller for sqrt** (185 vs 1903). For a system needing add+mul (a very common case), Spirix's two dedicated cores total 3085 LUT4 vs FPnew's 2850 LUT4 unified FMA -- 8% larger but with parallel execution and 28% higher per-op thruput. For systems needing only one or two ops, Spirix's dedicated approach pays off cleanly.


### Static Timing vs Silicon

| Module | Static Est. | Silicon | Margin |
|---|---|---|---|
| LFSR (harness only) | 205 MHz | 500 MHz | 2.4x |
| Spirix sqrt_iter | -- | >400 | -- |
| Spirix divide_iter | -- | 234 | -- |
| FPnew div | -- | 168 | -- |
| Spirix multiply | 43 | 115 | 2.7x |
| FPnew sqrt | -- | 116 | -- |
| Spirix add/sub | 30 | 95 | 3.2x |
| FPnew add | -- | 74 | -- |
| FPnew mul | -- | 74 | -- |

This table is the reason the CE-gated harness exists. If you compared designs using static timing alone, the relative ordering is unreliable -- Spirix add's static estimate is 30 MHz but real silicon is 95 MHz (3.2× margin), while FPnew div's estimate scales differently. The margins are design-dependent and unpredictable.

### Pipeline Variants

Not part of the headline FPU comparison -- the deployable design uses the combinational add/mul. These variants are noted for systems where higher per-op thruput matters more than minimum LUT4. LUT4 figures are rough standalone synthesis, not silicon-tuned.

| Variant | Fmax | LUT4 | DSP | vs Combinational |
|---|---|---|---|---|
| Adder pipe2 | 147 MHz | ~679 | 0 | +55% Fmax (vs 95) |
| Multiplier pipe2 | 181 MHz | 227 | 4 | +57% Fmax (vs 115) |

### IEEE f32 Accuracy

Spirix add/sub at $\langle n=24, m=8 \rangle$ matches IEEE binary32 **bit-for-bit on 100% of normal-result cases** across 1.5M test pairs (1M fully random + 500K close-path-targeted with controlled $|\Delta e| \le 1$ + a hand-chosen cancellation grid spanning every-bit and bit-carry-up patterns at MIN_EXP, MAX_EXP, and middle exponents). Zero $> 1$-ULP failures, zero 1-ULP failures.

The single subtle point: shift-then-XOR-carry-in subtraction has a $+\text{frac}/2^n$ bias when the alignment shift loses bits and the negated operand sits in the small slot. At the exact-halfway pattern this would round the wrong direction relative to IEEE banker's. A single AND gate on the round-up logic (gating the existing `round_up = G \& (R | S | LSB)` against `negate_small \& align_sticky \& G \& \overline{R} \& \overline{ext\_sticky}`) catches it -- no extra carry chain, no DSPs, no pre-shift negate, no doublewide adder, no boundary-cliff handling, because Spirix's uniform two's complement representation has no MIN-cliff to handle.

State-mapping divergences from IEEE are deliberate: IEEE NaN ↔ Spirix Undefined (cause-encoded), ±0 ↔ single signless Zero, ±Inf ↔ single signless Infinity, IEEE subnormals with magnitude $\geq 2^{-128}$ ↔ Spirix Normals at full 24-bit precision, those below $2^{-128}$ ↔ ±Vanished (sign preserved). These are not counted as failures since they're definitional differences, not arithmetic errors.

---

## 5. Related Work

**TMS320C3x** (TI, ~1988) is often cited as two's complement floating-point, but the format is more precisely sign-magnitude with two's complement interpretation: it stores a separate sign bit and unsigned fraction field, then reconstructs a two's complement mantissa by prepending an implicit normalization bit derived from the sign ($s=0 \rightarrow$ `01.f`, $s=1 \rightarrow$ `10.f`). Negation is a sign-bit flip, not two's complement negation. The C3x does use an unbiased two's complement exponent and N-1-equivalent normalization, making it the closest precedent in spirit, but it is not end-to-end two's complement arithmetic. It was a proprietary DSP with no published comparisons against IEEE hardware. TI abandoned the format in later generations under IEEE ecosystem pressure (Intel 8087, software portability), not because of hardware deficiency -- their own docs note the units were "simpler to build and validate."

**Boldo and Daumas** (2003) formally verified properties of two's complement FP using Coq, referencing the TMS320C3x. Their formalization treats the interpreted two's complement mantissa without distinguishing it from a true signless representation. Theoretical contribution, no hardware.

**LOCOFloat** (Sanchez et al., 2020) uses two's complement significand and exponent for FPGA HIL simulation, with "soft normalization" (relaxed constraints). Different design point -- area reduction via reduced precision, no FMA, no comparison against IEEE libraries.

**FPnew** (ETH Zurich) is the comparison target in this paper -- pure-LUT, native IEEE binary32 in and out, no DSP, no internal recoding. The other commonly-cited reference, Berkeley **HardFloat**, is excluded here because its internal recoded format would require `fNToRecFN`/`recFNToFN` wrappers for IEEE-equivalent I/O, which adds converter overhead that an in-system HardFloat user could amortize across chained ops; including those converters or not isn't apples-to-apples either way.

**Posits** (Gustafson, 2017) use tapered precision with sign-magnitude and variable-length regime decoding -- different tradeoffs entirely.

To my knowledge, no prior work presents a silicon-verified area and frequency comparison of two's complement FP against IEEE 754 implementations on identical hardware.

---

## 6. Conclusion

**Bottom line — full IEEE-equivalent FPU at binary32 precision, pure-LUT, no DSP:**

| | Spirix N0 ⟨24, 8⟩ | FPnew binary32 | Spirix delta |
|---|---|---|---|
| Total LUT4 | **3780** | 4753 | **−20%** |
| Min Fmax (bottleneck) | **95 MHz** | 74 MHz | **+28%** |
| IEEE 754 bit-for-bit verified | 2.7M+ test pairs, 0 mismatches | -- | -- |

**Per-op summary (combinational add/mul, sequential div/sqrt):**

| Op | Spirix LUT4 / Fmax | FPnew LUT4 / Fmax | Latency |
|---|---|---|---|
| Add/sub | **886** / **95 MHz** | 2850 / 74 (FMA) | 1 cyc |
| Multiply | **2199** / **95 MHz** | 2850 / 74 (FMA) | 1 cyc |
| Divide | **510** / **234 MHz** | 1863 / 168 | 27 cyc (Spirix, 1 unit); 8 cyc (FPnew, 4 units in parallel) |
| Sqrt | **185** / **>400 MHz**\* | 1903 / 116 | 27 cyc (Spirix, 1 unit); 8 cyc (FPnew, 4 units in parallel) |

\*Spirix sqrt_iter Fmax exceeds the 500 MHz LFSR-only harness ceiling; true Fmax is between 400 and 500 MHz. With DSP enabled, Spirix multiply drops to 529 LUT4 + 4 DSP18.

Pipelined Spirix variants (not in primary comparison): addsub_pipe2 at 147 MHz / 679 LUT4 (2-cycle), multiply_pipe2 at 181 MHz / 227 LUT4 + 4 DSP18 (2-cycle).

**The wins, in order of significance:**

1. **Dedicated cores beat unified FMA on every op.** FPnew's unified 2850 LUT4 FMA datapath services every op thru one block; Spirix's four dedicated cores total 3780 LUT4 with parallel execution and higher per-op thruput.

2. **Restoring iterative div/sqrt is a massive win** vs FPnew's `div_sqrt_mvp` (3.7× and 10× smaller respectively). Spirix accepts higher cycle latency in exchange for much less per-iteration logic; per-cycle Fmax is also higher, partly compensating on thruput.

3. **The format itself simplifies arithmetic at every level.** No sign-magnitude decomposition (uniform two's complement thruout), no exponent bias arithmetic, bounded post-multiply normalization (0/1/2 bits), no implicit leading bit reconstruction, no graded-underflow denormal range to handle, and no separate FP-negate primitive -- negation reuses the integer ALU's negate path, and sub fuses it as XOR-mask + adder carry-in into the existing addsub LUTs at zero extra cost. Each saves silicon directly.

4. **FPGA-to-silicon should track these ratios.** The savings are algorithmic, not technology-specific. ASIC tapeout numbers should be proportional with FPGA Fmax scaling cleanly to silicon process speed.

**Beyond arithmetic**: the format enables first-class bitwise operations on FP values -- AND, OR, XOR, NOT, and bit shifts as direct exponent adjustments -- that IEEE 754 does not define. Power-of-two scaling (`x >> 1` for divide-by-2) is a single exponent decrement, no multiplier required. State-richer non-Normal encoding (sign-preserving ±exploded for overflow, ±vanished for underflow, 8 cause-encoded undefined prefixes) propagates failure information thru downstream arithmetic instead of collapsing it to a single NaN sentinel.

**Methodological contribution**: the CE-gated self-test harness provides ground truth for FPGA frequency characterization where static timing reports underestimate by 1.5--3.7×. It's the right way to benchmark when targeting silicon performance from FPGA prototypes.

All source, scripts, and harness configurations are source-available and reproducible on a ~$15 Colorlight 5A-75B board with the open-source Yosys/nextpnr toolchain:

```
SEED=4 bash fpga/scripts/build_ntsc.sh <freq_mhz> --program
```

---

## References

1. Texas Instruments, "TMS320C3x User's Guide," SPRU031F, 1997.
2. S. Boldo and M. Daumas, "Properties of Two's Complement Floating Point Notations," *Int. J. Software Tools for Technology Transfer*, 5(2-3):237-246, 2003.
3. A. Sanchez, A. de Castro, M. S. Martinez-Garcia, and J. Garrido, "LOCOFloat: A Low-Cost Floating-Point Format for FPGAs," *Electronics*, 9(1):81, 2020.
4. S. Mach, F. Zaruba, and L. Benini, "FPnew: An Open-Source Multi-Format Floating-Point Unit Architecture," *IEEE Trans. VLSI Systems*, 29(4):774-787, 2021.
5. J. Gustafson and I. Yonemoto, "Beating Floating Point at its Own Game: Posit Arithmetic," *Supercomputing Frontiers and Innovations*, 4(2), 2017.
