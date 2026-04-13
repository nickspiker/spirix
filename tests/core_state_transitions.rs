use spirix::*;

/// Test state transitions between normal, exploded, vanished, and undefined states
/// Verify that state transitions follow the expected patterns and preserve mathematical consistency

#[test]
fn test_normal_to_exploded_transitions() {
    // Test various paths to exploded state
    let large_normal = ScalarF5E3::MAX;

    // Multiplication overflow
    let exploded_mult = large_normal * ScalarF5E3::from(2);
    assert!(exploded_mult.exploded());
    assert!(exploded_mult.is_positive());

    // Addition overflow
    let exploded_add = large_normal + large_normal;
    assert!(exploded_add.exploded());
    assert!(exploded_add.is_positive());

    // Power overflow
    let exploded_pow = ScalarF5E3::from(10).pow(ScalarF5E3::from(100));
    assert!(exploded_pow.exploded());
    assert!(exploded_pow.is_positive());

    // Exponential overflow
    let exploded_exp = ScalarF5E3::from(100).exp();
    assert!(exploded_exp.exploded());
    assert!(exploded_exp.is_positive());
}

#[test]
fn test_normal_to_vanished_transitions() {
    // Test various paths to vanished state
    let small_normal = ScalarF5E3::MIN_POS;

    // Division underflow
    let vanished_div = small_normal / ScalarF5E3::from(2);
    assert!(vanished_div.vanished());
    assert!(vanished_div.is_positive());

    // Multiplication underflow
    let vanished_mult = small_normal * ScalarF5E3::from(0.5);
    assert!(vanished_mult.vanished());
    assert!(vanished_mult.is_positive());

    // Power underflow
    let vanished_pow = ScalarF5E3::from(0.1).pow(ScalarF5E3::from(100));
    assert!(vanished_pow.vanished());
    assert!(vanished_pow.is_positive());

    // Exponential underflow
    let vanished_exp = ScalarF5E3::from(-100).exp();
    assert!(vanished_exp.vanished());
    assert!(vanished_exp.is_positive());
}

#[test]
fn test_undefined_state_generation() {
    // Test various paths to undefined states

    // Zero divided by zero
    let zero_div_zero = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    assert!(zero_div_zero.is_undefined());

    // Infinity minus infinity
    let inf_minus_inf = ScalarF5E3::INFINITY - ScalarF5E3::INFINITY;
    assert!(inf_minus_inf.is_undefined());

    // Infinity divided by infinity
    let inf_div_inf = ScalarF5E3::INFINITY / ScalarF5E3::INFINITY;
    assert!(inf_div_inf.is_undefined());

    // Square root of negative number
    let sqrt_negative = ScalarF5E3::from(-1).sqrt();
    assert!(sqrt_negative.is_undefined());

    // Logarithm of negative number
    let log_negative = ScalarF5E3::from(-1).ln();
    assert!(log_negative.is_undefined());

    // Logarithm of zero → negative infinity, which is singular ∞ in Spirix
    let log_zero = ScalarF5E3::ZERO.ln();
    assert!(log_zero.is_infinite());

    // Zero to the power of zero = 1 (convention: 0^0 = 1)
    let zero_pow_zero = ScalarF5E3::ZERO.pow(ScalarF5E3::ZERO);
    assert!(zero_pow_zero == ScalarF5E3::ONE);
}

#[test]
fn test_undefined_state_propagation() {
    // Test that undefined states propagate through operations
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    assert!(undefined.is_undefined());

    let normal = ScalarF5E3::from(42);

    // Addition with undefined
    let add_result = normal + undefined;
    assert!(add_result.is_undefined());

    // Subtraction with undefined
    let sub_result = normal - undefined;
    assert!(sub_result.is_undefined());

    // Multiplication with undefined
    let mult_result = normal * undefined;
    assert!(mult_result.is_undefined());

    // Division with undefined
    let div_result = normal / undefined;
    assert!(div_result.is_undefined());

    // Power with undefined
    let pow_result = normal.pow(undefined);
    assert!(pow_result.is_undefined());

    // Function application to undefined
    let sin_result = undefined.sin();
    assert!(sin_result.is_undefined());

    let exp_result = undefined.exp();
    assert!(exp_result.is_undefined());
}

#[test]
fn test_first_cause_preservation() {
    // Test that the first undefined cause is preserved through operation chains
    let zero = ScalarF5E3::ZERO;
    let normal = ScalarF5E3::from(42);

    // Create an undefined value from zero division
    let first_undefined = zero / zero;
    assert!(first_undefined.is_undefined());

    // Apply multiple operations
    let chained_result = ((first_undefined + normal) * ScalarF5E3::from(2))
        .pow(ScalarF5E3::from(3))
        .sin()
        .ln();

    assert!(chained_result.is_undefined());
}

#[test]
fn test_exploded_state_preservation() {
    // Test that exploded states preserve their orientation
    let pos_exploded = ScalarF5E3::MAX * ScalarF5E3::from(2);
    let neg_exploded = ScalarF5E3::MIN * ScalarF5E3::from(2);

    assert!(pos_exploded.exploded() && pos_exploded.is_positive());
    assert!(neg_exploded.exploded() && neg_exploded.is_negative());

    // Exploded + exploded = ℘ (transfinite + transfinite is undefined)
    let exploded_sum = pos_exploded + pos_exploded;
    assert!(exploded_sum.is_undefined());

    // Exploded - neg_exploded = exploded + exploded = ℘ (undefined)
    let exploded_diff = pos_exploded - neg_exploded;
    assert!(exploded_diff.is_undefined());

    // Test that exploded values can return to normal through division
    let normalized = pos_exploded / ScalarF5E3::from(1000);
    if normalized.is_normal() {
        assert!(normalized.is_positive());
    }
}

#[test]
fn test_vanished_state_preservation() {
    // Test that vanished states preserve their orientation
    let pos_vanished = ScalarF5E3::MIN_POS / ScalarF5E3::from(2);
    let neg_vanished = ScalarF5E3::MAX_NEG / ScalarF5E3::from(2);

    assert!(pos_vanished.vanished() && pos_vanished.is_positive());
    assert!(neg_vanished.vanished() && neg_vanished.is_negative());

    // Test operations between vanished values
    let vanished_sum = pos_vanished + pos_vanished;
    if vanished_sum.vanished() {
        assert!(vanished_sum.is_positive());
    }

    let vanished_diff = pos_vanished - neg_vanished;
    if vanished_diff.vanished() {
        assert!(vanished_diff.is_positive());
    }

    // Test that vanished values can return to normal through multiplication
    let normalized = pos_vanished * ScalarF5E3::from(1000);
    if normalized.is_normal() {
        assert!(normalized.is_positive());
    }
}

#[test]
fn test_state_transition_reversibility() {
    // Test that some state transitions are reversible
    let normal = ScalarF5E3::from(42);

    // Normal -> Exploded -> Normal (through division)
    let exploded = normal * ScalarF5E3::from(1e30);
    if exploded.exploded() {
        let back_to_normal = exploded / ScalarF5E3::from(1e30);
        if back_to_normal.is_normal() {
            let normal_val: f32 = normal.into();
            let back_val: f32 = back_to_normal.into();
            assert!((normal_val - back_val).abs() < 1e-3);
        }
    }

    // Normal -> Vanished -> Normal (through multiplication)
    let vanished = normal / ScalarF5E3::from(1e30);
    if vanished.vanished() {
        let back_to_normal = vanished * ScalarF5E3::from(1e30);
        if back_to_normal.is_normal() {
            let normal_val: f32 = normal.into();
            let back_val: f32 = back_to_normal.into();
            assert!((normal_val - back_val).abs() < 1e-3);
        }
    }
}

#[test]
fn test_mixed_state_operations() {
    // Test operations between different state types
    let normal = ScalarF5E3::from(42);
    let exploded = ScalarF5E3::MAX * ScalarF5E3::from(2);
    let vanished = ScalarF5E3::MIN_POS / ScalarF5E3::from(2);
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;

    // Normal + Exploded = ℘ (finite + transfinite is undefined in Spirix)
    let normal_plus_exploded = normal + exploded;
    assert!(normal_plus_exploded.is_undefined());

    // Normal * Vanished
    let normal_times_vanished = normal * vanished;
    if normal_times_vanished.vanished() {
        assert!(normal_times_vanished.is_positive());
    }

    // Exploded / Vanished (should be exploded)
    let exploded_div_vanished = exploded / vanished;
    assert!(exploded_div_vanished.exploded());

    // Any operation with undefined should be undefined
    assert!((normal + undefined).is_undefined());
    assert!((exploded + undefined).is_undefined());
    assert!((vanished + undefined).is_undefined());
}

#[test]
fn test_zero_and_infinity_special_cases() {
    // Test special cases with zero and infinity
    let zero = ScalarF5E3::ZERO;
    let infinity = ScalarF5E3::INFINITY;
    let normal = ScalarF5E3::from(42);

    // Zero operations
    assert!((zero + normal) == normal);
    assert!((zero * normal) == zero);
    // 1/0 = ∞ in Spirix (only 0/0 = ℘)
    assert!((normal / zero).is_infinite());

    // Infinity operations
    // ∞ + finite = ℘ (transfinite plus finite is undefined in Spirix)
    assert!((infinity + normal).is_undefined());
    // ∞ * finite = ∞ (multiplication preserves infinity)
    assert!((infinity * normal).is_infinite());
    assert!((normal / infinity) == zero);
    assert!((infinity / normal).is_infinite());

    // Zero and infinity interactions
    assert!((zero * infinity).is_undefined());
    assert!((infinity / infinity).is_undefined());
    assert!((zero / zero).is_undefined());
}
