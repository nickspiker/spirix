# Spirix Minimal Cores

Parameterized two's complement floating-point arithmetic in plain Verilog. No vendor primitives, no DSP inference directives, no external dependencies. Drop these files into any FPGA or ASIC project.

## Modules

| Module | Type | Latency | Interface |
|---|---|---|---|
| `spirix_addsub` | Add/Subtract | Combinational | `a, b, sub -> result` |
| `spirix_multiply` | Multiply | Combinational | `a, b -> result` |
| `spirix_divide` | Divide | FRAC_BITS+2 cycles | `clk, start -> done, result` |
| `spirix_sqrt` | Square Root | FRAC_BITS+2 cycles | `clk, start -> done, result` |
| `spirix_nr_divsqrt` | NR Divide/Sqrt | ~10-20 cycles | `clk, start, mode -> done, result` |

All modules are parameterized by `FRAC_BITS` and `EXP_BITS`. All include full edge case handling matching the Rust reference spec.

## Number Format

```
value = (fraction / 2^(FRAC_BITS-1)) * 2^exponent
```

- **Fraction**: signed two's complement, N1-normalized (top two bits always differ: `01...` positive, `10...` negative)
- **Exponent**: signed two's complement, unbiased
- **Sentinel**: `AMBIGUOUS_EXP = -2^(EXP_BITS-1)` encodes all non-normal states (zero, infinity, exploded, vanished, undefined)

No sign bit, no implicit leading one, no exponent bias, no positive/negative zero distinction.

## Why Floor-Only (No Rounding)

These cores truncate **away from infinity** rather than implementing banker's rounding (round-to-nearest-even).

Note: "away from infinity" is the correct Spirix-native term. Spirix has one infinity — unsigned, singular, the escaped state at overflow boundary. "Toward negative infinity" is an IEEE-ism that requires signed infinities to be directionally meaningful. On Spirix's number line, truncation moves results toward the representable interior, away from INFINITY.

**0. You always know you're short.** Floor guarantees the result is ≤ the true value. Always. The truncation error is bounded and directional — you can reason about your error budget. With banker's rounding you can overshoot, which means error sign is input-dependent and you need to track it.

**1. Exact remainders are exact.** Floor produces a remainder in `[0, divisor)` with no ambiguous regions. Round-toward-zero (the IEEE default for integer conversion) carves a double-wide capture basin at zero — integers at `±ε` both round toward 0, creating a flat spot in the tire, I mean, number line.

**2. Rounding is not free.** Banker's rounding requires guard/round/sticky calculation, a conditional increment, and rounding overflow detection (which fires on ~0.8% of additions and causes 521K mismatches if omitted in exhaustive 8-bit testing). Estimate ~25% area overhead on the adder datapath for rounding logic, with a negative timing impact. Not measured in isolation.

**3. Truncation is fast.** No conditional logic, no carry chain into the exponent. Just drop the bits. The speed difference is significant.

**4. Integer libraries don't provide rounding.** Spirix arithmetic is integer arithmetic — the fraction is a signed integer, the exponent is a signed integer. When you multiply two `i32` values in Rust, C or ASM, you get truncation. Nobody expects `3 * 5 / 2` to return 8 instead of 7. Floor is the native behavior of every integer ALU ever built.

The bench modules at `fpga/bench/spirix_*.v` (FRAC=25, EXP=8) add banker's rounding for the apples to apples IEEE f32 comparison in the paper. Those are the "with rounding" variants if you need them.

## Why Power-of-Two Widths

The Rust library types are `Scalar<iN, iM>` where N and M are Rust integer types: `i8`, `i16`, `i32`, `i64`, `i128`. The naming convention follows:

| Name | FRAC_BITS | EXP_BITS | Rust Type |
|---|---|---|---|
| F3E3 | 8 | 8 | `Scalar<i8, i8>` |
| F4E3 | 16 | 8 | `Scalar<i16, i8>` |
| F5E3 | 32 | 8 | `Scalar<i32, i8>` |
| F6E3 | 64 | 8 | `Scalar<i64, i8>` |
| F7E3 | 128 | 8 | `Scalar<i128, i8>` |

**FxEy means 2^x bit fraction, 2^y bit exponent.** F4E3 is a 16-bit fraction with an 8-bit exponent, not a 4-bit anything.

Power-of-two widths align with hardware integer types, memory bus widths, and register file entries. A 32-bit fraction + 8-bit exponent packs into a 40-bit bus naturally. Odd widths like 25-bit (used in `fpga/bench/` for binary32-equivalent precision) are useful for comparison benchmarks but not for production data paths where you want clean bus alignment.

The Verilog parameters accept any width (FRAC_BITS from 2-256, EXP_BITS from 2-256), so nothing stops you from using 25 or 53 or any other value. The power-of-two defaults are just the natural choice.

## Edge Cases

Every module handles the full Spirix state model:

- **Undefined** (fraction top 3 bits identical, not N0): passthrough
- **Zero** (frac=0, exp=AMBIG) and **Infinity** (frac=-1, exp=AMBIG): per-op rules
- **Exploded** (N1 frac, AMBIG exp) and **Vanished** (N2 frac, any exp): per-op rules
- **Normal** (N1 frac, non-AMBIG exp): computed result

Division and sqrt modules use an `S_SHORTCUT` FSM state to skip the iterative computation entirely for edge case inputs (1 cycle instead of FRAC+2).

For details on edge case behavior per operation, see the Rust reference at `src/implementations/`.

## Usage

Instantiate with your chosen precision:

```verilog
spirix_addsub #(.FRAC_BITS(32), .EXP_BITS(8)) adder (
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .sub(1'b0),
    .result_frac(r_frac), .result_exp(r_exp)
);

spirix_divide #(.FRAC_BITS(32), .EXP_BITS(8)) divider (
    .clk(clk), .start(start),
    .a_frac(a_frac), .a_exp(a_exp),
    .b_frac(b_frac), .b_exp(b_exp),
    .result_frac(r_frac), .result_exp(r_exp),
    .busy(busy), .done(done)
);
```

## Synthesis

Any Verilog-2001 toolchain. No vendor-specific primitives. Example with Yosys targeting ECP5:

```bash
yosys -p "read_verilog spirix_addsub.v; synth_ecp5 -nowidelut -top spirix_addsub"
```

For ASIC flows, these are pure combinational logic (addsub, multiply) or simple FSMs (divide, sqrt, nr_divsqrt) with no technology-specific cells.

The `spirix_nr_divsqrt` module exposes its multiplier port (`mul_a`, `mul_b`, `mul_prod`) so you can share the multiplier with other logic when the unit is idle, or connect it to a DSP block for higher performance.

## Verification

Test vector generators in `examples/` produce Rust-golden edge case and random vectors:

```bash
cargo run --example gen_div_edge_vectors     # division edge cases
cargo run --example gen_sqrt_edge_vectors    # sqrt edge cases
cargo run --example gen_addsub_edge_vectors  # add/sub edge cases
cargo run --example gen_mul_edge_vectors     # multiply edge cases
```

Simulation with Icarus Verilog:

```bash
iverilog -o tb spirix_divide.v your_testbench.v && vvp tb
```
