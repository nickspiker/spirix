//! ARM NEON SIMD implementations for Spirix (future)
//!
//! Placeholder for ARM NEON optimized implementations. Will provide feature parity with x86-64 AVX2/SSE implementations.

use crate::ScalarF4E4;

/// Subtract 4x ScalarF4E4 using NEON
///
/// # Safety
///
/// Requires NEON support (standard on all ARM64). Currently element-wise, bit-identical to `result[i] = a[i] - b[i]`.
#[target_feature(enable = "neon")]
pub unsafe fn scalar_subtract_batch_neon(
    a: &[ScalarF4E4],
    b: &[ScalarF4E4],
    result: &mut [ScalarF4E4],
) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());
    // No NEON kernel yet, so this is element-wise scalar subtract to stay bit-identical; the public dispatcher calls the scalar operator directly on aarch64.
    for i in 0..a.len() {
        result[i] = a[i] - b[i];
    }
}
