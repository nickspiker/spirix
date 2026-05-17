use spirix::*;
fn main() {
    let a = CircleF3E3::from((20usize, 10isize));
    eprintln!("(20+10i): r=0x{:x} i=0x{:x} exp=0x{:x}",
        a.real as u8, a.imaginary as u8, a.exponent as u8);
    eprintln!("  r={} i={}", a.r().to_f64(), a.i().to_f64());

    let q = a / 2f64;
    eprintln!("/2: r=0x{:x} i=0x{:x} exp=0x{:x}",
        q.real as u8, q.imaginary as u8, q.exponent as u8);
    eprintln!("  r={} i={}", q.r().to_f64(), q.i().to_f64());
}
