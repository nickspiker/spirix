//! Tests comparing bitwise `sqrt()` vs Newton-Raphson `sqrt_newton()`.
//!
//! `sqrt()` is the canonical bit-exact floor (subtractive restoring algorithm). `sqrt_newton()` is LUT-seeded Newton, which converges within ±1 ULP of the floor — it can't self-verify the floor because Spirix's multiply floors too (see the `sqrt_newton` docstring). These tests assert agreement to within 1 ULP at the fraction level, with matching exponent.
use spirix::{ScalarF3E3, ScalarF4E4, ScalarF5E5, ScalarF6E6, ScalarF7E7};

/// Asserts `bit` and `newt` are equal or differ by exactly 1 ULP at the fraction level (same exponent). Uses wrapping_add so it works uniformly for any signed fraction width without overflow concerns.
macro_rules! assert_within_1_ulp {
    ($bit:expr, $newt:expr, $width:literal, $val:expr) => {{
        let bit = $bit;
        let newt = $newt;
        let close = bit.exponent == newt.exponent
            && (bit.fraction == newt.fraction
                || bit.fraction.wrapping_add(1) == newt.fraction
                || newt.fraction.wrapping_add(1) == bit.fraction);
        assert!(
            close,
            "{}-bit sqrt agreement violated for val={}: bitwise={:#?} newton={:#?}",
            $width, $val, bit, newt
        );
    }};
}

#[test]
fn test_sqrt_8bit_methods_match() {
    for val in 0i8..=127 {
        let input = ScalarF3E3::from(val);
        if !input.is_normal() {
            continue;
        }
        assert_within_1_ulp!(input.sqrt(), input.sqrt_newton(), 8, val);
    }
}

#[test]
fn test_sqrt_16bit_methods_match() {
    for val in 0u8..=255 {
        let input = ScalarF4E4::from(val);
        if !input.is_normal() {
            continue;
        }
        assert_within_1_ulp!(input.sqrt(), input.sqrt_newton(), 16, val);
    }
}

#[test]
fn test_sqrt_32bit_methods_match() {
    // Representative sample.
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF5E5::from(val);
        if !input.is_normal() {
            continue;
        }
        assert_within_1_ulp!(input.sqrt(), input.sqrt_newton(), 32, val);
    }
}

#[test]
fn test_sqrt_64bit_methods_match() {
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF6E6::from(val);
        if !input.is_normal() {
            continue;
        }
        assert_within_1_ulp!(input.sqrt(), input.sqrt_newton(), 64, val);
    }
}

#[test]
fn test_sqrt_128bit_methods_match() {
    // Note: 128-bit sqrt_bb still uses multiplication (no U512 available).
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF7E7::from(val);
        if !input.is_normal() {
            continue;
        }
        assert_within_1_ulp!(input.sqrt(), input.sqrt_newton(), 128, val);
    }
}
