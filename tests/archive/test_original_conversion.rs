use spirix::ScalarF4E4;

#[test]
fn test_original_u8_conversion_user_case() {
    // User's case: casting 100, 101, ... 500 to u8
    // Should saturate at 255
    for i in 100..=500 {
        let s = ScalarF4E4::from(i);
        let result: u8 = s.into();

        let expected = if i > 255 { 255 } else { i as u8 };

        assert_eq!(
            result, expected,
            "Failed for i={}: got {}, expected {}",
            i, result, expected
        );
    }
    println!("✓ All values 100-500 convert correctly to u8!");
}

#[test]
fn test_original_i8_conversion_user_case() {
    // User's case: casting 100, 101, ... 500 to i8
    // Should saturate at 127
    for i in 100..=500 {
        let s = ScalarF4E4::from(i);
        let result: i8 = s.into();

        let expected = if i > 127 { 127 } else { i as i8 };

        assert_eq!(
            result, expected,
            "Failed for i={}: got {}, expected {}",
            i, result, expected
        );
    }
    println!("✓ All values 100-500 convert correctly to i8!");
}

#[test]
fn test_small_values() {
    // Test small values that my other test failed on
    for i in 0..=10 {
        let s = ScalarF4E4::from(i);
        let result: u8 = s.into();

        assert_eq!(
            result, i as u8,
            "Failed for i={}: got {}, expected {}",
            i, result, i
        );
    }
    println!("✓ Small values 0-10 work!");
}
