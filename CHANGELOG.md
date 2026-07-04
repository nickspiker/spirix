# Changelog

All notable changes to Spirix are documented here.

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