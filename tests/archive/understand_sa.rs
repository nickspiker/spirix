use spirix::{ScalarF4E4, Integer};

#[test]
fn understand_sa_conversion() {
    let s = ScalarF4E4::from(100);
    println!("Value 100: frac={} (0x{:04x}), exp={}", s.fraction, s.fraction as u16, s.exponent);

    // What does .sa() do?
    let shifted = s.fraction << 1isize;
    println!("  After <<1: {} (0x{:04x})", shifted, shifted as u16);

    // Call .sa() to convert to u8
    let as_u8: u8 = shifted.sa();
    println!("  After .sa() to u8: {}", as_u8);

    // Then shift right
    let final_result = as_u8 >> (8 - s.exponent as usize);
    println!("  After >> {}: {}", 8 - s.exponent as usize, final_result);

    assert_eq!(final_result, 100);
}
