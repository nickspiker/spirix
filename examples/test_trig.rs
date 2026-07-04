use spirix::ScalarF5E3;

fn f(x: ScalarF5E3) -> f32 {
    x.into()
}

fn main() {
    let x = ScalarF5E3::from(-0.99f32);
    let sin_x = x.sin();
    println!("sin(-0.99) = {} bits={:?}", f(sin_x), sin_x);

    let mag = sin_x.magnitude();
    println!("sin_x.magnitude() = {} bits={:?}", f(mag), mag);

    let one = ScalarF5E3::from(1);
    let one_minus = one - mag;
    println!(
        "one_minus_abs_x = {} bits={:?} neg={} van={}",
        f(one_minus),
        one_minus,
        one_minus.is_negligible(),
        one_minus.vanished()
    );

    let three_quarters =
        ScalarF5E3::from(0.5f32) + ScalarF5E3::from(0.5f32) * ScalarF5E3::from(0.5f32);
    println!("3/4 = {} bits={:?}", f(three_quarters), three_quarters);
    println!("mag < 3/4 = {}", mag < three_quarters);

    let omah = one_minus >> 1i8;
    println!("one_minus >> 1 = {} bits={:?}", f(omah), omah);

    let sqrt_term = omah.sqrt();
    println!(
        "sqrt(omah) = {} bits={:?} norm={}",
        f(sqrt_term),
        sqrt_term,
        sqrt_term.is_normal()
    );
}
