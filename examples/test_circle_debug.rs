use spirix::*;
fn main() {
    let three_s = ScalarF3E3::from(3);
    let four_s = ScalarF3E3::from(4);
    eprintln!("Scalar 3: frac=0x{:x} exp=0x{:x}", three_s.fraction as u8, three_s.exponent as u8);
    eprintln!("Scalar 4: frac=0x{:x} exp=0x{:x}", four_s.fraction as u8, four_s.exponent as u8);

    let c = CircleF3E3::from((3.0f32, 4i64));
    eprintln!("Circle (3,4): r=0x{:x} i=0x{:x} exp=0x{:x}", c.real as u8, c.imaginary as u8, c.exponent as u8);
    eprintln!("  r() = {}, i() = {}", c.r().to_f64(), c.i().to_f64());

    let two_c = CircleF3E3::from(2);
    eprintln!("Circle 2: r=0x{:x} i=0x{:x} exp=0x{:x}", two_c.real as u8, two_c.imaginary as u8, two_c.exponent as u8);

    let prod = c * 2u32;
    eprintln!("(3+4i)*2: r=0x{:x} i=0x{:x} exp=0x{:x}", prod.real as u8, prod.imaginary as u8, prod.exponent as u8);
    eprintln!("  r() = {}, i() = {}", prod.r().to_f64(), prod.i().to_f64());
}
