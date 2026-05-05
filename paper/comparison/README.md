# Spirix Paper — Comparison Harnesses

Standalone bit-accurate implementations of Spirix v0.1 N0 arithmetic at binary32-equivalent precision, paired with IEEE 754 binary32 comparison harnesses. The numbers reported in the paper come from this code.

This crate is **deliberately independent** of the production `spirix/` library. Each binary contains a self-contained implementation of one operation, so a reader can verify the algorithm in one place without crossing trait boundaries or generic instantiation. The algorithms here mirror the bit-level behavior of the silicon-verified Verilog cores.

## Format

- **FRAC = 24** — 24-bit two's complement fraction (non-power-of-2 width, custom)
- **EXP = 8** — 8-bit two's complement exponent
- **Total: 32 bits**, bit-equivalent to IEEE 754 binary32
- **N0 normalization** — stored MSB encodes sign via implicit complement (MSB=1 reads positive, MSB=0 reads negative); normal magnitudes occupy [+1, +2) ∪ [-2, -1) (asymmetric per two's complement)
- **AMBIG_EXP = i8::MAX (127)** — single sentinel exponent for non-normal states (zero, infinity, exploded, vanished, undefined)
- **Rounding: banker's** (round-to-nearest-even, RNE) — matches IEEE 754 default and the silicon Verilog cores

## Running

```sh
cd paper/comparison
cargo run --release --bin add_v01_vs_ieee
```

## Binaries

- `add_v01_vs_ieee` — Spirix v0.1 add vs native f32 add, 10M random pairs, categorized output
- `mul_v01_vs_ieee` — *(planned)*
- `fma_v01_vs_ieee` — *(planned)*

## Output categories

| Category | Meaning |
|---|---|
| **Exact** | Bit-for-bit match between Spirix and IEEE results |
| **1 ULP rounding** | Differ by one unit in the last place — valid rounding choice at boundary |
| **Spirix vanished, IEEE → 0** | Spirix produced a vanished state preserving phase; IEEE truncated to zero |
| **Spirix vanished, IEEE → denormal** | Spirix vanished; IEEE produced a denormal (reduced-precision sub-MIN_NORMAL value) |
| **Spirix exploded, IEEE → ±∞** | Spirix produced an exploded state preserving phase; IEEE truncated to signed infinity |
| **Spirix exploded, IEEE → finite** | Spirix exploded; IEEE produced a finite normal — should not happen in well-formed comparison |
| **Off by more (unexpected)** | Anything else — should be near zero count; investigate if not |

The first two categories are standard IEEE comparison results. The next four are *architectural distinctions* — Spirix and IEEE handle out-of-range results differently, so these are not arithmetic errors. The last category catches any unexpected delta and should always have zero count for properly-implemented algorithms.
