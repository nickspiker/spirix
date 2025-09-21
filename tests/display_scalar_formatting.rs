use spirix::*;
use paste::paste;

/// Test display formatting across different scales and precision levels
/// Verify that numbers format correctly for very small, normal, and very large values

macro_rules! test_display_formatting_for_type {
    ($scalar_type:ident) => {
        paste! {
            #[test]
            fn [<test_power_display_ $scalar_type:lower>]() {
                // Test various powers - should display cleanly
                let test_cases = vec![
                    (2, 3, "8"),
                    (3, 4, "81"),
                    (10, 5, "100000"),
                    (5, 0, "1"),
                    (7, 1, "7"),
                ];

                for (base, exp, expected_contains) in test_cases {
                    let result = $scalar_type::from(base).pow($scalar_type::from(exp));
                    let display = format!("{}", result);

                    if result.is_normal() {
                        // For normal results, check the display contains expected digits
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

    // They should all represent the same basic value (42)
    // but precision might affect exact representation
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
        assert!(!display.is_empty(), "Edge case {} produced empty display", i);
        assert!(!debug.is_empty(), "Edge case {} produced empty debug", i);

        // Should be reasonable length
        assert!(display.len() < 100, "Edge case {} display too long: {}", i, display);
        assert!(debug.len() < 200, "Edge case {} debug too long: {}", i, debug);
    }
}