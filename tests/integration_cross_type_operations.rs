use approx::assert_relative_eq;
use paste::paste;
use spirix::*;

/// Test operations between different types: Scalar ↔ Circle, different precisions, and Rust primitives
/// Verify conversion accuracy, mixed-type arithmetic, and precision preservation

// Macro to test cross-type operations for all combinations
macro_rules! test_cross_type_for_types {
    ($scalar_type:ident, $circle_type:ident) => {
        paste! {
            #[test]
            fn [<test_scalar_circle_conversions_ $scalar_type:lower>]() {
                // Test Scalar -> Circle conversion
                let scalar = $scalar_type::from(42.5);
                let circle: $circle_type = scalar.into();

                assert!(circle.is_real());
                assert_eq!(circle.r(), scalar);
                assert_eq!(circle.i(), $scalar_type::ZERO);

                // Test Circle -> Scalar conversion (real part)
                let complex_circle = $circle_type::from((3.0, 4.0));
                let real_part = complex_circle.r();
                let expected_real = $scalar_type::from(3.0);

                let real_val: f32 = real_part.into();
                let expected_val: f32 = expected_real.into();
                assert_relative_eq!(real_val, expected_val, epsilon = 1e-5);

                // Test imaginary part extraction
                let imag_part = complex_circle.i();
                let expected_imag = $scalar_type::from(4.0);

                let imag_val: f32 = imag_part.into();
                let expected_imag_val: f32 = expected_imag.into();
                assert_relative_eq!(imag_val, expected_imag_val, epsilon = 1e-5);
            }

            #[test]
            fn [<test_scalar_circle_arithmetic_ $scalar_type:lower>]() {
                let scalar = $scalar_type::from(2.0);
                let circle = $circle_type::from((3.0, 4.0));

                // Scalar + Circle
                let sum = scalar + circle;
                let sum_real: f32 = sum.r().into();
                let sum_imag: f32 = sum.i().into();
                assert_relative_eq!(sum_real, 5.0, epsilon = 1e-5);
                assert_relative_eq!(sum_imag, 4.0, epsilon = 1e-5);

                // Circle + Scalar
                let sum2 = circle + scalar;
                let sum2_real: f32 = sum2.r().into();
                let sum2_imag: f32 = sum2.i().into();
                assert_relative_eq!(sum2_real, 5.0, epsilon = 1e-5);
                assert_relative_eq!(sum2_imag, 4.0, epsilon = 1e-5);

                // Scalar * Circle
                let product = scalar * circle;
                let product_real: f32 = product.r().into();
                let product_imag: f32 = product.i().into();
                assert_relative_eq!(product_real, 6.0, epsilon = 1e-5);
                assert_relative_eq!(product_imag, 8.0, epsilon = 1e-5);

                // Circle / Scalar
                let quotient = circle / scalar;
                let quotient_real: f32 = quotient.r().into();
                let quotient_imag: f32 = quotient.i().into();
                assert_relative_eq!(quotient_real, 1.5, epsilon = 1e-5);
                assert_relative_eq!(quotient_imag, 2.0, epsilon = 1e-5);
            }

            #[test]
            fn [<test_rust_primitive_interop_ $scalar_type:lower>]() {
                let scalar = $scalar_type::from(10.0);

                // Test with i32
                let int_val = 5i32;
                let scalar_plus_int = scalar + int_val;
                let result_val: f32 = scalar_plus_int.into();
                assert_relative_eq!(result_val, 15.0, epsilon = 1e-5);

                // Test with f32
                let float_val = 2.5f32;
                let scalar_times_float = scalar * float_val;
                let mult_result: f32 = scalar_times_float.into();
                assert_relative_eq!(mult_result, 25.0, epsilon = 1e-5);

                // Test with f64
                let double_val = 0.5f64;
                let scalar_div_double = scalar / double_val;
                let div_result: f32 = scalar_div_double.into();
                assert_relative_eq!(div_result, 20.0, epsilon = 1e-5);

                // Test Circle with primitives
                let circle = $circle_type::from((3.0, 4.0));

                // Circle + real number (should affect real part only)
                let circle_plus_real = circle + 2.0f32;
                let circle_real: f32 = circle_plus_real.r().into();
                let circle_imag: f32 = circle_plus_real.i().into();
                assert_relative_eq!(circle_real, 5.0, epsilon = 1e-5);
                assert_relative_eq!(circle_imag, 4.0, epsilon = 1e-5);

                // Circle * real number
                let circle_times_real = circle * 2.0f32;
                let mult_real: f32 = circle_times_real.r().into();
                let mult_imag: f32 = circle_times_real.i().into();
                assert_relative_eq!(mult_real, 6.0, epsilon = 1e-5);
                assert_relative_eq!(mult_imag, 8.0, epsilon = 1e-5);
            }

            #[test]
            fn [<test_escaped_states_cross_type_ $scalar_type:lower>]() {
                // Test operations involving escaped states across types
                let exploded_scalar = $scalar_type::MAX * $scalar_type::from(2.0);
                let normal_circle = $circle_type::from((1.0, 1.0));

                // Exploded scalar with normal circle
                let result = exploded_scalar + normal_circle;
                assert!(result.r().exploded());
                assert!(result.i().is_normal());

                // Vanished scalar with normal circle
                let vanished_scalar = $scalar_type::MIN_POS / $scalar_type::from(2.0);
                let result2 = vanished_scalar * normal_circle;

                if result2.r().vanished() && result2.i().vanished() {
                    assert!(result2.r().is_positive());
                    assert!(result2.i().is_positive());
                }
            }

            #[test]
            fn [<test_magnitude_operations_ $scalar_type:lower>]() {
                // Test magnitude calculations
                let circle_345 = $circle_type::from((3.0, 4.0)); // Should have magnitude 5
                let magnitude = circle_345.magnitude();

                if magnitude.is_normal() {
                    let mag_val: f32 = magnitude.into();
                    assert_relative_eq!(mag_val, 5.0, epsilon = 1e-5);
                }

                // Test magnitude squared
                let mag_squared = circle_345.magnitude_squared();
                if mag_squared.is_normal() {
                    let mag_sq_val: f32 = mag_squared.into();
                    assert_relative_eq!(mag_sq_val, 25.0, epsilon = 1e-5);
                }

                // Test unit vector
                let unit = circle_345.unit();
                if unit.is_normal() {
                    let unit_mag = unit.magnitude();
                    if unit_mag.is_normal() {
                        let unit_mag_val: f32 = unit_mag.into();
                        assert_relative_eq!(unit_mag_val, 1.0, epsilon = 1e-5);
                    }
                }
            }

            #[test]
            fn [<test_complex_conjugate_ $scalar_type:lower>]() {
                let z = $circle_type::from((3.0, 4.0));
                let conj = z.conjugate();

                let conj_real: f32 = conj.r().into();
                let conj_imag: f32 = conj.i().into();

                assert_relative_eq!(conj_real, 3.0, epsilon = 1e-5);
                assert_relative_eq!(conj_imag, -4.0, epsilon = 1e-5);

                // Test conjugate properties: z * conj(z) = |z|²
                let product = z * conj;
                let mag_squared = z.magnitude_squared();

                if product.is_normal() && mag_squared.is_normal() {
                    let product_real: f32 = product.r().into();
                    let product_imag: f32 = product.i().into();
                    let mag_sq_val: f32 = mag_squared.into();

                    assert_relative_eq!(product_real, mag_sq_val, epsilon = 1e-4);
                    assert_relative_eq!(product_imag, 0.0, epsilon = 1e-5);
                }
            }
        }
    };
}

// Test all F×E combinations
test_cross_type_for_types!(ScalarF3E3, CircleF3E3);
test_cross_type_for_types!(ScalarF3E4, CircleF3E4);
test_cross_type_for_types!(ScalarF3E5, CircleF3E5);
test_cross_type_for_types!(ScalarF3E6, CircleF3E6);
test_cross_type_for_types!(ScalarF3E7, CircleF3E7);
test_cross_type_for_types!(ScalarF4E3, CircleF4E3);
test_cross_type_for_types!(ScalarF4E4, CircleF4E4);
test_cross_type_for_types!(ScalarF4E5, CircleF4E5);
test_cross_type_for_types!(ScalarF4E6, CircleF4E6);
test_cross_type_for_types!(ScalarF4E7, CircleF4E7);
test_cross_type_for_types!(ScalarF5E3, CircleF5E3);
test_cross_type_for_types!(ScalarF5E4, CircleF5E4);
test_cross_type_for_types!(ScalarF5E5, CircleF5E5);
test_cross_type_for_types!(ScalarF5E6, CircleF5E6);
test_cross_type_for_types!(ScalarF5E7, CircleF5E7);
test_cross_type_for_types!(ScalarF6E3, CircleF6E3);
test_cross_type_for_types!(ScalarF6E4, CircleF6E4);
test_cross_type_for_types!(ScalarF6E5, CircleF6E5);
test_cross_type_for_types!(ScalarF6E6, CircleF6E6);
test_cross_type_for_types!(ScalarF6E7, CircleF6E7);
test_cross_type_for_types!(ScalarF7E3, CircleF7E3);
test_cross_type_for_types!(ScalarF7E4, CircleF7E4);
test_cross_type_for_types!(ScalarF7E5, CircleF7E5);
test_cross_type_for_types!(ScalarF7E6, CircleF7E6);
test_cross_type_for_types!(ScalarF7E7, CircleF7E7);

#[test]
fn test_mixed_precision_conversions() {
    // Test conversions between different precision levels
    let low_prec = ScalarF3E3::from(42.0);
    let high_prec = ScalarF7E7::from(42.0);

    // Convert to same precision for comparison
    let low_as_f32: f32 = low_prec.into();
    let high_as_f32: f32 = high_prec.into();

    // Should represent the same mathematical value (within low precision limits)
    assert_relative_eq!(low_as_f32, high_as_f32, epsilon = 1e-2);

    // Test precision upgrade
    let upgraded: ScalarF7E7 = ScalarF7E7::from(low_as_f32);
    let upgraded_val: f32 = upgraded.into();
    assert_relative_eq!(low_as_f32, upgraded_val, epsilon = 1e-5);

    // Test precision downgrade (may lose precision)
    let downgraded: ScalarF3E3 = ScalarF3E3::from(high_as_f32);
    let downgraded_val: f32 = downgraded.into();
    // Allow larger epsilon due to precision loss
    assert_relative_eq!(high_as_f32, downgraded_val, epsilon = 1e-1);
}

#[test]
fn test_conversion_accuracy_stress() {
    // Stress test conversion accuracy with challenging values
    let challenging_values = [
        1e-10f32, // Very small
        1e10f32,  // Very large
        std::f32::consts::PI,
        std::f32::consts::E,
        1.0 / 3.0,      // Repeating decimal
        1.0 / 7.0,      // Another repeating decimal
        2.0_f32.sqrt(), // Irrational
    ];

    for &val in &challenging_values {
        // Test round-trip conversion through different precisions
        let low_prec = ScalarF3E3::from(val);
        let mid_prec = ScalarF5E3::from(val);
        let high_prec = ScalarF7E7::from(val);

        if low_prec.is_normal() && mid_prec.is_normal() && high_prec.is_normal() {
            let low_back: f32 = low_prec.into();
            let mid_back: f32 = mid_prec.into();
            let high_back: f32 = high_prec.into();

            // Higher precision should be more accurate
            assert!((val - high_back).abs() <= (val - mid_back).abs() + 1e-6);
            assert!((val - mid_back).abs() <= (val - low_back).abs() + 1e-3);
        }

        // Test complex conversion
        let circle = CircleF5E3::from((val, val * 2.0));
        if circle.is_normal() {
            let real_back: f32 = circle.r().into();
            let imag_back: f32 = circle.i().into();

            assert_relative_eq!(real_back, val, epsilon = 1e-4);
            assert_relative_eq!(imag_back, val * 2.0, epsilon = 1e-4);
        }
    }
}

#[test]
fn test_type_system_consistency() {
    // Test that the type system maintains mathematical consistency
    let scalar = ScalarF5E3::from(5.0);
    let circle_real = CircleF5E3::from(5.0); // Creates real circle

    // Real circle should behave like scalar
    let scalar_squared = scalar * scalar;
    let circle_squared = circle_real * circle_real;

    let scalar_sq_val: f32 = scalar_squared.into();
    let circle_sq_real: f32 = circle_squared.r().into();
    let circle_sq_imag: f32 = circle_squared.i().into();

    assert_relative_eq!(scalar_sq_val, circle_sq_real, epsilon = 1e-5);
    assert_relative_eq!(circle_sq_imag, 0.0, epsilon = 1e-5);

    // Test commutativity across types
    let mixed1 = scalar + circle_real;
    let mixed2 = circle_real + scalar;

    let mixed1_real: f32 = mixed1.r().into();
    let mixed1_imag: f32 = mixed1.i().into();
    let mixed2_real: f32 = mixed2.r().into();
    let mixed2_imag: f32 = mixed2.i().into();

    assert_relative_eq!(mixed1_real, mixed2_real, epsilon = 1e-5);
    assert_relative_eq!(mixed1_imag, mixed2_imag, epsilon = 1e-5);
}

#[test]
fn test_complex_arithmetic_identities() {
    // Test complex number identities
    let i = CircleF5E3::POS_I;
    let one = CircleF5E3::ONE;

    // i² = -1
    let i_squared = i * i;
    let i_sq_real: f32 = i_squared.r().into();
    let i_sq_imag: f32 = i_squared.i().into();

    assert_relative_eq!(i_sq_real, -1.0, epsilon = 1e-5);
    assert_relative_eq!(i_sq_imag, 0.0, epsilon = 1e-5);

    // Test Euler's identity components
    let z1 = CircleF5E3::from((1.0, 2.0)); // 1 + 2i
    let z2 = CircleF5E3::from((3.0, 4.0)); // 3 + 4i

    // (a+bi)(c+di) = (ac-bd) + (ad+bc)i
    let product = z1 * z2;
    let expected_real = 1.0 * 3.0 - 2.0 * 4.0; // = -5
    let expected_imag = 1.0 * 4.0 + 2.0 * 3.0; // = 10

    let prod_real: f32 = product.r().into();
    let prod_imag: f32 = product.i().into();

    assert_relative_eq!(prod_real, expected_real, epsilon = 1e-5);
    assert_relative_eq!(prod_imag, expected_imag, epsilon = 1e-5);
}

#[test]
fn test_undefined_propagation_cross_type() {
    // Test that undefined states propagate correctly across type boundaries
    let undefined_scalar = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    let normal_circle = CircleF5E3::from((3.0, 4.0));

    // Operations should propagate undefined state appropriately
    let result1 = undefined_scalar + normal_circle;
    assert!(result1.r().is_undefined());
    assert!(result1.i().is_normal()); // Only real part should be undefined

    let result2 = normal_circle * undefined_scalar;
    assert!(result2.r().is_undefined());
    assert!(result2.i().is_undefined()); // Both parts undefined after multiplication
}
