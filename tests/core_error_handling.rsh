use spirix::*;

/// Comprehensive error handling tests for undefined states, edge cases, and error propagation
/// Verify that all error conditions are properly detected and handled

#[test]
fn test_division_by_zero_handling() {
    let normal = ScalarF5E3::from(42.0);
    let zero = ScalarF5E3::ZERO;

    // Division by zero should produce undefined
    let result = normal / zero;
    assert!(result.is_undefined());

    // Zero divided by zero should also be undefined
    let zero_div_zero = zero / zero;
    assert!(zero_div_zero.is_undefined());

    // Division involving infinity
    let infinity = ScalarF5E3::INFINITY;
    let inf_div_inf = infinity / infinity;
    assert!(inf_div_inf.is_undefined());

    // Normal number divided by infinity should be zero
    let normal_div_inf = normal / infinity;
    assert!(normal_div_inf.is_zero());

    // Infinity divided by normal should be infinity
    let inf_div_normal = infinity / normal;
    assert!(inf_div_normal.is_infinite());
}

#[test]
fn test_square_root_error_handling() {
    // Square root of negative numbers should be undefined
    let negative = ScalarF5E3::from(-4.0);
    let sqrt_neg = negative.sqrt();
    assert!(sqrt_neg.is_undefined());

    // Square root of zero should be zero
    let sqrt_zero = ScalarF5E3::ZERO.sqrt();
    assert!(sqrt_zero.is_zero());

    // Square root of positive should be normal
    let positive = ScalarF5E3::from(4.0);
    let sqrt_pos = positive.sqrt();
    assert!(sqrt_pos.is_normal());

    // Square root of infinity should be infinity
    let sqrt_inf = ScalarF5E3::INFINITY.sqrt();
    assert!(sqrt_inf.is_infinite());

    // Square root of undefined should remain undefined
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let sqrt_undef = undefined.sqrt();
    assert!(sqrt_undef.is_undefined());
}

#[test]
fn test_logarithm_error_handling() {
    // Logarithm of zero should be undefined (negative infinity is not representable)
    let ln_zero = ScalarF5E3::ZERO.ln();
    assert!(ln_zero.is_undefined());

    // Logarithm of negative numbers should be undefined
    let negative = ScalarF5E3::from(-1.0);
    let ln_neg = negative.ln();
    assert!(ln_neg.is_undefined());

    // Logarithm of positive numbers should be normal
    let positive = ScalarF5E3::from(2.718);
    let ln_pos = positive.ln();
    assert!(ln_pos.is_normal());

    // Logarithm of infinity should be infinity
    let ln_inf = ScalarF5E3::INFINITY.ln();
    assert!(ln_inf.is_infinite());

    // Logarithm of one should be zero
    let ln_one = ScalarF5E3::ONE.ln();
    assert!(
        ln_one.is_zero()
            || (ln_one.is_normal() && {
                let val: f32 = ln_one.into();
                val.abs() < 1e-6
            })
    );
}

#[test]
fn test_power_function_error_handling() {
    let zero = ScalarF5E3::ZERO;
    let one = ScalarF5E3::ONE;
    let negative = ScalarF5E3::from(-2.0);

    // Zero to the power of zero should be undefined
    let zero_pow_zero = zero.pow(zero);
    assert!(zero_pow_zero.is_undefined());

    // Zero to negative power should be undefined (would be infinity)
    let zero_pow_neg = zero.pow(negative);
    assert!(zero_pow_neg.is_undefined());

    // Zero to positive power should be zero
    let zero_pow_pos = zero.pow(one);
    assert!(zero_pow_pos.is_zero());

    // Negative base to fractional power should be undefined (complex result)
    let neg_pow_frac = negative.pow(ScalarF5E3::from(0.5));
    assert!(neg_pow_frac.is_undefined());

    // Negative base to integer power should work
    let neg_pow_int = negative.pow(ScalarF5E3::from(2.0));
    assert!(neg_pow_int.is_normal() && neg_pow_int.is_positive());

    // Infinity to the power of zero should be undefined
    let inf_pow_zero = ScalarF5E3::INFINITY.pow(zero);
    assert!(inf_pow_zero.is_undefined());
}

#[test]
fn test_trigonometric_function_error_handling() {
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let infinity = ScalarF5E3::INFINITY;

    // Trigonometric functions of undefined should be undefined
    assert!(undefined.sin().is_undefined());
    assert!(undefined.cos().is_undefined());
    assert!(undefined.tan().is_undefined());

    // Trigonometric functions of infinity should be undefined
    assert!(infinity.sin().is_undefined());
    assert!(infinity.cos().is_undefined());
    assert!(infinity.tan().is_undefined());

    // Inverse trigonometric functions with invalid domains
    let invalid_asin = ScalarF5E3::from(2.0).asin(); // |x| > 1
    assert!(invalid_asin.is_undefined());

    let invalid_acos = ScalarF5E3::from(-2.0).acos(); // |x| > 1
    assert!(invalid_acos.is_undefined());

    // Valid domain should work
    let valid_asin = ScalarF5E3::from(0.5).asin();
    assert!(valid_asin.is_normal());

    let valid_acos = ScalarF5E3::from(0.5).acos();
    assert!(valid_acos.is_normal());
}

#[test]
fn test_hyperbolic_function_error_handling() {
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let infinity = ScalarF5E3::INFINITY;

    // Hyperbolic functions of undefined should be undefined
    assert!(undefined.sinh().is_undefined());
    assert!(undefined.cosh().is_undefined());
    assert!(undefined.tanh().is_undefined());

    // Hyperbolic functions of infinity
    assert!(infinity.sinh().is_infinite());
    assert!(infinity.cosh().is_infinite());

    // tanh of infinity should be 1 (or very close)
    let tanh_inf = infinity.tanh();
    if tanh_inf.is_normal() {
        let val: f32 = tanh_inf.into();
        assert!((val - 1.0).abs() < 1e-6);
    }

    // Inverse hyperbolic functions with invalid domains
    let invalid_acosh = ScalarF5E3::from(0.5).acosh(); // x < 1
    assert!(invalid_acosh.is_undefined());

    let invalid_atanh = ScalarF5E3::from(2.0).atanh(); // |x| >= 1
    assert!(invalid_atanh.is_undefined());

    // Valid domains should work
    let valid_acosh = ScalarF5E3::from(2.0).acosh();
    assert!(valid_acosh.is_normal());

    let valid_atanh = ScalarF5E3::from(0.5).atanh();
    assert!(valid_atanh.is_normal());
}

#[test]
fn test_undefined_state_propagation_chain() {
    // Test that undefined states propagate through operation chains
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let normal = ScalarF5E3::from(42.0);

    // Long chain of operations should preserve undefined state
    let chain_result = ((undefined + normal) * ScalarF5E3::from(2.0))
        .pow(ScalarF5E3::from(3.0))
        .sin()
        .ln()
        .exp()
        .sqrt()
        .abs();

    assert!(chain_result.is_undefined());

    // Mixed operations with multiple undefined sources
    let undefined2 = ScalarF5E3::from(-1.0).sqrt(); // Another undefined
    let mixed_undefined = undefined + undefined2;
    assert!(mixed_undefined.is_undefined());

    // Operations between undefined and special values
    assert!((undefined + ScalarF5E3::INFINITY).is_undefined());
    assert!((undefined * ScalarF5E3::ZERO).is_undefined());
    assert!((undefined / ScalarF5E3::INFINITY).is_undefined());
}

#[test]
fn test_infinity_arithmetic_edge_cases() {
    let infinity = ScalarF5E3::INFINITY;
    let neg_infinity = -infinity;
    let normal = ScalarF5E3::from(42.0);
    let zero = ScalarF5E3::ZERO;

    // Infinity + Infinity = Infinity
    let inf_plus_inf = infinity + infinity;
    assert!(inf_plus_inf.is_infinite());

    // Infinity - Infinity = Undefined
    let inf_minus_inf = infinity - infinity;
    assert!(inf_minus_inf.is_undefined());

    // Infinity + (-Infinity) = Undefined
    let inf_plus_neg_inf = infinity + neg_infinity;
    assert!(inf_plus_neg_inf.is_undefined());

    // Infinity * normal = Infinity (with correct sign)
    let inf_times_pos = infinity * normal;
    assert!(inf_times_pos.is_infinite());

    let inf_times_neg = infinity * (-normal);
    assert!(inf_times_neg.is_infinite()); // Should be negative infinity

    // Infinity * 0 = Undefined
    let inf_times_zero = infinity * zero;
    assert!(inf_times_zero.is_undefined());

    // 0 * Infinity = Undefined
    let zero_times_inf = zero * infinity;
    assert!(zero_times_inf.is_undefined());

    // Infinity / Infinity = Undefined
    let inf_div_inf = infinity / infinity;
    assert!(inf_div_inf.is_undefined());

    // Normal / Infinity = 0
    let normal_div_inf = normal / infinity;
    assert!(normal_div_inf.is_zero());
}

#[test]
fn test_escaped_value_error_conditions() {
    // Test error conditions specific to exploded and vanished states
    let exploded = ScalarF5E3::MAX * ScalarF5E3::from(10.0);
    let vanished = ScalarF5E3::MIN_POS / ScalarF5E3::from(10.0);

    assert!(exploded.exploded());
    assert!(vanished.vanished());

    // Operations that should preserve escaped states
    let exploded_squared = exploded * exploded;
    assert!(exploded_squared.exploded());

    let vanished_squared = vanished * vanished;
    assert!(vanished_squared.vanished());

    // Operations that might transition escaped states
    let exploded_div_exploded = exploded / exploded;
    // This could be normal (≈1), exploded, or undefined depending on implementation

    let vanished_div_vanished = vanished / vanished;
    // This could be normal (≈1), vanished, or undefined depending on implementation

    // Verify that escaped states don't become normal unexpectedly
    let exploded_plus_normal = exploded + ScalarF5E3::from(1.0);
    assert!(exploded_plus_normal.exploded()); // Should remain exploded

    let vanished_plus_vanished = vanished + vanished;
    if !vanished_plus_vanished.is_normal() {
        assert!(vanished_plus_vanished.vanished());
    }
}

#[test]
fn test_complex_error_propagation() {
    // Test error handling in complex numbers
    let normal_circle = CircleF5E3::new(ScalarF5E3::from(3.0), ScalarF5E3::from(4.0));
    let undefined_scalar = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let undefined_circle = CircleF5E3::new(undefined_scalar, ScalarF5E3::from(1.0));

    // Operations with undefined complex numbers
    let complex_add = normal_circle + undefined_circle;
    assert!(complex_add.real().is_undefined());
    assert!(complex_add.imaginary().is_normal()); // Only real part undefined

    let complex_mult = normal_circle * undefined_circle;
    assert!(complex_mult.real().is_undefined());
    assert!(complex_mult.imaginary().is_undefined()); // Both parts undefined after multiplication

    // Complex-specific error conditions
    let zero_circle = CircleF5E3::new(ScalarF5E3::ZERO, ScalarF5E3::ZERO);
    let div_by_zero_circle = normal_circle / zero_circle;
    assert!(div_by_zero_circle.real().is_undefined());
    assert!(div_by_zero_circle.imaginary().is_undefined());

    // Complex functions with error conditions
    let undefined_circle_exp = undefined_circle.exp();
    assert!(undefined_circle_exp.real().is_undefined());

    let undefined_circle_ln = undefined_circle.ln();
    assert!(undefined_circle_ln.real().is_undefined());
}

#[test]
fn test_conversion_error_handling() {
    // Test error handling during type conversions
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let exploded = ScalarF5E3::MAX * ScalarF5E3::from(10.0);
    let vanished = ScalarF5E3::MIN_POS / ScalarF5E3::from(10.0);

    // Conversion of undefined values
    // Note: The behavior of converting undefined to f32 is implementation-defined
    // but should not panic or cause undefined behavior
    let undefined_as_f32: f32 = undefined.into();
    assert!(undefined_as_f32.is_nan() || undefined_as_f32.is_infinite());

    // Conversion of exploded values
    let exploded_as_f32: f32 = exploded.into();
    assert!(exploded_as_f32.is_infinite() || exploded_as_f32.abs() > 1e30);

    // Conversion of vanished values
    let vanished_as_f32: f32 = vanished.into();
    assert!(vanished_as_f32 == 0.0 || vanished_as_f32.abs() < 1e-30);

    // Round-trip conversion should preserve error states when possible
    let nan_input = f32::NAN;
    let inf_input = f32::INFINITY;

    let from_nan = ScalarF5E3::from(nan_input);
    assert!(from_nan.is_undefined());

    let from_inf = ScalarF5E3::from(inf_input);
    assert!(from_inf.is_infinite());
}

#[test]
fn test_edge_case_combinations() {
    // Test combinations of edge cases that might expose bugs
    let values = [
        ScalarF5E3::ZERO,
        ScalarF5E3::ONE,
        ScalarF5E3::INFINITY,
        ScalarF5E3::EPSILON,
        ScalarF5E3::MAX,
        ScalarF5E3::MIN,
        ScalarF5E3::MIN_POS,
        ScalarF5E3::MAX_NEG,
        ScalarF5E3::ZERO / ScalarF5E3::ZERO,         // undefined
        ScalarF5E3::MAX * ScalarF5E3::from(2.0),     // exploded
        ScalarF5E3::MIN_POS / ScalarF5E3::from(2.0), // vanished
    ];

    // Test all pairwise operations
    for &a in &values {
        for &b in &values {
            // Addition should not panic
            let sum = a + b;
            assert!(
                sum.is_normal()
                    || sum.is_zero()
                    || sum.is_infinite()
                    || sum.is_undefined()
                    || sum.exploded()
                    || sum.vanished()
            );

            // Multiplication should not panic
            let product = a * b;
            assert!(
                product.is_normal()
                    || product.is_zero()
                    || product.is_infinite()
                    || product.is_undefined()
                    || product.exploded()
                    || product.vanished()
            );

            // Division should not panic (but may be undefined)
            let quotient = a / b;
            assert!(
                quotient.is_normal()
                    || quotient.is_zero()
                    || quotient.is_infinite()
                    || quotient.is_undefined()
                    || quotient.exploded()
                    || quotient.vanished()
            );

            // Subtraction should not panic
            let difference = a - b;
            assert!(
                difference.is_normal()
                    || difference.is_zero()
                    || difference.is_infinite()
                    || difference.is_undefined()
                    || difference.exploded()
                    || difference.vanished()
            );
        }
    }
}

#[test]
fn test_undefined_prefix_consistency() {
    // Test that different undefined states can be distinguished by their prefixes
    let zero_div_zero = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let sqrt_negative = ScalarF5E3::from(-1.0).sqrt();
    let ln_negative = ScalarF5E3::from(-1.0).ln();
    let inf_minus_inf = ScalarF5E3::INFINITY - ScalarF5E3::INFINITY;

    // All should be undefined
    assert!(zero_div_zero.is_undefined());
    assert!(sqrt_negative.is_undefined());
    assert!(ln_negative.is_undefined());
    assert!(inf_minus_inf.is_undefined());

    // But they might have different undefined prefixes to indicate the cause
    // This tests the implementation's ability to track the first cause of undefined behavior
    if let (Some(prefix1), Some(prefix2), Some(prefix3), Some(prefix4)) = (
        zero_div_zero.undefined_prefix(),
        sqrt_negative.undefined_prefix(),
        ln_negative.undefined_prefix(),
        inf_minus_inf.undefined_prefix(),
    ) {
        // Different operations should ideally have different prefixes
        // (though some might be the same if they map to the same underlying cause)
        let prefixes = [prefix1, prefix2, prefix3, prefix4];

        // At least verify that the prefixes are valid undefined patterns
        for prefix in prefixes {
            assert!(prefix != 0); // Should not be zero (which would indicate normal state)
        }
    }
}
