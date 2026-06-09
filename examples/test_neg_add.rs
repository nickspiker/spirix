use spirix::*;
fn main() {
    let neg_one = CircleF3E3::NEG_ONE;
    eprintln!("NEG_ONE: r=0x{:x} i=0x{:x} exp=0x{:x}",
        neg_one.real as u8, neg_one.imaginary as u8, neg_one.exponent as u8);
    eprintln!("  r() = {}", neg_one.r().to_f64());

    let neg_two = CircleF3E3::from(-2);
    eprintln!("Circle -2: r=0x{:x} i=0x{:x} exp=0x{:x}",
        neg_two.real as u8, neg_two.imaginary as u8, neg_two.exponent as u8);
    eprintln!("  r() = {}", neg_two.r().to_f64());

    let sum = neg_one + neg_one;
    eprintln!("(-1)+(-1): r=0x{:x} i=0x{:x} exp=0x{:x}",
        sum.real as u8, sum.imaginary as u8, sum.exponent as u8);
    eprintln!("  r() = {}", sum.r().to_f64());
}
