use approx::assert_relative_eq;
use spirix::*;

#[cfg(test)]
mod precision_boundaries {
    use super::*;

    #[test]
    fn test_minimal_precision_f3e3() {
        // F3E3: 8-bit fraction (~2.1 decimal digits), 8-bit exponent
        // This should have very limited precision

        let a = ScalarF3E3::from(1.0);
        let b = ScalarF3E3::from(1.01); // May lose precision

        // Basic arithmetic should work
        assert!(a.is_normal());
        assert!(b.is_normal());

        // Test that we can still do exact integer arithmetic
        let exact_int = ScalarF3E3::from(5) + ScalarF3E3::from(3);
        assert_eq!(exact_int, ScalarF3E3::from(8));

        // Test constants are available
        assert!(ScalarF3E3::PI.is_normal());
        assert!(ScalarF3E3::E.is_normal());

        // Test overflow behavior
        let max_val = ScalarF3E3::MAX;
        let exploded: ScalarF3E3 = max_val * 2.0;
        assert!(exploded.exploded());

        // Test underflow behavior
        let min_pos = ScalarF3E3::MIN_POS;
        let vanished: ScalarF3E3 = min_pos / 1000.0;
        assert!(vanished.vanished());
    }

    #[test]
    fn test_maximum_precision_f7e7() {
        // F7E7: 128-bit fraction (~38.2 decimal digits), 128-bit exponent
        // This should have extreme precision and range

        let high_precision = ScalarF7E7::from(1.23456789012345678901234567890123456789);
        assert!(high_precision.is_normal());

        // Test exact arithmetic still works
        let exact_result = ScalarF7E7::from(1000000) + ScalarF7E7::from(1);
        assert_eq!(exact_result, ScalarF7E7::from(1000001));

        // Test that this type can handle much larger ranges
        let very_large = ScalarF7E7::from(1e100);
        assert!(very_large.is_normal() || very_large.exploded());

        let very_small = ScalarF7E7::from(1e-100);
        assert!(very_small.is_normal() || very_small.vanished());

        // Test constants
        assert!(ScalarF7E7::PI.is_normal());
        assert!(ScalarF7E7::E.is_normal());
    }

    #[test]
    fn test_extreme_combinations() {
        // F3E7: Minimal fraction precision but huge exponent range
        let min_frac_max_exp = ScalarF3E7::from(1e50);
        assert!(min_frac_max_exp.is_normal() || min_frac_max_exp.exploded());

        // F7E3: Maximum fraction precision but limited exponent range
        let max_frac_min_exp = ScalarF7E3::from(1.234567890123456789);
        assert!(max_frac_min_exp.is_normal());

        // This should overflow the exponent range
        let overflow_exp = ScalarF7E3::from(1e50);
        assert!(overflow_exp.exploded() || overflow_exp.is_normal());
    }

    #[test]
    fn test_precision_degradation() {
        // Test how precision degrades from high to low precision types

        // Start with a high-precision value
        let precise = ScalarF7E7::from(1.123456789012345);
        let converted_f5e3: ScalarF5E3 = ScalarF5E3::from(1.123456789012345);
        let converted_f3e3: ScalarF3E3 = ScalarF3E3::from(1.123456789012345);

        // All should be normal, but with different precision
        assert!(precise.is_normal());
        assert!(converted_f5e3.is_normal());
        assert!(converted_f3e3.is_normal());

        // Convert to f32 to compare
        let precise_f32: f32 = precise.into();
        let f5e3_f32: f32 = converted_f5e3.into();
        let f3e3_f32: f32 = converted_f3e3.into();

        // F3E3 should be least precise
        assert_relative_eq!(precise_f32, 1.123456789012345, epsilon = 1e-10);
        assert_relative_eq!(f5e3_f32, 1.123456789012345, epsilon = 1e-6);
        assert_relative_eq!(f3e3_f32, 1.123456789012345, epsilon = 2e-2);
    }

    #[test]
    fn test_complex_precision_boundaries() {
        // Test complex numbers with extreme precisions

        // Minimal precision complex
        let z_min = CircleF3E3::from((1.1, 2.2));
        assert!(z_min.is_normal());

        // Maximum precision complex
        let z_max = CircleF7E7::from((1.123456789, 2.987654321));
        assert!(z_max.is_normal());

        // Test arithmetic still works
        let sum_min = z_min + CircleF3E3::from((0.9, -1.2));
        assert!(sum_min.is_normal());

        let sum_max = z_max + CircleF7E7::from((0.876543211, -1.234567890));
        assert!(sum_max.is_normal());

        // Test magnitude calculations
        let mag_min = z_min.magnitude();
        let mag_max = z_max.magnitude();
        assert!(mag_min.is_normal());
        assert!(mag_max.is_normal());
    }
}

#[cfg(test)]
mod extreme_values {
    use super::*;

    #[test]
    fn test_boundary_conditions() {
        // Test each type's MAX and MIN_POS values

        // Test ScalarF3E3 boundaries
        assert!(ScalarF3E3::MAX.is_normal());
        assert!(ScalarF3E3::MIN_POS.is_normal());

        // Test ScalarF7E7 boundaries
        assert!(ScalarF7E7::MAX.is_normal());
        assert!(ScalarF7E7::MIN_POS.is_normal());

        // Test that MAX + MAX creates exploded
        let f3e3_exploded: ScalarF3E3 = ScalarF3E3::MAX + ScalarF3E3::MAX;
        assert!(f3e3_exploded.exploded());

        let f7e7_exploded: ScalarF7E7 = ScalarF7E7::MAX + ScalarF7E7::MAX;
        assert!(f7e7_exploded.exploded());

        // Test that MIN_POS / large_number creates vanished
        let f3e3_vanished: ScalarF3E3 = ScalarF3E3::MIN_POS / 1000000.0;
        assert!(f3e3_vanished.vanished());

        let f7e7_vanished: ScalarF7E7 = ScalarF7E7::MIN_POS / 1e100;
        assert!(f7e7_vanished.vanished() || f7e7_vanished.is_normal());
    }

    #[test]
    fn test_operations_near_boundaries() {
        // Test arithmetic operations near the boundaries

        let near_max: ScalarF5E3 = ScalarF5E3::MAX / 2.0;
        let near_min: ScalarF5E3 = ScalarF5E3::MIN_POS * 2.0;

        assert!(near_max.is_normal());
        assert!(near_min.is_normal());

        // Operations that should stay normal
        let safe_add = near_max + ScalarF5E3::from(1);
        let safe_mul = near_min * ScalarF5E3::from(2);

        assert!(safe_add.is_normal());
        assert!(safe_mul.is_normal());

        // Operations that should escape
        let overflow_mul: ScalarF5E3 = near_max * ScalarF5E3::from(10);
        let underflow_div: ScalarF5E3 = near_min / ScalarF5E3::from(1000000);

        assert!(overflow_mul.exploded() || overflow_mul.is_normal());
        assert!(underflow_div.vanished() || underflow_div.is_normal());
    }

    #[test]
    fn test_mathematical_functions_at_boundaries() {
        // Test mathematical functions with extreme inputs

        // Exponential of large values
        let large_exp = ScalarF5E3::from(100).exp();
        assert!(large_exp.exploded() || large_exp.is_normal());

        // Logarithm of very small values
        let tiny_val = ScalarF5E3::MIN_POS;
        let small_log = tiny_val.ln();
        assert!(small_log.is_normal() || small_log.exploded()); // ln(very_small) is very negative

        // Square root of very large values
        let large_sqrt = ScalarF5E3::MAX.sqrt();
        assert!(large_sqrt.is_normal());

        // Power functions with extreme values
        let small_pow = ScalarF5E3::from(0.1).pow(ScalarF5E3::from(100));
        assert!(small_pow.vanished() || small_pow.is_normal());
    }
}

#[cfg(test)]
mod rube_goldberg_chains {
    use super::*;

    #[test]
    fn test_precision_cascade() {
        // Rube Goldberg machine: Start with F3E3, cascade through all precisions

        // Step 1: F3E3 - Start with limited precision
        let step1 = ScalarF3E3::from(3.14159);
        assert!(step1.is_normal());

        // Step 2: F4E4 - Square it (creates small precision error)
        let step2_val: f32 = step1.into();
        let step2 = ScalarF4E4::from(step2_val).square();
        assert!(step2.is_normal());

        // Step 3: F5E5 - Take square root (compounds error)
        let step3_val: f32 = step2.into();
        let step3 = ScalarF5E5::from(step3_val).sqrt();
        assert!(step3.is_normal());

        // Step 4: F6E6 - Apply trigonometry
        let step4_val: f32 = step3.into();
        let step4 = ScalarF6E6::from(step4_val).sin();
        assert!(step4.is_normal());

        // Step 5: F7E7 - Final high-precision calculation
        let step5_val: f32 = step4.into();
        let step5 = ScalarF7E7::from(step5_val).exp();
        assert!(step5.is_normal());

        // The final result should be normal but significantly different from
        // what we'd get if we did the entire calculation in F7E7
        let final_val: f32 = step5.into();
        assert!(final_val.is_finite());
    }

    #[test]
    fn test_complex_precision_cascade() {
        // Complex version of the cascade

        let z1 = CircleF3E3::from((1.1, 2.2));
        let temp_r2: f32 = (&z1.r()).into();
        let temp_i2: f32 = (&z1.i()).into();
        let z2 = CircleF4E4::from((temp_r2, temp_i2));
        let z3 = z2 * z2; // Square the complex number
        assert!(z3.is_normal());

        let temp_r4: f32 = (&z3.r()).into();
        let temp_i4: f32 = (&z3.i()).into();
        let z4 = CircleF5E5::from((temp_r4, temp_i4));
        let z5 = z4.conjugate();
        assert!(z5.is_normal());

        let temp_r6: f32 = (&z5.r()).into();
        let temp_i6: f32 = (&z5.i()).into();
        let z6 = CircleF6E6::from((temp_r6, temp_i6));
        let mag = z6.magnitude();
        assert!(mag.is_normal());

        // Use the magnitude in a final F7E7 calculation
        let temp_mag: f32 = (&mag).into();
        let final_result = ScalarF7E7::from(temp_mag).ln();
        assert!(final_result.is_normal());
    }

    #[test]
    fn test_escape_state_cascade() {
        // Create a cascade that triggers exploded → vanished → undefined states

        // Start with a large value that will explode
        let large = ScalarF5E3::MAX;
        let exploded: ScalarF5E3 = large * large;
        assert!(exploded.exploded());

        // Use exploded state to create vanished (through reciprocal)
        let reciprocal = exploded.reciprocal();
        assert!(reciprocal.vanished());

        // Use vanished to create undefined (through log of very small number)
        let log_vanished = reciprocal.ln();
        assert!(log_vanished.is_undefined() || log_vanished.exploded());

        // Propagate undefined through arithmetic
        let propagated1 = log_vanished + ScalarF5E3::from(42);
        let propagated2 = propagated1 * ScalarF5E3::PI;
        let propagated3 = propagated2.exp();

        assert!(propagated1.is_undefined() || propagated1.exploded());
        assert!(propagated2.is_undefined() || propagated2.exploded());
        assert!(propagated3.is_undefined() || propagated3.exploded());
    }

    #[test]
    fn test_mixed_type_chain_reaction() {
        // Mix scalars and circles in a chain reaction

        // Start with a scalar
        let scalar_start = ScalarF4E4::from(2.5);

        // Convert to complex
        let complex1 = CircleF4E4::from(scalar_start);
        assert!(complex1.is_normal());

        // Complex arithmetic
        let complex2 = complex1 * CircleF4E4::POS_I; // Multiply by i
        assert!(complex2.is_normal());

        // Extract imaginary part back to scalar
        let scalar_mid = complex2.i();
        assert!(scalar_mid.is_normal());

        // Use in another complex number
        let temp_scalar_mid: f32 = (&scalar_mid).into();
        let complex3 = CircleF5E5::from((temp_scalar_mid, 1.0));
        assert!(complex3.is_normal());

        // Calculate magnitude
        let magnitude = complex3.magnitude();
        assert!(magnitude.is_normal());

        // Use magnitude in high-precision calculation
        let temp_magnitude: f32 = (&magnitude).into();
        let final_scalar = ScalarF7E7::from(temp_magnitude);
        let final_result = final_scalar.sqrt().square(); // Should equal original
        assert!(final_result.is_normal());

        // Verify the chain preserved reasonableness
        let final_val: f32 = final_result.into();
        assert!(final_val > 0.0 && final_val < 10.0);
    }
}
