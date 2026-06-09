use spirix::*;
fn main() {
    for &val in &[3i8, 7, 11, 13, 17, 23, 100] {
        let s = ScalarF3E3::from(val);
        let bit = s.sqrt();
        let newt = s.sqrt_newton();
        println!("val={} sqrt()={} ({}) sqrt_newton()={} ({}) match={}",
            val, bit.fraction, bit, newt.fraction, newt,
            bit.fraction == newt.fraction);
    }
    let s = ScalarF3E3::from(3i8);
    let bit = s.sqrt();
    let newt = s.sqrt_newton();
    println!("sqrt() (bitwise/floor): {} (fr={}, exp={})", bit, bit.fraction, bit.exponent);
    println!("sqrt_newton():          {} (fr={}, exp={})", newt, newt.fraction, newt.exponent);
    println!();
    let bit_sq = bit.square();
    let newt_sq = newt.square();
    println!("bit.square()  = {} (fr={}, exp={})", bit_sq, bit_sq.fraction, bit_sq.exponent);
    println!("newt.square() = {} (fr={}, exp={})", newt_sq, newt_sq.fraction, newt_sq.exponent);
    println!("bit.square() > s ? {}", bit_sq > s);
    println!("newt.square() > s ? {}", newt_sq > s);
}
