# Migration Status

## Done ✅
### Committed
- Addition, Subtraction, Multiplication, Division, Modulus — inflate/deflate, truth tables pass
- Bitwise AND/OR/XOR, Comparison
- Classification gating on is_normal()
- is_positive/is_negative recursion fix
- Constants via PrimInt-derived functions
- IEEE conversions (±∞→exploded, ±0→zero, [∞]→NaN)
- Scalar→int via inflate + I256
- Undefined prefix overhaul (signed hex)
- Truth tables standardized (col OP row)
- Square specialized (single inflate, NEG_ONE special case)
- Sqrt: restoring binary (bit-exact floor) + Newton (LUT-seeded nearest)
- i128 inflate mask fix
- Into<f64>/Into<f32> for i64 and i128 widths
- Division Newton + LUT deletion
- Wrapping ops pass + power-of-2 multiplies → shifts

### Uncommitted (this session)
- **lb() rewrite**: direct bit-OR into u128 accumulator, ~15× faster than Scalar adds
- **scalar_negate fix**: replaced broken bit-trick with `Self::pos_one_normal()`
- **From<iN> fix**: handles negative integers (negate abs + power-of-2 boundary shift)
- **From<f64>/From<f32> fix**: cast to F before shifting (was overflowing for FRAC > intermediate width)
- **exp() sign fix**: `is_negative()` method instead of stored MSB check
- **Comment cleanup**: removed all "old format"/"new format" archaeology
- **Side effect**: subtraction `ZERO - ONE` boundary now passes → truth tables 6/6

## Still needs work

### Tests to write (NEXT)
- Exhaustive F3E3 conversion suite covering:
  - All 256 i8 values → Scalar<i8,i8> → i8 round-trip
  - All 256 stored Scalar<i8,i8> → f32, f64
  - Constants, primes, powers of 2, **negative powers of 2** (always a problem)
  - All edges: ZERO, ONE, NEG_ONE, MIN, MAX, MIN_POS, MAX_NEG, exploded/vanished/undefined
  - Cross-width Scalar→Scalar (i8↔i16↔i32↔i64↔i128)
  - Round-trip property: f64 → Scalar → f64 within ULP bound
- Exhaustive F3E3 truth tables for bitwise, powers/roots, trig
- Patent + website truth table sync

### Code still using old-format assumptions
- **normalize()** (`basic_scalar.rs`): counts leading_same of stored bits. Only called from constructors, but conceptually wrong.
- **Trigonometry** (`trigonometry/scalar.rs`): exponent boundary check at line 73 uses `FRAC-1` instead of `FRAC`. Otherwise composition of working ops.
- **Formatting** (`formatting/scalar.rs`): digit extraction assumes stored == effective. Cosmetic (Display impl).
- **Random** (`statistics/random.rs`): normalization loop counts leading_same of stored bits — doesn't apply now.
- **exponents/scalar_scalar.rs**: integer_power should work via repeated squaring; general power and logarithm need testing now that lb works.

### Currently failing lib tests
- `from_f32_is_const` / `from_f32_matches_runtime` — const-fn From<f32> path I haven't touched
- `into_f32_agrees_with_to_f32` / `into_f64_agrees_with_to_f64` — my Into<> fix likely diverged from `to_*()` method
- `to_f32_special_cases` / `to_f64_special_cases`
- 8 tensor tests (separate subsystem)

## Circle
Circle keeps old format. No changes to Circle code itself; only Scalar↔Circle boundary needs translation.
