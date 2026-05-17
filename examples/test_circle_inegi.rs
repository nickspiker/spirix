use spirix::*;
fn main() {
    let pos_i = CircleF3E3::POS_I;
    let neg_i = CircleF3E3::NEG_I;
    eprintln!("POS_I: r=0x{:x} i=0x{:x} exp=0x{:x}",
        pos_i.real as u8, pos_i.imaginary as u8, pos_i.exponent as u8);
    eprintln!("NEG_I: r=0x{:x} i=0x{:x} exp=0x{:x}",
        neg_i.real as u8, neg_i.imaginary as u8, neg_i.exponent as u8);

    let prod = pos_i * neg_i;
    eprintln!("i*(-i): r=0x{:x} i=0x{:x} exp=0x{:x}",
        prod.real as u8, prod.imaginary as u8, prod.exponent as u8);
    eprintln!("  r() = {}, i() = {}", prod.r().to_f64(), prod.i().to_f64());

    let diff = prod - 1i8;
    eprintln!("i*(-i) - 1: r=0x{:x} i=0x{:x} exp=0x{:x}",
        diff.real as u8, diff.imaginary as u8, diff.exponent as u8);
    eprintln!("  r() = {}, i() = {}", diff.r().to_f64(), diff.i().to_f64());

    let mag = diff.magnitude();
    eprintln!("magnitude = {}", mag.to_f64());
}
