//! x86-64 SIMD implementations for Spirix
//!
//! Provides an AVX2 implementation of ScalarF4E4 subtract that is bit-identical to the scalar operator.
//!
//! ## Feature Detection
//!
//! Functions are marked with `#[target_feature(enable = "...")]` and require `unsafe` to call. The parent module dispatches to them when the feature is enabled at compile time, or when [`has_avx2`] reports it at runtime.
//!
//! ## Implementation Strategy
//!
//! - **Lanes**: each ScalarF4E4 is one 32-bit lane of a __m256i, fraction in the low half and exponent in the high half
//! - **Common case**: both operands normal, aligned subtract with per-lane variable shifts, normalize via an exact float conversion of the isolated top bit
//! - **Everything else**: lanes holding Zero, Infinity, escaped or undefined operands, or results that explode or vanish, are recomputed with the scalar operator

mod scalar_scalar;

pub use scalar_scalar::*;

use core::sync::atomic::{AtomicU8, Ordering};

static AVX2_STATE: AtomicU8 = AtomicU8::new(0);

/// Reports whether the running CPU and OS support AVX2, cached after the first call.
///
/// The crate is `no_std`, so this reads CPUID and XCR0 directly instead of using `is_x86_feature_detected!`.
pub fn has_avx2() -> bool {
    match AVX2_STATE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let detected = detect_avx2();
            AVX2_STATE.store(if detected { 2 } else { 1 }, Ordering::Relaxed);
            detected
        }
    }
}

fn detect_avx2() -> bool {
    use core::arch::x86_64::{__cpuid, __cpuid_count};
    // CPUID is always present on x86-64.
    let max_leaf = __cpuid(0).eax;
    if max_leaf < 7 {
        return false;
    }
    let leaf1 = __cpuid(1);
    let osxsave = leaf1.ecx & (1 << 27) != 0;
    let avx = leaf1.ecx & (1 << 28) != 0;
    if !osxsave || !avx {
        return false;
    }
    // The OS must save both XMM (bit 1) and YMM (bit 2) state across context switches.
    let xcr0 = unsafe { read_xcr0() };
    if xcr0 & 0b110 != 0b110 {
        return false;
    }
    __cpuid_count(7, 0).ebx & (1 << 5) != 0
}

/// # Safety
///
/// Requires XSAVE support with OSXSAVE set, which the caller checks via CPUID leaf 1.
#[target_feature(enable = "xsave")]
unsafe fn read_xcr0() -> u64 {
    core::arch::x86_64::_xgetbv(0)
}
