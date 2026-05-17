use spirix::*;
fn main() {
    let six = CircleF3E3::from(6);
    let two = CircleF3E3::from(2);
    eprintln!("6: r=0x{:x} exp=0x{:x}", six.real as u8, six.exponent as u8);
    eprintln!("2: r=0x{:x} exp=0x{:x}", two.real as u8, two.exponent as u8);
    let q = six / two;
    eprintln!("6/2: r=0x{:x} exp=0x{:x}", q.real as u8, q.exponent as u8);
    eprintln!("r() = {}", q.r().to_f64());

    let a = CircleF3E3::from((3.0f32, 4i64));
    eprintln!("3+4i: r=0x{:x} exp=0x{:x}", a.real as u8, a.exponent as u8);
    let prod = a * 2u32;
    eprintln!("(3+4i)*2: r=0x{:x} i=0x{:x} exp=0x{:x}", prod.real as u8, prod.imaginary as u8, prod.exponent as u8);
    eprintln!("r={} i={}", prod.r().to_f64(), prod.i().to_f64());
}
