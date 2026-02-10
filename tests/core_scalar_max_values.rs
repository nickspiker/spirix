use spirix::ScalarF4E4;

#[test]
fn check_u16_max_representation() {
    let s = ScalarF4E4::from(65535u16);
    println!("Value 65535:");
    println!("  fraction={}, exponent={}", s.fraction, s.exponent);

    let result: u16 = s.into();
    println!("  Converts back to: {}", result);
    println!("  Difference: {}", 65535i32 - result as i32);
}

#[test]
fn check_u32_max_representation() {
    let s = ScalarF4E4::from(4294967295u32);
    println!("Value 2^32-1 (4294967295):");
    println!("  fraction={}, exponent={}", s.fraction, s.exponent);

    let result: u32 = s.into();
    println!("  Converts back to: {}", result);
    println!("  Difference: {}", 4294967295i64 - result as i64);
}
