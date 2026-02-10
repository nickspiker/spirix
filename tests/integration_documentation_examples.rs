use approx::assert_relative_eq;
use spirix::*;

/// Tests that verify all code examples from documentation compile and work correctly
/// Ensures that documentation stays in sync with actual implementation

#[test]
fn test_readme_basic_examples() {
    // Example from README: Basic Scalar usage
    let a = ScalarF5E3::from(42);
    let mut b = ScalarF5E3::from(-1);
    b /= 12; // Divide -1 by 12 and assign to b

    assert!(a.is_normal());
    assert!(b.is_normal());

    let a_val: f32 = a.into();
    let b_val: f32 = b.into();
    assert_relative_eq!(a_val, 42.0, epsilon = 1e-6);
    assert_relative_eq!(b_val, -1.0 / 12.0, epsilon = 1e-6);
}

#[test]
fn test_readme_undefined_tracking_example() {
    // Example from README: Undefined state tracking
    let a = ScalarF5E3::from(42);

    // Track undefined states while preserving first cause
    let zero_div_zero = (a - 42) / 0;
    assert!(zero_div_zero.is_undefined());

    let b = ScalarF5E3::from(-1) / 12;
    let still_undefined_zero_div_zero = (zero_div_zero + b).pow(-5.71).ln();
    assert!(still_undefined_zero_div_zero.is_undefined());

    // First cause should be preserved through the operation chain
    // (Implementation detail: the undefined prefix should indicate division by zero)
}

#[test]
fn test_readme_escaped_values_example() {
    // Example from README: Escaped values preserve phase
    let exploded = ScalarF5E3::MAX * 2;
    assert!(exploded.exploded() && exploded.is_positive());

    // Vanished values preserve phase too!
    let vanished = ScalarF5E3::MAX_NEG / 3;
    assert!(vanished.vanished() && vanished.is_negative());

    // Absolute operations can be applied to escaped values
    assert!(vanished.square().is_positive());
}

#[test]
fn test_scalar_documentation_examples() {
    // Examples from scalar.rs documentation

    // Create a Scalar with 32-bit fraction and 8-bit exponent
    let a = Scalar::<i32, i8>::from(42);
    assert!(a.is_normal());

    // Using a type alias for the same size
    let mut b = ScalarF5E3::from(-1);
    b /= 12; // Divide -1 by 12 and assign to b
    assert!(b.is_normal());

    // Track undefined states while preserving first cause
    let zero_div_zero = (a - 42) / 0;
    assert!(zero_div_zero.is_undefined());

    let still_undefined_zero_div_zero = (zero_div_zero + b).pow(-5.71).ln();
    assert!(still_undefined_zero_div_zero.is_undefined());

    // Escaped values preserve phase
    let exploded = ScalarF5E3::MAX * 2;
    assert!(exploded.exploded() && exploded.is_positive());

    // Vanished values preserve phase too!
    let vanished = ScalarF5E3::MAX_NEG / 3;
    assert!(vanished.vanished() && vanished.is_negative());

    // Absolute operations can be applied to escaped values
    assert!(vanished.square().is_positive());
}

#[test]
fn test_circle_documentation_examples() {
    // Examples from circle.rs documentation

    // Create a Circle with 32-bit fractions and 8-bit exponent
    let c = Circle::<i32, i8>::new(Scalar::<i32, i8>::from(3), Scalar::<i32, i8>::from(4));
    assert!(c.is_normal());

    // Using type alias for convenience
    let circle = CircleF5E3::new(ScalarF5E3::from(3.0), ScalarF5E3::from(4.0));
    assert!(circle.is_normal());

    // Test magnitude calculation (should be 5.0 for 3+4i)
    let magnitude = circle.magnitude();
    if magnitude.is_normal() {
        let mag_val: f32 = magnitude.into();
        assert_relative_eq!(mag_val, 5.0, epsilon = 1e-6);
    }

    // Test that complex arithmetic works
    let doubled = circle * ScalarF5E3::from(2.0);
    assert!(doubled.is_normal());

    let doubled_real: f32 = doubled.real().into();
    let doubled_imag: f32 = doubled.imaginary().into();
    assert_relative_eq!(doubled_real, 6.0, epsilon = 1e-6);
    assert_relative_eq!(doubled_imag, 8.0, epsilon = 1e-6);
}

#[test]
fn test_precision_configuration_examples() {
    // Test examples from precision configuration documentation

    // Different precision levels
    let low_prec = ScalarF3E3::from(3.14159);
    let mid_prec = ScalarF5E3::from(3.14159);
    let high_prec = ScalarF7E7::from(3.14159);

    assert!(low_prec.is_normal());
    assert!(mid_prec.is_normal());
    assert!(high_prec.is_normal());

    // Higher precision should be more accurate
    let low_val: f32 = low_prec.into();
    let mid_val: f32 = mid_prec.into();
    let high_val: f32 = high_prec.into();

    // All should be close to π, but higher precision should be closer
    let pi = std::f32::consts::PI;
    assert!((high_val - pi).abs() <= (mid_val - pi).abs() + 1e-6);
    assert!((mid_val - pi).abs() <= (low_val - pi).abs() + 1e-2);
}

#[test]
fn test_state_classification_examples() {
    // Examples demonstrating the state classification system

    // N-0 level: Zero and Infinity
    let zero = ScalarF5E3::ZERO;
    let infinity = ScalarF5E3::INFINITY;

    assert!(zero.is_zero());
    assert!(infinity.is_infinite());
    assert_eq!(zero.normalization_level(), 0);
    assert_eq!(infinity.normalization_level(), 0);

    // N-1 level: Normal and Exploded
    let normal = ScalarF5E3::from(42.0);
    let exploded = ScalarF5E3::MAX * ScalarF5E3::from(2.0);

    assert!(normal.is_normal());
    assert!(exploded.is_exploded());
    assert_eq!(normal.normalization_level(), 1);
    assert_eq!(exploded.normalization_level(), 1);

    // N-2 level: Vanished
    let vanished = ScalarF5E3::MIN_POSITIVE / ScalarF5E3::from(2.0);
    assert!(vanished.is_vanished());
    assert_eq!(vanished.normalization_level(), 2);

    // N-3+ level: Undefined
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    assert!(undefined.is_undefined());
    assert!(undefined.normalization_level() >= 3);
}

#[test]
fn test_mathematical_constants_examples() {
    // Test that mathematical constants are properly defined and accessible

    // Scalar constants
    assert!(ScalarF5E3::ZERO.is_zero());
    assert!(ScalarF5E3::ONE.is_normal());
    assert!(ScalarF5E3::PI.is_normal());
    assert!(ScalarF5E3::E.is_normal());
    assert!(ScalarF5E3::INFINITY.is_infinite());
    assert!(ScalarF5E3::EPSILON.is_normal());

    // Verify approximate values
    let pi: f32 = ScalarF5E3::PI.into();
    let e: f32 = ScalarF5E3::E.into();
    let one: f32 = ScalarF5E3::ONE.into();

    assert_relative_eq!(pi, std::f32::consts::PI, epsilon = 1e-6);
    assert_relative_eq!(e, std::f32::consts::E, epsilon = 1e-6);
    assert_relative_eq!(one, 1.0, epsilon = 1e-6);

    // Circle constants
    assert!(CircleF5E3::ZERO.is_zero());
    assert!(CircleF5E3::ONE.is_real());
    assert!(CircleF5E3::I.is_imaginary());

    // Verify that I is the imaginary unit
    let i = CircleF5E3::I;
    let i_squared = i * i;

    // i² should equal -1
    let real_part: f32 = i_squared.real().into();
    let imag_part: f32 = i_squared.imaginary().into();

    assert_relative_eq!(real_part, -1.0, epsilon = 1e-6);
    assert_relative_eq!(imag_part, 0.0, epsilon = 1e-6);
}

#[test]
fn test_conversion_examples() {
    // Test conversion examples from documentation

    // Rust primitive to Scalar
    let from_int = ScalarF5E3::from(42);
    let from_float = ScalarF5E3::from(3.14159);

    assert!(from_int.is_normal());
    assert!(from_float.is_normal());

    // Scalar to Rust primitive
    let back_to_int: i32 = from_int.into();
    let back_to_float: f32 = from_float.into();

    assert_eq!(back_to_int, 42);
    assert_relative_eq!(back_to_float, 3.14159, epsilon = 1e-6);

    // Cross-precision conversion
    let low_prec = ScalarF3E3::from(42.0);
    let as_f32: f32 = low_prec.into();
    let high_prec = ScalarF7E7::from(as_f32);

    assert!(low_prec.is_normal());
    assert!(high_prec.is_normal());

    let high_back: f32 = high_prec.into();
    assert_relative_eq!(as_f32, high_back, epsilon = 1e-6);

    // Scalar to Circle conversion
    let scalar = ScalarF5E3::from(5.0);
    let circle: CircleF5E3 = scalar.into();

    assert!(circle.is_real());
    let circle_real: f32 = circle.real().into();
    let circle_imag: f32 = circle.imaginary().into();

    assert_relative_eq!(circle_real, 5.0, epsilon = 1e-6);
    assert_relative_eq!(circle_imag, 0.0, epsilon = 1e-6);
}

#[test]
fn test_operation_examples() {
    // Test arithmetic operation examples

    let a = ScalarF5E3::from(10.0);
    let b = ScalarF5E3::from(3.0);

    // Basic arithmetic
    let sum = a + b;
    let diff = a - b;
    let product = a * b;
    let quotient = a / b;

    assert!(sum.is_normal());
    assert!(diff.is_normal());
    assert!(product.is_normal());
    assert!(quotient.is_normal());

    let sum_val: f32 = sum.into();
    let diff_val: f32 = diff.into();
    let product_val: f32 = product.into();
    let quotient_val: f32 = quotient.into();

    assert_relative_eq!(sum_val, 13.0, epsilon = 1e-6);
    assert_relative_eq!(diff_val, 7.0, epsilon = 1e-6);
    assert_relative_eq!(product_val, 30.0, epsilon = 1e-6);
    assert_relative_eq!(quotient_val, 10.0 / 3.0, epsilon = 1e-6);

    // Transcendental functions
    let angle = ScalarF5E3::from(std::f32::consts::PI / 4.0);
    let sin_val = angle.sin();
    let cos_val = angle.cos();
    let exp_val = ScalarF5E3::ONE.exp();
    let ln_val = ScalarF5E3::E.ln();

    assert!(sin_val.is_normal());
    assert!(cos_val.is_normal());
    assert!(exp_val.is_normal());
    assert!(ln_val.is_normal());

    let sin_f32: f32 = sin_val.into();
    let cos_f32: f32 = cos_val.into();
    let exp_f32: f32 = exp_val.into();
    let ln_f32: f32 = ln_val.into();

    assert_relative_eq!(sin_f32, (std::f32::consts::PI / 4.0).sin(), epsilon = 1e-5);
    assert_relative_eq!(cos_f32, (std::f32::consts::PI / 4.0).cos(), epsilon = 1e-5);
    assert_relative_eq!(exp_f32, std::f32::consts::E, epsilon = 1e-5);
    assert_relative_eq!(ln_f32, 1.0, epsilon = 1e-5);
}

#[test]
fn test_complex_operation_examples() {
    // Test complex number operation examples

    let z1 = CircleF5E3::new(ScalarF5E3::from(1.0), ScalarF5E3::from(2.0)); // 1 + 2i
    let z2 = CircleF5E3::new(ScalarF5E3::from(3.0), ScalarF5E3::from(4.0)); // 3 + 4i

    // Complex addition: (1+2i) + (3+4i) = 4+6i
    let sum = z1 + z2;
    let sum_real: f32 = sum.real().into();
    let sum_imag: f32 = sum.imaginary().into();

    assert_relative_eq!(sum_real, 4.0, epsilon = 1e-6);
    assert_relative_eq!(sum_imag, 6.0, epsilon = 1e-6);

    // Complex multiplication: (1+2i) * (3+4i) = (3-8) + (4+6)i = -5+10i
    let product = z1 * z2;
    let prod_real: f32 = product.real().into();
    let prod_imag: f32 = product.imaginary().into();

    assert_relative_eq!(prod_real, -5.0, epsilon = 1e-6);
    assert_relative_eq!(prod_imag, 10.0, epsilon = 1e-6);

    // Complex conjugate
    let z1_conj = z1.conjugate(); // 1 - 2i
    let conj_real: f32 = z1_conj.real().into();
    let conj_imag: f32 = z1_conj.imaginary().into();

    assert_relative_eq!(conj_real, 1.0, epsilon = 1e-6);
    assert_relative_eq!(conj_imag, -2.0, epsilon = 1e-6);

    // Magnitude: |3+4i| = 5
    let magnitude = z2.magnitude();
    let mag_val: f32 = magnitude.into();
    assert_relative_eq!(mag_val, 5.0, epsilon = 1e-6);
}

#[test]
fn test_testing_guide_examples() {
    // Examples from TESTING.md documentation

    // Property-based testing example
    let input = ScalarF5E3::from(42.0);
    let result = input.sqrt().square(); // Should approximately equal input

    if result.is_normal() {
        let input_val: f32 = input.into();
        let result_val: f32 = result.into();
        assert_relative_eq!(input_val, result_val, epsilon = 1e-5);
    }

    // Precision boundary testing
    let max_val = ScalarF5E3::MAX;
    let doubled = max_val * ScalarF5E3::from(2.0);
    assert!(doubled.exploded());

    let min_pos = ScalarF5E3::MIN_POS;
    let halved = min_pos / ScalarF5E3::from(2.0);
    assert!(halved.vanished());

    // Error condition testing
    let undefined = ScalarF5E3::ZERO / ScalarF5E3::ZERO;
    assert!(undefined.is_undefined());

    let sqrt_negative = ScalarF5E3::from(-1.0).sqrt();
    assert!(sqrt_negative.is_undefined());
}
