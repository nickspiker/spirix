/// Tests comparing Newton-Raphson sqrt() vs non-restoring sqrt_bb()
/// Ensures both algorithms produce bit-identical results
use spirix::{ScalarF3E3, ScalarF4E4, ScalarF5E5, ScalarF6E6, ScalarF7E7};

#[test]
fn test_sqrt_8bit_methods_match() {
    for val in 0i8..=127 {
        let input = ScalarF3E3::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        assert_eq!(
            newton.fraction, bitwise.fraction,
            "8-bit fraction mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
        assert_eq!(
            newton.exponent, bitwise.exponent,
            "8-bit exponent mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
    }
}

#[test]
fn test_sqrt_16bit_methods_match() {
    for val in 0u8..=255 {
        let input = ScalarF4E4::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        assert_eq!(
            newton.fraction, bitwise.fraction,
            "16-bit fraction mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
        assert_eq!(
            newton.exponent, bitwise.exponent,
            "16-bit exponent mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
    }
}

#[test]
fn test_sqrt_32bit_methods_match() {
    // Test a representative sample
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF5E5::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        assert_eq!(
            newton.fraction, bitwise.fraction,
            "32-bit fraction mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
        assert_eq!(
            newton.exponent, bitwise.exponent,
            "32-bit exponent mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
    }
}

#[test]
fn test_sqrt_64bit_methods_match() {
    // Test a representative sample
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF6E6::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        assert_eq!(
            newton.fraction, bitwise.fraction,
            "64-bit fraction mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
        assert_eq!(
            newton.exponent, bitwise.exponent,
            "64-bit exponent mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
    }
}

#[test]
fn test_sqrt_128bit_methods_match() {
    // Test a representative sample
    // Note: 128-bit sqrt_bb still uses multiplication (no U512 available)
    for val in (0u16..=10000).step_by(10) {
        let input = ScalarF7E7::from(val);
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        assert_eq!(
            newton.fraction, bitwise.fraction,
            "128-bit fraction mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
        assert_eq!(
            newton.exponent, bitwise.exponent,
            "128-bit exponent mismatch for val={}: newton={:#?} bitwise={:#?}",
            val, newton, bitwise
        );
    }
}
