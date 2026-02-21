use spirix::*;

#[test]
fn test_clamp_operations_work() {
    let a = ScalarF5E3::from(5u8);
    let b = ScalarF5E3::from(3i16);
    let c = ScalarF5E3::from(8f32);

    // Test that clamp works with Scalar types
    let clamped = a.clamp(b, c);
    let expected = ScalarF5E3::from(5);
    assert_eq!(clamped, expected);

    // Test value below range gets clamped to min
    let low_val = ScalarF5E3::from(1);
    let clamped_low = low_val.clamp(b, c);
    assert_eq!(clamped_low, b); // Should be 3

    // Test value above range gets clamped to max
    let high_val = ScalarF5E3::from(10);
    let clamped_high = high_val.clamp(b, c);
    assert_eq!(clamped_high, c); // Should be 8

    println!("All clamp operations work correctly!");
}
