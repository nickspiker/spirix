use spirix::ScalarF4E4;
use std::hint::black_box;

#[inline(never)]
pub fn sqrt_newton(input: ScalarF4E4) -> ScalarF4E4 {
    black_box(input).sqrt()
}

#[inline(never)]
pub fn sqrt_bitwise(input: ScalarF4E4) -> ScalarF4E4 {
    black_box(input).sqrt_bb()
}

fn main() {
    let input = ScalarF4E4::from(100u8);

    let result1 = sqrt_newton(input);
    let result2 = sqrt_bitwise(input);

    println!("Newton: {:#?}", result1);
    println!("Bitwise: {:#?}", result2);
}
