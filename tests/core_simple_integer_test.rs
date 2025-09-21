use spirix::*;

#[test]
fn test_simple_integer_cases() {
    println!("=== Simple Integer Detection Test ===");

    // Test the exact case from our debug output
    let neg_17 = ScalarF5E3::from(-17);
    let pos_42 = ScalarF5E3::from(42);
    let zero = ScalarF5E3::from(0);
    let neg_6 = ScalarF5E3::from(-6);

    println!(
        "ScalarF5E3::from(-17).is_integer() = {}",
        neg_17.is_integer()
    );
    println!(
        "ScalarF5E3::from(42).is_integer() = {}",
        pos_42.is_integer()
    );
    println!("ScalarF5E3::from(0).is_integer() = {}", zero.is_integer());
    println!("ScalarF5E3::from(-6).is_integer() = {}", neg_6.is_integer());

    // Test a few more negative numbers
    let neg_1 = ScalarF5E3::from(-1);
    let neg_100 = ScalarF5E3::from(-100);

    println!("ScalarF5E3::from(-1).is_integer() = {}", neg_1.is_integer());
    println!(
        "ScalarF5E3::from(-100).is_integer() = {}",
        neg_100.is_integer()
    );

    // Test some non-integers
    let pi = ScalarF5E3::from(3.14);
    let half = ScalarF5E3::from(0.5);
    let neg_half = ScalarF5E3::from(-0.5);

    println!("ScalarF5E3::from(3.14).is_integer() = {}", pi.is_integer());
    println!("ScalarF5E3::from(0.5).is_integer() = {}", half.is_integer());
    println!(
        "ScalarF5E3::from(-0.5).is_integer() = {}",
        neg_half.is_integer()
    );
}
