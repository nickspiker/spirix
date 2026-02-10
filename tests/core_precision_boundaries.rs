use approx::assert_relative_eq;
use paste::paste;
use spirix::*;

/// Test precision boundaries and overflow/underflow behavior across all precision levels
/// Tests the transition between normal, exploded, and vanished states

macro_rules! test_precision_boundaries_for_type {
    ($scalar_type:ident, $fraction_bits:literal, $exponent_bits:literal) => {
        paste! {
            #[test]
            fn [<test_overflow_underflow_ $scalar_type:lower>]() {
                // Test maximum finite value
                let max_val = $scalar_type::MAX;
                assert!(max_val.is_normal());
                assert!(max_val.is_positive());

                // Test that doubling MAX produces exploded state
                let doubled_max = max_val * $scalar_type::from(2);
                assert!(doubled_max.exploded());
                assert!(doubled_max.is_positive());

                // Test minimum finite value (most negative)
                let min_val = $scalar_type::MIN;
                assert!(min_val.is_normal());
                assert!(min_val.is_negative());

                // Test that doubling MIN produces exploded state
                let doubled_min = min_val * $scalar_type::from(2);
                assert!(doubled_min.exploded());
                assert!(doubled_min.is_negative());

                // Test smallest positive normal value
                let min_pos = $scalar_type::MIN_POS;
                assert!(min_pos.is_normal());
                assert!(min_pos.is_positive());

                // Test that halving MIN_POS produces vanished state
                let halved_min_pos = min_pos / $scalar_type::from(2);
                assert!(halved_min_pos.vanished());
                assert!(halved_min_pos.is_positive());

                // Test largest negative normal value (closest to zero)
                let max_neg = $scalar_type::MAX_NEG;
                assert!(max_neg.is_normal());
                assert!(max_neg.is_negative());

                // Test that halving MAX_NEG produces vanished state
                let halved_max_neg = max_neg / $scalar_type::from(2);
                assert!(halved_max_neg.vanished());
                assert!(halved_max_neg.is_negative());
            }

            #[test]
            fn [<test_zero_crossing_ $scalar_type:lower>]() {
                // Test two's complement behavior around zero
                let small_pos = $scalar_type::MIN_POS;
                let small_neg = -small_pos;

                assert!(small_pos.is_positive());
                assert!(small_neg.is_negative());

                // Test addition that crosses zero
                let sum = small_pos + small_neg;
                assert!(sum.is_zero());

                // Test subtraction that crosses zero
                let diff1 = small_pos - small_pos;
                assert!(diff1.is_zero());

                let diff2 = small_neg - small_neg;
                assert!(diff2.is_zero());

                // Test multiplication that crosses zero
                let product = small_pos * $scalar_type::ZERO;
                assert!(product.is_zero());
            }

            #[test]
            fn [<test_normalization_levels_ $scalar_type:lower>]() {
                // Test N0 level (zero and infinity)
                let zero = $scalar_type::ZERO;
                assert!(zero.is_zero());
                assert_eq!(zero.normalization_level(), 0);

                let infinity = $scalar_type::INFINITY;
                assert!(infinity.is_infinite());
                assert_eq!(infinity.normalization_level(), 0);

                // Test N1 level (normal and exploded)
                let normal = $scalar_type::from(42);
                assert!(normal.is_normal());
                assert_eq!(normal.normalization_level(), 1);

                let exploded = $scalar_type::MAX * $scalar_type::from(10);
                assert!(exploded.exploded());
                assert_eq!(exploded.normalization_level(), 1);

                // Test N2 level (vanished)
                let vanished = $scalar_type::MIN_POS / $scalar_type::from(10);
                assert!(vanished.vanished());
                assert_eq!(vanished.normalization_level(), 2);

                // Test N3+ level (undefined states)
                let undefined = $scalar_type::ZERO / $scalar_type::ZERO;
                assert!(undefined.is_undefined());
                assert!(undefined.normalization_level() >= 3);
            }

            #[test]
            fn [<test_escaped_value_operations_ $scalar_type:lower>]() {
                // Test that exploded values maintain orientation through operations
                let pos_exploded = $scalar_type::MAX * $scalar_type::from(2);
                let neg_exploded = $scalar_type::MIN * $scalar_type::from(2);

                assert!(pos_exploded.exploded() && pos_exploded.is_positive());
                assert!(neg_exploded.exploded() && neg_exploded.is_negative());

                // Test absolute operations on exploded values
                let exploded_product = pos_exploded * neg_exploded;
                assert!(exploded_product.exploded() && exploded_product.is_negative());

                let exploded_quotient = pos_exploded / neg_exploded;
                assert!(exploded_quotient.is_normal() && exploded_quotient.is_negative());

                // Test that vanished values maintain orientation through operations
                let pos_vanished = $scalar_type::MIN_POS / $scalar_type::from(2);
                let neg_vanished = $scalar_type::MAX_NEG / $scalar_type::from(2);

                assert!(pos_vanished.vanished() && pos_vanished.is_positive());
                assert!(neg_vanished.vanished() && neg_vanished.is_negative());

                // Test absolute operations on vanished values
                let vanished_product = pos_vanished * neg_vanished;
                assert!(vanished_product.vanished() && vanished_product.is_negative());

                let vanished_quotient = pos_vanished / neg_vanished;
                assert!(vanished_quotient.is_normal() && vanished_quotient.is_negative());
            }

            #[test]
            fn [<test_precision_specific_operations_ $scalar_type:lower>]() {
                // Test operations that are sensitive to the specific precision level
                let one = $scalar_type::ONE;
                let epsilon = one / $scalar_type::from(1000000); // Approximate epsilon

                // Test that 1 + epsilon != 1 for this precision
                let one_plus_eps = one + epsilon;
                assert_ne!(one_plus_eps, one);
                assert!(one_plus_eps.is_normal());

                // Test that operations smaller than epsilon vanish
                let tiny = epsilon / $scalar_type::from(2);
                if tiny.vanished() {
                    // For precisions where epsilon/2 vanishes
                    assert!(tiny.is_positive());
                } else {
                    // For high precisions where epsilon/2 is still normal
                    assert!(tiny.is_normal());
                    assert!(tiny < epsilon);
                }

                // Test precision-specific rounding behavior
                let third = $scalar_type::ONE / $scalar_type::from(3);
                let reconstructed = third * $scalar_type::from(3);

                // Due to finite precision, this might not equal exactly 1
                if reconstructed.is_normal() {
                    let diff = (reconstructed - one);
                    let abs_diff = if diff.is_negative() { -diff } else { diff };
                    assert!(abs_diff <= epsilon * $scalar_type::from(2)); // Allow small rounding error
                }
            }
        }
    };
}

// Test all precision combinations
test_precision_boundaries_for_type!(ScalarF3E3, 8, 8);
test_precision_boundaries_for_type!(ScalarF4E3, 16, 8);
test_precision_boundaries_for_type!(ScalarF5E3, 32, 8);
test_precision_boundaries_for_type!(ScalarF6E3, 64, 8);
test_precision_boundaries_for_type!(ScalarF7E3, 128, 8);

test_precision_boundaries_for_type!(ScalarF3E4, 8, 16);
test_precision_boundaries_for_type!(ScalarF4E4, 16, 16);
test_precision_boundaries_for_type!(ScalarF5E4, 32, 16);
test_precision_boundaries_for_type!(ScalarF6E4, 64, 16);
test_precision_boundaries_for_type!(ScalarF7E4, 128, 16);

test_precision_boundaries_for_type!(ScalarF3E5, 8, 32);
test_precision_boundaries_for_type!(ScalarF4E5, 16, 32);
test_precision_boundaries_for_type!(ScalarF5E5, 32, 32);
test_precision_boundaries_for_type!(ScalarF6E5, 64, 32);
test_precision_boundaries_for_type!(ScalarF7E5, 128, 32);

test_precision_boundaries_for_type!(ScalarF3E6, 8, 64);
test_precision_boundaries_for_type!(ScalarF4E6, 16, 64);
test_precision_boundaries_for_type!(ScalarF5E6, 32, 64);
test_precision_boundaries_for_type!(ScalarF6E6, 64, 64);
test_precision_boundaries_for_type!(ScalarF7E6, 128, 64);

test_precision_boundaries_for_type!(ScalarF3E7, 8, 128);
test_precision_boundaries_for_type!(ScalarF4E7, 16, 128);
test_precision_boundaries_for_type!(ScalarF5E7, 32, 128);
test_precision_boundaries_for_type!(ScalarF6E7, 64, 128);
test_precision_boundaries_for_type!(ScalarF7E7, 128, 128);

#[test]
fn test_cross_precision_boundary_comparisons() {
    // Test behavior when comparing values across precision boundaries
    let small_precision = ScalarF3E3::from(42);
    let large_precision = ScalarF7E7::from(42);

    // Both should represent the same mathematical value
    let small_as_f32: f32 = small_precision.into();
    let large_as_f32: f32 = large_precision.into();

    assert_relative_eq!(small_as_f32, large_as_f32, epsilon = 1e-2); // Account for precision difference

    // Test precision loss when converting high precision to low precision
    let pi_high = ScalarF7E7::PI;
    let pi_high_f32: f32 = pi_high.into();
    let pi_low = ScalarF3E3::from(pi_high_f32);

    let pi_high_as_f32: f32 = pi_high.into();
    let pi_low_as_f32: f32 = pi_low.into();

    // Low precision should be less accurate
    let high_abs = if pi_high_as_f32 < 0.0 {
        -pi_high_as_f32
    } else {
        pi_high_as_f32
    };
    let low_abs = if pi_low_as_f32 < 0.0 {
        -pi_low_as_f32
    } else {
        pi_low_as_f32
    };
    let diff_abs = if (pi_high_as_f32 - pi_low_as_f32) < 0.0 {
        -(pi_high_as_f32 - pi_low_as_f32)
    } else {
        pi_high_as_f32 - pi_low_as_f32
    };
    assert!(high_abs > low_abs || diff_abs <= 1e-2);
}

#[test]
fn test_two_complement_consistency() {
    // Test that two's complement representation is consistent across operations
    let values = [
        ScalarF5E3::from(-1),
        ScalarF5E3::from(0),
        ScalarF5E3::from(1),
        ScalarF5E3::from(42),
        ScalarF5E3::from(-42),
    ];

    for &val in &values {
        // Test that negation is consistent with two's complement
        let negated = -val;
        let double_negated = -negated;

        if val.is_normal() && double_negated.is_normal() {
            assert_eq!(val, double_negated);
        }

        // Test that subtraction is consistent with addition of negation
        let zero = ScalarF5E3::ZERO;
        let sub_result = zero - val;
        let add_neg_result = zero + (-val);

        if sub_result.is_normal() && add_neg_result.is_normal() {
            assert_eq!(sub_result, add_neg_result);
        }
    }
}
