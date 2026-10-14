# Changelog

All notable changes to Spirix are documented here.

## [Unreleased]

### Fixed
- Circle `<<`, `>>`, `<<=`, `>>=` still used the pre-AMBIG=0 sign-flip overflow tests, getting about half of all (exponent, shift) pairs wrong since 0.1.0: crossing 1.0 upward exploded (0.75 << 1), landing at or below [1, 2) from above vanished (3 >> 1, and x >> 0 for x in [1, 2)), and real overflow/underflow past the cycle edge wrapped silently to the far end of the range (2^126 << 3 gave 2^-127). Now mirrors the Scalar shift: cycle-widened position compared against the widened bounds, escapes keep the phase (exploded = fractions as-is, vanished = fractions >> 1). Scalar shifts were unaffected. Exhaustive regression sweep in `tests/shift_sweep.rs`.
- Circle add/sub (all six Circle/Scalar mixes) had no underflow check: near-cancellation at the bottom of the exponent range wrapped the exponent around the cycle, returning a normal value near 2^123 (or exploded) where the result should vanish. One shared bounds-checked tail now serves all six.
- Circle negate/conjugate vanished one binade early at the bottom of the range (`-x` at stored exponent 2), and an all-ones component renormalized one binade too far, halving the value.
- Circle AND/OR/XOR vanished one binade early (underflow test copied from Scalar, whose result exponent is one lower).
- Scalar `sin`/`cos`/`tan` of `MIN` returned 0/1/0: the exploded `|MIN|` was read as tiny. Now undefined, like `MAX`.
- `is_contiguous` rejected -1 (and every negative power of two at the low end) and accepted -2^FRAC and exploded values. Now a bounds check against `MIN_CONTIGUOUS`/`MAX_CONTIGUOUS`.
- Circle `MAX_CONTIGUOUS`/`MIN_CONTIGUOUS` scaled with twice the exponent width instead of the fraction width, right only when FRAC = 2*EXP (`CircleF5E3::MAX_CONTIGUOUS` was 32767.99, now 2^31-2).
- `{:#?}` exponent colours were swapped (integer binades showed the fractional colour).
- Circle OR against a far-smaller operand with a negative component ran `Scalar::normalize` on the other component, flipping its sign: (1 + 0.0156i) | tiny gave (-4.8e-7, -0.0156).
- Circle cross-width conversion (`CircleF5E4::from(&CircleF4E3)` and back) still used the old signed exponent encoding: ONE widened to about 4e9825, ZERO widened to a non-zero, narrowing 1000.0 gave 0.9765.
- `Circle::from((Scalar, Scalar))` misaligned components 128 or more binades apart: (2^64, 2^-64) gave (2^-64, 2^-64).
- Circle from an escaped Scalar (and so from out-of-range f64/f32) dropped the escape one class: `CircleF4E3::from(1e300)` was vanished, `from(1e-300)` undefined.
- Scalar narrowing (e.g. E4 to E3) out of range produced the wrong escape class or ZERO (-2^200 became ZERO); it now explodes/vanishes with the sign and phase kept.
- `Scalar::from(-2^-127)` (f64 and f32) returned ZERO instead of -vanished; `ScalarF7E3::from(i128::MIN)` exploded instead of returning -2^127 exactly.
- `to_i*`/`to_u*` at exponent widths of 16 bits or more wrapped the shift: 1.5·2^-32767 converted to 3.
- Circle `magnitude`, `magnitude_squared`, `sqrt`, and `ln` squared at the stored exponent and went undefined above about 2^62 or below 2^-62 at E3 (`sqrt` of 2^64 was ℘√↑); they now factor the shared exponent out.
- `sinh` of a large negative argument returned the positive vanished arm (`sinh(-112)` was +vanished); `sinh` and `cosh` now follow sign(x)·exp(|x|)/2 when an arm escapes, and `cosh(±89)` no longer explodes early.
- Circle/Circle and Scalar/Circle division wrapped the wide intermediate when a quotient component reached about 2, flipping its sign (0.22% of random Circle/Circle cases); Circle/Scalar now canonicalizes a de-normalized numerator first.
- `frac` of a tiny negative returned exactly 1.0, outside [0, 1).
- SIMD `scalar_subtract_batch` under AVX2 wrote its results into a temporary copy and returned the output untouched, and the AVX2 kernel used the old signed exponent compare with no normalization. The kernel now matches the scalar operator bit for bit (falling back per lane for escapes), and AVX2 is detected at runtime instead of only at compile time.

### Changed
- Scalar division floors its quotient, as the README documents, instead of truncating toward zero: inexact negative quotients are one unit lower than before. This also fixes reciprocals that truncated onto the unrepresentable -2^-127 and vanished.
- Circle `sqrt` uses the cancellation-free form (Im / (2 × larger root)), so low bits change for most inputs; the minor component of near-real roots was losing about 2^-10 of |root|.
- Out-of-range f64/f32 conversions keep the mantissa as escape phase instead of the canonical ±1 pattern.

### Deprecated
- `Scalar::normalize` is now a no-op: under N0 every bit pattern at a normal exponent is already canonical, and the old body corrupted values (1.5 became -0.5).

## [0.1.1] - 2026-07-04

### Fixed
- Shift and overflow panics in consumers' debug builds (overflow checks were masked in spirix's own profiles): square()'s exploded branch panicked thru pow → exp on boundary-adjacent significands, round()'s banker's tie at ±0.5, Circle construction from Scalars at E7 widths, and every escaped-path normalization shift. Wide shift primitives are now total (negative counts flip direction, counts past the width saturate to the mathematical limit); dev builds now run with overflow checks on so this class stays caught.
- The crates.io package no longer ships the patent/ and paper/ directories (also brought it under the upload size limit).

## [0.1.0] - 2026-07-03

First production-track release. Breaking change from the 0.0.x series.

### Changed (breaking)
- **New Scalar binary representation**: implicit sign bit for normal numbers (one extra bit of precision at every width) and the AMBIG=0 exponent convention. Stored 0.0.x values are not compatible. Circles remain unchanged.
- **Strict equality and ordering**: undefined, infinity, exploded, and vanished never compare equal to anything, including themselves. Escaped values order against normals where the answer is knowable; `partial_cmp` returns `None` for unordered pairs. Bit identity is available thru the pub `fraction`/`exponent` fields.
- Integer casts (`to_i32()` etc.) floor rather than truncate, matching Spirix's floor-based frac/division/modulus.
- Tensor/NN demo re-exports removed from the crate root; reach them via `spirix::tensor::`. `IntoScalars` is now re-exported at the root.

### Fixed
- `exp()`/`pow()` leaked wrong finite values instead of escaping past the exponent range (three distinct boundary bugs).
- `atan` computed the wrong continued fraction (missing k² numerators); now matches f64 to full precision. `atan2` inherited the fix, plus its own `is_negligible` sign bug on -2^k boundary values.
- Two-arg `log` resolves transfinite limits (log(0) = log(∞) = ∞, log base 0 = log base ∞ = 0) with correct value-vs-base undefined tags.
- `pow`: zero base resolves by exponent sign (0^+ = 0, 0^− = ∞); infinite base by pure sign dominance (∞^0.5 was returning zero); escaped bases resolve thru the multiply chain for integer exponents (class, parity sign, and phase all survive, so x^1 ≡ x) and by dominance for non-integer |p| > 1.
- Escaped-value phase (significand) now survives multiplication with the correct two's-complement sign convention; Circle orientation is preserved thru multiply, negate, conjugate, reciprocal, division, and integer/rotated powers.
- Circle: `Into<Complex<f64>>`/`<f32>` read every normal value at exactly half; sqrt's half-angle rounding could poison pure-real inputs with ℘√-; division wrapped the sign at the two's-complement fraction boundary and panicked on de-normalized operand pairs; pow/exp/ln/sqrt edge blocks resolved and correctly tagged.
- Circle classification is now a total partition: vanished/zero/infinite gained the ambiguous-exponent gate, and undefined catches every remaining AMBIG pattern.
- Two hangs found by fuzzing: `integer_power` spun forever on n = -2^MAX_EXP, and modulus used O(exp_diff) chunked reduction (effectively infinite at wide exponent widths; now modular exponentiation).

### Added
- Truth-table tests for every operation's class edges, both types, including escaped-orientation angle assertions.
- Numeric-reference suites: Scalar ops vs the IEEE f64 oracle over a shared value set; Circle ops vs `num_complex::Complex<f64>`; Scalar↔Circle real-axis consistency.
- Conversion reference: IEEE specials both directions, subnormal handling by width, integer-cast semantics, bit-exact f32 round-trip sweep.
- No-panic fuzz: every op total over raw bit patterns (exhaustive at F3E3 unary), including non-canonical patterns constructible thru the pub fields.
- `random()` statistical tests: class safety, [-1, +1) endpoints, per-binade occupancy (proper fraction extension).
- Circle `is_exploded()`/`is_vanished()` aliases for predicate-family parity with Scalar.

### Versions 0.0.8 thru 0.0.12
Final iterations of the old representation format; changes rolled into the 0.1.0 notes above.

## [0.0.7] - 2026-02-18

### Added
- **Tensor module** (`tensor`): Generic multi-dimensional array type `Tensor<T>` with element-wise arithmetic, `matmul`, `transpose`, `relu`, `scale`, and neural network primitives (`Linear`, `SimpleNet`, `SGD`, `mse_loss`, autodiff/backprop functions).
- **SIMD module** (`simd`): Architecture-dispatched `scalar_subtract_batch` with x86_64 AVX2/SSE4.2 and aarch64 NEON paths. Note: AVX2 normalization is in-progress; the function currently falls back to scalar arithmetic on paths where SIMD normalization is incomplete.
- **LUT module** (`lut`): Compile-time `SQRT_LUT` lookup table for Newton-Raphson square root seeding.
- **Statistics module**: `Scalar::sigmoid(x)` — logistic sigmoid function σ(x) = 1 / (1 + e^(-x)).
- `Scalar::sqrt_newton` — Newton-Raphson square root with LUT-seeded initial guess. Fixed termination condition (`new_y == y` instead of `new_y >= y`) to avoid oscillation-induced infinite loops.
- `Scalar::scalar_divide_scalar_newton` — LUT-seeded Newton-Raphson reciprocal division for 8–128-bit fraction widths.
- `Scalar::normalize` — previously `pub(crate)`, now fully public.

### Fixed
- Scalar-to-integer conversion: off-by-one in right-shift amount (`wrapping_add(1)` removed), correcting conversion of larger scalars to primitive integers.
- `format_circle` and `format_scalar`: missing closing `⦈`/`⦊` bracket on the zero branch of escaped values.
- `sqrt` Newton-Raphson termination: oscillation-induced infinite loop fixed.

### Documentation
- Extensive rustdoc added to `formatting/scalar.rs`, `formatting/circle.rs`, `formatting/colours.rs`, and `formatting/mod.rs`.
- README: added arithmetic truth tables for `+`, `−`, `×`, `÷` across all value states (normal, exploded, vanished, infinity, undefined).

## [0.0.6] - 2024 (initial public release-ish)

Initial release with core `Scalar<F, E>` and `Circle<F, E>` types, arithmetic, trigonometry, bitwise, comparison, modular arithmetic, and transcendental functions.

Prior versions were fixing cargo publish glitches