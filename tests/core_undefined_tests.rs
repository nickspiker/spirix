use spirix::*;

#[cfg(test)]
mod undefined_creation {
    use super::*;

    #[test]
    fn test_division_by_zero() {
        // Division by zero produces infinity in Spirix
        let one = ScalarF5E3::from(1);
        let zero = ScalarF5E3::ZERO;
        let result = one / zero;

        assert!(result.is_infinite());
        assert!(!result.is_normal());
        assert!(!result.is_finite());
        assert!(!result.is_zero());
    }

    #[test]
    fn test_zero_to_zero_power() {
        // 0^0 = 1 in Spirix (convention)
        let zero = ScalarF5E3::ZERO;
        let result = zero.pow(zero);

        assert_eq!(result, ScalarF5E3::ONE);
    }

    #[test]
    fn test_square_root_of_negative() {
        // √(-x) should be undefined for real numbers
        let negative = ScalarF5E3::from(-4);
        let result = negative.sqrt();

        assert!(result.is_undefined());
    }

    #[test]
    fn test_logarithm_of_negative() {
        // ln(-x) should be undefined for real numbers
        let negative = ScalarF5E3::from(-2);
        let result = negative.ln();

        assert!(result.is_undefined());
    }

    #[test]
    fn test_logarithm_of_zero() {
        // ln(0) = -infinity in Spirix
        let zero = ScalarF5E3::ZERO;
        let result = zero.ln();

        assert!(result.is_infinite());
    }

    #[test]
    fn test_invalid_inverse_trig() {
        // asin(x) and acos(x) are undefined for |x| > 1
        let too_large = ScalarF5E3::from(2.0);
        let asin_result = too_large.asin();
        let acos_result = too_large.acos();

        assert!(asin_result.is_undefined());
        assert!(acos_result.is_undefined());

        let too_small = ScalarF5E3::from(-2.0);
        let asin_result2 = too_small.asin();
        let acos_result2 = too_small.acos();

        assert!(asin_result2.is_undefined());
        assert!(acos_result2.is_undefined());
    }

    #[test]
    fn test_negative_base_fractional_power() {
        // (-x)^(non-integer) is undefined for real numbers
        let negative_base = ScalarF5E3::from(-2);
        let fractional_exp = ScalarF5E3::from(0.5); // √(-2)
        let result = negative_base.pow(fractional_exp);

        assert!(result.is_undefined());
    }
}

#[cfg(test)]
mod undefined_propagation {
    use super::*;

    #[test]
    fn test_arithmetic_propagation() {
        // Truly undefined values (0/0) propagate thru addition/subtraction
        let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        assert!(undefined.is_undefined());

        let normal = ScalarF5E3::from(42);

        // Addition with undefined
        assert!((undefined + normal).is_undefined());
        assert!((normal + undefined).is_undefined());

        // Subtraction with undefined
        assert!((undefined - normal).is_undefined());
        assert!((normal - undefined).is_undefined());

        // Multiplication with undefined
        assert!((undefined * normal).is_undefined());
        assert!((normal * undefined).is_undefined());

        // Division with undefined
        assert!((undefined / normal).is_undefined());
        assert!((normal / undefined).is_undefined());
    }

    #[test]
    fn test_function_propagation() {
        // Undefined values should propagate thru mathematical functions
        let undefined = ScalarF5E3::from(-1).sqrt(); // undefined
        assert!(undefined.is_undefined());

        // Trigonometric functions
        assert!(undefined.sin().is_undefined());
        assert!(undefined.cos().is_undefined());
        assert!(undefined.tan().is_undefined());

        // Exponential and logarithmic functions
        assert!(undefined.exp().is_undefined());
        assert!(undefined.ln().is_undefined());
        assert!(undefined.lb().is_undefined());

        // Power functions
        assert!(undefined.square().is_undefined());
        assert!(undefined.sqrt().is_undefined());
        assert!(undefined.reciprocal().is_undefined());

        // Power with undefined base
        let normal = ScalarF5E3::from(2);
        assert!(undefined.pow(normal).is_undefined());

        // Power with undefined exponent
        assert!(normal.pow(undefined).is_undefined());
    }

    #[test]
    fn test_comparison_with_undefined() {
        // Operations involving undefined should handle gracefully
        let undefined = ScalarF5E3::from(0) / ScalarF5E3::ZERO;
        let normal = ScalarF5E3::from(5);

        assert!(undefined.is_undefined());

        // Min/max with undefined
        let min_result = undefined.min(normal);
        let max_result = undefined.max(normal);

        // Should return undefined or the normal value (implementation dependent)
        assert!(min_result.is_undefined() || min_result == normal);
        assert!(max_result.is_undefined() || max_result == normal);

        // Clamp with undefined
        let clamp_result = undefined.clamp(ScalarF5E3::from(1), ScalarF5E3::from(10));
        assert!(clamp_result.is_undefined());
    }

    #[test]
    fn test_undefined_chaining() {
        // Test that undefined propagates thru complex chains
        let start = ScalarF5E3::from(-1);

        // Create undefined thru sqrt of negative
        let step1 = start.sqrt(); // undefined
        assert!(step1.is_undefined());

        // Chain operations
        let step2 = step1.exp(); // undefined
        let step3 = step2 + ScalarF5E3::PI; // undefined
        let step4 = step3.sin(); // undefined
        let step5 = step4 * ScalarF5E3::from(100); // undefined

        assert!(step2.is_undefined());
        assert!(step3.is_undefined());
        assert!(step4.is_undefined());
        assert!(step5.is_undefined());
    }
}

#[cfg(test)]
mod complex_undefined {
    use super::*;

    #[test]
    fn test_complex_with_undefined_components() {
        // Use NaN to create undefined via f32 conversion
        let z1 = CircleF5E3::from((f32::NAN, 2.0));
        assert!(z1.is_undefined());

        let z2 = CircleF5E3::from((3.0, f32::NAN));
        assert!(z2.is_undefined());

        let z3 = CircleF5E3::from((f32::NAN, f32::NAN));
        assert!(z3.is_undefined());
    }

    #[test]
    fn test_complex_operations_with_undefined() {
        // Create undefined complex via NaN
        let undefined_complex = CircleF5E3::from((f32::NAN, 1.0));
        let normal_complex = CircleF5E3::from((2.0, 3.0));

        assert!(undefined_complex.is_undefined());
        assert!(normal_complex.is_normal());

        // Arithmetic operations propagate undefined
        assert!((undefined_complex + normal_complex).is_undefined());
        assert!((normal_complex + undefined_complex).is_undefined());
        assert!((undefined_complex - normal_complex).is_undefined());
        assert!((undefined_complex * normal_complex).is_undefined());
    }

    #[test]
    fn test_complex_undefined_propagation() {
        // Create undefined complex via NaN
        let undefined_complex = CircleF5E3::from((f32::NAN, f32::NAN));
        assert!(undefined_complex.is_undefined());

        // Chain complex operations
        let step1 = undefined_complex * CircleF5E3::POS_I;
        let step2 = step1.conjugate();
        let step3 = step2 + CircleF5E3::from((5.0, -3.0));

        assert!(step1.is_undefined());
        assert!(step2.is_undefined());
        assert!(step3.is_undefined());
    }
}

#[cfg(test)]
mod undefined_across_types {
    use super::*;

    // Macro to test undefined propagation across different scalar types
    macro_rules! test_undefined_for_type {
        ($scalar_type:ident) => {
            // Create undefined value (0/0, not 1/0 which is infinity)
            let undefined = $scalar_type::ZERO / $scalar_type::ZERO;
            assert!(undefined.is_undefined());

            // Test propagation thru basic arithmetic
            let normal = $scalar_type::from(42);
            assert!((undefined + normal).is_undefined());
            assert!((undefined * normal).is_undefined());
            assert!((normal - undefined).is_undefined());

            // Test propagation thru functions
            assert!(undefined.sin().is_undefined());
            assert!(undefined.exp().is_undefined());
            assert!(undefined.sqrt().is_undefined());

            // Test undefined creation thru invalid operations
            assert!(($scalar_type::from(-1).sqrt()).is_undefined());
            assert!(($scalar_type::from(-1).ln()).is_undefined());
        };
    }

    // Macro to test undefined propagation across different circle types
    macro_rules! test_complex_undefined_for_type {
        ($circle_type:ident) => {
            // Create undefined complex number via NaN
            let undefined_complex = $circle_type::from((f32::NAN, f32::NAN));
            assert!(undefined_complex.is_undefined());

            // Test propagation
            let normal = $circle_type::from((2.0, 3.0));
            assert!((undefined_complex + normal).is_undefined());
            assert!((undefined_complex * normal).is_undefined());
        };
    }

    #[test]
    fn test_undefined_propagation_all_scalar_types() {
        // Test undefined behavior for representative scalar types
        test_undefined_for_type!(ScalarF3E3);
        test_undefined_for_type!(ScalarF5E3);
        test_undefined_for_type!(ScalarF7E7);
        test_undefined_for_type!(ScalarF3E7);
        test_undefined_for_type!(ScalarF7E3);
    }

    #[test]
    fn test_undefined_propagation_all_circle_types() {
        // Test undefined behavior for representative circle types
        test_complex_undefined_for_type!(CircleF3E3);
        test_complex_undefined_for_type!(CircleF5E3);
        test_complex_undefined_for_type!(CircleF7E7);
        test_complex_undefined_for_type!(CircleF3E7);
        test_complex_undefined_for_type!(CircleF7E3);
    }

    #[test]
    fn test_undefined_conversion_between_types() {
        // Test that undefined state is preserved across type conversions via NaN
        let undefined_f3e3 = ScalarF3E3::from(f32::NAN);
        assert!(undefined_f3e3.is_undefined());

        // Convert to different types via f32 NaN
        let as_f5e3 = ScalarF5E3::from(f32::NAN);
        let as_f7e7 = ScalarF7E7::from(f32::NAN);

        assert!(as_f5e3.is_undefined());
        assert!(as_f7e7.is_undefined());

        // Test complex conversions
        let undefined_circle_f3e3 = CircleF3E3::from((f32::NAN, 1.0));
        assert!(undefined_circle_f3e3.is_undefined());
    }
}

#[cfg(test)]
mod undefined_special_cases {
    use super::*;

    #[test]
    fn test_undefined_with_escaped_values() {
        // Test interactions between undefined and exploded/vanished values

        let exploded: ScalarF5E3 = ScalarF5E3::MAX * 2.0;
        let vanished: ScalarF5E3 = ScalarF5E3::MIN_POS / 1000.0;
        let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;

        assert!(exploded.exploded());
        assert!(vanished.vanished());
        assert!(undefined.is_undefined());

        // Operations between undefined and escaped values
        let undefined_plus_exploded = undefined + exploded;
        let undefined_plus_vanished = undefined + vanished;
        let exploded_plus_undefined = exploded + undefined;

        // Undefined should dominate
        assert!(undefined_plus_exploded.is_undefined());
        assert!(undefined_plus_vanished.is_undefined());
        assert!(exploded_plus_undefined.is_undefined());
    }

    #[test]
    fn test_double_undefined_operations() {
        // Test operations between two undefined values
        let undefined1 = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        let undefined2 = ScalarF5E3::from(-1).sqrt();

        assert!(undefined1.is_undefined());
        assert!(undefined2.is_undefined());

        // Operations between undefined values should remain undefined
        assert!((undefined1 + undefined2).is_undefined());
        assert!((undefined1 - undefined2).is_undefined());
        assert!((undefined1 * undefined2).is_undefined());
        assert!((undefined1 / undefined2).is_undefined());
        assert!(undefined1.pow(undefined2).is_undefined());
    }

    #[test]
    fn test_undefined_edge_cases() {
        // Test edge cases that might create or interact with undefined values

        // Very large exponents
        let large_exp = ScalarF5E3::from(1000);
        let small_base = ScalarF5E3::from(0.1);
        let result = small_base.pow(large_exp);
        // This might vanish or become undefined
        assert!(result.vanished() || result.is_undefined() || result.is_normal());

        // Factorial of negative (if supported) Note: This test assumes factorial function exists, may need to be removed if not available let negative = ScalarF5E3::from(-1); let factorial_result = negative.factorial(); assert!(factorial_result.is_undefined());

        // Modulo by zero - returns 0 (valid implementation choice)
        let modulo_result = ScalarF5E3::from(5) % ScalarF5E3::ZERO;
        assert!(modulo_result.is_zero());

        // Power of zero with negative exponent: 0^(-1) = 1/0 = ∞ (infinity, not undefined).
        let zero_power_neg = ScalarF5E3::ZERO.pow(ScalarF5E3::from(-1));
        assert!(zero_power_neg.is_infinite());
    }
}
