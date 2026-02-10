use spirix::ScalarF4E4;

#[test]
fn verify_conversions_with_display() {
    println!("\n=== Verifying u8 conversions ===");
    for i in [0, 1, 2, 100, 127, 128, 254, 255, 256, 500] {
        let s = ScalarF4E4::from(i);
        let as_u8: u8 = s.into();
        println!("  {} → Scalar({}) → u8: {}", i, s, as_u8);

        let expected_u8 = if i > 255 { 255 } else { i as u8 };
        assert_eq!(as_u8, expected_u8, "Mismatch for {}", i);
    }

    println!("\n=== Verifying i8 conversions ===");
    for i in [-128, -100, -1, 0, 1, 100, 127, 128, 200] {
        let s = ScalarF4E4::from(i);
        let as_i8: i8 = s.into();
        println!("  {} → Scalar({}) → i8: {}", i, s, as_i8);

        let expected_i8 = if i < -128 { -128 } else if i > 127 { 127 } else { i as i8 };
        assert_eq!(as_i8, expected_i8, "Mismatch for {}", i);
    }

    println!("\n=== Verifying boundary cases ===");
    let s = ScalarF4E4::from(65535u16);
    let as_u16: u16 = s.into();
    println!("  65535 → Scalar({}) → u16: {}", s, as_u16);

    let s = ScalarF4E4::from(4294967295u32);
    let as_u32: u32 = s.into();
    println!("  4294967295 → Scalar({}) → u32: {}", s, as_u32);

    println!("\n✓ All conversions verified");
}
