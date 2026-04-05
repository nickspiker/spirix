use spirix::ScalarF4E4;
use std::hint::black_box;

#[inline(never)]
pub fn sqrt_newton(input: ScalarF4E4) -> ScalarF4E4 {
    black_box(input).sqrt()
}

#[inline(never)]
pub fn sqrt_newton_method(input: ScalarF4E4) -> ScalarF4E4 {
    black_box(input).sqrt_newton()
}

fn main() {
    let input = ScalarF4E4::from(100u8);

    let result1 = sqrt_newton(input);
    let result2 = sqrt_newton_method(input);

    println!("sqrt:        {:#?}", result1);
    println!("sqrt_newton: {:#?}", result2);
}
