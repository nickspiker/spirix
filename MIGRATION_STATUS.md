# Migration Status

## Done ✅
- Addition, Subtraction, Multiplication, Division, Modulus (truth tables pass)
- Bitwise AND/OR/XOR/NOT, Comparison
- Classification gating on is_normal()
- is_positive/is_negative (method + recursion fix)
- Constants via PrimInt-derived functions
- IEEE conversions (±∞→exploded, ±0→zero, [∞]→NaN, subnormals)
- Scalar→int via inflate + I256 (fixed Into<iN>/Into<uN> for normals)
- From<iN> / From<uN> (negative handling + pow-of-2 boundary)
- From<f64> / From<f32> (runtime + const-fn)
- to_f64() / to_f32() (delegate to Into<>)
- Undefined prefix overhaul (signed hex)
- Truth tables standardized (col OP row); 6/6 pass
- Unary truth tables added (sqrt, lb/ln, exp/powb, square, not)
- Square specialized (single inflate, NEG_ONE special case)
- Sqrt: restoring binary (bit-exact floor) + Newton (LUT-seeded nearest)
- i128 inflate mask fix
- Division Newton + LUT deletion; scalar_power_scalar sign fix
- Wrapping ops pass + power-of-2 multiplies → shifts
- lb() direct-bit rewrite; scalar_negate fix; exp() sign fix
- Trig FRAC boundary fix (line 73)

## Circle ↔ Scalar Format Relationship (reference)

Circle and Scalar use different stored encodings for the same mathematical value:

| Format | Sign encoding | Magnitude bits | Value formula |
|---|---|---|---|
| Circle | Explicit (MSB = sign) | FRAC-1 | `stored * 2^(exp - FRAC + 1)` |
| Scalar | Implicit (~MSB = sign) | FRAC | `inflate(stored) * 2^(exp - FRAC)` |

**Translation equivalence:** `scalar_inflate = 2 × circle_stored` for the same value.

**Circle → Scalar** (extracting a component):
1. `sign_extend(circle_stored)` to wider type (no XOR needed)
2. `<< 1` (scale up by 2)
3. Renormalize to N-1 if the component wasn't individually normalized
4. `deflate` to Scalar's F (truncate low FRAC bits)

**Scalar → Circle**:
1. `inflate(scalar_stored, is_normal)` — for normals this **XORs with the mask** to undo implicit sign
2. `>> 1` (arithmetic shift, preserves sign)
3. `deflate` to Circle's F (truncate)

The XOR is asymmetric: needed going Scalar→Circle (to recover effective
magnitude from Scalar's implicit encoding), not needed the other direction.

## Still needs work

### Tests to write
- Exhaustive F3E3 truth tables for bitwise, powers/roots, trig
- Patent + website truth table sync
- Circle boundary ops (Scalar + Circle, Circle - Scalar, etc.)
- Exhaustive Circle↔Scalar round trips

### Boundary code still to convert
- `Scalar + Circle` and other mixed arithmetic (`implementations/addition/scalar_circle.rs` etc.)
- `Complex<f64>` / `Complex<f32>` → Circle via non-zero imaginary paths

### Circle-internal (not boundary)
- Circle reciprocal precision bug: `z.reciprocal()` ≠ `Circle::ONE / z` for same z.
  Both pure-Circle ops, but internal representations differ slightly.
  4 tests in `core_circle_reciprocal.rs` fail.

### Low-priority / cosmetic
- Formatting (`formatting/scalar.rs`) — works, but digit extraction assumes stored == effective. Display output is correct per testing; but format could be cleaner.
- Random (`statistics/random.rs`) — normalization loop uses leading_same of stored bits; concept doesn't apply to current Scalar format.
- `normalize()` in basic_scalar.rs — only called from Circle→Scalar conversion (now via `extract_component()` helper which doesn't use it).

### Tensor tests
- 8 failing tests in `tensor::*` — separate subsystem, may have its own format assumptions.
