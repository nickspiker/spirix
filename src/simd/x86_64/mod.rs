//! x86-64 SIMD implementations for Spirix
//!
//! Provides AVX2 and SSE4.2 optimized implementations for Spirix operations.
//!
//! ## Feature Detection
//!
//! Functions are marked with `#[target_feature(enable = "...")]` and require `unsafe` to call. The parent module handles runtime feature detection.
//!
//! ## Implementation Strategy
//!
//! - **Fractions**: i32 stored in __m256i (8x i32 with AVX2, 4x i32 with SSE)
//! - **Exponents**: i16 stored in __m128i (8x i16 with AVX2, 4x i16 with SSE)
//! - **Ambiguous handling**: Detect with SIMD compare, fallback to scalar for edge cases
//! - **Normalization**: Use lzcnt intrinsics (AVX2) or scalar fallback

mod scalar_scalar;

pub use scalar_scalar::*;
