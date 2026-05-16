use approx::assert_relative_eq;
use proptest::prelude::*;
use spirix::*;

/// Enhanced property-based tests for mathematical invariants and edge cases Tests two's complement representation properties and numerical stability

// Property test strategies for different value ranges
fn small_finite_scalar() -> impl Strategy<Value = ScalarF5E3> {
    (-1000.0f32..1000.0f32).prop_map(ScalarF5E3::from)
}

fn positive_scalar() -> impl Strategy<Value = ScalarF5E3> {
    (0.001f32..1000.0f32).prop_map(ScalarF5E3::from)
}

fn unit_interval_scalar() -> impl Strategy<Value = ScalarF5E3> {
    (-1.0f32..1.0f32).prop_map(ScalarF5E3::from)
}

fn small_circle() -> impl Strategy<Value = CircleF5E3> {
    ((-100.0f32..100.0f32), (-100.0f32..100.0f32)).prop_map(|(r, i)| CircleF5E3::from((r, i)))
}

proptest! {
    #[test]
    fn test_twos_complement_addition_properties(
        a in small_finite_scalar(),
        b in small_finite_scalar(),
        c in small_finite_scalar()
    ) {
        if a.is_normal() && b.is_normal() && c.is_normal() {
            // Commutativity: a + b = b + a
            let sum1 = a + b;
            let sum2 = b + a;
            if sum1.is_normal() && sum2.is_normal() {
                let val1: f32 = sum1.into();
                let val2: f32 = sum2.into();
                assert_relative_eq!(val1, val2, epsilon = 1e-6);
            }

            // Associativity: (a + b) + c = a + (b + c)
            let left = (a + b) + c;
            let right = a + (b + c);
            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-5);
            }

            // Identity: a + 0 = a
            let with_zero = a + ScalarF5E3::ZERO;
            if with_zero.is_normal() {
                let orig: f32 = a.into();
                let with_zero_val: f32 = with_zero.into();
                assert_relative_eq!(orig, with_zero_val, epsilon = 1e-6);
            }

            // Inverse: a + (-a) = 0 (for two's complement)
            let inverse_sum = a + (-a);
            if inverse_sum.is_normal() || inverse_sum.is_zero() {
                if inverse_sum.is_zero() {
                    assert!(inverse_sum == ScalarF5E3::ZERO);
                } else {
                    let inverse_val: f32 = inverse_sum.into();
                    assert_relative_eq!(inverse_val, 0.0, epsilon = 1e-6);
                }
            }
        }
    }

    #[test]
    fn test_twos_complement_multiplication_properties(
        a in small_finite_scalar(),
        b in small_finite_scalar(),
        c in small_finite_scalar()
    ) {
        if a.is_normal() && b.is_normal() && c.is_normal() {
            // Commutativity: a * b = b * a
            let prod1 = a * b;
            let prod2 = b * a;
            if prod1.is_normal() && prod2.is_normal() {
                let val1: f32 = prod1.into();
                let val2: f32 = prod2.into();
                assert_relative_eq!(val1, val2, epsilon = 1e-6);
            }

            // Associativity: (a * b) * c = a * (b * c)
            let left = (a * b) * c;
            let right = a * (b * c);
            if left.is_normal() && right.is_normal() {
                let left_val: f32 = left.into();
                let right_val: f32 = right.into();
                assert_relative_eq!(left_val, right_val, epsilon = 1e-5);
            }

            // Identity: a * 1 = a
            let with_one = a * ScalarF5E3::ONE;
            if with_one.is_normal() {
                let orig: f32 = a.into();
                let with_one_val: f32 = with_one.into();
                assert_relative_eq!(orig, with_one_val, epsilon = 1e-6);
            }

            // Zero property: a * 0 = 0
            let with_zero = a * ScalarF5E3::ZERO;
            assert!(with_zero.is_zero() || with_zero == ScalarF5E3::ZERO);

            // Distributivity: a * (b + c) = a * b + a * c
            let sum_bc = b + c;
            if sum_bc.is_normal() {
                let left_dist = a * sum_bc;
                let right_dist = (a * b) + (a * c);
                if left_dist.is_normal() && right_dist.is_normal() {
                    let left_val: f32 = left_dist.into();
                    let right_val: f32 = right_dist.into();
                    assert_relative_eq!(left_val, right_val, epsilon = 1e-3);
                }
            }
        }
    }

    #[test]
    fn test_division_properties(
        a in small_finite_scalar(),
        b in small_finite_scalar()
    ) {
        if a.is_normal() && b.is_normal() && !b.is_zero() {
            let quotient = a / b;
            if quotient.is_normal() {
                // Multiplication inverse: (a / b) * b = a
                let reconstructed = quotient * b;
                if reconstructed.is_normal() {
                    let orig: f32 = a.into();
                    let recon: f32 = reconstructed.into();
                    assert_relative_eq!(orig, recon, epsilon = 1e-5);
                }

                // Division by one: a / 1 = a
                if b == ScalarF5E3::ONE {
                    let orig: f32 = a.into();
                    let div_by_one: f32 = quotient.into();
                    assert_relative_eq!(orig, div_by_one, epsilon = 1e-6);
                }
            }
        }

        // Division by zero: in Spirix, nonzero/0 = infinity, 0/0 = undefined
        if a.is_normal() && !a.is_zero() {
            let div_by_zero = a / ScalarF5E3::ZERO;
            // 1/0 = infinity in Spirix (not undefined, not exploded, not normal)
            assert!(!div_by_zero.is_normal() && !div_by_zero.is_undefined());
        }
    }

    #[test]
    fn test_sign_preservation_properties(
        a in small_finite_scalar()
    ) {
        if a.is_normal() {
            // Double negation: -(-a) = a
            let double_neg = -(-a);
            if double_neg.is_normal() {
                let orig: f32 = a.into();
                let double_neg_val: f32 = double_neg.into();
                assert_relative_eq!(orig, double_neg_val, epsilon = 1e-6);
            }

            // Sign consistency with two's complement
            if a.is_positive() {
                assert!((-a).is_negative() || (-a).is_zero());
            } else if a.is_negative() {
                assert!((-a).is_positive() || (-a).is_zero());
            }

            // Absolute value properties
            let abs_a = a.magnitude();
            if abs_a.is_normal() {
                assert!(abs_a.is_positive() || abs_a.is_zero());

                // |a| * sign(a) = a (approximately)
                if !a.is_zero() {
                    let sign_a = a.sign();
                    if sign_a.is_normal() {
                        let reconstructed = abs_a * sign_a;
                        if reconstructed.is_normal() {
                            let orig: f32 = a.into();
                            let recon: f32 = reconstructed.into();
                            assert_relative_eq!(orig, recon, epsilon = 1e-6);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_power_function_properties(
        base in positive_scalar(),
        exp1 in unit_interval_scalar(),
        exp2 in unit_interval_scalar()
    ) {
        if base.is_normal() && base.is_positive() &&
           exp1.is_normal() && exp2.is_normal() {

            // Power of 1: a^1 = a
            let power_one = base.pow(ScalarF5E3::ONE);
            if power_one.is_normal() {
                let base_val: f32 = base.into();
                let power_val: f32 = power_one.into();
                assert_relative_eq!(base_val, power_val, epsilon = 1e-6);
            }

            // Power of 0: a^0 = 1 (for positive a)
            let power_zero = base.pow(ScalarF5E3::ZERO);
            if power_zero.is_normal() {
                let power_val: f32 = power_zero.into();
                assert_relative_eq!(power_val, 1.0, epsilon = 1e-6);
            }

            // Power rule: a^(m+n) = a^m * a^n
            let sum_exp = exp1 + exp2;
            if sum_exp.is_normal() {
                let power_sum = base.pow(sum_exp);
                let power1 = base.pow(exp1);
                let power2 = base.pow(exp2);

                if power_sum.is_normal() && power1.is_normal() && power2.is_normal() {
                    let product = power1 * power2;
                    if product.is_normal() {
                        let sum_val: f32 = power_sum.into();
                        let prod_val: f32 = product.into();
                        assert_relative_eq!(sum_val, prod_val, epsilon = 1e-4);
                    }
                }
            }
        }
    }

    #[test]
    fn test_trigonometric_properties(
        angle in unit_interval_scalar()
    ) {
        if angle.is_normal() {
            let sin_val = angle.sin();
            let cos_val = angle.cos();

            if sin_val.is_normal() && cos_val.is_normal() {
                // Pythagorean identity: sin²θ + cos²θ = 1
                let sin_sq = sin_val * sin_val;
                let cos_sq = cos_val * cos_val;
                let sum = sin_sq + cos_sq;

                if sum.is_normal() {
                    let sum_val: f32 = sum.into();
                    assert_relative_eq!(sum_val, 1.0, epsilon = 1e-5);
                }

                // Range properties: -1 ≤ sin(θ) ≤ 1, -1 ≤ cos(θ) ≤ 1
                let sin_f32: f32 = sin_val.into();
                let cos_f32: f32 = cos_val.into();
                assert!(sin_f32 >= -1.01 && sin_f32 <= 1.01); // Small tolerance for numerical error
                assert!(cos_f32 >= -1.01 && cos_f32 <= 1.01);

                // Even/odd properties: sin(-θ) = -sin(θ), cos(-θ) = cos(θ)
                let neg_angle = -angle;
                let sin_neg = neg_angle.sin();
                let cos_neg = neg_angle.cos();

                if sin_neg.is_normal() && cos_neg.is_normal() {
                    let neg_sin_val = -sin_val;
                    if neg_sin_val.is_normal() {
                        let sin_neg_f32: f32 = sin_neg.into();
                        let neg_sin_f32: f32 = neg_sin_val.into();
                        assert_relative_eq!(sin_neg_f32, neg_sin_f32, epsilon = 1e-6);
                    }

                    let cos_neg_f32: f32 = cos_neg.into();
                    assert_relative_eq!(cos_f32, cos_neg_f32, epsilon = 1e-6);
                }
            }
        }
    }

    #[test]
    fn test_exponential_logarithm_properties(
        x in positive_scalar()
    ) {
        if x.is_normal() && x.is_positive() {
            let ln_x = x.ln();

            if ln_x.is_normal() {
                // exp(ln(x)) = x
                let exp_ln_x = ln_x.exp();
                if exp_ln_x.is_normal() {
                    let orig: f32 = x.into();
                    let roundtrip: f32 = exp_ln_x.into();
                    assert_relative_eq!(orig, roundtrip, epsilon = 1e-5);
                }

                // Logarithm properties: ln(xy) = ln(x) + ln(y)
                let y = ScalarF5E3::from(2.0);
                let ln_y = y.ln();
                if ln_y.is_normal() {
                    let product = x * y;
                    if product.is_normal() {
                        let ln_product = product.ln();
                        let sum_logs = ln_x + ln_y;

                        if ln_product.is_normal() && sum_logs.is_normal() {
                            let product_log: f32 = ln_product.into();
                            let sum_val: f32 = sum_logs.into();
                            assert_relative_eq!(product_log, sum_val, epsilon = 1e-5);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_complex_arithmetic_properties(
        z1 in small_circle(),
        z2 in small_circle(),
        z3 in small_circle()
    ) {
        if z1.is_normal() && z2.is_normal() && z3.is_normal() {
            // Complex addition commutativity: z1 + z2 = z2 + z1
            let sum1 = z1 + z2;
            let sum2 = z2 + z1;
            if sum1.is_normal() && sum2.is_normal() {
                let sum1_real: f32 = sum1.r().into();
                let sum1_imag: f32 = sum1.i().into();
                let sum2_real: f32 = sum2.r().into();
                let sum2_imag: f32 = sum2.i().into();

                assert_relative_eq!(sum1_real, sum2_real, epsilon = 1e-6);
                assert_relative_eq!(sum1_imag, sum2_imag, epsilon = 1e-6);
            }

            // Complex multiplication distributivity: z1 * (z2 + z3) = z1 * z2 + z1 * z3
            let sum_z2_z3 = z2 + z3;
            if sum_z2_z3.is_normal() {
                let left = z1 * sum_z2_z3;
                let right = (z1 * z2) + (z1 * z3);

                if left.is_normal() && right.is_normal() {
                    let left_real: f32 = left.r().into();
                    let left_imag: f32 = left.i().into();
                    let right_real: f32 = right.r().into();
                    let right_imag: f32 = right.i().into();

                    assert_relative_eq!(left_real, right_real, epsilon = 1e-4);
                    assert_relative_eq!(left_imag, right_imag, epsilon = 1e-4);
                }
            }

            // Conjugate properties: conj(z1 + z2) = conj(z1) + conj(z2)
            let sum = z1 + z2;
            if sum.is_normal() {
                let conj_sum = sum.conjugate();
                let sum_conj = z1.conjugate() + z2.conjugate();

                if conj_sum.is_normal() && sum_conj.is_normal() {
                    let conj_sum_real: f32 = conj_sum.r().into();
                    let conj_sum_imag: f32 = conj_sum.i().into();
                    let sum_conj_real: f32 = sum_conj.r().into();
                    let sum_conj_imag: f32 = sum_conj.i().into();

                    assert_relative_eq!(conj_sum_real, sum_conj_real, epsilon = 1e-6);
                    assert_relative_eq!(conj_sum_imag, sum_conj_imag, epsilon = 1e-6);
                }
            }

            // Magnitude properties: |z * w| = |z| * |w|
            let product = z1 * z2;
            if product.is_normal() {
                let mag_product = product.magnitude();
                let mag1 = z1.magnitude();
                let mag2 = z2.magnitude();

                if mag_product.is_normal() && mag1.is_normal() && mag2.is_normal() {
                    let product_mag = mag1 * mag2;
                    if product_mag.is_normal() {
                        let mag_prod_val: f32 = mag_product.into();
                        let prod_mag_val: f32 = product_mag.into();
                        assert_relative_eq!(mag_prod_val, prod_mag_val, epsilon = 1e-5);
                    }
                }
            }
        }
    }

    #[test]
    fn test_numerical_stability_properties(
        x in small_finite_scalar(),
        y in small_finite_scalar()
    ) {
        if x.is_normal() && y.is_normal() {
            // Test that small perturbations don't cause large changes (when possible)
            let epsilon = ScalarF5E3::ONE / ScalarF5E3::from(1000000.0); // Approximate epsilon
            let x_plus_eps = x + epsilon;

            if x_plus_eps.is_normal() && x_plus_eps != x {
                // Basic operations should be stable
                let diff1 = (x * y) - (x_plus_eps * y);
                let diff2 = (x + y) - (x_plus_eps + y);

                if diff1.is_normal() && diff2.is_normal() {
                    let diff1_val: f32 = diff1.magnitude().into();
                    let diff2_val: f32 = diff2.magnitude().into();
                    let y_val: f32 = y.magnitude().into();
                    let eps_val: f32 = epsilon.into();

                    // Changes should be proportional to epsilon
                    assert!(diff1_val <= y_val * eps_val * 2.0);
                    assert!(diff2_val <= eps_val * 2.0);
                }
            }
        }
    }

    #[test]
    fn test_escaped_value_consistency(
        a in small_finite_scalar(),
        large_factor in 1e10f32..1e20f32
    ) {
        if a.is_normal() && !a.is_zero() {
            let large_scalar = ScalarF5E3::from(large_factor);
            if large_scalar.is_normal() {
                // Create exploded value
                let exploded = a * large_scalar;

                if exploded.exploded() {
                    // Sign should be preserved
                    if a.is_positive() {
                        assert!(exploded.is_positive());
                    } else if a.is_negative() {
                        assert!(exploded.is_negative());
                    }

                    // Operations with exploded should maintain exploded state (mostly)
                    let exploded_plus_normal = exploded + a;
                    assert!(exploded_plus_normal.exploded());

                    // Division by large number might bring back to normal
                    let normalized = exploded / large_scalar;
                    if normalized.is_normal() {
                        // Should be approximately the original value
                        let orig_val: f32 = a.into();
                        let norm_val: f32 = normalized.into();
                        assert_relative_eq!(orig_val, norm_val, epsilon = 1e-3);
                    }
                }
            }
        }
    }
}
