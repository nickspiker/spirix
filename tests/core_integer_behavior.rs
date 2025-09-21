use spirix::*;

#[test]
fn test_integer_detection_comprehensive() {
    println!("=== Testing integer detection ===");

    // Positive integers
    println!("Positive integers:");
    println!("0: {}", ScalarF5E3::from(0).is_integer());
    println!("1: {}", ScalarF5E3::from(1).is_integer());
    println!("42: {}", ScalarF5E3::from(42).is_integer());
    println!("1000: {}", ScalarF5E3::from(1000).is_integer());

    // Negative integers
    println!("\nNegative integers:");
    println!("-1: {}", ScalarF5E3::from(-1).is_integer());
    println!("-42: {}", ScalarF5E3::from(-42).is_integer());
    println!("-17: {}", ScalarF5E3::from(-17).is_integer());
    println!("-1000: {}", ScalarF5E3::from(-1000).is_integer());

    // Non-integers
    println!("\nNon-integers:");
    println!("3.14: {}", ScalarF5E3::from(3.14).is_integer());
    println!("-2.5: {}", ScalarF5E3::from(-2.5).is_integer());
    println!("0.1: {}", ScalarF5E3::from(0.1).is_integer());

    // Edge case: zero
    println!("\nSpecial case - zero:");
    let zero = ScalarF5E3::ZERO;
    println!("ScalarF5E3::ZERO: {}", zero.is_integer());
    println!("ScalarF5E3::from(0): {}", ScalarF5E3::from(0).is_integer());
    println!(
        "ScalarF5E3::from(0.0): {}",
        ScalarF5E3::from(0.0).is_integer()
    );

    // Test if it's related to sign detection
    println!("\n=== Sign information ===");
    let neg_int = ScalarF5E3::from(-42);
    let pos_int = ScalarF5E3::from(42);

    println!("ScalarF5E3::from(-42):");
    println!("  is_integer(): {}", neg_int.is_integer());
    println!("  is_negative(): {}", neg_int.is_negative());
    println!("  is_positive(): {}", neg_int.is_positive());
    println!("  is_normal(): {}", neg_int.is_normal());

    println!("ScalarF5E3::from(42):");
    println!("  is_integer(): {}", pos_int.is_integer());
    println!("  is_negative(): {}", pos_int.is_negative());
    println!("  is_positive(): {}", pos_int.is_positive());
    println!("  is_normal(): {}", pos_int.is_normal());
}

#[test]
fn test_mathematical_integer_operations() {
    println!("\n=== Mathematical operations on integers ===");

    let pos_int = ScalarF5E3::from(6);
    let neg_int = ScalarF5E3::from(-6);

    // Test operations that should preserve integer nature
    println!("Positive 6:");
    println!("  Original: is_integer() = {}", pos_int.is_integer());
    println!(
        "  Squared: is_integer() = {}",
        pos_int.square().is_integer()
    );
    let pos_times_2: ScalarF5E3 = pos_int * 2.0;
    println!("  Times 2: is_integer() = {}", pos_times_2.is_integer());

    println!("\nNegative -6:");
    println!("  Original: is_integer() = {}", neg_int.is_integer());
    println!(
        "  Squared: is_integer() = {}",
        neg_int.square().is_integer()
    ); // Should be positive 36
    let neg_times_2: ScalarF5E3 = neg_int * 2.0;
    println!("  Times 2: is_integer() = {}", neg_times_2.is_integer()); // Should be negative 12

    // Test the square of negative -> positive
    let neg_squared = neg_int.square();
    let neg_squared_val: f32 = neg_squared.into();
    println!("  Squared value: {} (should be 36)", neg_squared_val);
    println!("  Squared is_positive(): {}", neg_squared.is_positive());
}

#[test]
fn test_zero_variants() {
    println!("\n=== Different ways to create zero ===");

    let zero_const = ScalarF5E3::ZERO;
    let zero_int = ScalarF5E3::from(0);
    let zero_float = ScalarF5E3::from(0.0);
    let zero_computed = ScalarF5E3::from(5) - ScalarF5E3::from(5);

    println!("ScalarF5E3::ZERO.is_integer(): {}", zero_const.is_integer());
    println!(
        "ScalarF5E3::from(0).is_integer(): {}",
        zero_int.is_integer()
    );
    println!(
        "ScalarF5E3::from(0.0).is_integer(): {}",
        zero_float.is_integer()
    );
    println!("(5-5).is_integer(): {}", zero_computed.is_integer());

    // Are they all considered the same zero?
    println!("\nZero equality:");
    println!("ZERO == from(0): {}", zero_const == zero_int);
    println!("ZERO == from(0.0): {}", zero_const == zero_float);
    println!("ZERO == computed: {}", zero_const == zero_computed);
}
