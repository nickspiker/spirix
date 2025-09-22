use spirix::*;

fn debug_contiguous(val: ScalarF3E3) {
    println!("=== Debug is_contiguous({}) ===", val.to_f64());

    if !val.is_normal() || val.exponent.is_negative() {
        println!("Not normal or negative exponent -> return is_zero()");
        return;
    }

    println!("is_normal: {}, exponent >= 0: {}", val.is_normal(), !val.exponent.is_negative());

    // Check the constants
    let exponent_bits = 3isize; // E::EXPONENT_BITS for i8 should be 3
    let isize_bits = std::mem::size_of::<isize>() as isize * 8;
    println!("EXPONENT_BITS: {}, isize bits: {}", exponent_bits, isize_bits);

    if exponent_bits >= isize_bits {
        println!("Branch 1: EXPONENT_BITS >= isize bits");
        if val.exponent > 7i8 { // F::FRACTION_BITS should be 7 for i8
            println!("exponent > FRACTION_BITS -> false");
            return;
        }
        if val.exponent == 7i8 {
            println!("exponent == FRACTION_BITS -> true");
            return;
        }
    } else {
        println!("Branch 2: EXPONENT_BITS < isize bits");
        let exponent_isize: isize = val.exponent as isize;
        if exponent_isize > 7isize { // F::FRACTION_BITS
            println!("exponent > FRACTION_BITS -> false");
            return;
        }
        if exponent_isize == 7isize {
            println!("exponent == FRACTION_BITS -> true");
            return;
        }
    }

    // The problematic calculation
    let mut frac_bits = 7isize; // F::FRACTION_BITS
    let exponent: isize = val.exponent as isize;
    println!("Before subtraction: frac_bits={}, exponent={}", frac_bits, exponent);

    if exponent > frac_bits {
        println!("ERROR: exponent > frac_bits, would cause underflow!");
        return;
    }

    frac_bits -= exponent;
    println!("After subtraction: frac_bits={}", frac_bits);

    if frac_bits < 0 {
        println!("ERROR: frac_bits is negative!");
        return;
    }

    // This is where it might panic
    println!("About to shift by {} bits", frac_bits);
    let mask = (1i8 << frac_bits) - 1;
    println!("mask = {:08b}", mask);

    let result = (val.fraction & mask) == 0;
    println!("(fraction & mask) == 0: {}", result);
}

fn main() {
    let val = ScalarF3E3::from(5i64);
    debug_contiguous(val);
}