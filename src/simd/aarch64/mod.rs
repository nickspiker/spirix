//! ARM NEON SIMD implementations for Spirix (future)
//!
//! Placeholder for ARM NEON optimized implementations. Will provide feature parity with x86-64 AVX2/SSE implementations.

use crate::ScalarF4E4;

/// Subtract 4x ScalarF4E4 using NEON
///
/// # Safety
///
/// Requires NEON support (standard on all ARM64). Processes 4× i32 fractions and 4× i16 exponents per iteration.
#[target_feature(enable = "neon")]
pub unsafe fn scalar_subtract_batch_neon(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    // TODO: Implement NEON version
    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
