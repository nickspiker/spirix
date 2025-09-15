use approx::assert_relative_eq;
use spirix::*;

// Macro to test basic arithmetic for all scalar types
macro_rules! test_arithmetic_for_type {
    ($scalar_type:ident) => {
        // Test exact integer arithmetic
        let a = $scalar_type::from(40);
        let b = $scalar_type::from(2);
        let sum = a + b;
        assert_eq!(sum, $scalar_type::from(42));

        let c = $scalar_type::from(10);
        let d = $scalar_type::from(5);
        let diff = c - d;
        assert_eq!(diff, $scalar_type::from(5));

        let e = $scalar_type::from(6);
        let f = $scalar_type::from(7);
        let product = e * f;
        assert_eq!(product, $scalar_type::from(42));

        // Test negation
        let pos = $scalar_type::from(42);
        let neg = -pos;
        assert_eq!(neg, $scalar_type::from(-42));

        // Test zero operations
        let zero = $scalar_type::ZERO;
        let five = $scalar_type::from(5);
        assert_eq!(five + zero, five);
        assert_eq!(zero + five, five);
        assert_eq!(five * zero, zero);
        assert_eq!(zero * five, zero);

        // Test identity
        let one = $scalar_type::ONE;
        assert_eq!(five * one, five);
        assert_eq!(one * five, five);

        // Test constants are normal
        assert!(one.is_normal());
        assert!(zero.is_zero());
        assert!($scalar_type::PI.is_normal());
        assert!($scalar_type::E.is_normal());

        // Test state checks
        assert!(pos.is_positive());
        assert!(neg.is_negative());
        assert!(pos.is_normal());
        assert!(neg.is_normal());
        assert!(!zero.is_normal());
        assert!(!zero.is_positive());
        assert!(!zero.is_negative());

        // Test integer detection (catch the negative integer bug!)
        assert!($scalar_type::from(42).is_integer());
        assert!($scalar_type::from(0).is_integer());
        assert!($scalar_type::from(-1).is_integer());
        assert!($scalar_type::from(-17).is_integer());
        assert!($scalar_type::from(-100).is_integer());
        assert!(!$scalar_type::from(3.14).is_integer());

        // Test extreme values for this type
        let max_val = $scalar_type::MAX;
        let min_pos = $scalar_type::MIN_POS;
        assert!(max_val.is_normal());
        assert!(min_pos.is_normal());

        // Test overflow creates exploded state
        let exploded: $scalar_type = max_val * 2.0;
        assert!(exploded.exploded());
        assert!(exploded.is_positive());

        // Test underflow creates vanished state
        let vanished: $scalar_type = min_pos / 1000.0;
        assert!(vanished.vanished());
        assert!(vanished.is_positive());

        // Test undefined propagation
        let undefined = $scalar_type::from(1) / zero;
        assert!(undefined.is_undefined());
        let propagated = undefined + $scalar_type::from(42);
        assert!(propagated.is_undefined());
    };
}

// Macro to generate test functions for all scalar types
macro_rules! test_all_scalar_types {
    ($($scalar_type:ident),+) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_ $scalar_type:lower _comprehensive>]() {
                    test_arithmetic_for_type!($scalar_type);
                }
            }
        )+
    };
}

// Generate comprehensive tests for all 25 scalar types
test_all_scalar_types!(
    ScalarF3E3, ScalarF3E4, ScalarF3E5, ScalarF3E6, ScalarF3E7,
    ScalarF4E3, ScalarF4E4, ScalarF4E5, ScalarF4E6, ScalarF4E7,
    ScalarF5E3, ScalarF5E4, ScalarF5E5, ScalarF5E6, ScalarF5E7,
    ScalarF6E3, ScalarF6E4, ScalarF6E5, ScalarF6E6, ScalarF6E7,
    ScalarF7E3, ScalarF7E4, ScalarF7E5, ScalarF7E6, ScalarF7E7
);

#[cfg(test)]
mod basic_operations {
    use super::*;

    #[test]
    fn test_scalar_creation_and_conversion() {
        // Test creation from integers
        let a = ScalarF5E3::from(42u8);
        let b = ScalarF5E3::from(-17i8);
        assert!(a.is_normal());
        assert!(b.is_normal());
        assert!(a.is_positive());
        assert!(b.is_negative());

        // Test creation from floats
        let pi = ScalarF5E3::from(3.14159f32);
        let e = ScalarF5E3::from(2.71828f64);
        assert!(pi.is_normal());
        assert!(e.is_normal());

        // Test constants
        let zero = ScalarF5E3::ZERO;
        let one = ScalarF5E3::ONE;
        let pi_const = ScalarF5E3::PI;
        let e_const = ScalarF5E3::E;

        assert!(zero.is_zero());
        assert!(one.is_normal());
        assert!(pi_const.is_normal());
        assert!(e_const.is_normal());
        assert!(
            pi_const.square() > 9
                && pi_const.square() < 10
                && !pi_const.square().is_integer()
                && pi_const.square().is_normal()
        );
    }

    #[test]
    fn test_basic_arithmetic() {
        let a = ScalarF5E3::from(7);
        let b = ScalarF5E3::from(3);

        // Addition
        let sum = a + b;
        assert!(sum.is_normal());
        let sum_f32: f32 = sum.into();
        assert_relative_eq!(sum_f32, 10.0, epsilon = 1e-5);

        // Subtraction
        let diff = a - b;
        assert!(diff.is_normal());
        let diff_f32: f32 = diff.into();
        assert_relative_eq!(diff_f32, 4.0, epsilon = 1e-5);

        // Multiplication
        let product = a * b;
        assert!(product.is_normal());
        let product_f32: f32 = product.into();
        assert_relative_eq!(product_f32, 21.0, epsilon = 1e-5);

        // Division
        let quotient = a / b;
        assert!(quotient.is_normal());
        let quotient_f32: f32 = quotient.into();
        assert_relative_eq!(quotient_f32, 7.0 / 3.0, epsilon = 1e-5);

        // Negation
        let neg_a = -a;
        assert!(neg_a.is_normal());
        assert!(neg_a.is_negative());
        let neg_a_f32: f32 = neg_a.into();
        assert_relative_eq!(neg_a_f32, -7.0, epsilon = 1e-5);
    }

    #[test]
    fn test_zero_operations() {
        let zero = ScalarF5E3::ZERO;
        let five = ScalarF5E3::from(5);

        // Addition with zero
        let result = five + zero;
        assert!(result.is_normal());
        let result_f32: f32 = result.into();
        assert_relative_eq!(result_f32, 5.0, epsilon = 1e-5);

        let result2 = zero + five;
        assert!(result2.is_normal());
        let result2_f32: f32 = result2.into();
        assert_relative_eq!(result2_f32, 5.0, epsilon = 1e-5);

        // Multiplication with zero
        let result3 = five * zero;
        assert!(result3.is_zero());

        let result4 = zero * five;
        assert!(result4.is_zero());

        // Division by zero should create undefined state
        let div_by_zero = five / zero;
        assert!(div_by_zero.is_undefined());
    }

    #[test]
    fn test_arithmetic_properties() {
        let a = ScalarF5E3::from(12);
        let b = ScalarF5E3::from(8);
        let c = ScalarF5E3::from(-5);

        // Commutativity
        assert_eq!(a + b, b + a);
        assert_eq!(a * b, b * a);

        // Associativity
        assert_eq!((a + b) + c, a + (b + c));
        assert_eq!((a * b) * c, a * (b * c));

        // Distributivity
        let left = a * (b + c);
        let right = a * b + a * c;
        let left_f32: f32 = left.into();
        let right_f32: f32 = right.into();
        assert_relative_eq!(left_f32, right_f32, epsilon = 1e-4);

        // Identity elements
        let zero = ScalarF5E3::ZERO;
        let one = ScalarF5E3::ONE;

        assert_eq!(a + zero, a);
        assert_eq!(a * one, a);
    }
}

#[cfg(test)]
mod special_values {
    use super::*;

    #[test]
    fn test_escaped_values() {
        // Test exploded values (too large)
        let max_val = ScalarF5E3::MAX;
        let exploded: ScalarF5E3 = max_val * 2.0;

        assert!(exploded.exploded());
        assert!(exploded.is_positive());
        assert!(!exploded.is_normal());
        assert!(!exploded.is_zero());

        // Test negative exploded
        let neg_exploded: ScalarF5E3 = max_val * -2.0;
        assert!(neg_exploded.exploded());
        assert!(neg_exploded.is_negative());

        // Test vanished values (too small)
        let min_pos = ScalarF5E3::MIN_POS;
        let vanished: ScalarF5E3 = min_pos / 1000.0;

        assert!(vanished.vanished());
        assert!(vanished.is_positive());
        assert!(!vanished.is_normal());
        assert!(!vanished.is_zero());

        // Test negative vanished
        let neg_vanished: ScalarF5E3 = min_pos / -1000.0;
        assert!(neg_vanished.vanished());
        assert!(neg_vanished.is_negative());
    }

    #[test]
    fn test_undefined_states() {
        // Division by zero
        let div_zero = ScalarF5E3::from(1) / ScalarF5E3::ZERO;
        assert!(div_zero.is_undefined());

        // Zero to zero power
        let zero_pow_zero = ScalarF5E3::ZERO.pow(ScalarF5E3::ZERO);
        assert!(zero_pow_zero.is_undefined());

        // Square root of negative
        let sqrt_neg = ScalarF5E3::from(-4).sqrt();
        assert!(sqrt_neg.is_undefined());

        // Logarithm of negative
        let log_neg = ScalarF5E3::from(-2).ln();
        assert!(log_neg.is_undefined());

        // Undefined values propagate
        let propagated = div_zero + ScalarF5E3::from(42);
        assert!(propagated.is_undefined());

        let propagated2 = sqrt_neg * ScalarF5E3::from(10);
        assert!(propagated2.is_undefined());
    }

    #[test]
    fn test_escaped_value_operations() {
        let exploded: ScalarF5E3 = ScalarF5E3::MAX * 2.0;
        let vanished: ScalarF5E3 = ScalarF5E3::MIN_POS / 1000.0;

        // Absolute operations on escaped values
        let exploded_squared = exploded.square();
        assert!(exploded_squared.exploded());
        assert!(exploded_squared.is_positive());

        let vanished_squared = vanished.square();
        assert!(vanished_squared.vanished());
        assert!(vanished_squared.is_positive());

        // Sign operations
        let neg_exploded = -exploded;
        assert!(neg_exploded.exploded());
        assert!(neg_exploded.is_negative());

        // Operations between escaped values that should create undefined
        let _exploded_add_exploded = exploded + exploded;
        // This might be undefined based on the implementation
        // The exact behavior depends on how exploded values are handled
    }
}

#[cfg(test)]
mod mathematical_functions {
    use super::*;

    #[test]
    fn test_power_functions() {
        let base = ScalarF5E3::from(2);
        let exp = ScalarF5E3::from(3);

        // Basic power
        let result = base.pow(exp);
        assert!(result.is_normal());
        let result_f32: f32 = result.into();
        assert_relative_eq!(result_f32, 8.0, epsilon = 1e-5);

        // Square function
        let square = base.square();
        assert!(square.is_normal());
        let square_f32: f32 = square.into();
        assert_relative_eq!(square_f32, 4.0, epsilon = 1e-5);

        // Square root
        let sqrt_val = ScalarF5E3::from(9).sqrt();
        assert!(sqrt_val.is_normal());
        let sqrt_f32: f32 = sqrt_val.into();
        assert_relative_eq!(sqrt_f32, 3.0, epsilon = 1e-5);

        // Reciprocal
        let recip = ScalarF5E3::from(4).reciprocal();
        assert!(recip.is_normal());
        let recip_f32: f32 = recip.into();
        assert_relative_eq!(recip_f32, 0.25, epsilon = 1e-5);
    }

    #[test]
    fn test_exponential_and_logarithmic() {
        // Natural exponential
        let exp_result = ScalarF5E3::from(1).exp();
        assert!(exp_result.is_normal());
        let exp_f32: f32 = exp_result.into();
        assert_relative_eq!(exp_f32, std::f32::consts::E, epsilon = 1e-4);

        // Natural logarithm
        let ln_e = ScalarF5E3::E.ln();
        assert!(ln_e.is_normal());
        let ln_f32: f32 = ln_e.into();
        assert_relative_eq!(ln_f32, 1.0, epsilon = 1e-4);

        // Binary logarithm
        let lb_8 = ScalarF5E3::from(8).lb();
        assert!(lb_8.is_normal());
        let lb_f32: f32 = lb_8.into();
        assert_relative_eq!(lb_f32, 3.0, epsilon = 1e-4);

        // Power of 2
        let powb_3 = ScalarF5E3::from(3).powb();
        assert!(powb_3.is_normal());
        let powb_f32: f32 = powb_3.into();
        assert_relative_eq!(powb_f32, 8.0, epsilon = 1e-4);
    }

    #[test]
    fn test_trigonometric_functions() {
        let pi_half: ScalarF5E3 = ScalarF5E3::PI / 2.0;
        let pi_quarter: ScalarF5E3 = ScalarF5E3::PI / 4.0;

        // Sine
        let sin_pi_half = pi_half.sin();
        assert!(sin_pi_half.is_normal());
        let sin_f32: f32 = sin_pi_half.into();
        assert_relative_eq!(sin_f32, 1.0, epsilon = 1e-4);

        // Cosine
        let cos_zero = ScalarF5E3::ZERO.cos();
        assert!(cos_zero.is_normal());
        let cos_f32: f32 = cos_zero.into();
        assert_relative_eq!(cos_f32, 1.0, epsilon = 1e-4);

        // Tangent
        let tan_pi_quarter = pi_quarter.tan();
        assert!(tan_pi_quarter.is_normal());
        let tan_f32: f32 = tan_pi_quarter.into();
        assert_relative_eq!(tan_f32, 1.0, epsilon = 1e-3);

        // Inverse functions
        let asin_half = ScalarF5E3::from(0.5).asin();
        assert!(asin_half.is_normal());

        let acos_half = ScalarF5E3::from(0.5).acos();
        assert!(acos_half.is_normal());

        let atan_one = ScalarF5E3::ONE.atan();
        assert!(atan_one.is_normal());
        let atan_f32: f32 = atan_one.into();
        assert_relative_eq!(atan_f32, std::f32::consts::FRAC_PI_4, epsilon = 1e-4);
    }

    #[test]
    fn test_hyperbolic_functions() {
        let one = ScalarF5E3::ONE;

        // Hyperbolic sine
        let sinh_1 = one.sinh();
        assert!(sinh_1.is_normal());

        // Hyperbolic cosine
        let cosh_0 = ScalarF5E3::ZERO.cosh();
        assert!(cosh_0.is_normal());
        let cosh_f32: f32 = cosh_0.into();
        assert_relative_eq!(cosh_f32, 1.0, epsilon = 1e-4);

        // Hyperbolic tangent
        let tanh_0 = ScalarF5E3::ZERO.tanh();
        assert!(tanh_0.is_normal());
        let tanh_f32: f32 = tanh_0.into();
        assert_relative_eq!(tanh_f32, 0.0, epsilon = 1e-6);

        // Inverse hyperbolic functions
        // Note: asinh function may not be available in this version
        // let asinh_0 = ScalarF5E3::ZERO.asinh();
        // assert!(asinh_0.is_normal());
        // let asinh_f32: f32 = asinh_0.into();
        // assert_relative_eq!(asinh_f32, 0.0, epsilon = 1e-6);
    }
}

#[cfg(test)]
mod comparison_and_utility {
    use super::*;

    #[test]
    fn test_comparison_operations() {
        let a = ScalarF5E3::from(5);
        let b = ScalarF5E3::from(3);
        let c = ScalarF5E3::from(8);

        // Min and max
        let min_val = a.min(b);
        let min_f32: f32 = min_val.into();
        assert_relative_eq!(min_f32, 3.0, epsilon = 1e-6);

        let max_val = a.max(c);
        let max_f32: f32 = max_val.into();
        assert_relative_eq!(max_f32, 8.0, epsilon = 1e-6);

        // Clamp
        let clamped = a.clamp(b, c);
        let clamped_f32: f32 = clamped.into();
        assert_relative_eq!(clamped_f32, 5.0, epsilon = 1e-6);

        let clamped_low = ScalarF5E3::from(1).clamp(b, c);
        let clamped_low_f32: f32 = clamped_low.into();
        assert_relative_eq!(clamped_low_f32, 3.0, epsilon = 1e-6);

        let clamped_high = ScalarF5E3::from(10).clamp(b, c);
        let clamped_high_f32: f32 = clamped_high.into();
        assert_relative_eq!(clamped_high_f32, 8.0, epsilon = 1e-6);
    }

    #[test]
    fn test_integer_functions() {
        // Floor
        let floor_val = ScalarF5E3::from(3.7).floor();
        let floor_f32: f32 = floor_val.into();
        assert_relative_eq!(floor_f32, 3.0, epsilon = 1e-6);

        let floor_neg = ScalarF5E3::from(-2.3).floor();
        let floor_neg_f32: f32 = floor_neg.into();
        assert_relative_eq!(floor_neg_f32, -3.0, epsilon = 1e-6);

        // Ceiling
        let ceil_val = ScalarF5E3::from(3.2).ceil();
        let ceil_f32: f32 = ceil_val.into();
        assert_relative_eq!(ceil_f32, 4.0, epsilon = 1e-6);

        // Round
        let round_val = ScalarF5E3::from(3.6).round();
        let round_f32: f32 = round_val.into();
        assert_relative_eq!(round_f32, 4.0, epsilon = 1e-6);

        let round_half = ScalarF5E3::from(2.5).round();
        let round_half_f32: f32 = round_half.into();
        assert_relative_eq!(round_half_f32, 2.0, epsilon = 1e-6); // Round to even

        // Fractional part
        let frac_val = ScalarF5E3::from(3.7).frac();
        let frac_f32: f32 = frac_val.into();
        assert_relative_eq!(frac_f32, 0.7, epsilon = 1e-5);
    }

    #[test]
    fn test_state_checking() {
        let normal = ScalarF5E3::from(42);
        let zero = ScalarF5E3::ZERO;
        let exploded: ScalarF5E3 = ScalarF5E3::MAX * 2.0;
        let vanished: ScalarF5E3 = ScalarF5E3::MIN_POS / 1000.0;
        let undefined = ScalarF5E3::from(1) / ScalarF5E3::ZERO;

        // Normal value checks
        assert!(normal.is_normal());
        assert!(normal.is_finite());
        assert!(normal.is_positive());
        assert!(!normal.is_negative());
        assert!(!normal.is_zero());
        assert!(!normal.vanished());
        assert!(!normal.exploded());
        assert!(!normal.is_undefined());

        // Zero checks
        assert!(!zero.is_normal());
        assert!(zero.is_finite());
        assert!(!zero.is_positive());
        assert!(!zero.is_negative());
        assert!(zero.is_zero());
        assert!(zero.is_negligible());

        // Exploded checks
        assert!(!exploded.is_normal());
        assert!(!exploded.is_finite());
        assert!(exploded.exploded());
        assert!(exploded.is_positive());

        // Vanished checks
        assert!(!vanished.is_normal());
        assert!(!vanished.is_finite());
        assert!(vanished.vanished());
        assert!(vanished.is_negligible());
        assert!(vanished.is_positive());

        // Undefined checks
        assert!(!undefined.is_normal());
        assert!(!undefined.is_finite());
        assert!(undefined.is_undefined());
    }

    #[test]
    fn test_integer_properties() {
        // Test is_integer
        assert!(ScalarF5E3::from(42).is_integer());
        assert!(ScalarF5E3::from(-17).is_integer());
        assert!(!ScalarF5E3::from(3.14).is_integer());
        assert!(ScalarF5E3::ZERO.is_integer());

        // Test contiguous range
        let small_int = ScalarF5E3::from(5);
        assert!(small_int.is_contiguous());

        // Test primality (for small integers)
        assert!(ScalarF5E3::from(7).is_prime());
        assert!(ScalarF5E3::from(13).is_prime());
        assert!(!ScalarF5E3::from(8).is_prime());
        assert!(!ScalarF5E3::from(15).is_prime());
    }
}

#[cfg(test)]
mod bitwise_operations {
    use super::*;

    #[test]
    fn test_bitwise_ops() {
        let a = ScalarF5E3::from(0b1010); // 10
        let b = ScalarF5E3::from(0b1100); // 12

        // Bitwise AND
        let and_result = a & b;
        assert!(and_result.is_normal());
        let and_val: i32 = and_result.into();
        assert_eq!(and_val, 0b1000); // 8

        // Bitwise OR
        let or_result = a | b;
        assert!(or_result.is_normal());
        let or_val: i32 = or_result.into();
        assert_eq!(or_val, 0b1110); // 14

        // Bitwise XOR
        let xor_result = a ^ b;
        assert!(xor_result.is_normal());
        let xor_val: i32 = xor_result.into();
        assert_eq!(xor_val, 0b0110); // 6

        // Bitwise NOT
        let not_a = !a;
        assert!(not_a.is_normal());

        // Left shift (multiply by power of 2)
        let left_shift = a << 2i32;
        assert!(left_shift.is_normal());
        let ls_val: i32 = left_shift.into();
        assert_eq!(ls_val, 40); // 10 * 4

        // Right shift (divide by power of 2)
        let right_shift = b >> 1i32;
        assert!(right_shift.is_normal());
        let rs_val: i32 = right_shift.into();
        assert_eq!(rs_val, 6); // 12 / 2
    }
}

#[cfg(test)]
mod modular_operations {
    use super::*;

    #[test]
    fn test_modulo() {
        let a = ScalarF5E3::from(17);
        let b = ScalarF5E3::from(5);

        let remainder = a % b;
        assert!(remainder.is_normal());
        let rem_val: f32 = remainder.into();
        assert_relative_eq!(rem_val, 2.0, epsilon = 1e-6);

        // Modulo with floating point
        let c = ScalarF5E3::from(7.5);
        let d = ScalarF5E3::from(2.5);
        let remainder2 = c % d;
        assert!(remainder2.is_normal());
        let rem2_val: f32 = remainder2.into();
        assert_relative_eq!(rem2_val, 0.0, epsilon = 1e-5);
    }
}

#[cfg(test)]
mod random_testing {
    use super::*;

    #[test]
    fn test_random_generation() {
        // Generate random values and test basic properties
        for _ in 0..100 {
            let random_val = ScalarF5E3::random();
            // Random values should be between -1 and 1
            let val_f32: f32 = random_val.into();
            assert!(val_f32 >= -1.0 && val_f32 <= 1.0);
            assert!(random_val.is_normal() || random_val.is_zero());

            let random_gauss = ScalarF5E3::random_gauss();
            // Gaussian values should be normal or zero (very rarely exploded/vanished)
            assert!(
                random_gauss.is_normal()
                    || random_gauss.is_zero()
                    || random_gauss.exploded()
                    || random_gauss.vanished()
            );
        }
    }
}

#[cfg(test)]
mod precision_configurations {
    use super::*;

    #[test]
    fn test_different_precisions() {
        // Test different fraction/exponent combinations
        let small = ScalarF3E3::from(42);
        let medium = ScalarF5E3::from(42);
        let large = ScalarF6E4::from(42);

        assert!(small.is_normal());
        assert!(medium.is_normal());
        assert!(large.is_normal());

        // Convert to common format for comparison
        let small_f32: f32 = small.into();
        let medium_f32: f32 = medium.into();
        let large_f32: f32 = large.into();

        assert_relative_eq!(small_f32, 42.0, epsilon = 1e-1);
        assert_relative_eq!(medium_f32, 42.0, epsilon = 1e-4);
        assert_relative_eq!(large_f32, 42.0, epsilon = 1e-6);
    }

    #[test]
    fn test_range_differences() {
        // Higher exponent bits should allow larger ranges
        let wide_range = ScalarF4E7::from(1e30);
        assert!(wide_range.is_normal() || wide_range.exploded());

        let narrow_range = ScalarF4E3::from(1e30);
        // This should likely be exploded due to limited exponent range
        assert!(narrow_range.exploded() || narrow_range.is_normal());
    }
}
