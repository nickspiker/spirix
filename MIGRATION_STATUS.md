# New Format Migration Status

## Committed and verified
- **Addition, Subtraction** — inflate/deflate pipeline, truth tables pass ✅
- **Multiplication** — inflate/deflate, truth table pass ✅
- **Division** — abs+inflate+unsigned div, truth table pass ✅
- **Modulus** — proper remainder + all escape paths, truth table pass ✅
- **Bitwise AND/OR/XOR** — inflate/wide-op/deflate, flagged for review ✅
- **Comparison** — cmp_unsigned correct for new format ✅
- **Classification** — is_undefined/is_vanished/is_exploded/is_uniform now gate on is_normal() ✅
- **is_positive/is_negative** — fixed infinite recursion ✅
- **Constants** — PrimInt-derived functions, no trait bounds ✅
- **Conversions** — IEEE ±∞→exploded, ±0→zero, [∞]→NaN ✅
- **Scalar→int** — inflate-based via I256, to_iN delegated to Into ✅
- **Undefined prefixes** — complete overhaul, signed hex, new allocation ✅
- **Truth tables** — standardized col OP row, all 5 ops verified ✅
- **Square** — delegates to multiplication (295→5 lines) ✅
- **Sqrt** — restoring binary (bit-exact floor) + Newton-Raphson (nearest) ✅

## Done but NOT committed (review these diffs!)
- `exponents/scalar.rs` — square(), sqrt(), sqrt_newton() rewritten
- sqrt: restoring binary, subtractive method, matches hardware spirix_sqrt_iter
- sqrt_newton: LUT-seeded, nearest-not-floor, oscillation detection
- Both verified: F3E3 127/127 match (except 1 floor vs nearest), F5E3 10000/10000

## Needs migration (old format code)
### normalize() — CRITICAL
`basic_scalar.rs:2042` — operates on stored fraction bits, not effective values.
Counts leading_same of STORED bits. In new format, stored leading_same ≠ effective
leading_same. However: floor() works by accident because low bits of stored = low
bits of effective. And normalize() is only called from constructors (From, conversion
paths), not arithmetic (which uses inflate/deflate directly). So the impact may be
limited, but it's conceptually wrong and should be rewritten.

### lb() (binary log) — HIGH
`exponents/scalar.rs:131` — builds raw fraction bits directly in stored encoding.
Then calls normalize(). Both are wrong for new format. Affects: ln(), lb(), exp(),
powb(). Rewrite to use inflate/deflate or compose from migrated ops.

### exp() — HIGH (depends on lb fix)
`exponents/scalar.rs:~275` — Taylor series that manipulates fraction bits directly.
Since it uses migrated arithmetic ops (multiply, add), it MIGHT work despite
old-format fraction construction. Needs testing.

### Trigonometry — MEDIUM
`trigonometry/scalar.rs` — uses migrated ops (add, sub, mul, div, frac, floor).
Mostly composition of already-working ops. One issue: exponent boundary check at
line 73 uses `FRAC-1` instead of `FRAC`. Floor() and frac() actually work correctly
for new format (low bits of stored = low bits of effective, mask approach valid).

### Formatting — MEDIUM
`formatting/scalar.rs` — digit extraction assumes stored bits = value.
Needs inflate before extracting digits. Purely cosmetic (Display impl).

### Random — LOW
`statistics/random.rs` — normalization loop counts leading_same of stored bits.
In new format, all stored patterns are valid normals, so the old normalization
concept doesn't directly apply. Needs rethinking of the exponent distribution
strategy.

### exponents/scalar_scalar.rs — MEDIUM
integer_power, logarithm, general power. Depends on square (fixed) and lb (broken).
integer_power uses repeated squaring which now works. General power uses lb (broken).

## Known bugs
1. **Subtraction escape path**: `ZERO - ONE` doesn't produce correct sign (boundary test fails)
2. **normalize()**: operates on stored bits instead of effective values
3. **lb()**: builds raw fraction bits in stored encoding

## Circle
Circle keeps old format — no changes needed to Circle-specific code.
Circle↔Scalar conversion boundary needs format translation (into_scalars, from_scalars).

## Tests to write
- Exhaustive F3E3 truth tables for bitwise (AND/OR/XOR)
- Exhaustive F3E3 truth tables for powers/roots/trig
- Patent and website truth table sync

## Restoring sqrt status (NOT YET WORKING)
The restoring binary sqrt code compiles but produces wrong results. Two issues:

1. **Radicand shift off by 1**: inflate values are 2x larger than old-format fractions.
   Old: `fraction << (FRAC+1 - even)`. New: should be `inflate << (FRAC - even)`.

2. **Output encoding**: the unsigned sqrt result needs conversion to new-format stored
   encoding. Old code just truncated `(result >> FRAC) as stored_type`. New format
   needs `(result >> FRAC) ^ mask` or equivalent XOR to get the implicit-sign encoding.

Newton-Raphson sqrt (sqrt_newton) works correctly and is verified against f64 gold.
The restoring binary (sqrt) should be fixed for bit-exactness but is NOT correct yet.

Keep both implementations — once restoring is fixed, bench to compare speed.
