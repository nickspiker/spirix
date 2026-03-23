//! SIMD-optimized implementations for Spirix operations
//!
//! This module provides platform-specific SIMD implementations for common
//! Spirix operations, with runtime feature detection and fallback to scalar.
//!
//! ## Architecture
//!
//! - **x86_64**: AVX2 and SSE4.2 implementations
//! - **aarch64**: ARM NEON implementations (future)
//! - **wasm**: WebAssembly SIMD128 (future)
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
//! ```

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use crate::ScalarF4E4;

/// Subtract arrays of ScalarF4E4 using SIMD when available
///
/// Automatically dispatches to the best available implementation:
/// - x86-64: AVX2 (8×) → SSE4.2 (4×) → scalar
/// - ARM: NEON (4×) → scalar
/// - WASM: SIMD128 → scalar
pub fn scalar_subtract_batch(a: &[ScalarF4E4], b: &[ScalarF4E4], result: &mut [ScalarF4E4]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        return unsafe { x86_64::scalar_subtract_batch_avx2(a, b, result) };
    }

    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2"), target_feature = "sse4.2"))]
    {
        return unsafe { x86_64::scalar_subtract_batch_sse42(a, b, result) };
    }

    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    {
        return unsafe { aarch64::scalar_subtract_batch_neon(a, b, result) };
    }

    // Fallback to scalar implementation
    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
