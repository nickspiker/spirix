use approx::assert_relative_eq;
use spirix::*;

#[cfg(test)]
mod scalar_conversions {
    use super::*;

    #[test]
    fn test_exact_integer_conversions() {
        // Integer values should convert exactly between types
        let original = 42;

        let f3e3 = ScalarF3E3::from(original);
        let f5e3 = ScalarF5E3::from(original);
        let f7e7 = ScalarF7E7::from(original);

        // All should represent the same integer exactly
        assert_eq!(f3e3, ScalarF3E3::from(42));
        assert_eq!(f5e3, ScalarF5E3::from(42));
        assert_eq!(f7e7, ScalarF7E7::from(42));

        // Converting back to i32 should give exact result
        let back_f3e3: i32 = f3e3.into();
        let back_f5e3: i32 = f5e3.into();
        let back_f7e7: i32 = f7e7.into();

        assert_eq!(back_f3e3, original);
        assert_eq!(back_f5e3, original);
        assert_eq!(back_f7e7, original);
    }

    #[test]
    fn test_upward_conversions() {
        // Converting from lower to higher precision should preserve value

        // F3E3 → F5E3 → F7E7
        let original_f3e3 = ScalarF3E3::from(3.14);
        let temp_val: f32 = (&original_f3e3).into();
        let converted_f5e3 = ScalarF5E3::from(temp_val);
        let temp_val2: f32 = (&converted_f5e3).into();
        let converted_f7e7 = ScalarF7E7::from(temp_val2);

        // All should be normal
        assert!(original_f3e3.is_normal());
        assert!(converted_f5e3.is_normal());
        assert!(converted_f7e7.is_normal());

        // Values should be approximately equal
        let val_f3e3: f32 = original_f3e3.into();
        let val_f5e3: f32 = converted_f5e3.into();
        let val_f7e7: f32 = converted_f7e7.into();

        assert_relative_eq!(val_f3e3, val_f5e3, epsilon = 1e-2);
        assert_relative_eq!(val_f5e3, val_f7e7, epsilon = 1e-6);
    }

    #[test]
    fn test_downward_conversions() {
        // Converting from higher to lower precision may lose precision

        let high_precision = 1.123456789012345;

        let f7e7 = ScalarF7E7::from(high_precision);
        let f5e3 = ScalarF5E3::from(high_precision);
        let f3e3 = ScalarF3E3::from(high_precision);

        assert!(f7e7.is_normal());
        assert!(f5e3.is_normal());
        assert!(f3e3.is_normal());

        // Convert back to f64 to check precision loss
        let back_f7e7: f64 = f7e7.into();
        let back_f5e3: f64 = f5e3.into();
        let back_f3e3: f64 = f3e3.into();

        // F7E7 should be most precise
        assert_relative_eq!(back_f7e7, high_precision, epsilon = 1e-10);
        assert_relative_eq!(back_f5e3, high_precision, epsilon = 1e-6);
        assert_relative_eq!(back_f3e3, high_precision, epsilon = 1e-1);
    }

    #[test]
    fn test_range_overflow_conversions() {
        // Test what happens when converting values outside target range

        // Large value that fits in F7E7 but might not in F3E3
        let large_val = 1e20;
        let f7e7_large = ScalarF7E7::from(large_val);
        let f3e3_large = ScalarF3E3::from(large_val);

        assert!(f7e7_large.is_normal() || f7e7_large.exploded());
        // F3E3 should likely explode due to limited exponent range
        assert!(f3e3_large.exploded() || f3e3_large.is_normal());

        // Small value test
        let small_val = 1e-20;
        let f7e7_small = ScalarF7E7::from(small_val);
        let f3e3_small = ScalarF3E3::from(small_val);

        assert!(f7e7_small.is_normal() || f7e7_small.vanished());
        // F3E3 should likely vanish due to limited precision
        assert!(f3e3_small.vanished() || f3e3_small.is_normal());
    }

    #[test]
    fn test_cross_dimensional_conversions() {
        // Test conversions between different fraction/exponent combinations

        // High fraction, low exponent (F7E3) vs Low fraction, high exponent (F3E7)
        let precise_limited = ScalarF7E3::from(1.23456789012345);
        let imprecise_wide = ScalarF3E7::from(1.23456789012345);

        assert!(precise_limited.is_normal());
        assert!(imprecise_wide.is_normal());

        // Convert back to check precision
        let precise_back: f64 = precise_limited.into();
        let imprecise_back: f64 = imprecise_wide.into();

        // F7E3 should maintain more decimal precision
        assert_relative_eq!(precise_back, 1.23456789012345, epsilon = 1e-10);
        assert_relative_eq!(imprecise_back, 1.23456789012345, epsilon = 1e-1);

        // But F3E7 should handle larger ranges
        let large_test = 1e50;
        let precise_large = ScalarF7E3::from(large_test);
        let imprecise_large = ScalarF3E7::from(large_test);

        // F7E3 should explode, F3E7 should handle it
        assert!(precise_large.exploded());
        assert!(imprecise_large.is_normal() || imprecise_large.exploded());
    }

    #[test]
    fn test_state_preservation_across_conversions() {
        // Test that special states are preserved across conversions

        // Zero should remain zero
        let zero_f3e3 = ScalarF3E3::ZERO;
        let temp_zero: f32 = (&zero_f3e3).into();
        let zero_f7e7 = ScalarF7E7::from(temp_zero);
        assert!(zero_f3e3.is_zero());
        assert!(zero_f7e7.is_zero());

        // Exploded values should remain exploded (or become exploded)
        let exploded_f7e7: ScalarF7E7 = ScalarF7E7::MAX * 2.0;
        assert!(exploded_f7e7.exploded());

        let temp_exploded: f32 = (&exploded_f7e7).into();
        let converted_f3e3 = ScalarF3E3::from(temp_exploded);
        // Should be exploded or undefined
        assert!(converted_f3e3.exploded() || converted_f3e3.is_undefined());

        // Undefined should propagate
        let undefined = ScalarF5E3::from(1) / ScalarF5E3::ZERO;
        assert!(undefined.is_undefined());

        let temp_undefined: f32 = (&undefined).into();
        let undefined_converted = ScalarF7E7::from(temp_undefined);
        assert!(undefined_converted.is_undefined());
    }
}

#[cfg(test)]
mod circle_conversions {
    use super::*;

    #[test]
    fn test_complex_exact_conversions() {
        // Complex numbers with integer components should convert exactly

        let original = (3, 4);
        let z_f3e3 = CircleF3E3::from((original.0 as f32, original.1 as f32));
        let z_f5e3 = CircleF5E3::from((original.0 as f32, original.1 as f32));
        let z_f7e7 = CircleF7E7::from((original.0 as f32, original.1 as f32));

        assert!(z_f3e3.is_normal());
        assert!(z_f5e3.is_normal());
        assert!(z_f7e7.is_normal());

        // Real and imaginary parts should be exact
        let real_f3e3: i32 = (&z_f3e3.r()).into();
        let imag_f3e3: i32 = (&z_f3e3.i()).into();
        let real_f5e3: i32 = (&z_f5e3.r()).into();
        let imag_f5e3: i32 = (&z_f5e3.i()).into();
        let real_f7e7: i32 = (&z_f7e7.r()).into();
        let imag_f7e7: i32 = (&z_f7e7.i()).into();

        assert_eq!(real_f3e3, original.0);
        assert_eq!(imag_f3e3, original.1);
        assert_eq!(real_f5e3, original.0);
        assert_eq!(imag_f5e3, original.1);
        assert_eq!(real_f7e7, original.0);
        assert_eq!(imag_f7e7, original.1);
    }

    #[test]
    fn test_complex_precision_degradation() {
        // Test precision loss in complex number conversions

        let high_precision = (1.123456789, 2.987654321);

        let z_f7e7 = CircleF7E7::from(high_precision);
        let z_f5e3 = CircleF5E3::from(high_precision);
        let z_f3e3 = CircleF3E3::from(high_precision);

        assert!(z_f7e7.is_normal());
        assert!(z_f5e3.is_normal());
        assert!(z_f3e3.is_normal());

        // Check precision of real parts
        let real_f7e7: f64 = z_f7e7.r().into();
        let real_f5e3: f64 = z_f5e3.r().into();
        let real_f3e3: f64 = z_f3e3.r().into();

        assert_relative_eq!(real_f7e7, high_precision.0, epsilon = 1e-10);
        assert_relative_eq!(real_f5e3, high_precision.0, epsilon = 1e-6);
        assert_relative_eq!(real_f3e3, high_precision.0, epsilon = 1e-1);

        // Check precision of imaginary parts
        let imag_f7e7: f64 = z_f7e7.i().into();
        let imag_f5e3: f64 = z_f5e3.i().into();
        let imag_f3e3: f64 = z_f3e3.i().into();

        assert_relative_eq!(imag_f7e7, high_precision.1, epsilon = 1e-10);
        assert_relative_eq!(imag_f5e3, high_precision.1, epsilon = 1e-6);
        assert_relative_eq!(imag_f3e3, high_precision.1, epsilon = 1e-1);
    }

    #[test]
    fn test_complex_magnitude_across_types() {
        // Test that magnitude calculations are consistent across types

        let components = (3.0, 4.0); // Should give magnitude of 5.0

        let z_f3e3 = CircleF3E3::from(components);
        let z_f5e3 = CircleF5E3::from(components);
        let z_f7e7 = CircleF7E7::from(components);

        let mag_f3e3 = z_f3e3.magnitude();
        let mag_f5e3 = z_f5e3.magnitude();
        let mag_f7e7 = z_f7e7.magnitude();

        assert!(mag_f3e3.is_normal());
        assert!(mag_f5e3.is_normal());
        assert!(mag_f7e7.is_normal());

        // All should be approximately 5.0
        let mag_val_f3e3: f32 = mag_f3e3.into();
        let mag_val_f5e3: f32 = mag_f5e3.into();
        let mag_val_f7e7: f32 = mag_f7e7.into();

        assert_relative_eq!(mag_val_f3e3, 5.0, epsilon = 1e-1);
        assert_relative_eq!(mag_val_f5e3, 5.0, epsilon = 1e-5);
        assert_relative_eq!(mag_val_f7e7, 5.0, epsilon = 1e-10);
    }

    #[test]
    fn test_complex_state_preservation() {
        // Test that complex number states are preserved across conversions

        // Zero complex numbers
        let zero_f3e3 = CircleF3E3::ZERO;
        let temp_real: f32 = (&zero_f3e3.r()).into();
        let temp_imag: f32 = (&zero_f3e3.i()).into();
        let zero_f7e7 = CircleF7E7::from((temp_real, temp_imag));

        assert!(zero_f3e3.is_zero());
        assert!(zero_f7e7.is_zero());

        // Complex numbers with exploded components
        let exploded_scalar: ScalarF5E3 = ScalarF5E3::MAX * 2.0;
        assert!(exploded_scalar.exploded());

        let temp_exp_val: f32 = (&exploded_scalar).into();
        let z_with_exploded = CircleF5E3::from((temp_exp_val, 1.0));
        // Should be exploded or undefined
        assert!(z_with_exploded.exploded() || z_with_exploded.is_undefined());

        // Undefined complex numbers
        let undefined_scalar = ScalarF5E3::from(1) / ScalarF5E3::ZERO;
        let temp_undef_val: f32 = (&undefined_scalar).into();
        let z_undefined = CircleF5E3::from((temp_undef_val, 1.0));
        assert!(z_undefined.is_undefined());
    }
}

#[cfg(test)]
mod conversion_matrices {
    use super::*;

    // Macro to test conversion between two scalar types
    macro_rules! test_scalar_conversion {
        ($from_type:ident, $to_type:ident) => {
            let original_val = 42.0;
            let from_scalar = $from_type::from(original_val);
            let temp_conv: f32 = (&from_scalar).into();
            let to_scalar = $to_type::from(temp_conv);

            // Both should be normal for reasonable values
            assert!(from_scalar.is_normal());
            assert!(to_scalar.is_normal());

            // Values should be approximately equal
            let from_f32: f32 = from_scalar.into();
            let to_f32: f32 = to_scalar.into();
            assert_relative_eq!(from_f32, to_f32, epsilon = 1.0);
        };
    }

    // Macro to test conversion between two circle types
    macro_rules! test_circle_conversion {
        ($from_type:ident, $to_type:ident) => {
            let original_val = (3.0, 4.0);
            let from_circle = $from_type::from(original_val);
            let temp_r: f32 = (&from_circle.r()).into();
            let temp_i: f32 = (&from_circle.i()).into();
            let to_circle = $to_type::from((temp_r, temp_i));

            assert!(from_circle.is_normal());
            assert!(to_circle.is_normal());

            // Magnitudes should be approximately equal
            let from_mag: f32 = from_circle.magnitude().into();
            let to_mag: f32 = to_circle.magnitude().into();
            assert_relative_eq!(from_mag, to_mag, epsilon = 1.0);
        };
    }

    #[test]
    fn test_scalar_conversion_matrix() {
        // Test a representative sample of scalar conversions
        // (Testing all 625 combinations would be excessive)

        // Low precision conversions
        test_scalar_conversion!(ScalarF3E3, ScalarF3E4);
        test_scalar_conversion!(ScalarF3E3, ScalarF5E3);
        test_scalar_conversion!(ScalarF3E3, ScalarF7E7);

        // Medium precision conversions
        test_scalar_conversion!(ScalarF5E3, ScalarF3E3);
        test_scalar_conversion!(ScalarF5E3, ScalarF5E5);
        test_scalar_conversion!(ScalarF5E3, ScalarF7E3);

        // High precision conversions
        test_scalar_conversion!(ScalarF7E7, ScalarF3E3);
        test_scalar_conversion!(ScalarF7E7, ScalarF5E5);
        test_scalar_conversion!(ScalarF7E7, ScalarF6E6);

        // Cross-dimensional conversions
        test_scalar_conversion!(ScalarF3E7, ScalarF7E3);
        test_scalar_conversion!(ScalarF7E3, ScalarF3E7);
    }

    #[test]
    fn test_circle_conversion_matrix() {
        // Test a representative sample of circle conversions

        // Low precision conversions
        test_circle_conversion!(CircleF3E3, CircleF3E4);
        test_circle_conversion!(CircleF3E3, CircleF5E3);
        test_circle_conversion!(CircleF3E3, CircleF7E7);

        // Medium precision conversions
        test_circle_conversion!(CircleF5E3, CircleF3E3);
        test_circle_conversion!(CircleF5E3, CircleF5E5);
        test_circle_conversion!(CircleF5E3, CircleF7E3);

        // High precision conversions
        test_circle_conversion!(CircleF7E7, CircleF3E3);
        test_circle_conversion!(CircleF7E7, CircleF5E5);
        test_circle_conversion!(CircleF7E7, CircleF6E6);
    }

    #[test]
    fn test_boundary_conversions() {
        // Test conversions of boundary values

        // MAX values
        let max_f3e3 = ScalarF3E3::MAX;
        let temp_max: f32 = (&max_f3e3).into();
        let max_as_f7e7 = ScalarF7E7::from(temp_max);
        assert!(max_f3e3.is_normal());
        assert!(max_as_f7e7.is_normal());

        // MIN_POS values
        let min_f7e7 = ScalarF7E7::MIN_POS;
        let temp_min: f32 = (&min_f7e7).into();
        let min_as_f3e3 = ScalarF3E3::from(temp_min);
        assert!(min_f7e7.is_normal());
        // Might vanish due to precision loss
        assert!(min_as_f3e3.is_normal() || min_as_f3e3.vanished());

        // PI constant conversions
        let pi_f3e3: f32 = (&ScalarF3E3::PI).into();
        let pi_f7e7: f32 = (&ScalarF7E7::PI).into();

        // Should be approximately equal to π
        assert_relative_eq!(pi_f3e3, std::f32::consts::PI, epsilon = 1e-1);
        assert_relative_eq!(pi_f7e7, std::f32::consts::PI, epsilon = 1e-6);
    }
}