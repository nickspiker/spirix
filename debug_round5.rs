use spirix::*;

fn main() {
    // Test multiple values to understand the representation
    let vals = [
        ("2.0", ScalarF5E3::from(2.0)),
        ("2.5", ScalarF5E3::from(2.5)),
        ("3.0", ScalarF5E3::from(3.0)),
        ("3.5", ScalarF5E3::from(3.5)),
        ("4.0", ScalarF5E3::from(4.0)),
    ];

    for (name, val) in vals {
        println!("{}: exp={}, frac={:032b}, value={}",
                 name, val.exponent, val.fraction, val.to_f64());

        // Calculate where the fractional boundary should be
        let width = 30; // F::FRACTION_BITS - 1 for i32
        let e = val.exponent as isize;
        let frac_bits = width - e;

        if frac_bits > 0 {
            let frac_mask = (1i32 << frac_bits) - 1;
            let integer_part = val.fraction & !frac_mask;
            let fractional_part = val.fraction & frac_mask;

            println!("  frac_bits={}, frac_mask={:032b}", frac_bits, frac_mask);
            println!("  integer_part={:032b}, fractional_part={:032b}", integer_part, fractional_part);
        }
        println!();
    }
}