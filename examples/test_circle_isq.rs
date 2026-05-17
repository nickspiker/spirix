use spirix::*;
fn main() {
    let pos_i = CircleF3E3::POS_I;
    eprintln!("POS_I: r=0x{:x} i=0x{:x} exp=0x{:x}",
        pos_i.real as u8, pos_i.imaginary as u8, pos_i.exponent as u8);

    let sq = pos_i.square();
    eprintln!("POS_I^2: r=0x{:x} i=0x{:x} exp=0x{:x}",
        sq.real as u8, sq.imaginary as u8, sq.exponent as u8);
    eprintln!("  r() = {}, i() = {}", sq.r().to_f64(), sq.i().to_f64());

    let neg_one = CircleF3E3::NEG_ONE;
    eprintln!("NEG_ONE: r=0x{:x} i=0x{:x} exp=0x{:x}",
        neg_one.real as u8, neg_one.imaginary as u8, neg_one.exponent as u8);
    eprintln!("  r() = {}, i() = {}", neg_one.r().to_f64(), neg_one.i().to_f64());

    eprintln!("sq == -1isize ? {}", sq == -1isize);
    eprintln!("sq == NEG_ONE ? {}", sq == neg_one);
}
