use spirix::*;

// Macro to test basic arithmetic for all circle types
macro_rules! test_arithmetic_for_type {
    ($circle_type:ident) => {
        // Test creation from tuples with mixed types
        let a = $circle_type::from((40u8, 2i8));
        let b = $circle_type::from((3f32, -5i16));
        let sum = a + b;
        assert!(sum.r() == 43u16);
        assert!(sum.i() == -3i32);

        // Test creation from single values (real-only)
        let real_only = $circle_type::from(15i64);
        assert!(real_only.r() == 15u128);
        assert!(real_only.i().is_zero());

        // Test basic arithmetic with mixed types
        let c = $circle_type::from((12i16, 8f32));
        let d = $circle_type::from((5i32, 3f64));
        let diff = c - d;
        assert!(diff.r() == 7usize);
        assert!(diff.i() == 5isize);

        // Test multiplication with mixed types
        let e = $circle_type::from((3f32, 4i64));
        let f = 2u32;
        let product = e * f;
        assert!(product.r() == 6i128);
        assert!(product.i() == 8f32);

        // Test division with mixed types
        let g = $circle_type::from((15usize, 9isize));
        let h = 3f64;
        let quotient = g / h;
        assert!(quotient.r() == 5f32);
        assert!(quotient.i() == 3f64);

        // Test component extraction
        let complex = $circle_type::from((3.5f32, 2.75f64));
        assert!(complex.r() == 3.5);
        assert!(complex.i() == 2.75);

        // Test negation
        let pos = $circle_type::from((42f32, -17f64));
        let neg = -pos;
        assert!(neg.r() == -42);
        assert!(neg.i() == 17);

        // Test zero operations with mixed types
        let zero = $circle_type::ZERO;
        assert!($circle_type::from((5u32, 3i16)) + zero == $circle_type::from((5usize, 3isize)));
        assert!(zero + $circle_type::from((7isize, -2f32)) == $circle_type::from((7f64, -2f32)));
        assert!($circle_type::from((3i16, 4u32)) * zero == zero);
        assert!(zero * $circle_type::from((8u64, -5i128)) == zero);

        // Test identity with mixed types
        let one = $circle_type::ONE;
        assert!($circle_type::from((5i32, 3f32)) * one == $circle_type::from((5f64, 3f64)));
        assert!(one * $circle_type::from((9u16, -7i64)) == $circle_type::from((9f32, -7f64)));

        // Test imaginary unit
        let i = $circle_type::POS_I;
        assert!(i.r().is_zero());
        assert!(i.i() == 1u32);
        assert!(i.square() == -1f32);

        let neg_i = $circle_type::NEG_I;
        assert!(neg_i.r().is_zero());
        assert!(neg_i.i() == -1i32);
        assert!(neg_i.square() == -1f64);

        // Test constants are normal
        assert!(one.is_normal());
        assert!(zero.is_zero());
        assert!(i.is_normal());
        assert!(neg_i.is_normal());
        assert!($circle_type::PI.is_normal());
        assert!($circle_type::E.is_normal());

        // Test state checks
        let normal = $circle_type::from((42f32, 17i64));
        assert!(normal.is_normal());
        assert!(!zero.is_normal());

        // Test complex-specific operations
        let complex_val = $circle_type::from((3f32, 4i16));

        // Conjugate
        let conj = complex_val.conjugate();
        assert!(conj.r() == 3f64);
        assert!(conj.i() == -4i32);

        // Magnitude
        let mag = complex_val.magnitude();
        assert!(mag == 5f32);

        // Magnitude squared
        let mag_sq = complex_val.magnitude_squared();
        assert!(mag_sq == 25u64);

        // Sign (unit vector)
        let unit = complex_val.sign();
        let unit_mag = unit.magnitude();
        assert!((unit_mag - 1f32).magnitude() < 0.1);

        // Test extreme values for this type
        let max_val = $circle_type::MAX;
        assert!(max_val.is_normal());

        // Test overflow creates exploded state
        let exploded: $circle_type = max_val * 2;
        assert!(exploded.exploded());

        // Test underflow creates vanished state
        let min_pos = $circle_type::MIN_POS;
        let vanished: $circle_type = min_pos.square();
        assert!(vanished.vanished());

        // Test undefined propagation
        let undefined = $circle_type::from(0) / zero;
        assert!(undefined.is_undefined());
        let propagated = undefined + $circle_type::from((42f32, 17i64));
        assert!(propagated.is_undefined());
    };
}

// Define all 25 circle types in one place
macro_rules! all_circle_types {
    ($macro_name:ident) => {
        $macro_name!(
            CircleF3E3, CircleF3E4, CircleF3E5, CircleF3E6, CircleF3E7, CircleF4E3, CircleF4E4,
            CircleF4E5, CircleF4E6, CircleF4E7, CircleF5E3, CircleF5E4, CircleF5E5, CircleF5E6,
            CircleF5E7, CircleF6E3, CircleF6E4, CircleF6E5, CircleF6E6, CircleF6E7, CircleF7E3,
            CircleF7E4, CircleF7E5, CircleF7E6, CircleF7E7
        );
    };
}

// Macro to generate test functions for all circle types
macro_rules! test_all_circle_types {
    ($($circle_type:ident),+) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_ $circle_type:lower _comprehensive>]() {
                    test_arithmetic_for_type!($circle_type);
                }
            }
        )+
    };
}

// Generate comprehensive tests for all 25 circle types
all_circle_types!(test_all_circle_types);

#[cfg(test)]
mod basic_operations {
    use super::*;

    // Macro to test circle creation and conversion for any circle type
    macro_rules! test_circle_creation_for_type {
        ($circle_type:ident) => {
            // Test creation from mixed tuple types
            let a = $circle_type::from((42u8, -17i16));
            let b = $circle_type::from((100u32, 50i64));
            assert!(a.is_normal());
            assert!(b.is_normal());
            assert!(a.r() == 42);
            assert!(a.i() == -17);
            assert!(b.r() == 100);
            assert!(b.i() == 50);

            // Test creation from mixed float tuples
            let pi_e = $circle_type::from((3.14159f32, 2.71828f64));
            assert!(pi_e.is_normal());
            assert!(pi_e.r() > 3f32 && pi_e.r() < 4f32);
            assert!(pi_e.i() > 2f32 && pi_e.i() < 3f32);

            // Test creation from single values
            let real_only = $circle_type::from(7.5f64);
            assert!(real_only.r() == 7.5);
            assert!(real_only.i().is_zero());

            // Test constants
            let zero = $circle_type::ZERO;
            let one = $circle_type::ONE;
            let pos_i = $circle_type::POS_I;
            let neg_i = $circle_type::NEG_I;
            let pi_const = $circle_type::PI;
            let e_const = $circle_type::E;

            assert!(zero.is_zero());
            assert!(one.is_normal());
            assert!(pos_i.is_normal());
            assert!(neg_i.is_normal());
            assert!(pi_const.is_normal());
            assert!(e_const.is_normal());

            // Test imaginary unit properties
            assert!(pos_i.square() == -1isize);
            assert!(neg_i.square() == -1isize);
            assert!((pos_i * neg_i - 1i8).magnitude() < 0.1f32);

            // Test mixed type conversions
            assert!(a.r() == 42usize);
            assert!(a.i() == -17isize);
            assert!(one.r() == 1f32);
            assert!(zero.r() == 0f64);
        };
    }

    // Generate circle creation tests for all circle types
    macro_rules! generate_circle_creation_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_circle_creation_ $circle_type:lower>]() {
                        test_circle_creation_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_circle_creation_tests);

    // Macro to test basic arithmetic with mixed types for any circle type
    macro_rules! test_basic_arithmetic_for_type {
        ($circle_type:ident) => {
            // Test case 1: (7, 3) + (3, 2) = (10, 5)
            let a1 = $circle_type::from((7i8, 3f32));
            let b1 = $circle_type::from((3f32, 2i16));
            let sum1 = a1 + b1;
            assert!(sum1.r() == 10);
            assert!(sum1.i() == 5);

            // Test case 2: (12, 8) - (5, 3) = (7, 5)
            let a2 = $circle_type::from((12u16, 8i32));
            let b2 = $circle_type::from((5i32, 3f64));
            let diff2 = a2 - b2;
            assert!(diff2.r() == 7u64);
            assert!(diff2.i() == 5i128);

            // Test case 3: (1, 2) * (3, 4) = (-5, 10)
            let a3 = $circle_type::from((1f32, 2f64));
            let b3 = $circle_type::from((3i64, 4u32));
            let product3 = a3 * b3;
            assert!(product3.r() == -5i128);
            assert!(product3.i() == 10f32);

            // Test case 4: (20, 10) / (2, 0) = (10, 5)
            let a4 = $circle_type::from((20usize, 10isize));
            let b4 = 2f64;
            let quotient4 = a4 / b4;
            assert!(quotient4.r() == 10f64);
            assert!(quotient4.i() == 5f32);

            // Test case 5: (15, 25) + scalar = (17, 25)
            let a5 = $circle_type::from((15i128, 25u32));
            let scalar_add = a5 + 2f32;
            assert!(scalar_add.r() == 17f64);
            assert!(scalar_add.i() == 25);

            // Test case 6: (100, 37) - scalar = (63, 37)
            let a6 = $circle_type::from((100f64, 37u8));
            let scalar_sub = a6 - 37i16;
            assert!(scalar_sub.r() == 63i16);
            assert!(scalar_sub.i() == 37);

            // Test case 7: scalar * (9, 7) = (27, 21)
            let a7 = $circle_type::from((9i8, 7f32));
            let scalar_mult = 3u64 * a7;
            assert!(scalar_mult.r() == 27);
            assert!(scalar_mult.i() == 21);

            // Test case 9: (0, 0) + (42, 17) = (42, 17)
            let zero = $circle_type::ZERO;
            let fortytwo_seventeen = $circle_type::from((42i64, 17f32));
            let sum9 = zero + fortytwo_seventeen;
            assert!(sum9.r() == 42);
            assert!(sum9.i() == 17);

            // Test negation
            let pos = $circle_type::from((17f64, -23i32));
            let neg = -pos;
            assert!(neg.r() == -17);
            assert!(neg.i() == 23);
        };
    }

    // Generate basic arithmetic tests for all circle types
    macro_rules! generate_basic_arithmetic_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_basic_arithmetic_ $circle_type:lower>]() {
                        test_basic_arithmetic_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_basic_arithmetic_tests);

    // Macro to test zero operations for any circle type
    macro_rules! test_zero_operations_for_type {
        ($circle_type:ident) => {
            let zero = $circle_type::ZERO;
            let five_three = $circle_type::from((5u8, 3i16));

            // Addition with zero - mixed types
            let result = five_three + zero;
            assert!(result.r() == 5i16);
            assert!(result.i() == 3u32);

            let result2 = zero + $circle_type::from((7isize, -2f32));
            assert!(result2.r() == 7f32);
            assert!(result2.i() == -2f64);

            // Multiplication with zero - mixed types
            let result3 = $circle_type::from((3u32, 4i64)) * zero;
            assert!(result3 == zero);

            let result4 = zero * $circle_type::from((9f64, -1i8));
            assert!(result4.is_zero());

            // 0/0 is undefined
            let div_by_zero = $circle_type::from((0i8, 0u16)) / zero;
            assert!(div_by_zero.is_undefined());
        };
    }

    // Macro to test arithmetic properties for any circle type
    macro_rules! test_arithmetic_properties_for_type {
        ($circle_type:ident) => {
            let a = $circle_type::from((12u16, 5i32));
            let b = $circle_type::from((8i32, -3f32));
            let c = $circle_type::from((2f32, 7i64));

            // Commutativity with mixed types
            assert!(a + b == b + a);
            // Note: multiplication is NOT commutative for complex numbers

            // Associativity
            assert!((a + b) + c == a + (b + c));

            // Identity elements with mixed types
            let zero = $circle_type::ZERO;
            let one = $circle_type::ONE;

            assert!(a + zero == a);
            assert!(
                one * $circle_type::from((42isize, 17f64)) == $circle_type::from((42f32, 17f32))
            );
        };
    }

    // Generate zero operations tests for all circle types
    macro_rules! generate_zero_operations_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_zero_operations_ $circle_type:lower>]() {
                        test_zero_operations_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate arithmetic properties tests for all circle types
    macro_rules! generate_arithmetic_properties_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_arithmetic_properties_ $circle_type:lower>]() {
                        test_arithmetic_properties_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_zero_operations_tests);
    all_circle_types!(generate_arithmetic_properties_tests);
}

#[cfg(test)]
mod special_values {
    use super::*;

    // Macro to test escaped values for any circle type
    macro_rules! test_escaped_values_for_type {
        ($circle_type:ident) => {
            // Test exploded values (too large)
            let max_val = $circle_type::MAX;
            let exploded = max_val * 2f32;

            assert!(exploded.exploded());
            assert!(!exploded.is_normal());
            assert!(!exploded.is_zero());

            // Test negative exploded from imaginary overflow
            let exploded_imag = $circle_type::from((1f32, max_val.r())) * 2f64;
            assert!(exploded_imag.exploded());

            // Test vanished values (too small)
            let min_pos = $circle_type::MIN_POS;
            let vanished = min_pos / 1000u16;

            assert!(vanished.vanished());
            assert!(!vanished.is_normal());
            assert!(!vanished.is_zero());

            // Test vanished from imaginary underflow
            let vanished_imag = $circle_type::from((1f32, min_pos.r())) / 1000i32;
            assert!(vanished_imag.vanished());
        };
    }

    // Macro to test undefined states for any circle type
    macro_rules! test_undefined_states_for_type {
        ($circle_type:ident) => {
            // Zero over zero
            let zero_div_zero = 0u8 / $circle_type::ZERO;
            assert!(zero_div_zero.is_undefined());

            // Zero to zero power (by convention, 0^0 = 1 in this implementation)
            let zero_pow_zero = $circle_type::ZERO.pow(0);
            assert!(zero_pow_zero.r() == 1u128);
            assert!(zero_pow_zero.i().is_zero());

            // Square root of negative real part (for complex numbers, this should work)
            let complex_with_neg = $circle_type::from((-4i16, 0f32));
            let sqrt_complex = complex_with_neg.sqrt();
            // For complex numbers, sqrt of negative real should give imaginary result
            assert!(sqrt_complex.is_normal() || sqrt_complex.vanished());

            // Undefined values propagate
            let propagated = zero_div_zero + $circle_type::from((42i64, 17f32));
            assert!(propagated.is_undefined());

            let propagated2 = zero_div_zero * $circle_type::from((10f64, -5i8));
            assert!(propagated2.is_undefined());
        };
    }

    // Macro to test escaped value operations for any circle type
    macro_rules! test_escaped_operations_for_type {
        ($circle_type:ident) => {
            let exploded = $circle_type::MAX * 2usize;
            let vanished = $circle_type::MIN_POS / 1000isize;

            // Absolute operations on escaped values
            let exploded_squared = exploded.square();
            assert!(exploded_squared.exploded());

            let vanished_squared = vanished.square();
            assert!(vanished_squared.vanished());

            // Sign operations
            let neg_exploded = -exploded;
            assert!(neg_exploded.exploded());
        };
    }

    // Generate escaped values tests for all circle types
    macro_rules! generate_escaped_values_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_escaped_values_ $circle_type:lower>]() {
                        test_escaped_values_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate undefined states tests for all circle types
    macro_rules! generate_undefined_states_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_undefined_states_ $circle_type:lower>]() {
                        test_undefined_states_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate escaped operations tests for all circle types
    macro_rules! generate_escaped_operations_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_escaped_operations_ $circle_type:lower>]() {
                        test_escaped_operations_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_escaped_values_tests);
    all_circle_types!(generate_undefined_states_tests);
    all_circle_types!(generate_escaped_operations_tests);
}

#[cfg(test)]
mod mathematical_functions {
    use super::*;

    // Macro to test power functions for any circle type
    macro_rules! test_power_functions_for_type {
        ($circle_type:ident) => {
            let base = $circle_type::from((2u8, 1i16));
            let exp = 2f32;

            // Basic power with mixed types
            let result = base.pow(exp);
            // (2+i)² = 4 + 4i - 1 = 3 + 4i
            assert!((result.r() - 3f32).magnitude() < 0.1);
            assert!((result.i() - 4f32).magnitude() < 0.1);

            // Square function
            let square = base.square();
            assert!((square.r() - 3f64).magnitude() < 0.1);
            assert!((square.i() - 4f64).magnitude() < 0.1);

            // Square root
            let sqrt_val = $circle_type::from(9f32).sqrt();
            assert!(sqrt_val == 3f64);

            // Reciprocal
            let recip = $circle_type::from((2usize, 0isize)).reciprocal();
            assert!((recip.r() - 0.5f32).magnitude() < 0.1);
            assert!(recip.i().magnitude() < 0.1);
        };
    }

    // Macro to test exponential and logarithmic functions for any circle type
    macro_rules! test_exp_log_for_type {
        ($circle_type:ident) => {
            // Natural exponential
            let exp_result = $circle_type::from((1isize, 0f32)).exp();
            let e_const = $circle_type::E;
            assert!((exp_result.r() - e_const.r()).magnitude() < 0.1f32);

            // Natural logarithm
            let ln_e = e_const.ln();
            assert!((ln_e.r() - 1u128).magnitude() < 0.1f32);
        };
    }

    // Macro to test trigonometric functions for any circle type
    macro_rules! test_trig_functions_for_type {
        ($circle_type:ident) => {
            let pi_half = $circle_type::PI / 2f32;
            let zero = $circle_type::ZERO;

            // Sine: sin(0) = 0
            let sin_zero = zero.sin();
            assert!(sin_zero.r().magnitude() < 0.1);
            assert!(sin_zero.i().magnitude() < 0.1);

            // Cosine: cos(0) = 1
            let cos_zero = zero.cos();
            assert!((cos_zero.r() - 1u16).magnitude() < 0.1);
            assert!(cos_zero.i().magnitude() < 0.1);

            // Tangent: tan(0) = 0
            let tan_zero = zero.tan();
            assert!(tan_zero.r().magnitude() < 0.1);
            assert!(tan_zero.i().magnitude() < 0.1);

            // Test with pure imaginary input
            let pure_imag = $circle_type::from((0f32, 1i64));
            let sin_i = pure_imag.sin();
            // sin(i) = i * sinh(1) - purely imaginary result
            assert!(sin_i.r().magnitude() < 0.1);
            assert!(sin_i.i() != 0f64);
        };
    }

    // Macro to test hyperbolic functions for any circle type
    macro_rules! test_hyperbolic_for_type {
        ($circle_type:ident) => {
            let zero = $circle_type::ZERO;
            let one = $circle_type::ONE;

            // Hyperbolic sine: sinh(0) = 0
            let sinh_0 = zero.sinh();
            assert!(sinh_0.r().magnitude() < 0.1);
            assert!(sinh_0.i().magnitude() < 0.1);

            // Hyperbolic cosine: cosh(0) = 1
            let cosh_0 = zero.cosh();
            assert!((cosh_0.r() - 1u32).magnitude() < 0.1);
            assert!(cosh_0.i().magnitude() < 0.1);

            // Hyperbolic tangent: tanh(0) = 0
            let tanh_0 = zero.tanh();
            assert!(tanh_0.r().magnitude() < 0.1);
            assert!(tanh_0.i().magnitude() < 0.1);

            // Test with pure imaginary
            let pure_imag = $circle_type::from((0f32, 1f64));
            let sinh_i = pure_imag.sinh();
            // sinh(i) = i * sin(1) - purely imaginary
            assert!(sinh_i.r().magnitude() < 0.1);
            assert!(sinh_i.i() != 0i64);
        };
    }

    // Generate power function tests for all circle types
    macro_rules! generate_power_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_power_functions_ $circle_type:lower>]() {
                        test_power_functions_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate exp/log tests for all circle types
    macro_rules! generate_exp_log_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_exp_log_ $circle_type:lower>]() {
                        test_exp_log_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate trigonometric tests for all circle types
    macro_rules! generate_trig_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_trig_ $circle_type:lower>]() {
                        test_trig_functions_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate hyperbolic tests for all circle types
    macro_rules! generate_hyperbolic_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_hyperbolic_ $circle_type:lower>]() {
                        test_hyperbolic_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_power_tests);
    all_circle_types!(generate_exp_log_tests);
    all_circle_types!(generate_trig_tests);
    all_circle_types!(generate_hyperbolic_tests);
}

#[cfg(test)]
mod comparison_and_utility {
    use super::*;

    // Macro to test equality operations for any circle type (no ordering for complex numbers)
    macro_rules! test_equality_for_type {
        ($circle_type:ident) => {
            let a = $circle_type::from((5u8, 3i16));
            let b = $circle_type::from((5f32, 3f64));
            let c = $circle_type::from((8i32, -2i64));

            // Equality with mixed types
            assert!(a == b);
            assert!(a != c);

            // Self-equality
            assert!(a == a);
            assert!(c == c);

            // Zero equality
            let zero = $circle_type::ZERO;
            assert!(zero == $circle_type::from((0u8, 0i16)));
            assert!($circle_type::from((0f32, 0f64)) == zero);
        };
    }

    // Macro to test state checking for any circle type
    macro_rules! test_state_checking_for_type {
        ($circle_type:ident) => {
            let normal = $circle_type::from((42u64, 17i32));
            let zero = $circle_type::ZERO;
            let exploded = $circle_type::MAX * 2f32;
            let vanished = $circle_type::MIN_POS / 1000f64;
            let undefined = $circle_type::INFINITY / $circle_type::INFINITY;

            // Normal value checks
            assert!(normal.is_normal());
            assert!(normal.is_finite());
            assert!(!normal.is_zero());
            assert!(!normal.vanished());
            assert!(!normal.exploded());
            assert!(!normal.is_undefined());

            // Zero checks
            assert!(!zero.is_normal());
            assert!(zero.is_finite());
            assert!(zero.is_zero());
            assert!(zero.is_negligible());

            // Exploded checks
            assert!(!exploded.is_normal());
            assert!(!exploded.is_finite());
            assert!(exploded.exploded());

            // Vanished checks
            assert!(!vanished.is_normal());
            assert!(!vanished.is_finite());
            assert!(vanished.vanished());
            assert!(vanished.is_negligible());

            // Undefined checks
            assert!(!undefined.is_normal());
            assert!(!undefined.is_finite());
            assert!(undefined.is_undefined());
        };
    }

    // Generate equality tests for all circle types
    macro_rules! generate_equality_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_equality_ $circle_type:lower>]() {
                        test_equality_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    // Generate state checking tests for all circle types
    macro_rules! generate_state_checking_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_state_checking_ $circle_type:lower>]() {
                        test_state_checking_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_equality_tests);
    all_circle_types!(generate_state_checking_tests);
}

#[cfg(test)]
mod bitwise_operations {
    use super::*;

    // Macro to test bitwise operations for any circle type
    macro_rules! test_bitwise_for_type {
        ($circle_type:ident) => {
            let a = $circle_type::from((42i32, 51i32)); // 0b101010, 0b110011
            let b = $circle_type::from((85i32, 102i32)); // 0b1010101, 0b1100110

            // Bitwise AND: (42 & 85, 51 & 102) = (0, 34)
            let and_result = a & b;
            assert!(and_result.r() == 0i32);
            assert!(and_result.i() == 34i32);

            // Bitwise OR: (42 | 85, 51 | 102) = (127, 119)
            let or_result = a | b;
            assert!(or_result.r() == 127i32);
            assert!(or_result.i() == 119i32);

            // Bitwise XOR: (42 ^ 85, 51 ^ 102) = (127, 85)
            let xor_result = a ^ b;
            assert!(xor_result.r() == 127i32);
            assert!(xor_result.i() == 85i32);

            // Test commutativity
            assert!(a & b == b & a);
            assert!(a | b == b | a);
            assert!(a ^ b == b ^ a);

            // Test with zero
            let zero = $circle_type::from((0i32, 0i32));
            assert!(a & zero == zero);
            assert!(a | zero == a);
            assert!(a ^ zero == a);

            // Test with all ones pattern
            let ones = $circle_type::from((-1i32, -1i32));
            assert!(a & ones == a);
            assert!(a | ones == ones);

            // Left shift: (42 << 1, 51 << 1) = (84, 102)
            let left_shift = a << 1i32;
            assert!(left_shift.r() == 84i32);
            assert!(left_shift.i() == 102i32);

            // Right shift: (84 >> 1, 102 >> 1) = (42, 51)
            let right_shift = left_shift >> 1i32;
            assert!(right_shift.r() == 42i32);
            assert!(right_shift.i() == 51i32);
            assert!(right_shift == a);
        };
    }

    // Generate bitwise tests for all circle types
    macro_rules! generate_bitwise_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_bitwise_ $circle_type:lower>]() {
                        test_bitwise_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_bitwise_tests);
}

#[cfg(test)]
mod modular_operations {
    use super::*;

    // Macro to test modular operations for any circle type
    macro_rules! test_modulo_for_type {
        ($circle_type:ident) => {
            // Basic modulus with Circle-Circle
            let a = $circle_type::from((17u8, 13i16));
            let b = $circle_type::from((5i16, 4f32));
            let remainder = a % b;
            // Complex modulus implementation varies, just test it doesn't crash
            assert!(remainder.is_normal() || remainder.is_zero() || remainder.is_undefined());

            // Component-wise modulo
            let component_mod = a.modulo(b);
            // Component-wise: (17 % 5, 13 % 4) = (2, 1)
            assert!(component_mod.r() == 2f32);
            assert!(component_mod.i() == 1f64);

            // Circle-Scalar modulus
            let scalar_mod = a % 3u64;
            assert!(scalar_mod.is_normal() || scalar_mod.is_zero() || scalar_mod.is_undefined());

            // Circle-Scalar component-wise modulo
            let component_scalar_mod = a.modulo(3i32);
            // (17 % 3, 13 % 3) = (2, 1)
            assert!(component_scalar_mod.r() == 2i128);
            assert!(component_scalar_mod.i() == 1f32);

            // Zero cases
            let zero = $circle_type::ZERO;

            // 0 % anything = 0
            assert!((zero % $circle_type::from((5u16, 3i32))).is_zero());
            assert!((zero % 7f64).is_zero());

            // anything % 0 = 0 (special Spirix behavior)
            assert!(($circle_type::from((42, 17)) % zero).is_zero());
            assert!(($circle_type::from((17f32, 23i64)) % zero).is_zero());

            // Special escaped value cases
            let exploded = $circle_type::MAX * 2usize;
            let vanished = $circle_type::MIN_POS / 1000isize;

            // Exploded % anything = undefined or exploded
            let exploded_mod = exploded % $circle_type::from((5f32, 3f64));
            assert!(exploded_mod.is_undefined() || exploded_mod.exploded());

            // anything % vanished = undefined
            let mod_vanished = $circle_type::from((42u32, 17i8)) % vanished;
            assert!(mod_vanished.is_undefined());

            // Exploded/vanished % 0 = 0 (zero takes precedence)
            assert!((exploded % zero).is_zero());
            assert!((vanished % zero).is_zero());

            // Undefined propagation
            let undefined = $circle_type::INFINITY / $circle_type::INFINITY;
            assert!((undefined % $circle_type::from((5, 3))).is_undefined());
            assert!(($circle_type::from((5, 3)) % undefined).is_undefined());
        };
    }

    // Generate modular tests for all circle types
    macro_rules! generate_modular_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_modulo_ $circle_type:lower>]() {
                        test_modulo_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_modular_tests);
}

#[cfg(test)]
mod random_testing {
    use super::*;

    // Macro to test random generation for any circle type
    macro_rules! test_random_for_type {
        ($circle_type:ident) => {
            const SAMPLE_SIZE: usize = 50; // Smaller sample for complex numbers
            let mut uniform_samples = Vec::with_capacity(SAMPLE_SIZE);
            let mut gauss_samples = Vec::with_capacity(SAMPLE_SIZE);

            // Collect samples for statistical analysis
            for _ in 0..SAMPLE_SIZE {
                let random_val = $circle_type::random();
                assert!(random_val.is_normal() || random_val.is_zero() || random_val.vanished());

                if random_val.is_normal() {
                    uniform_samples.push(random_val);
                }

                let random_gauss = $circle_type::random_gauss();
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
                // Test that uniform samples are within unit circle (for uniform distribution)
                let mut within_unit_circle = 0u32;
                for &sample in &uniform_samples {
                    if sample.magnitude() <= 1.5 {
                        // Allow some tolerance
                        within_unit_circle += 1;
                    }
                }

                // Most uniform samples should be reasonably sized
                let within_ratio = within_unit_circle as f64 / uniform_samples.len() as f64;
                assert!(
                    within_ratio > 0.3,
                    "Too many uniform samples outside reasonable range: {:.2}",
                    within_ratio
                );
            }

            // Test that random values have proper mixed type compatibility
            let rand_test = $circle_type::random();
            assert!(rand_test == rand_test); // Self-equality
            assert!(rand_test.magnitude() >= 0u8); // Magnitude is non-negative

            let gauss_test = $circle_type::random_gauss();
            let combined = rand_test + gauss_test;
            assert!(
                combined.is_normal()
                    || combined.is_zero()
                    || combined.exploded()
                    || combined.vanished()
                    || combined.is_undefined()
            );

            // Test complex-specific properties
            let complex_rand = $circle_type::random();
            let real_part = complex_rand.r();
            let imag_part = complex_rand.i();
            assert!(real_part.is_normal() || real_part.is_zero() || real_part.vanished());
            assert!(imag_part.is_normal() || imag_part.is_zero() || imag_part.vanished());
        };
    }

    // Generate random tests for all circle types
    macro_rules! generate_random_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_random_ $circle_type:lower>]() {
                        test_random_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_random_tests);
}

#[cfg(test)]
mod complex_specific {
    use super::*;

    // Macro to test Circle-specific operations for any circle type
    macro_rules! test_complex_operations_for_type {
        ($circle_type:ident) => {
            // Test conjugate
            let complex = $circle_type::from((3f32, 4i16));
            let conj = complex.conjugate();
            assert!(conj.r() == 3f64);
            assert!(conj.i() == -4i32);

            // Conjugate of conjugate is original
            assert!(conj.conjugate() == complex);

            // Conjugate of real number is itself
            let real_only = $circle_type::from(42f64);
            assert!(real_only.conjugate() == real_only);

            // Test magnitude and magnitude_squared
            let complex_3_4 = $circle_type::from((3f32, 4f64));
            let mag = complex_3_4.magnitude();
            let mag_sq = complex_3_4.magnitude_squared();

            assert!((mag - 5f32).magnitude() < 0.1);
            assert!((mag_sq - 25u64).magnitude() < 0.1);

            // Magnitude relationship: |z|² = z * z̄
            let product_with_conj = complex_3_4 * complex_3_4.conjugate();
            assert!((product_with_conj.r() - mag_sq).magnitude() < 0.1);
            assert!(product_with_conj.i().magnitude() < 0.1);

            // Test sign (unit vector)
            let unit = complex_3_4.sign();
            let unit_mag = unit.magnitude();
            assert!((unit_mag - 1f32).magnitude() < 0.1);

            // Sign of zero is undefined
            let zero_sign = $circle_type::ZERO.sign();
            assert!(zero_sign.is_undefined());

            // Test imaginary unit properties
            let i = $circle_type::POS_I;
            let neg_i = $circle_type::NEG_I;

            // i² = -1
            assert!(i.square().r() == -1f32);
            assert!(i.square().i().magnitude() < 0.1);

            // (-i)² = -1
            assert!(neg_i.square().r() == -1f64);
            assert!(neg_i.square().i().magnitude() < 0.1);

            // i * (-i) = 1
            let i_times_neg_i = i * neg_i;
            assert!((i_times_neg_i.r() - 1f32).magnitude() < 0.1);
            assert!(i_times_neg_i.i().magnitude() < 0.1);

            // Conjugate of i is -i
            assert!(i.conjugate() == neg_i);
            assert!(neg_i.conjugate() == i);

            // Test Euler's formula: e^(iπ) = -1
            let i_pi = $circle_type::POS_I * $circle_type::PI;
            let euler = i_pi.exp();
            assert!((euler.r() + 1f32).magnitude() < 0.1);
            assert!(euler.i().magnitude() < 0.1);

            // Test real and imaginary part extraction
            let complex_test = $circle_type::from((7.5f32, -2.3f64));
            assert!(complex_test.r() == 7.5);
            assert!(complex_test.i() == -2.3);

            // Real part of real number
            let real_num = $circle_type::from(42i64);
            assert!(real_num.r() == 42u128);
            assert!(real_num.i().is_zero());
        };
    }

    // Generate complex-specific tests for all circle types
    macro_rules! generate_complex_specific_tests {
        ($($circle_type:ident),+) => {
            $(
                paste::paste! {
                    #[test]
                    fn [<test_complex_operations_ $circle_type:lower>]() {
                        test_complex_operations_for_type!($circle_type);
                    }
                }
            )+
        };
    }

    all_circle_types!(generate_complex_specific_tests);
}
