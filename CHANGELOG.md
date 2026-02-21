# Changelog

All notable changes to Spirix are documented here.

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