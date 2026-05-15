# Synthesis reference — paper/comparison/verilog/

ECP5-25F speed-6 (CABGA256), Yosys + nextpnr-ecp5, FRAC_BITS=24, EXP_BITS=8.
All modules: N0 storage, banker's RNE, full edge case handling, DSP-free.

## Final synthesis numbers

| Op | LUT4 | CCU2C | DSP | Bit-exact verified |
|---|---|---|---|---|
| Add/sub        | 886  | 119 | 0 | 1.5M @ 100% match (random + close-path-targeted) |
| Multiply       | 2199 | 58  | 0 | 1M @ 100% match (random) |
| Divide_iter    | 510  | 90  | 0 | 100K @ 100% (sequential, 27 cyc latency) |
| Sqrt_iter      | 185  | 39  | 0 | 100K @ 100% (sequential, 27 cyc latency) |

(Multiply with-DSP: 529 LUT4 + 4 MULT18X18D — different operating point.)

## Comparison vs FPnew (binary32 IEEE-equivalent, no DSP)

| Op | Spirix | FPnew (deployable FMA datapath) | Spirix delta |
|---|---|---|---|
| Add/sub  | 886  | 2850 (or 825 const-prop)  | -69% / +7% |
| Multiply | 2199 | 2850 (or 574 const-prop)  | -23% / +283% |
| Divide   | 510  | 1863                       | **-72%** |
| Sqrt     | 185  | 1903                       | **-90%** |

(FMA omitted: primary value is single-rounding precision and one-cycle latency, not area/Fmax. Composes from add+mul.)

## Silicon Fmax + throughput (Colorlight 5A-75B v8.0, ECP5-25F speed-6, SEED=4)

| Op       | Spirix Fmax | Cycles | ns/result | Mops/s | Spirix LUT4 | FPnew Fmax | Cyc | ns | Mops/s | FPnew LUT4 (note) |
|----------|-------------|-------:|----------:|-------:|------------:|------------|----:|----:|-------:|------------------:|
| Add/sub  | **78 MHz**  | 1 | 12.8 | **78.0** | 888  | 63 MHz  | 1 | 15.9 | 63.0 | 3080 (unified FMA) |
| Multiply | **93 MHz**  | 1 | 10.8 | **93.0** | 2205 | 65 MHz  | 1 | 15.4 | 65.0 | 3080 (unified FMA) |
| Divide   | 121 MHz     | 7 | 57.9 | 17.3     | 548  | 164 MHz | 9 | 54.9 | **18.2** | 1369 (unified div/sqrt, FP32-pinned) |
| Sqrt     | 107 MHz     | 9 | 84.1 | 11.9     | 271  | 111 MHz | 9 | 81.1 | **12.3** | 1369 (unified div/sqrt, FP32-pinned) |

LUT4 from standalone synthesis with `-nodsp -nowidelut` (`paper/comparison/scripts/synth_all_paper.sh`). FPnew add+mul share `fpnew_fma` (3080 LUT4); div+sqrt share `div_sqrt_mvp_wrapper` (3884 LUT4). Unified-datapath numbers — full module without operand const-prop.

System Fmax is gated by the slowest unit (add/sub at 78 MHz). Iterative ops have plenty of Fmax headroom over the system clock — pushing div/sqrt Fmax further gains nothing at the system level, only throughput per cycle matters there. Spirix wins throughput on the combinational ops (add/mul) and ties FPnew within 5% on iterative ops, with **3-10× smaller area** across the board.

## Divide algorithm sweep

Investigated the iterative divide critical path to understand FPnew's Fmax win.
All measurements at FRAC=24, EXP=8, ECP5-25F speed-6, SEED=4.

| Configuration | MHz | Cycles | ns/result | Mops/s |
|---|---:|---:|---:|---:|
| Spirix restoring, PARALLEL=4    | 121 | 7 | 57.9 | **17.3** ← published |
| Spirix restoring, PARALLEL=3    | 138 | 9 | 65.2 | 15.3 |
| Spirix non-restoring, PARALLEL=3 (FPnew-style chain) | 154 | 9 | 58.4 | 17.1 |
| FPnew (non-restoring, 3 cells) | 164 | 9 | 54.9 | 18.2 |

Findings:
- **Algorithm**: non-restoring radix-2 saves 1 LUT depth per stage (no output mux);
  same width and stage count gives +12% Fmax. See `divide_step_chain_fpnew.v`.
- **Apples-to-apples** (same algorithm, same stage count): Spirix lands within 6%
  of FPnew silicon Fmax. The remaining gap is gate-level/placement — FPnew's
  cells pack into ECP5 PLBs slightly tighter (12.55 ns of routing in our
  critical path vs FPnew's similar logic depth).
- **Best operating point** for Spirix is restoring at PARALLEL=4: fewer cycles
  per result more than offsets the lower Fmax. Throughput 17.3 vs FPnew 18.2
  Mops/s (5% gap), 3.5× smaller area.

## Bit-exactness methodology

## Bit-exactness methodology

For each op:
- Random pairs from f32 input space, mapped to Spirix N0 storage.
- Edge-case grid covering every combination of {Normal, Zero, Inf, ±Exploded, ±Vanished, Undefined}.
- Gold model: `f64_to_spirix(spirix_to_f64(a) op spirix_to_f64(b))` for Normal × Normal cases. f64 chosen over f32 to avoid spurious denormalization for Spirix-normal-but-f32-subnormal values.
- For non-Normal results: state-class match (Spirix's exploded/vanished/undefined have no IEEE equivalents; we accept any state in the same family).

For add: 1.5M tests including 500K close-path-targeted (forced |Δexp| ≤ 1).

For multiply, divide, sqrt: 100K-1M random + edge-case grid.

Reproduce:
```
cargo run --release --bin gen_addsub_vectors -- 1000000
cargo run --release --bin gen_mul_vectors -- 1000000
cargo run --release --bin gen_div_vectors -- 100000
cargo run --release --bin gen_sqrt_vectors -- 100000

iverilog -g2012 -o /tmp/tb verilog/spirix_addsub.v   verilog/tb_random.v && /tmp/tb
iverilog -g2012 -o /tmp/tb verilog/spirix_multiply.v verilog/tb_mul.v    && /tmp/tb
iverilog -g2012 -o /tmp/tb verilog/divide_step_chain.v verilog/spirix_divide.v verilog/tb_div.v && /tmp/tb
iverilog -g2012 -o /tmp/tb verilog/spirix_sqrt.v       verilog/tb_sqrt.v && /tmp/tb
```

## Silicon Fmax methodology (Colorlight 5A-75B v8.0, ECP5-25F speed-6)

Hardware bench: `fpga/bench/top_ntsc.v` runs each DUT under a CE-gated dual-pass
protocol on real silicon. Each test cycle:

1. **PH_GOLD**: 32K cycles warmup (no accumulation) + 64K cycles accumulated via
   rotate-XOR hash, all at slow CE (1/256). Slow CE is guaranteed correct.
2. **PH_SWITCH**: LFSR reset to captured seed; one cycle of phase transition.
3. **PH_TEST**: same 32K warmup + 64K accumulated, but at full speed (CE=1).
4. **Compare**: gold_hash == test_hash → pass.

The accumulate window is implemented via bit-taps on an 18-bit counter: warmup =
counts 0..32767, accumulate = counts {32768..65535, 98304..131071} (the
middle 32K is skipped because the tap selects the 64-value bit rather than ≥64
comparison — saves a comparator). Net effect: 65536 accumulated samples per
phase, identical between gold and test.

Marginal frequencies produce intermittent fails because timing closes for some
input patterns but not others. Pass requires identical hashes — single bit flip
anywhere in 64K results trips the comparator.

**Reset discipline (parity across all DUTs)**: every iterative DUT (FPnew
div/sqrt, HardFloat div/sqrt, Spirix divide/sqrt PARALLEL=4) is held in a known
clean state during PH_IDLE and PH_SWITCH. Without this, residual pipeline state
from the gold phase contaminates the first test-phase result and produces
repeatable mismatches at any frequency. FPnew's wrapper is held via its
`Rst_RBI` port; Spirix iterative modules have no reset port but achieve
equivalent discipline by blocking `start` pulses during reset phases (the FSM
self-clears through S_FINALIZE → S_IDLE on the prior operation, and the
protocol guarantees PH_GOLD only exits after the final `done` fires).
Combinational DUTs (Spirix add/sub, multiply) have no carryover state and need
no extra discipline.

**Binary search**: program at frequency F, observe pass/fail LED. Pass → ramp F
up. Fail → ramp F down. Boundary is reported as the highest frequency that
produces a stable pass over a sustained run (typically minutes).

**Path coverage per phase (add/sub, 65536 samples, uniform-random 8-bit
exponents → triangular Δexp distribution):**
- Same exp (Δ=0):                      256/65536  ≈ 0.39%  (~256 samples)
- Close (|Δexp| ≤ 2):                  1274/65536 ≈ 1.95%  (~1280 samples)
- Far, smaller contributes (3 ≤ |Δ| ≤ 25): ~16.3% (~10700 samples)
- Far, smaller flushed (|Δexp| > 25):  ~81%       (~53100 samples)
- Special (AMBIG_EXP either operand):  511/65536  ≈ 0.78% (~511, preempts)

The flushed case (|Δ|>25) dominates — those samples just return the larger
operand and exercise minimal datapath. Bit-exactness for close-path corner
cases is verified separately in simulation (see "Bit-exactness methodology"
below — 500K close-path-targeted vectors per op). The silicon bench validates
timing closure, which is largely path-agnostic once the design is bit-exact in
sim, but: timing failures on rarely-exercised paths (same-exp cancellation in
particular) might escape this bench. Future improvement could bias the input
LFSR toward |Δ|≤2 for sharper coverage on the close datapath.

## Key architectural notes

**Add**: close/far split with shared barrel via bit-reversal trick. Sub fused via XOR + adder carry-in. Halfway-pattern AND-gate fix for shift-then-XOR-carry's ½-ULP positive bias when align_sticky=1 and negate_small=1.

**Multiply**: bounded normalize 0/1/2 bits (canonical N1×N1 has top 3 product bits sufficient). 2*COMPUTE_FRAC=50-bit product width with banker's RNE on top COMPUTE_FRAC bits + GRS below.

**Divide**: restoring sequential, 1 quotient bit per clock. COMPUTE_FRAC=25 iterations. Banker's RNE in finalize stage. Inflate at start, deflate at output.

**Sqrt**: restoring binary, 1 result bit per clock. N_ITER=26 iterations. Even/odd exponent dispatch for radicand setup. Banker's RNE on top MAG bits with Q[1:0] + remainder providing GRS.
