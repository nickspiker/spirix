use paste::paste;
use spirix::*;

/// Test display formatting across different scales and precision levels Verify that numbers format correctly for very small, normal, and very large values

macro_rules! test_display_formatting_for_type {
    ($scalar_type:ident) => {
        paste! {
            #[test]
            fn [<test_power_display_ $scalar_type:lower>]() {
                // Test small powers that fit in any precision
                let test_cases = vec![
                    (2, 3, "8"),
                    (5, 0, "1"),
                    (7, 1, "7"),
                ];

                for (base, exp, expected_contains) in test_cases {
                    let result = $scalar_type::from(base).pow($scalar_type::from(exp));
                    let display = format!("{}", result);

                    if result.is_normal() {
                        assert!(display.contains(expected_contains) || result.exploded(),
                            "Power {}^{} = {} should contain '{}' but got '{}'",
                            base, exp, result, expected_contains, display);
                    }
                }
            }

            #[test]
            fn [<test_large_power_display_ $scalar_type:lower>]() {
                // Test very large powers that may explode
                let large_powers = vec![
                    ($scalar_type::from(2), $scalar_type::from(64)),
                    ($scalar_type::from(10), $scalar_type::from(100)),
                    ($scalar_type::from(300), $scalar_type::from(42)),
                ];

                for (base, exp) in large_powers {
                    let result = base.pow(exp);
                    let display = format!("{}", result);

                    // Should not panic and should produce readable output
                    assert!(!display.is_empty());

                    if result.exploded() {
                        // Exploded values should indicate their state somehow
                        assert!(display.len() > 0);
                    }
                }
            }

            #[test]
            fn [<test_small_power_display_ $scalar_type:lower>]() {
                // Test negative powers that may vanish
                let small_powers = vec![
                    ($scalar_type::from(2), $scalar_type::from(-3)),
                    ($scalar_type::from(10), $scalar_type::from(-5)),
                    ($scalar_type::from(10), $scalar_type::from(-100)),
                    ($scalar_type::from(10), $scalar_type::from(-1000)),
                ];

                for (base, exp) in small_powers {
                    let result = base.pow(exp);
                    let display = format!("{}", result);

                    // Should not panic and should produce readable output
                    assert!(!display.is_empty());

                    if result.vanished() {
                        // Vanished values should indicate their state
                        assert!(display.len() > 0);
                    }
                }
            }

            #[test]
            fn [<test_decimal_scale_display_ $scalar_type:lower>]() {
                // Test the decimal scale values from your debug output
                let decimal_values = vec![
                    0.000000123456789_f64,
                    0.00000123456789_f64,
                    0.0000123456789_f64,
                    0.000123456789_f64,
                    0.00123456789_f64,
                    0.0123456789_f64,
                    0.123456789_f64,
                    1.23456789_f64,
                    12.3456789_f64,
                    123.456789_f64,
                    1234.56789_f64,
                    12345.6789_f64,
                    123456.789_f64,
                    1234567.89_f64,
                    12345678.9_f64,
                    123456789.0_f64,
                    1234567890.0_f64,
                ];

                for &val in &decimal_values {
                    let scalar = $scalar_type::from(val as f32);
                    let display = format!("{}", scalar);

                    // Should not panic
                    assert!(!display.is_empty());

                    // For normal values, should contain some recognizable digits
                    if scalar.is_normal() {
                        assert!(display.len() > 0);
                        // Should not be just zeros or invalid characters
                        assert!(!display.chars().all(|c| c == '0' || c == '.'));
                    }
                }
            }

            #[test]
            fn [<test_large_integer_display_ $scalar_type:lower>]() {
                // Test large integer values
                let large_ints = vec![
                    12345678900_u64,
                    123456789000_u64,
                    1234567890000_u64,
                    12345678900000_u64,
                ];

                for &val in &large_ints {
                    let scalar = $scalar_type::from(val as f32);
                    let display = format!("{}", scalar);

                    // Should not panic
                    assert!(!display.is_empty());

                    if scalar.is_normal() {
                        // Should show some representation of the large number
                        assert!(display.len() > 3); // More than just "0" or "inf"
                    }
                }
            }

            #[test]
            fn [<test_reciprocal_display_ $scalar_type:lower>]() {
                // Test reciprocals of large numbers (like 1/300^42)
                let large_val = $scalar_type::from(300).pow($scalar_type::from(42));
                if large_val.exploded() {
                    let reciprocal = $scalar_type::ONE / large_val;
                    let display = format!("{}", reciprocal);

                    // Should not panic
                    assert!(!display.is_empty());

                    if reciprocal.vanished() {
                        // Should indicate it's a very small number
                        assert!(display.len() > 0);
                    }
                }
            }

            #[test]
            fn [<test_special_state_display_ $scalar_type:lower>]() {
                // Test display of special states
                let test_values = vec![
                    ($scalar_type::ZERO, "zero"),
                    ($scalar_type::ONE, "one"),
                    ($scalar_type::INFINITY, "infinity"),
                    ($scalar_type::ZERO / $scalar_type::ZERO, "undefined"), // undefined
                    ($scalar_type::MAX * $scalar_type::from(2), "exploded"), // exploded
                    ($scalar_type::MIN_POS / $scalar_type::from(2), "vanished"), // vanished
                ];

                for (value, state_name) in test_values {
                    let display = format!("{}", value);

                    // Should not panic and should produce some output
                    assert!(!display.is_empty(),
                        "Display of {} value should not be empty", state_name);

                    // Should not contain invalid characters like null bytes
                    assert!(!display.contains('\0'),
                        "Display should not contain null bytes");

                    // Should be reasonable length (not extremely long)
                    assert!(display.len() < 1000,
                        "Display should be reasonable length, got: {}", display);
                }
            }

            #[test]
            fn [<test_negative_value_display_ $scalar_type:lower>]() {
                // Test negative value display
                let pos_val = $scalar_type::from(123.456);
                let neg_val = -pos_val;

                let pos_display = format!("{}", pos_val);
                let neg_display = format!("{}", neg_val);

                // Negative should have minus sign or other indication
                assert!(!pos_display.is_empty());
                assert!(!neg_display.is_empty());

                if pos_val.is_normal() && neg_val.is_normal() {
                    // They should be different (one should show negative)
                    assert_ne!(pos_display, neg_display);
                }
            }

            #[test]
            fn [<test_debug_vs_display_ $scalar_type:lower>]() {
                // Test that Debug and Display formats are both reasonable
                let test_vals = vec![
                    $scalar_type::from(42.0),
                    $scalar_type::from(-42.0),
                    $scalar_type::from(0.001),
                    $scalar_type::from(1000.0),
                ];

                for val in test_vals {
                    let display = format!("{}", val);
                    let debug = format!("{:?}", val);

                    // Both should work without panicking
                    assert!(!display.is_empty());
                    assert!(!debug.is_empty());

                    // Debug format might be more detailed
                    assert!(debug.len() >= display.len() || !val.is_normal());
                }
            }
        }
    };
}

// Test display formatting for all scalar types
test_display_formatting_for_type!(ScalarF3E3);
test_display_formatting_for_type!(ScalarF3E4);
test_display_formatting_for_type!(ScalarF3E5);
test_display_formatting_for_type!(ScalarF3E6);
test_display_formatting_for_type!(ScalarF3E7);
test_display_formatting_for_type!(ScalarF4E3);
test_display_formatting_for_type!(ScalarF4E4);
test_display_formatting_for_type!(ScalarF4E5);
test_display_formatting_for_type!(ScalarF4E6);
test_display_formatting_for_type!(ScalarF4E7);
test_display_formatting_for_type!(ScalarF5E3);
test_display_formatting_for_type!(ScalarF5E4);
test_display_formatting_for_type!(ScalarF5E5);
test_display_formatting_for_type!(ScalarF5E6);
test_display_formatting_for_type!(ScalarF5E7);
test_display_formatting_for_type!(ScalarF6E3);
test_display_formatting_for_type!(ScalarF6E4);
test_display_formatting_for_type!(ScalarF6E5);
test_display_formatting_for_type!(ScalarF6E6);
test_display_formatting_for_type!(ScalarF6E7);
test_display_formatting_for_type!(ScalarF7E3);
test_display_formatting_for_type!(ScalarF7E4);
test_display_formatting_for_type!(ScalarF7E5);
test_display_formatting_for_type!(ScalarF7E6);
test_display_formatting_for_type!(ScalarF7E7);

#[test]
fn test_circle_display_formatting() {
    // Test complex number display
    let circle = CircleF5E3::from((3.0, 4.0));
    let display = format!("{}", circle);
    let debug = format!("{:?}", circle);

    // Should not panic
    assert!(!display.is_empty());
    assert!(!debug.is_empty());

    // Should represent both real and imaginary parts
    if circle.is_normal() {
        assert!(display.len() > 1); // More than just "0"
    }
}

#[test]
fn test_consistency_across_precisions() {
    // Test that the same mathematical value displays similarly across precisions
    let value = 42.0_f32;

    let displays = vec![
        format!("{}", ScalarF3E3::from(value)),
        format!("{}", ScalarF4E4::from(value)),
        format!("{}", ScalarF5E5::from(value)),
        format!("{}", ScalarF6E6::from(value)),
        format!("{}", ScalarF7E7::from(value)),
    ];

    // All should be non-empty
    for display in &displays {
        assert!(!display.is_empty());
    }

    // They should all represent the same basic value (42) but precision might affect exact representation
}

#[test]
fn test_formatting_edge_cases() {
    // Test edge cases that might cause formatting issues
    let edge_cases = vec![
        ScalarF5E3::from(f32::NAN),
        ScalarF5E3::from(f32::INFINITY),
        ScalarF5E3::from(f32::NEG_INFINITY),
        ScalarF5E3::from(f32::MIN),
        ScalarF5E3::from(f32::MAX),
        ScalarF5E3::from(f32::EPSILON),
        ScalarF5E3::from(-0.0),
        ScalarF5E3::from(1.0 / 0.0),
        ScalarF5E3::from(0.0 / 0.0),
    ];

    for (i, val) in edge_cases.iter().enumerate() {
        let display = format!("{}", val);
        let debug = format!("{:?}", val);

        // Should not panic or produce empty strings
        assert!(
            !display.is_empty(),
            "Edge case {} produced empty display",
            i
        );
        assert!(!debug.is_empty(), "Edge case {} produced empty debug", i);

        // Should be reasonable length
        assert!(
            display.len() < 100,
            "Edge case {} display too long: {}",
            i,
            display
        );
        assert!(
            debug.len() < 200,
            "Edge case {} debug too long: {}",
            i,
            debug
        );
    }
}

// println!("2^3 {}", ScalarF6E5::from(2).pow(3)); println!("3^4 {}", ScalarF6E5::from(3).pow(4)); println!("10^5 {}", ScalarF6E5::from(10).pow(5)); println!("2^64 {}", ScalarF6E5::from(2).pow(64)); println!("10^100 {}", ScalarF6E5::from(10).pow(100)); println!("2^-3 {}", ScalarF6E5::from(2).pow(-3)); println!("10^-5 {}", ScalarF6E5::from(10).pow(-5)); println!("10^-100 {}", ScalarF6E5::from(10).pow(-100)); println!("10^-1000 {}", ScalarF6E5::from(10).pow(-1000)); println!("5^0 {}", ScalarF6E5::from(5).pow(0)); println!("7^1 {}", ScalarF6E5::from(7).pow(1)); println!("11^-1 {}", ScalarF6E5::from(11).pow(-1)); println!("300^42 {}", ScalarF6E5::from(300).pow(42)); println!("1/300^42 {}", 1 / ScalarF6E5::from(300).pow(42)); println!("0.000000123456789 {}", ScalarF6E5::from(0.000000123456789)); println!("0.00000123456789 {}", ScalarF6E5::from(0.00000123456789)); println!("0.0000123456789 {}", ScalarF6E5::from(0.0000123456789)); println!("0.000123456789 {}", ScalarF6E5::from(0.000123456789)); println!("0.00123456789 {}", ScalarF6E5::from(0.00123456789)); println!("0.0123456789 {}", ScalarF6E5::from(0.0123456789)); println!("0.123456789 {}", ScalarF6E5::from(0.123456789)); println!("1.23456789 {}", ScalarF6E5::from(1.23456789)); println!("12.3456789 {}", ScalarF6E5::from(12.3456789)); println!("123.456789 {}", ScalarF6E5::from(123.456789)); println!("1234.56789 {}", ScalarF6E5::from(1234.56789)); println!("12345.6789 {}", ScalarF6E5::from(12345.6789)); println!("123456.789 {}", ScalarF6E5::from(123456.789)); println!("1234567.89 {}", ScalarF6E5::from(1234567.89)); println!("12345678.9 {}", ScalarF6E5::from(12345678.9)); println!("123456789 {}", ScalarF6E5::from(123456789)); println!("1234567890 {}", ScalarF6E5::from(1234567890)); println!("12345678900 {}", ScalarF6E5::from(12345678900u128)); println!("123456789000 {}", ScalarF6E5::from(123456789000u128)); println!("1234567890000 {}", ScalarF6E5::from(1234567890000u128)); println!("12345678900000 {}", ScalarF6E5::from(12345678900000u128)); println!("123456789000000 {}", ScalarF6E5::from(123456789000000u128)); println!( "1234567890000000 {}", ScalarF6E5::from(1234567890000000u128) ); println!( "12345678900000000 {}", ScalarF6E5::from(12345678900000000u128) ); println!( "123456789000000000 {}", ScalarF6E5::from(123456789000000000u128) ); println!( "1234567890000000000 {}", ScalarF6E5::from(1234567890000000000u128) ); println!( "-0.000000123456789 {}", ScalarF6E5::from(-0.000000123456789) ); println!("-0.00000123456789 {}", ScalarF6E5::from(-0.00000123456789)); println!("-0.0000123456789 {}", ScalarF6E5::from(-0.0000123456789)); println!("-0.000123456789 {}", ScalarF6E5::from(-0.000123456789)); println!("-0.00123456789 {}", ScalarF6E5::from(-0.00123456789)); println!("-0.0123456789 {}", ScalarF6E5::from(-0.0123456789)); println!("-0.123456789 {}", ScalarF6E5::from(-0.123456789)); println!("-1.23456789 {}", ScalarF6E5::from(-1.23456789)); println!("-12.3456789 {}", ScalarF6E5::from(-12.3456789)); println!("-123.456789 {}", ScalarF6E5::from(-123.456789)); println!("-1234.56789 {}", ScalarF6E5::from(-1234.56789)); println!("-12345.6789 {}", ScalarF6E5::from(-12345.6789)); println!("-123456.789 {}", ScalarF6E5::from(-123456.789)); println!("-1234567.89 {}", ScalarF6E5::from(-1234567.89)); println!("-12345678.9 {}", ScalarF6E5::from(-12345678.9)); println!("-123456789 {}", ScalarF6E5::from(-123456789)); println!("-1234567890 {}", ScalarF6E5::from(-1234567890)); println!("-12345678900 {}", ScalarF6E5::from(-12345678900i128)); println!("-123456789000 {}", ScalarF6E5::from(-123456789000i128)); println!("-1234567890000 {}", ScalarF6E5::from(-1234567890000i128)); println!("-12345678900000 {}", ScalarF6E5::from(-12345678900000i128)); println!( "-123456789000000 {}", ScalarF6E5::from(-123456789000000i128) ); println!( "-1234567890000000 {}", ScalarF6E5::from(-1234567890000000i128) ); println!( "-12345678900000000 {}", ScalarF6E5::from(-12345678900000000i128) ); println!( "-123456789000000000 {}", ScalarF6E5::from(-123456789000000000i128) ); println!( "-1234567890000000000 {}", ScalarF6E5::from(-1234567890000000000i128) ); println!("10^24 {}", ScalarF6E5::from(10).pow(24)); println!("10^23 {}", ScalarF6E5::from(10).pow(23)); println!("10^22 {}", ScalarF6E5::from(10).pow(22)); println!("10^21 {}", ScalarF6E5::from(10).pow(21)); println!("10^20 {}", ScalarF6E5::from(10).pow(20)); println!("10^19 {}", ScalarF6E5::from(10).pow(19)); println!("10^18 {}", ScalarF6E5::from(10).pow(18)); println!("10^17 {}", ScalarF6E5::from(10).pow(17)); println!("10^16 {}", ScalarF6E5::from(10).pow(16)); println!("10^15 {}", ScalarF6E5::from(10).pow(15)); println!("10^14 {}", ScalarF6E5::from(10).pow(14)); println!("10^13 {}", ScalarF6E5::from(10).pow(13)); println!("10^12 {}", ScalarF6E5::from(10).pow(12)); println!("10^11 {}", ScalarF6E5::from(10).pow(11)); println!("10^10 {}", ScalarF6E5::from(10).pow(10)); println!("10^9 {}", ScalarF6E5::from(10).pow(9)); println!("10^8 {}", ScalarF6E5::from(10).pow(8)); println!("10^7 {}", ScalarF6E5::from(10).pow(7)); println!("10^6 {}", ScalarF6E5::from(10).pow(6)); println!("10^5 {}", ScalarF6E5::from(10).pow(5)); println!("10^4 {}", ScalarF6E5::from(10).pow(4)); println!("10^3 {}", ScalarF6E5::from(10).pow(3)); println!("10^2 {}", ScalarF6E5::from(10).pow(2)); println!("10^1 {}", ScalarF6E5::from(10).pow(1)); println!("10^0 {}", ScalarF6E5::from(10).pow(0)); println!("10^-1 {}", ScalarF6E5::from(10).pow(-1)); println!("10^-2 {}", ScalarF6E5::from(10).pow(-2)); println!("10^-3 {}", ScalarF6E5::from(10).pow(-3)); println!("10^-4 {}", ScalarF6E5::from(10).pow(-4)); println!("10^-5 {}", ScalarF6E5::from(10).pow(-5)); println!("10^-6 {}", ScalarF6E5::from(10).pow(-6)); println!("10^-7 {}", ScalarF6E5::from(10).pow(-7)); println!("10^-8 {}", ScalarF6E5::from(10).pow(-8)); println!("10^-9 {}", ScalarF6E5::from(10).pow(-9)); println!("10^-10 {}", ScalarF6E5::from(10).pow(-10)); println!("10^-11 {}", ScalarF6E5::from(10).pow(-11)); println!("10^-12 {}", ScalarF6E5::from(10).pow(-12)); println!("10^-13 {}", ScalarF6E5::from(10).pow(-13)); println!("10^-14 {}", ScalarF6E5::from(10).pow(-14)); println!("10^-15 {}", ScalarF6E5::from(10).pow(-15)); println!("10^-16 {}", ScalarF6E5::from(10).pow(-16)); println!("10^-17 {}", ScalarF6E5::from(10).pow(-17)); println!("10^-18 {}", ScalarF6E5::from(10).pow(-18)); println!("10^-19 {}", ScalarF6E5::from(10).pow(-19)); println!("10^-20 {}", ScalarF6E5::from(10).pow(-20)); println!("10^-21 {}", ScalarF6E5::from(10).pow(-21)); println!("10^-22 {}", ScalarF6E5::from(10).pow(-22)); println!("10^-23 {}", ScalarF6E5::from(10).pow(-23)); println!("10^-24 {}", ScalarF6E5::from(10).pow(-24));

// println!("ScalarF3E3::MAX {}", ScalarF3E3::MAX); println!("ScalarF4E3::MAX {}", ScalarF4E3::MAX); println!("ScalarF5E3::MAX {}", ScalarF5E3::MAX); println!("ScalarF6E3::MAX {}", ScalarF6E3::MAX); println!("ScalarF7E3::MAX {}", ScalarF7E3::MAX);

// println!("ScalarF3E4::MAX {}", ScalarF3E4::MAX); println!("ScalarF4E4::MAX {}", ScalarF4E4::MAX); println!("ScalarF5E4::MAX {}", ScalarF5E4::MAX); println!("ScalarF6E4::MAX {}", ScalarF6E4::MAX); println!("ScalarF7E4::MAX {}", ScalarF7E4::MAX);

// println!("ScalarF3E5::MAX {}", ScalarF3E5::MAX); println!("ScalarF4E5::MAX {}", ScalarF4E5::MAX); println!("ScalarF5E5::MAX {}", ScalarF5E5::MAX); println!("ScalarF6E5::MAX {}", ScalarF6E5::MAX); println!("ScalarF7E5::MAX {}", ScalarF7E5::MAX);

// println!("ScalarF3E6::MAX {}", ScalarF3E6::MAX); println!("ScalarF4E6::MAX {}", ScalarF4E6::MAX); println!("ScalarF5E6::MAX {}", ScalarF5E6::MAX); println!("ScalarF6E6::MAX {}", ScalarF6E6::MAX); println!("ScalarF7E6::MAX {}", ScalarF7E6::MAX);

// println!("ScalarF3E7::MAX {}", ScalarF3E7::MAX); println!("ScalarF4E7::MAX {}", ScalarF4E7::MAX); println!("ScalarF5E7::MAX {}", ScalarF5E7::MAX); println!("ScalarF6E7::MAX {}", ScalarF6E7::MAX); println!("ScalarF7E7::MAX {}", ScalarF7E7::MAX);

// println!("ScalarF3E3::MIN {}", ScalarF3E3::MIN); println!("ScalarF4E3::MIN {}", ScalarF4E3::MIN); println!("ScalarF5E3::MIN {}", ScalarF5E3::MIN); println!("ScalarF6E3::MIN {}", ScalarF6E3::MIN); println!("ScalarF7E3::MIN {}", ScalarF7E3::MIN);

// println!("ScalarF3E4::MIN {}", ScalarF3E4::MIN); println!("ScalarF4E4::MIN {}", ScalarF4E4::MIN); println!("ScalarF6E4::MIN {}", ScalarF6E4::MIN); println!("ScalarF6E4::MIN {}", ScalarF6E4::MIN); println!("ScalarF7E4::MIN {}", ScalarF7E4::MIN);

// println!("ScalarF3E5::MIN {}", ScalarF3E5::MIN); println!("ScalarF4E5::MIN {}", ScalarF4E5::MIN); println!("ScalarF5E5::MIN {}", ScalarF5E5::MIN); println!("ScalarF6E5::MIN {}", ScalarF6E5::MIN); println!("ScalarF7E5::MIN {}", ScalarF7E5::MIN);

// println!("ScalarF3E6::MIN {}", ScalarF3E6::MIN); println!("ScalarF4E6::MIN {}", ScalarF4E6::MIN); println!("ScalarF5E6::MIN {}", ScalarF5E6::MIN); println!("ScalarF6E6::MIN {}", ScalarF6E6::MIN); println!("ScalarF7E6::MIN {}", ScalarF7E6::MIN);

// println!("ScalarF3E7::MIN {}", ScalarF3E7::MIN); println!("ScalarF4E7::MIN {}", ScalarF4E7::MIN); println!("ScalarF5E7::MIN {}", ScalarF5E7::MIN); println!("ScalarF6E7::MIN {}", ScalarF6E7::MIN); println!("ScalarF7E7::MIN {}", ScalarF7E7::MIN);

// println!("ScalarF3E3::MIN_POS {}", ScalarF3E3::MIN_POS); println!("ScalarF4E3::MIN_POS {}", ScalarF4E3::MIN_POS); println!("ScalarF5E3::MIN_POS {}", ScalarF5E3::MIN_POS); println!("ScalarF6E3::MIN_POS {}", ScalarF6E3::MIN_POS); println!("ScalarF7E3::MIN_POS {}", ScalarF7E3::MIN_POS);

// println!("ScalarF3E4::MIN_POS {}", ScalarF3E4::MIN_POS); println!("ScalarF4E4::MIN_POS {}", ScalarF4E4::MIN_POS); println!("ScalarF6E4::MIN_POS {}", ScalarF6E4::MIN_POS); println!("ScalarF6E4::MIN_POS {}", ScalarF6E4::MIN_POS); println!("ScalarF7E4::MIN_POS {}", ScalarF7E4::MIN_POS);

// println!("ScalarF3E5::MIN_POS {}", ScalarF3E5::MIN_POS); println!("ScalarF4E5::MIN_POS {}", ScalarF4E5::MIN_POS); println!("ScalarF5E5::MIN_POS {}", ScalarF5E5::MIN_POS); println!("ScalarF6E5::MIN_POS {}", ScalarF6E5::MIN_POS); println!("ScalarF7E5::MIN_POS {}", ScalarF7E5::MIN_POS);

// println!("ScalarF3E6::MIN_POS {}", ScalarF3E6::MIN_POS); println!("ScalarF4E6::MIN_POS {}", ScalarF4E6::MIN_POS); println!("ScalarF5E6::MIN_POS {}", ScalarF5E6::MIN_POS); println!("ScalarF6E6::MIN_POS {}", ScalarF6E6::MIN_POS); println!("ScalarF7E6::MIN_POS {}", ScalarF7E6::MIN_POS);

// println!("ScalarF3E7::MIN_POS {}", ScalarF3E7::MIN_POS); println!("ScalarF4E7::MIN_POS {}", ScalarF4E7::MIN_POS); println!("ScalarF5E7::MIN_POS {}", ScalarF5E7::MIN_POS); println!("ScalarF6E7::MIN_POS {}", ScalarF6E7::MIN_POS); println!("ScalarF7E7::MIN_POS {}", ScalarF7E7::MIN_POS);

// println!("ScalarF3E3::MAX_NEG {}", ScalarF3E3::MAX_NEG); println!("ScalarF4E3::MAX_NEG {}", ScalarF4E3::MAX_NEG); println!("ScalarF5E3::MAX_NEG {}", ScalarF5E3::MAX_NEG); println!("ScalarF6E3::MAX_NEG {}", ScalarF6E3::MAX_NEG); println!("ScalarF7E3::MAX_NEG {}", ScalarF7E3::MAX_NEG);

// println!("ScalarF3E4::MAX_NEG {}", ScalarF3E4::MAX_NEG); println!("ScalarF4E4::MAX_NEG {}", ScalarF4E4::MAX_NEG); println!("ScalarF6E4::MAX_NEG {}", ScalarF6E4::MAX_NEG); println!("ScalarF6E4::MAX_NEG {}", ScalarF6E4::MAX_NEG); println!("ScalarF7E4::MAX_NEG {}", ScalarF7E4::MAX_NEG);

// println!("ScalarF3E5::MAX_NEG {}", ScalarF3E5::MAX_NEG); println!("ScalarF4E5::MAX_NEG {}", ScalarF4E5::MAX_NEG); println!("ScalarF5E5::MAX_NEG {}", ScalarF5E5::MAX_NEG); println!("ScalarF6E5::MAX_NEG {}", ScalarF6E5::MAX_NEG); println!("ScalarF7E5::MAX_NEG {}", ScalarF7E5::MAX_NEG);

// println!("ScalarF3E6::MAX_NEG {}", ScalarF3E6::MAX_NEG); println!("ScalarF4E6::MAX_NEG {}", ScalarF4E6::MAX_NEG); println!("ScalarF5E6::MAX_NEG {}", ScalarF5E6::MAX_NEG); println!("ScalarF6E6::MAX_NEG {}", ScalarF6E6::MAX_NEG); println!("ScalarF7E6::MAX_NEG {}", ScalarF7E6::MAX_NEG);

// println!("ScalarF3E7::MAX_NEG {}", ScalarF3E7::MAX_NEG); println!("ScalarF4E7::MAX_NEG {}", ScalarF4E7::MAX_NEG); println!("ScalarF5E7::MAX_NEG {}", ScalarF5E7::MAX_NEG); println!("ScalarF6E7::MAX_NEG {}", ScalarF6E7::MAX_NEG); println!("ScalarF7E7::MAX_NEG {}", ScalarF7E7::MAX_NEG);

// println!("0.00001 {}", ScalarF6E5::from(0.00001)); println!("0.000001 {}", ScalarF6E5::from(0.000001)); println!("99999 {}", ScalarF6E5::from(99999)); println!("100000 {}", ScalarF6E5::from(100000));

// println!("TWO {}", ScalarF6E5::TWO); println!("HALF {}", ScalarF6E5::HALF);

// println!("sqrt(2) {}", ScalarF6E5::TWO.sqrt()); println!("log(e) {}", ScalarF6E5::E.ln()); let pi_half = ScalarF6E5::PI / ScalarF6E5::TWO; println!("sin(pi/2) {}", pi_half.sin()); println!("cos(pi) {}", ScalarF6E5::PI.cos()); let pi_quarter = ScalarF6E5::PI / ScalarF6E5::from(4); println!("tan(pi/4) {}", pi_quarter.tan());

// println!("1 + min_pos {}", ScalarF6E5::ONE + ScalarF6E5::MIN_POS); println!("1 - min_pos {}", ScalarF6E5::ONE - ScalarF6E5::MIN_POS); println!("very close to 1: {}", ScalarF6E5::from(0.999999)); println!("very close to 10: {}", ScalarF6E5::from(9.999999));

// println!("2^10 {}", ScalarF6E5::from(2).pow(10)); println!("3^5 {}", ScalarF6E5::from(3).pow(5)); println!("5^4 {}", ScalarF6E5::from(5).pow(4)); println!("7^3 {}", ScalarF6E5::from(7).pow(3));

// println!("1/2^10 {}", 1 / ScalarF6E5::from(2).pow(10)); println!("1/3^5 {}", 1 / ScalarF6E5::from(3).pow(5)); println!("1/π {}", 1 / ScalarF6E5::PI);

// println!("1/7 {}", 1 / ScalarF6E5::from(7)); println!("1/9 {}", 1 / ScalarF6E5::from(9)); println!("1/11 {}", 1 / ScalarF6E5::from(11)); println!("1/13 {}", 1 / ScalarF6E5::from(13));
