//! SIMD-optimized implementations for Spirix operations
//!
//! This module provides platform-specific batch implementations of Spirix operations. Every batch function returns results bit-identical to applying the scalar operator element-wise, for every input class.
//!
//! ## Architecture
//!
//! - **x86_64**: AVX2 kernel, selected at compile time with `target-feature=+avx2` or at runtime via CPUID; element-wise scalar otherwise
//! - **aarch64**: element-wise scalar (NEON kernel not yet written)
//! - **wasm**: element-wise scalar (SIMD128 kernel not yet written)
//!
//! ## Usage
//!
//! ```rust
//! use spirix::simd::scalar_subtract_batch;
//! use spirix::ScalarF4E4;
//!
//! let a = vec![ScalarF4E4::from(1.0); 1000];
//! let b = vec![ScalarF4E4::from(0.5); 1000];
//! let mut result = vec![ScalarF4E4::ZERO; 1000];
//!
//! scalar_subtract_batch(&a, &b, &mut result);
//! assert!(result.iter().all(|r| *r == 0.5));
//! ```

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use crate::ScalarF4E4;

/// Subtracts arrays of ScalarF4E4 element-wise, bit-identical to `result[i] = a[i] - b[i]`.
///
/// Dispatches to the best available implementation:
/// - x86-64: AVX2 (8 lanes) when enabled at compile time or detected at runtime, otherwise element-wise scalar
/// - other targets: element-wise scalar
///
/// # Panics
///
/// Panics if the three slices differ in length.
// With AVX2 enabled at compile time the early return makes the scalar loop statically unreachable; that is the intended dispatch shape, not dead code.
#[allow(unreachable_code)]
pub fn scalar_subtract_batch(a: &[ScalarF4E4], b: &[ScalarF4E4], result: &mut [ScalarF4E4]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        return unsafe { x86_64::scalar_subtract_batch_avx2(a, b, result) };
    }

    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    {
        if x86_64::has_avx2() {
            return unsafe { x86_64::scalar_subtract_batch_avx2(a, b, result) };
        }
    }

    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
