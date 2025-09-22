use spirix::*;

// Macro to test basic arithmetic for all scalar types
macro_rules! test_arithmetic_for_type {
    ($scalar_type:ident) => {
        // Test exact integer arithmetic with mixed types
        let a = $scalar_type::from(40u8);
        let b = 2i8;
        let sum = a + b;
        assert!(sum == 42u16);

        let c = 12i16;
        let d = $scalar_type::from(5i32);
        let diff = c - d;
        assert!(diff == 7u64);

        let e = $scalar_type::from(6i64);
        let f = 7i128;
        let product = e * f;
        assert!(product == 42u128);

        // Additional mixed type tests
        let g = $scalar_type::from(15usize);
        let h = 3isize;
        let quotient = g / h;
        assert!(quotient == 5f32);

        let i = 8.5f64;
        let j = $scalar_type::from(2u32);
        let mixed_result = i * j;
        assert!(mixed_result == 17.);

        // Test negation
        let pos = $scalar_type::from(42f32);
        let neg = -pos;
        assert!(neg == -42f64);

        // Test zero operations with mixed types
        let zero = $scalar_type::ZERO;
        assert!(5u32 + zero == 5usize);
        assert!(zero + 7isize == 7.);
        assert!(3i16 * zero == zero);
        assert!(zero * 8u64 == zero);

        // Test identity with mixed types
        let one = $scalar_type::ONE;
        assert!(5i32 * one == 5f32);
        assert!(one * 9u16 == 9f64);

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

        // Test primality - comprehensive prime checking
        assert!($scalar_type::from(2).is_prime());
        assert!($scalar_type::from(3).is_prime());
        assert!($scalar_type::from(5).is_prime());
        assert!($scalar_type::from(7).is_prime());
        assert!($scalar_type::from(11).is_prime());
        assert!($scalar_type::from(13).is_prime());
        assert!($scalar_type::from(17).is_prime());
        assert!($scalar_type::from(19).is_prime());
        assert!($scalar_type::from(23).is_prime());

        // Test non-primes
        assert!(!$scalar_type::from(1).is_prime());
        assert!(!$scalar_type::from(4).is_prime());
        assert!(!$scalar_type::from(6).is_prime());
        assert!(!$scalar_type::from(8).is_prime());
        assert!(!$scalar_type::from(9).is_prime());
        assert!(!$scalar_type::from(10).is_prime());
        assert!(!$scalar_type::from(12).is_prime());
        assert!(!$scalar_type::from(14).is_prime());
        assert!(!$scalar_type::from(15).is_prime());
        assert!(!$scalar_type::from(16).is_prime());
        assert!(!$scalar_type::from(18).is_prime());
        assert!(!$scalar_type::from(20).is_prime());
        assert!(!$scalar_type::from(21).is_prime());
        assert!(!$scalar_type::from(22).is_prime());

        // Test edge cases for primality
        assert!(!$scalar_type::from(0).is_prime());
        assert!(!$scalar_type::from(-2).is_prime());
        assert!(!$scalar_type::from(-7).is_prime());
        assert!(!$scalar_type::from(1.5).is_prime());
        assert!(!$scalar_type::from(-2.5).is_prime());
        assert!(!$scalar_type::INFINITY.is_prime());

        // Test extreme values for this type
        let max_val = $scalar_type::MAX;
        let min_pos = $scalar_type::MIN_POS;
        assert!(max_val.is_normal());
        assert!(min_pos.is_normal());

        // Test overflow creates exploded state
        let exploded: $scalar_type = max_val * 2;
        assert!(exploded.exploded());
        assert!(exploded.is_positive());

        // Test underflow creates vanished state
        let vanished: $scalar_type = min_pos.square();
        assert!(vanished.vanished());
        assert!(vanished.is_positive());

        // Test undefined propagation
        let undefined = $scalar_type::from(0) / zero;
        assert!(undefined.is_undefined());
        let propagated = undefined + $scalar_type::from(42);
        assert!(propagated.is_undefined());
    };
}

// Define all 25 scalar types in one place
macro_rules! all_scalar_types {
    ($macro_name:ident) => {
        $macro_name!(
            ScalarF3E3, ScalarF3E4, ScalarF3E5, ScalarF3E6, ScalarF3E7, ScalarF4E3, ScalarF4E4,
            ScalarF4E5, ScalarF4E6, ScalarF4E7, ScalarF5E3, ScalarF5E4, ScalarF5E5, ScalarF5E6,
            ScalarF5E7, ScalarF6E3, ScalarF6E4, ScalarF6E5, ScalarF6E6, ScalarF6E7, ScalarF7E3,
            ScalarF7E4, ScalarF7E5, ScalarF7E6, ScalarF7E7
        );
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
all_scalar_types!(test_all_scalar_types);

#[cfg(test)]
mod basic_operations {
    use super::*;

    // Macro to test scalar creation and conversion for any scalar type
    macro_rules! test_scalar_creation_for_type {
        ($scalar_type:ident) => {
            // Test creation from mixed integer types
            let a = $scalar_type::from(42u8);
            let b = $scalar_type::from(-17i16);
            let c = $scalar_type::from(100u32);
            let d = $scalar_type::from(-50i64);
            assert!(a.is_normal());
            assert!(b.is_normal());
            assert!(c.is_normal());
            assert!(d.is_normal());
            assert!(a.is_positive());
            assert!(b.is_negative());
            assert!(c.is_positive());
            assert!(d.is_negative());

            // Test creation from mixed float types
            let pi = $scalar_type::from(3.14159f32);
            let e = $scalar_type::from(2.71828f64);
            assert!(pi.is_normal());
            assert!(e.is_normal());

            // Test constants
            let zero = $scalar_type::ZERO;
            let one = $scalar_type::ONE;
            let pi_const = $scalar_type::PI;
            let e_const = $scalar_type::E;

            assert!(zero.is_zero());
            assert!(one.is_normal());
            assert!(pi_const.is_normal());
            assert!(e_const.is_normal());
            assert!(
                pi_const.square() > 9u8
                    && pi_const.square() < 10i32
                    && !pi_const.square().is_integer()
                    && pi_const.square().is_normal()
            );

            // Test mixed type conversions
            assert!(a == 42usize);
            assert!(b == -17isize);
            assert!(one == 1f32);
            assert!(zero == 0f64);
        };
    }

    // Generate scalar creation tests for all scalar types
    macro_rules! generate_scalar_creation_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_scalar_creation_ $scalar_type:lower>]() {
                        test_scalar_creation_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_scalar_creation_tests);

    // Macro to test basic arithmetic with mixed types for any scalar type
    macro_rules! test_basic_arithmetic_for_type {
        ($scalar_type:ident) => {
            // Test case 1: 7 + 3 = 10
            let a1 = $scalar_type::from(7i8);
            let b1 = 3f32;
            let sum1 = a1 + b1;
            assert!(sum1 == 10);

            // Test case 2: 12 - 5 = 7
            let a2 = $scalar_type::from(12u16);
            let b2 = 5i32;
            let diff2 = a2 - b2;
            assert!(diff2 == 7u64);

            // Test case 3: 6 * 8 = 48
            let a3 = 6f32;
            let b3 = $scalar_type::from(8i64);
            let product3 = a3 * b3;
            assert!(product3 == 48u128);
            // Test case 4: 20 / 4 = 5
            let a4 = $scalar_type::from(20usize);
            let b4 = 4isize;
            let quotient4 = a4 / b4;
            assert!(quotient4 == 5f64);

            // Test case 5: 15 + 25 = 40
            let a5 = 15i128;
            let b5 = $scalar_type::from(25u32);
            let sum5 = a5 + b5;
            assert!(sum5 == 40.);

            // Test case 6: 100 - 37 = 63
            let a6 = $scalar_type::from(100f64);
            let b6 = 37u8;
            let diff6 = a6 - b6;
            assert!(diff6 == 63i16);

            // Test case 7: 9 * 7 = 63
            let a7 = 9i8;
            let b7 = $scalar_type::from(7f32);
            let product7 = a7 * b7;
            assert!(product7 == 63);

            // Test case 8: 144 / 12 = 12
            let a8 = $scalar_type::from(144u64);
            let b8 = $scalar_type::from(12);
            let quotient8 = a8 / b8;
            assert!(quotient8 == 12);

            // Test case 9: 0 + 42 = 42
            let zero = $scalar_type::ZERO;
            let fortytwo = 42i64;
            let sum9 = zero + fortytwo;
            assert!(sum9 == 42);

            // Test negation
            let pos = $scalar_type::from(17f64);
            let neg = -pos;
            assert!(neg == -17);
            assert!(neg.is_negative());
        };
    }

    // Generate basic arithmetic tests for all scalar types
    macro_rules! generate_basic_arithmetic_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_basic_arithmetic_ $scalar_type:lower>]() {
                        test_basic_arithmetic_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_basic_arithmetic_tests);

    // Macro to test zero operations for any scalar type
    macro_rules! test_zero_operations_for_type {
        ($scalar_type:ident) => {
            let zero = $scalar_type::ZERO;
            let five = 5u8;

            // Addition with zero - mixed types
            let result = five + zero;
            assert!(result == 5i16);

            let result2 = zero + 7isize;
            assert!(result2 == 7f32);

            // Multiplication with zero - mixed types
            let result3 = 3u32 * zero;
            assert!(result3 == zero);

            let result4 = zero * 9f64;
            assert!(result4.is_zero());

            // 0/0 is undefined
            let div_by_zero = 0i8 / zero;
            assert!(div_by_zero.is_undefined());
        };
    }

    // Macro to test arithmetic properties for any scalar type
    macro_rules! test_arithmetic_properties_for_type {
        ($scalar_type:ident) => {
            let a = $scalar_type::from(12u16);
            let b = 8i32;
            let c = $scalar_type::from(-5f32);

            // Commutativity with mixed types
            assert!(a + b == b + a);
            assert!(a * b == b * a);

            // Associativity
            assert!((a + b) + c == a + (b + c));
            assert!((a * b) * c == a * (b * c));

            // Identity elements with mixed types
            let zero = $scalar_type::ZERO;
            let one = $scalar_type::ONE;

            assert!(a + zero == a);
            assert!(one * 42isize == 42);
        };
    }

    // Generate zero operations tests for all scalar types
    macro_rules! generate_zero_operations_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_zero_operations_ $scalar_type:lower>]() {
                        test_zero_operations_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate arithmetic properties tests for all scalar types
    macro_rules! generate_arithmetic_properties_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_arithmetic_properties_ $scalar_type:lower>]() {
                        test_arithmetic_properties_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_zero_operations_tests);
    all_scalar_types!(generate_arithmetic_properties_tests);
}

#[cfg(test)]
mod special_values {
    use super::*;

    // Macro to test escaped values for any scalar type
    macro_rules! test_escaped_values_for_type {
        ($scalar_type:ident) => {
            // Test exploded values (too large)
            let max_val = $scalar_type::MAX;
            let exploded = max_val * 2f32;

            assert!(exploded.exploded());
            assert!(exploded.is_positive());
            assert!(!exploded.is_normal());
            assert!(!exploded.is_zero());

            // Test negative exploded
            let neg_exploded = max_val * -2f64;
            assert!(neg_exploded.exploded());
            assert!(neg_exploded.is_negative());

            // Test vanished values (too small)
            let min_pos = $scalar_type::MIN_POS;
            let vanished = min_pos / 1000u16;

            assert!(vanished.vanished());
            assert!(vanished.is_positive());
            assert!(!vanished.is_normal());
            assert!(!vanished.is_zero());

            // Test negative vanished
            let neg_vanished = min_pos / -1000i32;
            assert!(neg_vanished.vanished());
            assert!(neg_vanished.is_negative());
        };
    }

    // Macro to test undefined states for any scalar type
    macro_rules! test_undefined_states_for_type {
        ($scalar_type:ident) => {
            // Zero over zero
            let zero_div_zero = 0u8 / $scalar_type::ZERO;
            assert!(zero_div_zero.is_undefined());

            // Zero to zero power
            let zero_pow_zero = $scalar_type::ZERO.pow($scalar_type::ZERO);
            assert!(zero_pow_zero.is_undefined());

            // Square root of negative
            let sqrt_neg = $scalar_type::from(-4i16).sqrt();
            assert!(sqrt_neg.is_undefined());

            // Logarithm of negative
            let log_neg = $scalar_type::from(-2f32).ln();
            assert!(log_neg.is_undefined());

            // Undefined values propagate
            let propagated = zero_div_zero + 42i64;
            assert!(propagated.is_undefined());

            let propagated2 = sqrt_neg * 10f64;
            assert!(propagated2.is_undefined());
        };
    }

    // Macro to test escaped value operations for any scalar type
    macro_rules! test_escaped_operations_for_type {
        ($scalar_type:ident) => {
            let exploded = $scalar_type::MAX * 2usize;
            let vanished = $scalar_type::MIN_POS / 1000isize;

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
        };
    }

    // Generate escaped values tests for all scalar types
    macro_rules! generate_escaped_values_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_escaped_values_ $scalar_type:lower>]() {
                        test_escaped_values_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate undefined states tests for all scalar types
    macro_rules! generate_undefined_states_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_undefined_states_ $scalar_type:lower>]() {
                        test_undefined_states_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate escaped operations tests for all scalar types
    macro_rules! generate_escaped_operations_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_escaped_operations_ $scalar_type:lower>]() {
                        test_escaped_operations_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_escaped_values_tests);
    all_scalar_types!(generate_undefined_states_tests);
    all_scalar_types!(generate_escaped_operations_tests);
}

#[cfg(test)]
mod mathematical_functions {
    use super::*;

    // Macro to test power functions for any scalar type
    macro_rules! test_power_functions_for_type {
        ($scalar_type:ident) => {
            let base = $scalar_type::from(2u8);
            let exp = 3i16;

            // Basic power with mixed types
            let result = base.pow(exp);
            assert!(result == 8u32);
            assert!(result.is_normal());

            // Square function
            let square = base.square();
            assert!(square == 4i64);
            assert!(square.is_normal());

            // Square root
            let sqrt_val = $scalar_type::from(9f32).sqrt();
            assert!(sqrt_val == 3f64);
            assert!(sqrt_val.is_normal());

            // Reciprocal
            let recip = $scalar_type::from(4usize).reciprocal();
            assert!(recip == 0.25);
            assert!(recip.is_normal());
        };
    }

    // Macro to test exponential and logarithmic functions for any scalar type
    macro_rules! test_exp_log_for_type {
        ($scalar_type:ident) => {
            // Natural exponential
            let exp_result = $scalar_type::from(1isize).exp();
            assert!(exp_result.is_normal());

            // Natural logarithm
            let ln_e = $scalar_type::E.ln();
            assert!(ln_e == 1u128);
            assert!(ln_e.is_normal());

            // Binary logarithm
            let lb_8 = $scalar_type::from(8i32).lb();
            assert!(lb_8 == 3i8);
            assert!(lb_8.is_normal());

            // Power of 2
            let powb_3 = $scalar_type::from(3u64).powb();
            assert!(powb_3 == 8);
            assert!(powb_3.is_normal());
        };
    }

    // Macro to test trigonometric functions for any scalar type
    macro_rules! test_trig_functions_for_type {
        ($scalar_type:ident) => {
            let pi_half = $scalar_type::PI / 2f32;
            let pi_quarter = $scalar_type::PI / 4f64;

            // Sine: sin(π/2) = 1
            let sin_pi_half = pi_half.sin();
            assert!((sin_pi_half - 1u16).magnitude() < 0.0001);
            assert!(sin_pi_half.is_normal());

            // Sine: sin(0) = 0
            let sin_zero = $scalar_type::ZERO.sin();
            assert!(sin_zero == 0i32);

            // Cosine: cos(0) = 1
            let cos_zero = $scalar_type::ZERO.cos();
            assert!(cos_zero == 1u16);

            // Cosine: cos(π/2) ≈ 0
            let cos_pi_half = pi_half.cos();
            assert!(cos_pi_half.magnitude() < 0.0001);

            // Tangent: tan(π/4) = 1
            let tan_pi_quarter = pi_quarter.tan();
            assert!((tan_pi_quarter - 1f32).magnitude() < 0.0001);

            // Tangent: tan(0) = 0
            let tan_zero = $scalar_type::ZERO.tan();
            assert!(tan_zero == 0f64);
            assert!(tan_zero.is_zero());

            // Inverse functions with known values
            let asin_half = $scalar_type::from(0.5).asin();
            assert!(asin_half.is_normal());
            assert!(asin_half > 0i64);
            assert!(asin_half < pi_half);

            let acos_half = $scalar_type::from(0.5).acos();
            assert!(acos_half.is_normal());
            assert!(acos_half > 0);
            assert!(acos_half < $scalar_type::PI);

            // atan(1) = π/4
            let atan_one = $scalar_type::ONE.atan();
            assert!(atan_one == pi_quarter);

            // atan(0) = 0
            let atan_zero = $scalar_type::ZERO.atan();
            assert!(atan_zero == 0u128);
            assert!(atan_zero.is_zero());
        };
    }

    // Macro to test hyperbolic functions for any scalar type
    macro_rules! test_hyperbolic_for_type {
        ($scalar_type:ident) => {
            let one = $scalar_type::ONE;
            let zero = $scalar_type::ZERO;

            // Hyperbolic sine: sinh(0) = 0
            let sinh_0 = zero.sinh();
            assert!(sinh_0 == 0u8);
            assert!(sinh_0.is_zero());

            // Hyperbolic sine: sinh(1) > 0
            let sinh_1 = one.sinh();
            assert!(sinh_1 > 1i16);

            // Hyperbolic cosine: cosh(0) = 1
            let cosh_0 = zero.cosh();
            assert!(cosh_0 == 1u32);

            // Hyperbolic cosine: cosh(1) > 1
            let cosh_1 = one.cosh();
            assert!(cosh_1 > 1f32);

            // Hyperbolic tangent: tanh(0) = 0
            let tanh_0 = zero.tanh();
            assert!(tanh_0 == 0f64);
            assert!(tanh_0.is_zero());

            // Hyperbolic tangent: tanh(1) ∈ (0,1)
            let tanh_1 = one.tanh();
            assert!(tanh_1 > 0i64);
            assert!(tanh_1 < 1usize);
            assert!(tanh_1.is_normal());

            // Hyperbolic identities: cosh²(x) - sinh²(x) = 1
            let cosh_squared = cosh_1.square();
            let sinh_squared = sinh_1.square();
            let identity = cosh_squared - sinh_squared;
            assert!((identity - 1u128).magnitude() < 0.0001);
        };
    }

    // Generate power function tests for all scalar types
    macro_rules! generate_power_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_power_functions_ $scalar_type:lower>]() {
                        test_power_functions_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate exp/log tests for all scalar types
    macro_rules! generate_exp_log_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_exp_log_ $scalar_type:lower>]() {
                        test_exp_log_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate trigonometric tests for all scalar types
    macro_rules! generate_trig_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_trig_ $scalar_type:lower>]() {
                        test_trig_functions_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate hyperbolic tests for all scalar types
    macro_rules! generate_hyperbolic_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_hyperbolic_ $scalar_type:lower>]() {
                        test_hyperbolic_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_power_tests);
    all_scalar_types!(generate_exp_log_tests);
    all_scalar_types!(generate_trig_tests);
    all_scalar_types!(generate_hyperbolic_tests);
}

#[cfg(test)]
mod comparison_and_utility {
    use super::*;

    // Macro to test comparison operations for any scalar type
    macro_rules! test_comparison_for_type {
        ($scalar_type:ident) => {
            let a = $scalar_type::from(5u8);
            let b = 3i16;
            let c = $scalar_type::from(8f32);

            // Min and max with mixed types
            let min_val = a.min(b);
            assert!(min_val == 3u32);

            let max_val = a.max(c);
            assert!(max_val == 8i64);

            // Clamp with mixed types
            let clamped = a.clamp($scalar_type::from(b), c);
            assert!(clamped == 5f64);

            let clamped_low = $scalar_type::from(1usize).clamp($scalar_type::from(b), c);
            assert!(clamped_low == 3isize);

            let clamped_high = $scalar_type::from(10u128).clamp($scalar_type::from(b), c);
            assert!(clamped_high == 8);
        };
    }

    // Macro to test integer functions for any scalar type
    macro_rules! test_integer_functions_for_type {
        ($scalar_type:ident) => {
            // Floor with mixed types
            let floor_val = $scalar_type::from(3.7f32).floor();
            assert!(floor_val == 3i8);

            let floor_neg = $scalar_type::from(-2.3f64).floor();
            assert!(floor_neg == -3i32);

            // Ceiling
            let ceil_val = $scalar_type::from(3.2).ceil();
            assert!(ceil_val == 4u16);

            // Round
            let round_val = $scalar_type::from(3.6).round();
            assert!(round_val == 4i128);

            // Banker's rounding (round to even) tests
            let round_half_even = $scalar_type::from(2.5f32).round();
            assert!(round_half_even == 2u8); // Round to even

            let round_half_odd = $scalar_type::from(3.5f64).round();
            assert!(round_half_odd == 4i16); // Round to even

            let round_half_even2 = $scalar_type::from(4.5).round();
            assert!(round_half_even2 == 4u32); // Round to even

            let round_half_odd2 = $scalar_type::from(5.5).round();
            assert!(round_half_odd2 == 6i64); // Round to even

            // Negative banker's rounding
            let round_neg_half_even = $scalar_type::from(-2.5f32).round();
            assert!(round_neg_half_even == -2isize); // Round to even

            let round_neg_half_odd = $scalar_type::from(-3.5).round();
            assert!(round_neg_half_odd == -4i128); // Round to even

            // Fractional part
            let frac_val = $scalar_type::from(3.7).frac();
            assert!(frac_val.is_normal());
        };
    }

    // Macro to test state checking for any scalar type
    macro_rules! test_state_checking_for_type {
        ($scalar_type:ident) => {
            let normal = $scalar_type::from(42u64);
            let zero = $scalar_type::ZERO;
            let exploded = $scalar_type::MAX * 2f32;
            let vanished = $scalar_type::MIN_POS / 1000f64;
            let undefined = $scalar_type::INFINITY / $scalar_type::INFINITY;

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
        };
    }

    // Macro to test integer properties for any scalar type
    macro_rules! test_integer_properties_for_type {
        ($scalar_type:ident) => {
            // Test is_integer with mixed types
            assert!($scalar_type::from(42u8).is_integer());
            assert!($scalar_type::from(-17i16).is_integer());
            assert!(!$scalar_type::from(3.14f32).is_integer());
            assert!($scalar_type::ZERO.is_integer());

            // Test contiguous range
            let small_int = $scalar_type::from(5i64);
            assert!(small_int.is_contiguous());

            // Test primality (for small integers)
            assert!($scalar_type::from(7u32).is_prime());
            assert!($scalar_type::from(13usize).is_prime());
            assert!(!$scalar_type::from(8i128).is_prime());
            assert!(!$scalar_type::from(15f64).is_prime());
        };
    }

    // Generate comparison tests for all scalar types
    macro_rules! generate_comparison_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_comparison_ $scalar_type:lower>]() {
                        test_comparison_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate integer function tests for all scalar types
    macro_rules! generate_integer_function_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_integer_functions_ $scalar_type:lower>]() {
                        test_integer_functions_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate state checking tests for all scalar types
    macro_rules! generate_state_checking_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_state_checking_ $scalar_type:lower>]() {
                        test_state_checking_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    // Generate integer properties tests for all scalar types
    macro_rules! generate_integer_properties_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_integer_properties_ $scalar_type:lower>]() {
                        test_integer_properties_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_comparison_tests);
    all_scalar_types!(generate_integer_function_tests);
    all_scalar_types!(generate_state_checking_tests);
    all_scalar_types!(generate_integer_properties_tests);
}

#[cfg(test)]
mod bitwise_operations {
    use super::*;

    // Macro to test bitwise operations for any scalar type
    macro_rules! test_bitwise_for_type {
        ($scalar_type:ident) => {
            let a = $scalar_type::from(0b101010u8);  // 42
            let b = 0b1010101i16;                    // 85

            // Bitwise AND: 42 & 85 = 0b101010 & 0b1010101 = 0b000000 = 0
            let and_result = a & b;
            assert!(and_result == 0u32);

            // Bitwise OR: 42 | 85 = 0b101010 | 0b1010101 = 0b1111111 = 127
            let or_result = a | b;
            assert!(or_result == 127i64);

            // Bitwise XOR: 42 ^ 85 = 0b101010 ^ 0b1010101 = 0b1111111 = 127
            let xor_result = a ^ b;
            assert!(xor_result == 127u128);

            // Test reverse operations (primitive OP Scalar)
            assert!(b & a == 0);
            assert!(b | a == 127);
            assert!(b ^ a == 127);

            // Left shift: 42 << 2 = 168
            let left_shift = a << 2i32;
            assert!(left_shift == $scalar_type::from(168));

            // Right shift: 84 >> 1 = 42
            let right_shift = $scalar_type::from(84i16) >> 1isize;
            assert!(right_shift == $scalar_type::from(42));
        };
    }

    // Generate bitwise tests for all scalar types
    macro_rules! generate_bitwise_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_bitwise_ $scalar_type:lower>]() {
                        test_bitwise_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_bitwise_tests);
}

#[cfg(test)]
mod modular_operations {
    use super::*;

    // Macro to test modular operations for any scalar type
    macro_rules! test_modulo_for_type {
        ($scalar_type:ident) => {
            // Basic modulus: 17 % 5 = 2
            let a = $scalar_type::from(17u8);
            let b = 5i16;
            let remainder = a % b;
            assert!(remainder == 2f32);
            assert!(remainder.is_normal());

            // Modulus preserves sign of divisor: -7 % 3 = 2 (not -1)
            let neg_a = $scalar_type::from(-7i32);
            let pos_b = 3u64;
            let remainder_sign = neg_a % pos_b;
            assert!(remainder_sign == 2f64);
            assert!(remainder_sign.is_positive());

            // Modulus with negative divisor: 7 % -3 = -2
            let pos_a = $scalar_type::from(7usize);
            let neg_b = -3isize;
            let remainder_neg = pos_a % neg_b;
            assert!(remainder_neg == -2i128);
            assert!(remainder_neg.is_negative());

            // Floating point modulus: 7.5 % 2.5 = 0
            let c = $scalar_type::from(7.5f64);
            let d = 2.5f32;
            let remainder2 = c % d;
            assert!(remainder2 == 0u32);
            assert!(remainder2.is_zero());

            // Fractional modulus: 3.7 % 1.2 ≈ 0.1
            let frac_a = $scalar_type::from(3.7f32);
            let frac_b = 1.2f64;
            let frac_remainder = frac_a % frac_b;
            assert!(frac_remainder.is_normal());
            assert!(frac_remainder > 0u8);
            assert!(frac_remainder < 1i8);

            // Zero cases
            let zero = $scalar_type::ZERO;

            // 0 % anything = 0
            assert!((zero % 5u16).is_zero());
            assert!((0i64 % $scalar_type::PI).is_zero());

            // anything % 0 = 0
            assert!(($scalar_type::from(42) % zero).is_zero());
            assert!((17f32 % zero).is_zero());

            // Special escaped value cases
            let exploded = $scalar_type::MAX * 2usize;
            let vanished = $scalar_type::MIN_POS / 1000isize;

            // Exploded % anything with matching signs = exploded value
            let exploded_mod = exploded % $scalar_type::from(5f32);
            assert!(exploded_mod.exploded());
            assert!(exploded_mod.is_positive());

            // Exploded % anything with differing signs = undefined
            let exploded_diff_sign = exploded % $scalar_type::from(-5f64);
            assert!(exploded_diff_sign.is_undefined());

            // anything % vanished = undefined
            let mod_vanished = $scalar_type::from(42u32) % vanished;
            assert!(mod_vanished.is_undefined());

            // Vanished % anything with matching signs = vanished
            let vanished_mod = vanished % $scalar_type::from(5i128);
            assert!(vanished_mod.vanished());
            assert!(vanished_mod.is_positive());

            // Exploded/vanished % 0 = 0
            assert!((exploded % zero).is_zero());
            assert!((vanished % zero).is_zero());
            assert!((0u8 % exploded).is_zero());
            assert!((0i8 % vanished).is_zero());

            // Undefined propagation
            let undefined = $scalar_type::INFINITY / $scalar_type::INFINITY;
            assert!((undefined % $scalar_type::from(5)).is_undefined());
            assert!(($scalar_type::from(5) % undefined).is_undefined());
        };
    }

    // Generate modular tests for all scalar types
    macro_rules! generate_modular_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_modulo_ $scalar_type:lower>]() {
                        test_modulo_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_modular_tests);
}

#[cfg(test)]
mod random_testing {
    use super::*;

    // Macro to test random generation for any scalar type
    macro_rules! test_random_for_type {
        ($scalar_type:ident) => {
            const SAMPLE_SIZE: usize = 100;
            let mut uniform_samples = Vec::with_capacity(SAMPLE_SIZE);
            let mut gauss_samples = Vec::with_capacity(SAMPLE_SIZE);
            let mut positive_count = 0u32;
            let mut negative_count = 0u32;
            let mut zero_count = 0u32;

            // Collect samples for statistical analysis
            for _ in 0..SAMPLE_SIZE {
                let random_val = $scalar_type::random();
                assert!(random_val.is_normal() || random_val.is_zero() || random_val.vanished());

                if random_val.is_normal() {
                    uniform_samples.push(random_val);
                    if random_val.is_positive() {
                        positive_count += 1;
                    } else if random_val.is_negative() {
                        negative_count += 1;
                    }
                } else if random_val.is_zero() {
                    zero_count += 1;
                }

                let random_gauss = $scalar_type::random_gauss();
                assert!(
                    random_gauss.is_normal()
                        || random_gauss.is_zero()
                        || random_gauss.exploded()
                        || random_gauss.vanished()
                );

                if random_gauss.is_normal() {
                    gauss_samples.push(random_gauss);
                }
            }

            // Statistical tests for uniform distribution
            if !uniform_samples.is_empty() {
                // Should have roughly balanced positive/negative split (within reason)
                let total_nonzero = positive_count + negative_count;
                if total_nonzero > 10 {
                    let pos_ratio =
                        $scalar_type::from(positive_count) / $scalar_type::from(total_nonzero);
                    assert!(
                        pos_ratio > 0.2 && pos_ratio < 0.8,
                        "Uniform distribution too skewed: {}/{} positive",
                        positive_count,
                        total_nonzero
                    );
                }

                // Calculate some basic stats for uniform samples
                let mut sum = $scalar_type::ZERO;
                let mut abs_sum = $scalar_type::ZERO;
                for &sample in &uniform_samples {
                    sum = sum + sample;
                    abs_sum = abs_sum + sample.magnitude();
                }

                // Mean should be close to zero for uniform [-1,1] distribution
                let mean = sum / $scalar_type::from(uniform_samples.len());
                assert!(mean.magnitude() < 0.5, "Uniform mean too far from zero");

                // Average absolute value should be reasonable for [-1,1] uniform
                let avg_abs = abs_sum / $scalar_type::from(uniform_samples.len());
                assert!(avg_abs > 0.1 && avg_abs < 1.0, "Uniform spread suspicious");
            }

            // Statistical tests for Gaussian distribution
            if gauss_samples.len() > 20 {
                // Count values within 1, 2, 3 standard deviations
                let mut within_1_sigma = 0u32;
                let mut within_2_sigma = 0u32;
                let mut within_3_sigma = 0u32;

                for &sample in &gauss_samples {
                    let abs_val = sample.magnitude();
                    if abs_val < 1.0 {
                        within_1_sigma += 1;
                    }
                    if abs_val < 2.0 {
                        within_2_sigma += 1;
                    }
                    if abs_val < 3.0 {
                        within_3_sigma += 1;
                    }
                }

                let total = $scalar_type::from(gauss_samples.len());
                let ratio_1s = $scalar_type::from(within_1_sigma) / total;
                let ratio_2s = $scalar_type::from(within_2_sigma) / total;

                // For normal distribution: ~68% within 1σ, ~95% within 2σ
                // We'll be lenient due to small sample size and discrete precision
                assert!(
                    ratio_1s > 0.4 && ratio_1s < 0.9,
                    "Gaussian 1-sigma ratio suspicious: {:.2}",
                    ratio_1s
                );
                assert!(
                    ratio_2s > 0.7 && ratio_2s < 1.0,
                    "Gaussian 2-sigma ratio suspicious: {:.2}",
                    ratio_2s
                );
            }

            // Test that random values have proper mixed type compatibility
            let rand_test = $scalar_type::random();
            assert!(rand_test == rand_test); // Self-equality
            assert!(rand_test.magnitude() >= 0u8); // Abs is non-negative

            let gauss_test = $scalar_type::random_gauss();
            let combined = rand_test + gauss_test;
            assert!(
                combined.is_normal()
                    || combined.is_zero()
                    || combined.exploded()
                    || combined.vanished()
                    || combined.is_undefined()
            );
        };
    }

    // Generate random tests for all scalar types
    macro_rules! generate_random_tests {
        ($($scalar_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_random_ $scalar_type:lower>]() {
                        test_random_for_type!($scalar_type);
                    }
                }
            )+
        };
    }

    all_scalar_types!(generate_random_tests);
}
