use spirix::*;
fn main() {
    let two = CircleF3E3::from(2);
    let three = CircleF3E3::from(3);
    eprintln!("2: r=0x{:x} exp=0x{:x}", two.real as u8, two.exponent as u8);
    eprintln!(
        "3: r=0x{:x} exp=0x{:x}",
        three.real as u8, three.exponent as u8
    );
    let product = two * three;
    eprintln!(
        "2*3: r=0x{:x} exp=0x{:x}",
        product.real as u8, product.exponent as u8
    );
    eprintln!("r() = {}", product.r().to_f64());
}
