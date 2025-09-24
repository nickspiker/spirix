use approx::assert_relative_eq;
use spirix::*;

/// Test mathematical function accuracy, identities, and special cases
/// Verify transcendental functions, power operations, and mathematical relationships

#[test]
fn test_trigonometric_identities() {
    let test_angles = [
        0.0,
        std::f32::consts::PI / 6.0,
        std::f32::consts::PI / 4.0,
        std::f32::consts::PI / 3.0,
        std::f32::consts::PI / 2.0,
        std::f32::consts::PI,
        3.0 * std::f32::consts::PI / 2.0,
        2.0 * std::f32::consts::PI,
        -std::f32::consts::PI / 4.0,
    ];

    for &angle in &test_angles {
        let theta = ScalarF5E3::from(angle);

        if theta.is_normal() {
            let sin_val = theta.sin();
            let cos_val = theta.cos();
            let tan_val = theta.tan();

            if sin_val.is_normal() && cos_val.is_normal() {
                // Test Pythagorean identity: sin²θ + cos²θ = 1
                let sin_squared = sin_val * sin_val;
                let cos_squared = cos_val * cos_val;
                let sum = sin_squared + cos_squared;

                if sum.is_normal() {
                    let sum_val: f32 = sum.into();
                    assert_relative_eq!(sum_val, 1.0, epsilon = 1e-6);
                }

                // Test tan identity: tan θ = sin θ / cos θ
                if cos_val != ScalarF5E3::ZERO && tan_val.is_normal() {
                    let computed_tan = sin_val / cos_val;
                    if computed_tan.is_normal() {
                        let tan_result: f32 = tan_val.into();
                        let computed_result: f32 = computed_tan.into();
                        assert_relative_eq!(tan_result, computed_result, epsilon = 1e-5);
                    }
                }
            }

            // Test angle addition formulas
            let half_angle = ScalarF5E3::from(angle / 2.0);
            if half_angle.is_normal() {
                let sin_half = half_angle.sin();
                let cos_half = half_angle.cos();

                if sin_half.is_normal() && cos_half.is_normal() {
                    // sin(2α) = 2sin(α)cos(α)
                    let sin_double_computed = ScalarF5E3::from(2.0) * sin_half * cos_half;
                    if sin_double_computed.is_normal() && sin_val.is_normal() {
                        let expected: f32 = sin_val.into();
                        let computed: f32 = sin_double_computed.into();
                        assert_relative_eq!(expected, computed, epsilon = 1e-4);
                    }
                }
            }
        }
    }
}

#[test]
fn test_exponential_logarithm_identities() {
    let test_values = [0.1, 0.5, 1.0, 2.0, std::f32::consts::E, 10.0, 100.0];

    for &val in &test_values {
        let x = ScalarF5E3::from(val);

        if x.is_normal() && x.is_positive() {
            let ln_x = x.ln();
            let exp_ln_x = ln_x.exp();

            // Test exp(ln(x)) = x
            if ln_x.is_normal() && exp_ln_x.is_normal() {
                let original: f32 = x.into();
                let reconstructed: f32 = exp_ln_x.into();
                assert_relative_eq!(original, reconstructed, epsilon = 1e-5);
            }

            // Test ln(e^x) = x for reasonable values
            if val < 10.0 {
                // Avoid overflow
                let exp_x = x.exp();
                if exp_x.is_normal() {
                    let ln_exp_x = exp_x.ln();
                    if ln_exp_x.is_normal() {
                        let original: f32 = x.into();
                        let reconstructed: f32 = ln_exp_x.into();
                        assert_relative_eq!(original, reconstructed, epsilon = 1e-5);
                    }
                }
            }

            // Test logarithm properties: ln(ab) = ln(a) + ln(b)
            if val < 10.0 {
                let y = ScalarF5E3::from(val * 1.5);
                if y.is_normal() && y.is_positive() {
                    let ln_y = y.ln();
                    let product = x * y;

                    if ln_x.is_normal() && ln_y.is_normal() && product.is_normal() {
                        let ln_product = product.ln();
                        let sum_logs = ln_x + ln_y;

                        if ln_product.is_normal() && sum_logs.is_normal() {
                            let product_log: f32 = ln_product.into();
                            let sum_result: f32 = sum_logs.into();
                            assert_relative_eq!(product_log, sum_result, epsilon = 1e-4);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_power_function_properties() {
    let bases = [0.5, 1.0, 2.0, std::f32::consts::E, 10.0];
    let exponents = [0.0, 0.5, 1.0, 2.0, -1.0, -0.5];

    for &base_val in &bases {
        for &exp_val in &exponents {
            let base = ScalarF5E3::from(base_val);
            let exponent = ScalarF5E3::from(exp_val);

            if base.is_normal() && base.is_positive() && exponent.is_normal() {
                let result = base.pow(exponent);

                if result.is_normal() {
                    // Test a^0 = 1
                    if exp_val == 0.0 {
                        let result_val: f32 = result.into();
                        assert_relative_eq!(result_val, 1.0, epsilon = 1e-6);
                    }

                    // Test a^1 = a
                    if exp_val == 1.0 {
                        let base_val: f32 = base.into();
                        let result_val: f32 = result.into();
                        assert_relative_eq!(base_val, result_val, epsilon = 1e-6);
                    }

                    // Test (a^m)^n = a^(mn)
                    if base_val < 5.0 && exp_val < 2.0 && exp_val > 0.0 {
                        let intermediate = base.pow(exponent);
                        if intermediate.is_normal() {
                            let second_exp = ScalarF5E3::from(2.0);
                            let double_power = intermediate.pow(second_exp);
                            let combined_exp = exponent * second_exp;
                            let direct_power = base.pow(combined_exp);

                            if double_power.is_normal() && direct_power.is_normal() {
                                let double_val: f32 = double_power.into();
                                let direct_val: f32 = direct_power.into();
                                assert_relative_eq!(double_val, direct_val, epsilon = 1e-4);
                            }
                        }
                    }

                    // Test sqrt(x) = x^(1/2)
                    if exp_val == 0.5 {
                        let sqrt_result = base.sqrt();
                        if sqrt_result.is_normal() {
                            let pow_val: f32 = result.into();
                            let sqrt_val: f32 = sqrt_result.into();
                            assert_relative_eq!(pow_val, sqrt_val, epsilon = 1e-5);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_inverse_functions() {
    let test_values = [-0.99, -0.5, 0.0, 0.5, 0.99];

    for &val in &test_values {
        let x = ScalarF5E3::from(val);

        if x.is_normal() {
            // Test arcsin(sin(x)) = x (for valid domain)
            if val.abs() <= std::f32::consts::PI / 2.0 {
                let sin_x = x.sin();
                if sin_x.is_normal() {
                    let arcsin_sin_x = sin_x.asin();
                    if arcsin_sin_x.is_normal() {
                        let original: f32 = x.into();
                        let reconstructed: f32 = arcsin_sin_x.into();
                        assert_relative_eq!(original, reconstructed, epsilon = 1e-5);
                    }
                }
            }

            // Test arccos(cos(x)) = |x| (for x in [0, π])
            if val >= 0.0 && val <= std::f32::consts::PI {
                let cos_x = x.cos();
                if cos_x.is_normal() {
                    let arccos_cos_x = cos_x.acos();
                    if arccos_cos_x.is_normal() {
                        let original: f32 = x.into();
                        let reconstructed: f32 = arccos_cos_x.into();
                        assert_relative_eq!(original, reconstructed, epsilon = 1e-5);
                    }
                }
            }

            // Test arctan(tan(x)) = x (for valid domain)
            if val.abs() < std::f32::consts::PI / 2.0 - 0.1 {
                let tan_x = x.tan();
                if tan_x.is_normal() {
                    let arctan_tan_x = tan_x.atan();
                    if arctan_tan_x.is_normal() {
                        let original: f32 = x.into();
                        let reconstructed: f32 = arctan_tan_x.into();
                        assert_relative_eq!(original, reconstructed, epsilon = 1e-5);
                    }
                }
            }
        }
    }
}

#[test]
fn test_hyperbolic_functions() {
    let test_values = [-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0];

    for &val in &test_values {
        let x = ScalarF5E3::from(val);

        if x.is_normal() {
            let sinh_x = x.sinh();
            let cosh_x = x.cosh();
            let tanh_x = x.tanh();

            if sinh_x.is_normal() && cosh_x.is_normal() {
                // Test hyperbolic identity: cosh²x - sinh²x = 1
                let cosh_squared = cosh_x * cosh_x;
                let sinh_squared = sinh_x * sinh_x;
                let difference = cosh_squared - sinh_squared;

                if difference.is_normal() {
                    let diff_val: f32 = difference.into();
                    assert_relative_eq!(diff_val, 1.0, epsilon = 1e-6);
                }

                // Test tanh x = sinh x / cosh x
                if cosh_x != ScalarF5E3::ZERO && tanh_x.is_normal() {
                    let computed_tanh = sinh_x / cosh_x;
                    if computed_tanh.is_normal() {
                        let tanh_val: f32 = tanh_x.into();
                        let computed_val: f32 = computed_tanh.into();
                        assert_relative_eq!(tanh_val, computed_val, epsilon = 1e-5);
                    }
                }
            }

            // Test relationship with exponential
            let exp_x = x.exp();
            let exp_neg_x = (-x).exp();

            if exp_x.is_normal() && exp_neg_x.is_normal() {
                // sinh x = (e^x - e^(-x))/2
                let computed_sinh = (exp_x - exp_neg_x) / ScalarF5E3::from(2.0);
                if computed_sinh.is_normal() && sinh_x.is_normal() {
                    let sinh_val: f32 = sinh_x.into();
                    let computed_val: f32 = computed_sinh.into();
                    assert_relative_eq!(sinh_val, computed_val, epsilon = 1e-5);
                }

                // cosh x = (e^x + e^(-x))/2
                let computed_cosh = (exp_x + exp_neg_x) / ScalarF5E3::from(2.0);
                if computed_cosh.is_normal() && cosh_x.is_normal() {
                    let cosh_val: f32 = cosh_x.into();
                    let computed_val: f32 = computed_cosh.into();
                    assert_relative_eq!(cosh_val, computed_val, epsilon = 1e-5);
                }
            }
        }
    }
}

#[test]
fn test_complex_function_identities() {
    // Test Euler's formula: e^(iθ) = cos(θ) + i*sin(θ)
    let angles = [
        0.0,
        std::f32::consts::PI / 4.0,
        std::f32::consts::PI / 2.0,
        std::f32::consts::PI,
        3.0 * std::f32::consts::PI / 2.0,
    ];

    for &angle in &angles {
        let theta = ScalarF5E3::from(angle);

        if theta.is_normal() {
            // Create i*θ
            let i_theta = CircleF5E3::new(ScalarF5E3::ZERO, theta);

            // Calculate e^(i*θ)
            let euler_result = i_theta.exp();

            // Calculate cos(θ) + i*sin(θ)
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();
            let trig_result = CircleF5E3::new(cos_theta, sin_theta);

            if euler_result.is_normal()
                && trig_result.is_normal()
                && cos_theta.is_normal()
                && sin_theta.is_normal()
            {
                let euler_real: f32 = euler_result.real().into();
                let euler_imag: f32 = euler_result.imaginary().into();
                let trig_real: f32 = trig_result.real().into();
                let trig_imag: f32 = trig_result.imaginary().into();

                assert_relative_eq!(euler_real, trig_real, epsilon = 1e-5);
                assert_relative_eq!(euler_imag, trig_imag, epsilon = 1e-5);
            }
        }
    }
}

#[test]
fn test_de_moivre_theorem() {
    // Test De Moivre's theorem: (cos θ + i sin θ)^n = cos(nθ) + i sin(nθ)
    let angles = [
        std::f32::consts::PI / 6.0,
        std::f32::consts::PI / 4.0,
        std::f32::consts::PI / 3.0,
    ];
    let powers = [2.0, 3.0, 4.0];

    for &angle in &angles {
        for &power in &powers {
            let theta = ScalarF5E3::from(angle);
            let n = ScalarF5E3::from(power);

            if theta.is_normal() && n.is_normal() {
                let cos_theta = theta.cos();
                let sin_theta = theta.sin();

                if cos_theta.is_normal() && sin_theta.is_normal() {
                    // Left side: (cos θ + i sin θ)^n
                    let complex_unit = CircleF5E3::new(cos_theta, sin_theta);
                    let left_side = complex_unit.pow(n);

                    // Right side: cos(nθ) + i sin(nθ)
                    let n_theta = n * theta;
                    if n_theta.is_normal() {
                        let cos_n_theta = n_theta.cos();
                        let sin_n_theta = n_theta.sin();

                        if cos_n_theta.is_normal() && sin_n_theta.is_normal() {
                            let right_side = CircleF5E3::new(cos_n_theta, sin_n_theta);

                            if left_side.is_normal() && right_side.is_normal() {
                                let left_real: f32 = left_side.real().into();
                                let left_imag: f32 = left_side.imaginary().into();
                                let right_real: f32 = right_side.real().into();
                                let right_imag: f32 = right_side.imaginary().into();

                                assert_relative_eq!(left_real, right_real, epsilon = 1e-4);
                                assert_relative_eq!(left_imag, right_imag, epsilon = 1e-4);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_special_function_values() {
    // Test known special values
    let tolerance = 1e-6;

    // sin(π/2) = 1
    let sin_pi_2 = (ScalarF5E3::PI / ScalarF5E3::from(2.0)).sin();
    if sin_pi_2.is_normal() {
        let val: f32 = sin_pi_2.into();
        assert_relative_eq!(val, 1.0, epsilon = tolerance);
    }

    // cos(π) = -1
    let cos_pi = ScalarF5E3::PI.cos();
    if cos_pi.is_normal() {
        let val: f32 = cos_pi.into();
        assert_relative_eq!(val, -1.0, epsilon = tolerance);
    }

    // ln(e) = 1
    let ln_e = ScalarF5E3::E.ln();
    if ln_e.is_normal() {
        let val: f32 = ln_e.into();
        assert_relative_eq!(val, 1.0, epsilon = tolerance);
    }

    // e^0 = 1
    let exp_0 = ScalarF5E3::ZERO.exp();
    if exp_0.is_normal() {
        let val: f32 = exp_0.into();
        assert_relative_eq!(val, 1.0, epsilon = tolerance);
    }

    // sqrt(4) = 2
    let sqrt_4 = ScalarF5E3::from(4.0).sqrt();
    if sqrt_4.is_normal() {
        let val: f32 = sqrt_4.into();
        assert_relative_eq!(val, 2.0, epsilon = tolerance);
    }

    // 2^3 = 8
    let two_cubed = ScalarF5E3::from(2.0).pow(ScalarF5E3::from(3.0));
    if two_cubed.is_normal() {
        let val: f32 = two_cubed.into();
        assert_relative_eq!(val, 8.0, epsilon = tolerance);
    }
}
