use spirix::ScalarF4E4;

#[test]
fn debug_conversion_to_u8() {
    let s = ScalarF4E4::from(1);
    println!("Value 1:");
    println!("  fraction={}, exponent={}", s.fraction, s.exponent);
    println!("  fraction_bits={}", std::mem::size_of::<i16>() * 8 - 1);
    println!("  shift_amount={}", (std::mem::size_of::<i16>() * 8 - 1) as isize - s.exponent as isize);

    let result = s.to_u8();
    println!("  result={}", result);

    // Manual calculation
    let fraction_bits = (std::mem::size_of::<i16>() * 8 - 1) as isize;
    let shift_amount = fraction_bits - s.exponent as isize;
    let manual = (s.fraction as i64 >> shift_amount) as u8;
    println!("  manual calculation: {} >> {} = {}", s.fraction, shift_amount, manual);
}
