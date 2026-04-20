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
1. `circle_stored << 1` — plain shift; Circle's sign is already explicit in the
   MSB, no sign inference needed. Working in F directly or via a wider type
   gives the same low-FRAC bits.
2. **Normalize** so the Scalar's MSB lands in the right spot: count leading
   same bits, shift further, and adjust the exponent to compensate.
3. Truncate back to Scalar's F.

**Scalar → Circle**:
1. `inflate(scalar_stored, true)` — for normals this **XORs with the mask** to
   un-hide the implicit sign and recover the effective (FRAC+1)-bit signed value.
   Required because Scalar's stored doesn't carry the sign as a regular bit.
2. `>> 1` (arithmetic right shift, preserves sign).
3. Truncate to Circle's F.

The XOR is asymmetric: only needed going Scalar→Circle (to undo the implicit-
sign encoding). Circle→Scalar is just a shift + normalization — no XOR, no
sign inference, because Circle already carries the effective value directly.

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
