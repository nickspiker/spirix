use spirix::*;

fn main() {
    let val = ScalarF3E3::from(5i64);
    println!("Value 5:");
    println!("  is_normal: {}", val.is_normal());
    println!("  exponent: {}", val.exponent);
    println!("  is_negative: {}", val.exponent.is_negative());
    println!("  is_contiguous: {}", val.is_contiguous());

    // Let's also test some edge cases
    let vals = [1i64, 5i64, 50i64, 127i64, 128i64];
    for v in vals {
        let scalar = ScalarF3E3::from(v);
        println!("Value {}: is_contiguous = {}", v, scalar.is_contiguous());
    }
}