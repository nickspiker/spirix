use spirix::*;

fn main() {
    let half = ScalarF5E3::from(2.5f32);

    println!("2.5 representation:");
    println!("  Exponent: {:?}", half.exponent);
    println!("  Fraction: {:b}", half.fraction);

    // Simulate my round logic
    let width = 31 - 1; // F::FRACTION_BITS - 1 for i32
    let e: isize = half.exponent as isize;
    let frac_bits = width - e;

    println!("  Width: {}", width);
    println!("  E: {}", e);
    println!("  Frac bits: {}", frac_bits);

    let one: i32 = 1;
    let frac_mask = (one << frac_bits) - one;
    let fractional_part = half.fraction & frac_mask;
    let half_bit = one << (frac_bits - 1);

    println!("  Half bit: {:b}", half_bit);
    println!("  Frac mask: {:b}", frac_mask);
    println!("  Fractional part: {:b}", fractional_part);
    println!("  Is exactly half? {}", fractional_part == half_bit);

    // Check integer part for evenness
    let integer_lsb = one << frac_bits;
    let is_odd = (half.fraction & integer_lsb) != 0;

    println!("  Integer LSB mask: {:b}", integer_lsb);
    println!("  Masked result: {:b}", half.fraction & integer_lsb);
    println!("  Is odd? {}", is_odd);
}