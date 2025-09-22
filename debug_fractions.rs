use spirix::*;

fn main() {
    println!("F3E3 info:");
    println!("F::FRACTION_BITS = {}", std::mem::size_of::<i8>() * 8 - 1);

    let val = ScalarF3E3::from(5i64);
    println!("Value 5:");
    println!("  exponent = {}", val.exponent);
    println!("  fraction = {:08b}", val.fraction);

    // Let's manually calculate what frac_bits should be
    let fraction_bits = std::mem::size_of::<i8>() * 8 - 1; // Should be 7 for i8
    let exponent = val.exponent as isize;
    println!("Manual calc:");
    println!("  FRACTION_BITS = {}", fraction_bits);
    println!("  exponent = {}", exponent);

    if exponent <= fraction_bits as isize {
        let frac_bits = fraction_bits - exponent as usize;
        println!("  frac_bits = {} - {} = {}", fraction_bits, exponent, frac_bits);
    } else {
        println!("  exponent > FRACTION_BITS, should be non-contiguous");
    }
}