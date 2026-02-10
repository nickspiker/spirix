use spirix::ScalarF4E4;

#[test]
fn trace_working_u8_formula() {
    let s = ScalarF4E4::from(100);
    println!("Value 100:");
    println!("  fraction={} (0x{:04x}), exponent={}", s.fraction, s.fraction as u16, s.exponent);

    // Original formula that WORKS
    let shifted = (s.fraction << 1isize) as i64;
    let target_bits = 8usize;
    let exponent = s.exponent as usize;
    let shift_amount = target_bits - exponent;
    let result = (shifted >> shift_amount) as u8;

    println!("  shifted (<<1) = {} (0x{:x})", shifted, shifted);
    println!("  shift_amount (8 - {}) = {}", exponent, shift_amount);
    println!("  result = {} >> {} = {}", shifted, shift_amount, result);

    assert_eq!(result, 100);
}
