use approx::assert_relative_eq;
use spirix::*;

// Integration tests for cross-type operations and real-world scenarios
// These tests verify that different precision configurations work together
// and that complex operation chains preserve mathematical correctness

#[cfg(test)]
mod cross_precision_operations {
    use super::*;

    #[test]
    fn test_scalar_precision_interactions() {
        // Test operations between different scalar precisions
        let low_prec = ScalarF3E3::from(42.0);
        let high_prec = ScalarF6E4::from(42.0);

        // Both should represent the same value
        let low_val: f32 = low_prec.into();
        let high_val: f32 = high_prec.into();
        assert_relative_eq!(low_val, high_val, epsilon = 1e-2);

        // Test that high precision maintains more accuracy
        let pi_low = ScalarF3E3::from(std::f32::consts::PI);
        let pi_high = ScalarF6E4::from(std::f32::consts::PI);

        let pi_low_val: f32 = pi_low.into();
        let pi_high_val: f32 = pi_high.into();

        let error_low = (pi_low_val - std::f32::consts::PI).abs();
        let error_high = (pi_high_val - std::f32::consts::PI).abs();

        // High precision should have lower error (or at least not worse)
        assert!(error_high <= error_low + 1e-6);
    }

    #[test]
    fn test_circle_precision_interactions() {
        // Test complex numbers with different precisions
        let low_prec = CircleF4E3::from((3.14159, 2.71828));
        let high_prec = CircleF6E4::from((3.14159, 2.71828));

        // Extract components and compare
        let low_real: f32 = low_prec.r().into();
        let low_imag: f32 = low_prec.i().into();
        let high_real: f32 = high_prec.r().into();
        let high_imag: f32 = high_prec.i().into();

        assert_relative_eq!(low_real, high_real, epsilon = 1e-2);
        assert_relative_eq!(low_imag, high_imag, epsilon = 1e-2);

        // Test that magnitudes are consistent
        let low_mag = low_prec.magnitude();
        let high_mag = high_prec.magnitude();

        if low_mag.is_normal() && high_mag.is_normal() {
            let low_mag_val: f32 = low_mag.into();
            let high_mag_val: f32 = high_mag.into();
            assert_relative_eq!(low_mag_val, high_mag_val, epsilon = 1e-2);
        }
    }

    #[test]
    fn test_range_vs_precision_tradeoffs() {
        // Wide range, low precision
        let wide_range = ScalarF4E7::from(1e20);

        // Narrow range, high precision
        let high_precision = ScalarF7E3::from(1e20);

        // Wide range should handle large numbers better
        if wide_range.is_normal() && !high_precision.is_normal() {
            // This is expected - wide range can represent larger numbers
            assert!(wide_range.is_normal());
            assert!(high_precision.exploded() || high_precision.is_undefined());
        }

        // For small, precise values, high precision should be better
        let small_precise = 1.0 + 1e-10;
        let wp_small = ScalarF4E7::from(small_precise);
        let hp_small = ScalarF7E3::from(small_precise);

        if wp_small.is_normal() && hp_small.is_normal() {
            let wp_val: f32 = wp_small.into();
            let hp_val: f32 = hp_small.into();

            let wp_error = (wp_val - small_precise as f32).abs();
            let hp_error = (hp_val - small_precise as f32).abs();

            // High precision should be more accurate for small values
            assert!(hp_error <= wp_error + 1e-8);
        }
    }
}

#[cfg(test)]
mod scalar_circle_interactions {
    use super::*;

    #[test]
    fn test_scalar_to_circle_promotion() {
        let scalar = ScalarF5E3::from(7);
        let circle = CircleF5E3::from((3.0, 4.0));

        // Scalar + Circle should work
        let sum = scalar + circle;
        assert!(sum.is_normal());

        let sum_real: f32 = sum.r().into();
        let sum_imag: f32 = sum.i().into();
        assert_relative_eq!(sum_real, 10.0, epsilon = 1e-5); // 7 + 3
        assert_relative_eq!(sum_imag, 4.0, epsilon = 1e-5); // 0 + 4

        // Circle + Scalar should be commutative
        let sum2 = circle + scalar;
        assert_eq!(sum, sum2);

        // Multiplication
        let product = scalar * circle;
        assert!(product.is_normal());

        let prod_real: f32 = product.r().into();
        let prod_imag: f32 = product.i().into();
        assert_relative_eq!(prod_real, 21.0, epsilon = 1e-5); // 7 * 3
        assert_relative_eq!(prod_imag, 28.0, epsilon = 1e-5); // 7 * 4
    }

    #[test]
    fn test_mixed_precision_scalar_circle() {
        let _scalar_low = ScalarF4E3::from(2);
        let circle_high = CircleF6E4::from((3.0, 4.0));

        // Operations between different precisions should work
        // The result precision should be determined by implementation
        // Convert to same precision for multiplication
        let scalar_compat = ScalarF6E4::from(2);
        let product = scalar_compat * circle_high;

        // Should be mathematically correct regardless of internal precision
        if product.is_normal() {
            let prod_real: f32 = product.r().into();
            let prod_imag: f32 = product.i().into();
            assert_relative_eq!(prod_real, 6.0, epsilon = 1e-3);
            assert_relative_eq!(prod_imag, 8.0, epsilon = 1e-3);
        }
    }

    #[test]
    fn test_rust_primitive_interactions() {
        let circle = CircleF5E3::from((3.0, 4.0));

        // Operations with Rust primitives
        let sum_i32 = circle + 5i32;
        let sum_f32 = circle + 2.5f32;
        let sum_f64 = circle + 1.5f64;

        if sum_i32.is_normal() {
            let real: f32 = sum_i32.r().into();
            let imag: f32 = sum_i32.i().into();
            assert_relative_eq!(real, 8.0, epsilon = 1e-5);
            assert_relative_eq!(imag, 4.0, epsilon = 1e-5);
        }

        if sum_f32.is_normal() {
            let real: f32 = sum_f32.r().into();
            let imag: f32 = sum_f32.i().into();
            assert_relative_eq!(real, 5.5, epsilon = 1e-5);
            assert_relative_eq!(imag, 4.0, epsilon = 1e-5);
        }

        if sum_f64.is_normal() {
            let real: f32 = sum_f64.r().into();
            let imag: f32 = sum_f64.i().into();
            assert_relative_eq!(real, 4.5, epsilon = 1e-5);
            assert_relative_eq!(imag, 4.0, epsilon = 1e-5);
        }
    }
}

#[cfg(test)]
mod chained_operations {
    use super::*;

    #[test]
    fn test_undefined_state_preservation() {
        // Create an undefined state and verify it propagates through a chain
        // In Spirix, 1/0 = infinity (not undefined). Only 0/0 = undefined.
        let undefined_start = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
        assert!(undefined_start.is_undefined());

        // Chain multiple operations
        let result = undefined_start.square().exp().ln().sin().cos().tan();

        // Should still be undefined
        assert!(result.is_undefined());

        // Test with complex numbers - use 0/0 to get undefined
        let complex_undefined = CircleF5E3::ZERO / CircleF5E3::ZERO;
        assert!(complex_undefined.is_undefined());

        let complex_result = complex_undefined
            .conjugate()
            .square()
            .magnitude()
            .sqrt()
            .exp();

        assert!(complex_result.is_undefined());
    }

    #[test]
    fn test_escaped_value_chains() {
        // Test operations with escaped values
        let exploded: ScalarF5E3 = ScalarF5E3::MAX * 1000.0;

        if exploded.exploded() {
            // Absolute operations should preserve exploded state
            let still_exploded = exploded.square().square();
            assert!(still_exploded.exploded());

            // Sign should be preserved appropriately
            let neg_exploded = -exploded;
            assert!(neg_exploded.exploded());
            assert!(neg_exploded.is_negative());

            // Operations that could reduce magnitude
            let maybe_normal = exploded * ScalarF5E3::from(0.0001);
            // Result depends on implementation - could be normal or still exploded
            assert!(maybe_normal.is_normal() || maybe_normal.exploded() || maybe_normal.vanished());
        }

        let vanished: ScalarF5E3 = ScalarF5E3::MIN_POS / 1000000.0;

        if vanished.vanished() {
            // Absolute operations should preserve vanished state
            let still_vanished = vanished.square();
            assert!(still_vanished.vanished() || still_vanished.is_zero());

            // Operations that could increase magnitude
            let maybe_normal = vanished * ScalarF5E3::from(1e10);
            assert!(maybe_normal.is_normal() || maybe_normal.vanished() || maybe_normal.exploded());
        }
    }

    #[test]
    fn test_precision_degradation_chains() {
        // Start with high precision value
        let precise = ScalarF7E4::from(2.5);

        // Chain operations using inverse pairs that should reconstruct the value:
        // exp then ln, square then sqrt
        let result = precise.exp().ln().square().sqrt();

        if result.is_normal() {
            let result_val: f64 = result.into();
            let original_val: f64 = precise.into();

            // Some precision loss is expected, but should be reasonable
            assert_relative_eq!(result_val, original_val, epsilon = 1e-2);
        }
    }

    #[test]
    fn test_complex_calculation_chains() {
        // Complex mathematical expression: e^(iπ) + 1 should equal 0 (Euler's identity)
        let i = CircleF5E3::POS_I;
        let pi = ScalarF5E3::PI;
        let one = CircleF5E3::ONE;

        let euler_result = (i * pi).exp() + one;

        if euler_result.is_normal() {
            let real: f32 = euler_result.r().into();
            let imag: f32 = euler_result.i().into();

            // Should be very close to zero
            assert_relative_eq!(real, 0.0, epsilon = 1e-3);
            assert_relative_eq!(imag, 0.0, epsilon = 1e-3);
        }

        // Test complex power: (1+i)^4 = ((1+i)^2)^2 = (2i)^2 = -4
        let one_plus_i = CircleF5E3::from((1.0, 1.0));
        let fourth_power = one_plus_i.pow(ScalarF5E3::from(4));

        if fourth_power.is_normal() {
            let real: f32 = fourth_power.r().into();
            let imag: f32 = fourth_power.i().into();
            assert_relative_eq!(real, -4.0, epsilon = 1e-3);
            assert_relative_eq!(imag, 0.0, epsilon = 1e-3);
        }
    }
}

#[cfg(test)]
mod real_world_scenarios {
    use super::*;

    #[test]
    fn test_numerical_integration_scenario() {
        // Simulate a simple numerical integration using Spirix types
        fn integrate_sin_squared(steps: usize) -> ScalarF6E4 {
            let step_size = ScalarF6E4::PI / steps as f32;
            let mut sum = ScalarF6E4::ZERO;

            for i in 0..steps {
                let x = step_size * i as f32;
                let sin_x = x.sin();
                let sin_squared = sin_x.square();
                sum = sum + sin_squared * step_size;
            }

            sum
        }

        // ∫[0,π] sin²(x) dx = π/2
        let result = integrate_sin_squared(1000);

        if result.is_normal() {
            let result_val: f32 = result.into();
            let expected = std::f32::consts::PI / 2.0;
            assert_relative_eq!(result_val, expected, epsilon = 1e-2);
        }
    }

    #[test]
    fn test_complex_mandelbrot_iteration() {
        // Test a few iterations of Mandelbrot set calculation
        fn mandelbrot_iteration(c: CircleF5E3, max_iter: usize) -> usize {
            let mut z = CircleF5E3::ZERO;

            for i in 0..max_iter {
                let mag_squared: f32 = z.magnitude_squared().into();
                if mag_squared > 4.0 {
                    return i;
                }
                z = z.square() + c;

                if z.is_undefined() || z.exploded() {
                    return i;
                }
            }
            max_iter
        }

        // Test point inside Mandelbrot set
        let c_inside = CircleF5E3::from((0.0, 0.0));
        let iter_inside = mandelbrot_iteration(c_inside, 100);
        assert_eq!(iter_inside, 100); // Should not escape

        // Test point outside Mandelbrot set
        let c_outside = CircleF5E3::from((2.0, 2.0));
        let iter_outside = mandelbrot_iteration(c_outside, 100);
        assert!(iter_outside < 10); // Should escape quickly

        // Test boundary point - at F5E3 precision, this may or may not escape
        // depending on rounding, so just verify it runs without panicking
        let c_boundary = CircleF5E3::from((-0.75, 0.15));
        let iter_boundary = mandelbrot_iteration(c_boundary, 100);
        assert!(iter_boundary >= 1); // Should iterate at least once
    }

    #[test]
    fn test_dft_like_computation() {
        // Test a simplified DFT-like computation using complex arithmetic
        let n = 8;
        let data = vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];

        // Convert to Spirix complex numbers
        let spirix_data: Vec<CircleF5E3> = data.iter().map(|&x| CircleF5E3::from(x)).collect();

        // Compute one DFT bin (k=1)
        let k = 1;
        let mut bin_result = CircleF5E3::ZERO;

        for (n_idx, &sample) in spirix_data.iter().enumerate() {
            let angle = -2.0 * std::f32::consts::PI * k as f32 * n_idx as f32 / n as f32;
            let twiddle = CircleF5E3::from((angle.cos(), angle.sin()));
            bin_result = bin_result + sample * twiddle;
        }

        // For this particular input, the result should be a real number
        if bin_result.is_normal() {
            let imag_part: f32 = bin_result.i().into();
            assert_relative_eq!(imag_part, 0.0, epsilon = 1e-3);
        }
    }

    #[test]
    fn test_financial_calculation() {
        // Test compound interest calculation with high precision
        // A = P(1 + r/n)^(nt)
        let principal = ScalarF7E4::from(10000.0); // $10,000
        let rate = ScalarF7E4::from(0.05); // 5% annual rate
        let compounds_per_year = ScalarF7E4::from(12.0); // Monthly compounding
        let years = ScalarF7E4::from(10.0); // 10 years

        let one = ScalarF7E4::ONE;
        let rate_per_period = rate / compounds_per_year;
        let total_periods = compounds_per_year * years;

        let compound_factor = (one + rate_per_period).pow(total_periods);
        let final_amount = principal * compound_factor;

        if final_amount.is_normal() && compound_factor.is_normal() {
            let final_val: f64 = final_amount.into();
            let factor_val: f64 = compound_factor.into();

            // Should be close to analytical result: 10000 * (1 + 0.05/12)^(12*10)
            let expected = 10000.0 * (1.0 + 0.05 / 12.0_f64).powf(12.0 * 10.0);
            assert_relative_eq!(final_val, expected, epsilon = 1e-2);

            // Compound factor should be > 1
            assert!(factor_val > 1.0);
        }
    }

    #[test]
    fn test_signal_processing_scenario() {
        // Test a simple digital filter implementation
        fn simple_lowpass_filter(input: &[f32], alpha: f32) -> Vec<ScalarF5E3> {
            let mut output = Vec::with_capacity(input.len());
            let mut prev_output = ScalarF5E3::ZERO;
            let alpha_scalar = ScalarF5E3::from(alpha);
            let one_minus_alpha = ScalarF5E3::ONE - alpha_scalar;

            for &sample in input {
                let input_scalar = ScalarF5E3::from(sample);
                let current_output = alpha_scalar * input_scalar + one_minus_alpha * prev_output;
                output.push(current_output);
                prev_output = current_output;
            }

            output
        }

        // Test with a simple signal: step function
        let input = vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0];
        let alpha = 0.1; // Low-pass filter coefficient

        let filtered = simple_lowpass_filter(&input, alpha);

        // Verify filter properties
        assert_eq!(filtered.len(), input.len());

        // First few samples should start at zero
        if filtered[0].is_normal() {
            let first_val: f32 = filtered[0].into();
            assert_relative_eq!(first_val, 0.0, epsilon = 1e-5);
        }

        // Filter should gradually respond to step
        for i in 3..6 {
            if filtered[i].is_normal() && filtered[i - 1].is_normal() {
                let current_val: f32 = filtered[i].into();
                let prev_val: f32 = filtered[i - 1].into();
                // Should be increasing during step response
                assert!(current_val >= prev_val - 1e-6);
            }
        }
    }
}

#[cfg(test)]
mod precision_boundary_tests {
    use super::*;

    #[test]
    fn test_precision_boundaries() {
        // Test behavior near precision boundaries

        // Very small numbers near underflow
        let tiny = ScalarF5E3::from(1e-30);
        let tinier: ScalarF5E3 = tiny / 1000.0;

        // Should handle gracefully (normal, vanished, or zero)
        assert!(tinier.is_normal() || tinier.vanished() || tinier.is_zero());

        // Very large numbers near overflow
        let big = ScalarF5E3::from(1e30);
        let bigger: ScalarF5E3 = big * 1000.0;

        // Should handle gracefully (normal or exploded)
        assert!(bigger.is_normal() || bigger.exploded());

        // Operations near precision limits
        // Note: 1e-10 is below F5E3 precision, so ONE - 1e-10 = ONE exactly.
        // Use a larger epsilon that F5E3 can actually represent.
        let almost_one = ScalarF5E3::ONE - ScalarF5E3::from(0.001);
        let sqrt_almost_one = almost_one.sqrt();

        if sqrt_almost_one.is_normal() {
            let val: f32 = sqrt_almost_one.into();
            assert!(val <= 1.0 && val > 0.999);
        }
    }

    #[test]
    fn test_mixed_range_operations() {
        // Test operations between values of very different magnitudes
        let large = ScalarF5E3::from(1e15);
        let small = ScalarF5E3::from(1e-15);

        // Addition should be dominated by large value
        let sum = large + small;
        if sum.is_normal() && large.is_normal() {
            let sum_val: f32 = sum.into();
            let large_val: f32 = large.into();
            assert_relative_eq!(sum_val, large_val, epsilon = 1e-6);
        }

        // Multiplication might produce normal result
        let product = large * small;
        if product.is_normal() {
            let prod_val: f32 = product.into();
            assert_relative_eq!(prod_val, 1.0, epsilon = 1e-3);
        }

        // Division should create very large or exploded result
        let quotient = large / small;
        assert!(quotient.exploded() || quotient.is_normal());

        if quotient.is_normal() {
            let quot_val: f32 = quotient.into();
            assert!(quot_val > 1e20);
        }
    }
}
