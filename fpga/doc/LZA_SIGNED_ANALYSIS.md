# Leading Zero/Same Anticipation for Signed Two's Complement

## Summary

LZA (Leading Zero Anticipation) is a standard technique in IEEE 754 floating-point
adders to predict the normalization shift amount in parallel with the carry-propagate
addition. This eliminates the serial add→CLZ→barrel dependency chain in the close path.

**All known LZA algorithms assume sign-magnitude representation with a guaranteed-positive
result.** They do not extend to signed two's complement normalization where the result
sign is unknown until the carry chain completes.

This document records our investigation into adapting LZA for Spirix's N1-normalized
signed two's complement floating-point format.

## Context

Spirix close path: for operands with exp_diff ≤ 1, massive cancellation can occur.
The result must be N1-normalized (top 2 bits differ), requiring a leading-same-digit
count (LSDC) and barrel shift. The serial chain add→LSDC→barrel is the critical path
bottleneck that prevents sharing the barrel shifter in a 3-stage pipeline.

N1 normalization counts "leading same" bits (sign extension), not "leading zeros" as
in IEEE 754. For a positive result, leading same = leading 0s. For negative, leading
same = leading 1s. The result sign depends on the carry-out — unknown without the
carry chain.

## Approaches Tested

### 1. Half-Sum XOR-Adjacent (HS_S = A ^ B)

T = A XOR B gives the carry-save sum (sum without carries). XOR-adjacent on T
(T[i] ^ T[i-1]) detects transitions in the propagate pattern.

- Works when T pattern is a solid run (all propagate, no generate/kill "holes")
- Fails when isolated G (A[i] AND B[i]) or Z (~A[i] AND ~B[i]) positions create
  false positives — carries propagate thru holes, shifting the actual boundary
- **Result: 6006 failures / 1M tests** (completely wrong for same-sign close)
- After adding eff_sub detection (LZA only for opposite-sign): **1165 failures / 1M**
- Remaining failures off by 3-4 positions, not the expected ±1

### 2. HS_S | HS_C (XOR | AND<<1)

Combined half-sum and half-carry. Marks positions where either a sum bit or a
carry-out bit is set.

- **90.6% within ±1** on close-path constrained random inputs (N1-normalized,
  exp_diff ≤ 1, opposite leading bits)
- Better than P/G/Z but not bounded — can still be off by 2+ positions
- Not usable as a proper LZA without a correction mechanism that itself requires
  the actual count (circular)

### 3. Textbook P/G/Z (Schmookler-Nowka / Quach-Flynn)

f[i] = P[i] XOR P[i-1] XOR G[i-1]

Where P = A XOR B (propagate), G = A AND B (generate). This is the standard
textbook LZA formulation that adds carry-generation correction to XOR-adjacent.

- Designed for unsigned magnitude subtraction A - B with A > B
- **~75% within ±1** on close-path inputs — worse than HS_S|HS_C
- The formulation solves a different problem (leading zeros of a known-positive
  result) and fails on signed leading-same detection

### 4. Suzuki Formulation

Alternative LZA prediction function from the literature.

- **Same ~75% result** as Schmookler-Nowka
- Same underlying assumption: unsigned, positive result

### 5. Sign Extension

Tested whether sign-extending the operands before LZA computation would help by
providing additional context bits.

- **Null result**: HS_S|HS_C with sign extension is bit-for-bit identical to
  without (redundant bit carries no new information)
- P/G/Z actually gets worse in the extended domain

## Why It Doesn't Work

The fundamental issue is that "leading same" in signed two's complement depends on
the result sign, which is determined by the carry-out of the addition:

1. **Positive result (01...)**: leading same = count of leading 0s
2. **Negative result (10...)**: leading same = count of leading 1s

For close-path operands (one 01..., one 10...), the result sign depends on which
operand has larger magnitude — which IS the carry-out. Without the carry chain,
you don't know whether you're counting leading 0s or leading 1s.

Every published LZA avoids this by either:
- Using sign-magnitude (result is always positive after magnitude subtraction)
- Running dual LZAs and selecting based on result sign
- Using a dual adder (sum and 2's complement simultaneously)

Intel US7024439 (expired) is the closest — sign-agnostic approach — but still
resolves to a positive value before normalizing.

## Conclusion

Post-sum CLZ (compressBy2 + tree) is the correct approach for Spirix's signed
close path. The CLZ is ~3-4 LUT4 levels and fits alongside the adder in a
pipeline stage, giving 68 MHz on ECP5 in the 4-stage shared-barrel design.

A novel signed-LZA that predicts leading-same-digit count without sign resolution
would enable a 3-stage shared-barrel pipeline (~570 LUT4, ~70 MHz, 3-cycle latency).
This remains an open research problem.

## Pipeline Design Comparison

| Design                        | LUT4 | Fmax     | FFs | Latency |
|-------------------------------|------|----------|-----|---------|
| Combinational (spirix_add)    | 797  | ~39 MHz  | 0   | 0       |
| 3-stage, two barrels (pipe)   | 619  | ~64 MHz  | 261 | 3       |
| 3-stage, shared barrel (3s)   | 640  | ~55 MHz  | 274 | 3       |
| **4-stage, shared barrel (4)** | **573** | **~68 MHz** | **339** | **4** |
| HardFloat addRecFN (IEEE)     | 635  | ~40 MHz  | 0   | 0       |
| cvfpu FMA (IEEE)              | 1845 | ~17 MHz  | —   | 0       |

## Files

- `rtl/spirix_add_pipe3lza.v` — experimental 3-stage LZA pipeline (not working)
- `rtl/spirix_add_pipe3s.v` — 3-stage shared barrel, no LZA (working, but slower)
- `rtl/spirix_add_pipe4.v` — 4-stage shared barrel (best working design)
- `rtl/spirix_add_pipe.v` — 3-stage, two barrels (best 3-stage design)
