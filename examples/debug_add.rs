use spirix::ScalarF3E3;

fn main() {
    let a = ScalarF3E3::ONE;
    let b = ScalarF3E3::NEG_ONE;
    println!("ONE     = frac={} exp={}", a.fraction, a.exponent);
    println!("NEG_ONE = frac={} exp={}", b.fraction, b.exponent);
    let exact = a + b;
    let closefar = a.scalar_add_scalar_closefar(&b);
    println!("\nExact:    frac={} exp={}", exact.fraction, exact.exponent);
    println!("CloseFar: frac={} exp={}", closefar.fraction, closefar.exponent);
}