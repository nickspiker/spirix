use approx::assert_relative_eq;
use proptest::prelude::*;
use spirix::*;

// Property-based tests for mathematical invariants and properties
// These tests use randomly generated inputs to verify mathematical properties hold

#[cfg(test)]
mod scalar_properties {
    use super::*;

    // Strategy for generating reasonable scalar values for testing
    fn scalar_strategy() -> impl Strategy<Value = f32> {
        prop_oneof![
            // Normal range values
            -1000.0f32..1000.0f32,
            // Small values
            -1.0f32..1.0f32,
            // Integer values
            -100.0f32..100.0f32,
            // Edge cases
            Just(0.0f32),
            Just(1.0f32),
            Just(-1.0f32),
        ]
    }

    proptest! {
        #[test]
        fn test_scalar_addition_commutativity(
            a in scalar_strategy(),
            b in scalar_strategy()
        ) {
            let sa = ScalarF5E3::from(a);
            let sb = ScalarF5E3::from(b);

            let left = sa + sb;
            let right = sb + sa;

            // Only test if both results are normal (avoid undefined/escaped values)
            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_scalar_multiplication_commutativity(
            a in scalar_strategy(),
            b in scalar_strategy()
        ) {
            let sa = ScalarF5E3::from(a);
            let sb = ScalarF5E3::from(b);

            let left = sa * sb;
            let right = sb * sa;

            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-5);
            }
        }

        #[test]
        fn test_scalar_addition_associativity(
            a in scalar_strategy(),
            b in scalar_strategy(),
            c in scalar_strategy()
        ) {
            let sa = ScalarF5E3::from(a);
            let sb = ScalarF5E3::from(b);
            let sc = ScalarF5E3::from(c);

            let left = (sa + sb) + sc;
            let right = sa + (sb + sc);

            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-4);
            }
        }

        #[test]
        fn test_scalar_multiplication_associativity(
            a in scalar_strategy(),
            b in scalar_strategy(),
            c in scalar_strategy()
        ) {
            let sa = ScalarF5E3::from(a);
            let sb = ScalarF5E3::from(b);
            let sc = ScalarF5E3::from(c);

            let left = (sa * sb) * sc;
            let right = sa * (sb * sc);

            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-4);
            }
        }

        #[test]
        fn test_scalar_distributivity(
            a in scalar_strategy(),
            b in scalar_strategy(),
            c in scalar_strategy()
        ) {
            let sa = ScalarF5E3::from(a);
            let sb = ScalarF5E3::from(b);
            let sc = ScalarF5E3::from(c);

            let left = sa * (sb + sc);
            let right = sa * sb + sa * sc;

            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-3);
            }
        }

        #[test]
        fn test_scalar_additive_identity(a in scalar_strategy()) {
            let sa = ScalarF5E3::from(a);
            let zero = ScalarF5E3::ZERO;

            let result = sa + zero;

            if sa.is_normal() && result.is_normal() {
                let orig_val: f32 = sa.into();
                let result_val: f32 = result.into();
                assert_relative_eq!(orig_val, result_val, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_scalar_multiplicative_identity(a in scalar_strategy()) {
            let sa = ScalarF5E3::from(a);
            let one = ScalarF5E3::ONE;

            let result = sa * one;

            if sa.is_normal() && result.is_normal() {
                let orig_val: f32 = sa.into();
                let result_val: f32 = result.into();
                assert_relative_eq!(orig_val, result_val, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_scalar_additive_inverse(a in scalar_strategy()) {
            let sa = ScalarF5E3::from(a);
            let neg_sa = -sa;
            let sum = sa + neg_sa;

            if sa.is_normal() && neg_sa.is_normal() && sum.is_normal() {
                let sum_val: f32 = sum.into();
                assert_relative_eq!(sum_val, 0.0, epsilon = 1e-5);
            }
        }

        #[test]
        fn test_scalar_multiplicative_inverse(a in scalar_strategy()) {
            // Skip zero and very small values
            if a.abs() > 1e-3 {
                let sa = ScalarF5E3::from(a);
                let recip = sa.reciprocal();
                let product = sa * recip;

                if sa.is_normal() && recip.is_normal() && product.is_normal() {
                    let product_val: f32 = product.into();
                    assert_relative_eq!(product_val, 1.0, epsilon = 1e-4);
                }
            }
        }

        #[test]
        fn test_scalar_power_laws(
            base in 0.1f32..10.0f32,
            exp1 in -3.0f32..3.0f32,
            exp2 in -3.0f32..3.0f32
        ) {
            let sb = ScalarF5E3::from(base);
            let se1 = ScalarF5E3::from(exp1);
            let se2 = ScalarF5E3::from(exp2);

            // Test a^(x+y) = a^x * a^y
            let left = sb.pow(se1 + se2);
            let right = sb.pow(se1) * sb.pow(se2);

            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-2);
            }
        }

        #[test]
        fn test_scalar_exp_ln_inverse(a in 0.01f32..100.0f32) {
            let sa = ScalarF5E3::from(a);
            let ln_sa = sa.ln();
            let exp_ln_sa = ln_sa.exp();

            if sa.is_normal() && ln_sa.is_normal() && exp_ln_sa.is_normal() {
                let orig_val: f32 = sa.into();
                let result_val: f32 = exp_ln_sa.into();
                assert_relative_eq!(orig_val, result_val, epsilon = 1e-3);
            }
        }

        #[test]
        fn test_scalar_trig_identities(a in -10.0f32..10.0f32) {
            let sa = ScalarF5E3::from(a);
            let sin_a = sa.sin();
            let cos_a = sa.cos();

            if sin_a.is_normal() && cos_a.is_normal() {
                // sin²(a) + cos²(a) = 1
                let sin_squared = sin_a.square();
                let cos_squared = cos_a.square();
                let identity = sin_squared + cos_squared;

                if identity.is_normal() {
                    let identity_val: f32 = identity.into();
                    assert_relative_eq!(identity_val, 1.0, epsilon = 1e-3);
                }
            }
        }

        #[test]
        fn test_scalar_sqrt_square_inverse(a in 0.0f32..100.0f32) {
            let sa = ScalarF5E3::from(a);
            let sqrt_sa = sa.sqrt();
            let sqrt_squared = sqrt_sa.square();

            if sa.is_normal() && sqrt_sa.is_normal() && sqrt_squared.is_normal() {
                let orig_val: f32 = sa.into();
                let result_val: f32 = sqrt_squared.into();
                assert_relative_eq!(orig_val, result_val, epsilon = 1e-4);
            }
        }
    }
}

#[cfg(test)]
mod circle_properties {
    use super::*;

    // Strategy for generating complex numbers
    pub fn circle_strategy() -> impl Strategy<Value = (f32, f32)> {
        prop_oneof![
            // Normal range
            (-10.0f32..10.0f32, -10.0f32..10.0f32),
            // Small values
            (-1.0f32..1.0f32, -1.0f32..1.0f32),
            // Pure real
            (-10.0f32..10.0f32, Just(0.0f32)),
            // Pure imaginary
            (Just(0.0f32), -10.0f32..10.0f32),
        ]
    }

    proptest! {
        #[test]
        fn test_complex_addition_commutativity(
            (a_real, a_imag) in super::circle_properties::circle_strategy(),
            (b_real, b_imag) in circle_strategy()
        ) {
            let ca = CircleF5E3::from((a_real, a_imag));
            let cb = CircleF5E3::from((b_real, b_imag));

            let left = ca + cb;
            let right = cb + ca;

            if left.is_normal() && right.is_normal() {
                let left_real: f32 = left.r().into();
                let left_imag: f32 = left.i().into();
                let right_real: f32 = right.r().into();
                let right_imag: f32 = right.i().into();

                assert_relative_eq!(left_real, right_real, epsilon = 1e-6);
                assert_relative_eq!(left_imag, right_imag, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_complex_multiplication_commutativity(
            (a_real, a_imag) in super::circle_properties::circle_strategy(),
            (b_real, b_imag) in circle_strategy()
        ) {
            let ca = CircleF5E3::from((a_real, a_imag));
            let cb = CircleF5E3::from((b_real, b_imag));

            let left = ca * cb;
            let right = cb * ca;

            if left.is_normal() && right.is_normal() {
                let left_real: f32 = left.r().into();
                let left_imag: f32 = left.i().into();
                let right_real: f32 = right.r().into();
                let right_imag: f32 = right.i().into();

                assert_relative_eq!(left_real, right_real, epsilon = 1e-5);
                assert_relative_eq!(left_imag, right_imag, epsilon = 1e-5);
            }
        }

        #[test]
        fn test_complex_conjugate_properties(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let z = CircleF5E3::from((a_real, a_imag));
            let conj_z = z.conjugate();
            let conj_conj_z = conj_z.conjugate();

            if z.is_normal() && conj_z.is_normal() && conj_conj_z.is_normal() {
                // conjugate(conjugate(z)) = z
                let orig_real: f32 = z.r().into();
                let orig_imag: f32 = z.i().into();
                let double_conj_real: f32 = conj_conj_z.r().into();
                let double_conj_imag: f32 = conj_conj_z.i().into();

                assert_relative_eq!(orig_real, double_conj_real, epsilon = 1e-6);
                assert_relative_eq!(orig_imag, double_conj_imag, epsilon = 1e-6);

                // z * conjugate(z) = |z|²
                let product = z * conj_z;
                let magnitude_squared = z.magnitude_squared();

                if product.is_normal() && magnitude_squared.is_normal() {
                    let product_real: f32 = product.r().into();
                    let product_imag: f32 = product.i().into();
                    let mag_sq_val: f32 = magnitude_squared.into();

                    assert_relative_eq!(product_real, mag_sq_val, epsilon = 1e-4);
                    assert_relative_eq!(product_imag, 0.0, epsilon = 1e-5);
                }
            }
        }

        #[test]
        fn test_complex_magnitude_properties(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let z = CircleF5E3::from((a_real, a_imag));
            let magnitude = z.magnitude();
            let magnitude_squared = z.magnitude_squared();

            if z.is_normal() && magnitude.is_normal() && magnitude_squared.is_normal() {
                // |z|² = |z| * |z|
                let mag_val: f32 = magnitude.into();
                let mag_sq_val: f32 = magnitude_squared.into();
                let mag_squared_calculated = mag_val * mag_val;

                assert_relative_eq!(mag_sq_val, mag_squared_calculated, epsilon = 1e-4);

                // |z| >= 0
                assert!(mag_val >= -1e-6); // Allow small epsilon for floating point

                // |z| = 0 iff z = 0
                if mag_val < 1e-6 {
                    let real_val: f32 = z.r().into();
                    let imag_val: f32 = z.i().into();
                    assert!(real_val.abs() < 1e-5 && imag_val.abs() < 1e-5);
                }
            }
        }

        #[test]
        fn test_complex_multiplication_magnitude_property(
            (a_real, a_imag) in super::circle_properties::circle_strategy(),
            (b_real, b_imag) in circle_strategy()
        ) {
            let ca = CircleF5E3::from((a_real, a_imag));
            let cb = CircleF5E3::from((b_real, b_imag));

            let product = ca * cb;
            let mag_product = product.magnitude();
            let mag_a = ca.magnitude();
            let mag_b = cb.magnitude();
            let mag_product_expected = mag_a * mag_b;

            if ca.is_normal() && cb.is_normal() && product.is_normal() &&
               mag_product.is_normal() && mag_a.is_normal() && mag_b.is_normal() &&
               mag_product_expected.is_normal() {
                // |z₁ * z₂| = |z₁| * |z₂|
                let mag_prod_val: f32 = mag_product.into();
                let mag_expected_val: f32 = mag_product_expected.into();

                assert_relative_eq!(mag_prod_val, mag_expected_val, epsilon = 1e-4);
            }
        }

        #[test]
        fn test_complex_additive_identity(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let z = CircleF5E3::from((a_real, a_imag));
            let zero = CircleF5E3::ZERO;
            let result = z + zero;

            if z.is_normal() && result.is_normal() {
                let orig_real: f32 = z.r().into();
                let orig_imag: f32 = z.i().into();
                let result_real: f32 = result.r().into();
                let result_imag: f32 = result.i().into();

                assert_relative_eq!(orig_real, result_real, epsilon = 1e-6);
                assert_relative_eq!(orig_imag, result_imag, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_complex_multiplicative_identity(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let z = CircleF5E3::from((a_real, a_imag));
            let one = CircleF5E3::ONE;
            let result = z * one;

            if z.is_normal() && result.is_normal() {
                let orig_real: f32 = z.r().into();
                let orig_imag: f32 = z.i().into();
                let result_real: f32 = result.r().into();
                let result_imag: f32 = result.i().into();

                assert_relative_eq!(orig_real, result_real, epsilon = 1e-6);
                assert_relative_eq!(orig_imag, result_imag, epsilon = 1e-6);
            }
        }

        #[test]
        fn test_complex_unit_circle_property(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let z = CircleF5E3::from((a_real, a_imag));

            // Skip zero
            if a_real.abs() > 1e-6 || a_imag.abs() > 1e-6 {
                let unit = z.sign();
                let magnitude = unit.magnitude();

                if unit.is_normal() && magnitude.is_normal() {
                    let mag_val: f32 = magnitude.into();
                    // Unit vector should have magnitude 1
                    assert_relative_eq!(mag_val, 1.0, epsilon = 1e-4);

                    // z = |z| * sign(z)
                    let z_magnitude = z.magnitude();
                    if z_magnitude.is_normal() {
                        let reconstructed = z_magnitude * unit;
                        if reconstructed.is_normal() {
                            let orig_real: f32 = z.r().into();
                            let orig_imag: f32 = z.i().into();
                            let recon_real: f32 = reconstructed.r().into();
                            let recon_imag: f32 = reconstructed.i().into();

                            assert_relative_eq!(orig_real, recon_real, epsilon = 1e-3);
                            assert_relative_eq!(orig_imag, recon_imag, epsilon = 1e-3);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod undefined_propagation_properties {
    use super::*;

    #[test]
    fn test_undefined_addition_propagation() {
        // In Spirix, 1/0 = infinity (not undefined). Only 0/0 = undefined.
        let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        let normal = ScalarF5E3::from(42);

        assert!(undefined.is_undefined());

        // Undefined + normal = undefined
        let result1 = undefined + normal;
        assert!(result1.is_undefined());

        // normal + undefined = undefined
        let result2 = normal + undefined;
        assert!(result2.is_undefined());

        // undefined + undefined = undefined
        let result3 = undefined + undefined;
        assert!(result3.is_undefined());
    }

    #[test]
    fn test_undefined_multiplication_propagation() {
        // In Spirix, 1/0 = infinity (not undefined). Only 0/0 = undefined.
        let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        let normal = ScalarF5E3::from(3);
        let zero = ScalarF5E3::ZERO;

        assert!(undefined.is_undefined());

        // undefined * normal = undefined
        let result1 = undefined * normal;
        assert!(result1.is_undefined());

        // undefined * zero = undefined (not zero!)
        let result2 = undefined * zero;
        assert!(result2.is_undefined());

        // Test that first cause is preserved through chain of operations
        let chained = undefined.square().exp().ln().sin();
        assert!(chained.is_undefined());
    }

    #[test]
    fn test_complex_undefined_propagation() {
        // In Spirix, 1/0 = infinity (not undefined). Only 0/0 = undefined.
        let undefined_scalar = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        let normal_circle = CircleF5E3::from((3.0, 4.0));
        let undefined_circle = CircleF5E3::ZERO / CircleF5E3::ZERO;

        assert!(undefined_scalar.is_undefined());
        assert!(undefined_circle.is_undefined());

        // Complex undefined propagation
        let result1 = undefined_circle + normal_circle;
        assert!(result1.is_undefined());

        let result2 = undefined_circle.magnitude();
        assert!(result2.is_undefined());

        let result3 = undefined_circle.conjugate();
        assert!(result3.is_undefined());
    }
}

#[cfg(test)]
mod escaped_value_properties {
    use super::*;

    #[test]
    fn test_exploded_value_properties() {
        // Create exploded values
        let large_pos: ScalarF5E3 = ScalarF5E3::MAX * 2.0;
        let large_neg: ScalarF5E3 = ScalarF5E3::MAX * -2.0;

        // These should be exploded if the library supports escaped values
        if large_pos.exploded() && large_neg.exploded() {
            // Sign should be preserved
            assert!(large_pos.is_positive());
            assert!(large_neg.is_negative());

            // Absolute operations should preserve exploded state
            let squared_pos = large_pos.square();
            let squared_neg = large_neg.square();

            if squared_pos.exploded() && squared_neg.exploded() {
                // Both squares should be positive
                assert!(squared_pos.is_positive());
                assert!(squared_neg.is_positive());
            }

            // Negation should flip sign
            let neg_large_pos = -large_pos;
            let neg_large_neg = -large_neg;

            if neg_large_pos.exploded() && neg_large_neg.exploded() {
                assert!(neg_large_pos.is_negative());
                assert!(neg_large_neg.is_positive());
            }
        }
    }

    #[test]
    fn test_vanished_value_properties() {
        // Create vanished values
        let tiny_pos: ScalarF5E3 = ScalarF5E3::MIN_POS / 1000.0;
        let tiny_neg: ScalarF5E3 = ScalarF5E3::MIN_POS / -1000.0;

        // These should be vanished if the library supports escaped values
        if tiny_pos.vanished() && tiny_neg.vanished() {
            // Sign should be preserved
            assert!(tiny_pos.is_positive());
            assert!(tiny_neg.is_negative());

            // Should be negligible
            assert!(tiny_pos.is_negligible());
            assert!(tiny_neg.is_negligible());

            // Absolute operations should preserve vanished state
            let squared_pos = tiny_pos.square();
            let squared_neg = tiny_neg.square();

            if squared_pos.vanished() && squared_neg.vanished() {
                // Both squares should be positive
                assert!(squared_pos.is_positive());
                assert!(squared_neg.is_positive());
            }
        }
    }
}

#[cfg(test)]
mod normalization_invariants {
    use super::*;

    proptest! {
        #[test]
        fn test_scalar_normalization_consistency(a in -1000.0f32..1000.0f32) {
            let sa = ScalarF5E3::from(a);

            // Check that the internal state is consistent
            if sa.is_normal() {
                assert!(!sa.is_zero());
                assert!(!sa.exploded());
                assert!(!sa.vanished());
                assert!(!sa.is_undefined());
                assert!(sa.is_finite());
            }

            if sa.is_zero() {
                assert!(!sa.is_normal());
                assert!(!sa.is_positive());
                assert!(!sa.is_negative());
                assert!(sa.is_finite());
                assert!(sa.is_negligible());
            }

            if sa.exploded() {
                assert!(!sa.is_normal());
                assert!(!sa.is_finite());
                assert!(!sa.is_zero());
                assert!(sa.is_positive() || sa.is_negative()); // Must have sign
            }

            if sa.vanished() {
                assert!(!sa.is_normal());
                assert!(!sa.is_finite());
                assert!(!sa.is_zero());
                assert!(sa.is_negligible());
                assert!(sa.is_positive() || sa.is_negative()); // Must have sign
            }

            if sa.is_undefined() {
                assert!(!sa.is_normal());
                assert!(!sa.is_finite());
                assert!(!sa.is_zero());
            }
        }

        #[test]
        fn test_circle_normalization_consistency(
            (a_real, a_imag) in super::circle_properties::circle_strategy()
        ) {
            let ca = CircleF5E3::from((a_real, a_imag));

            // Check that circle state is consistent
            if ca.is_normal() {
                assert!(!ca.is_zero());
                assert!(!ca.exploded());
                assert!(!ca.vanished());
                assert!(!ca.is_undefined());
                assert!(ca.is_finite());

                // Components should be extractable
                let real_part = ca.r();
                let imag_part = ca.i();
                assert!(real_part.is_normal() || real_part.is_zero());
                assert!(imag_part.is_normal() || imag_part.is_zero());
            }

            if ca.is_zero() {
                assert!(!ca.is_normal());
                assert!(ca.is_finite());

                // Both components should be zero
                let real_part = ca.r();
                let imag_part = ca.i();
                assert!(real_part.is_zero());
                assert!(imag_part.is_zero());
            }
        }

        #[test]
        fn test_conversion_consistency(a in -100.0f32..100.0f32) {
            let sa = ScalarF5E3::from(a);

            if sa.is_normal() {
                // Convert back to f32 and compare
                let back_to_f32: f32 = sa.into();
                assert_relative_eq!(a, back_to_f32, epsilon = 1e-4);

                // Create from the converted value should be identical
                let sa2 = ScalarF5E3::from(back_to_f32);
                if sa2.is_normal() {
                    let val1: f32 = sa.into();
                    let val2: f32 = sa2.into();
                    assert_relative_eq!(val1, val2, epsilon = 1e-6);
                }
            }
        }
    }
}
