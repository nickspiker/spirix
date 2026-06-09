use spirix::*;

fn main() {
    let infinity = ScalarF5E3::from(42_i8) / 0_i8;
    let undef_add = infinity + 5_i8;
    let undef_sub = infinity - infinity;
    println!("∞ + 5  = {}, is_undefined={} is_infinite={}", undef_add, undef_add.is_undefined(), undef_add.is_infinite());
    println!("∞ − ∞  = {}, is_undefined={} is_infinite={}", undef_sub, undef_sub.is_undefined(), undef_sub.is_infinite());

    let undef_inf_plus_zero = infinity + 0_i8;
    println!("∞ + 0  = {}, is_undefined={}", undef_inf_plus_zero, undef_inf_plus_zero.is_undefined());

    let tiny_positive = ScalarF5E5::MIN_POS.square();
    let sin_tiny = tiny_positive.sin();
    println!("tiny={}, sin(tiny)={}, equal? {}", tiny_positive, sin_tiny, sin_tiny == tiny_positive);
    println!("  tiny raw frac={} exp={}", tiny_positive.fraction, tiny_positive.exponent);
    println!("  sin  raw frac={} exp={}", sin_tiny.fraction, sin_tiny.exponent);

    let undefined = ScalarF5E3::ZERO / 0_i8;
    println!("undef==undef -> {}", undefined == undefined);
    println!("undef!=undef -> {}", undefined != undefined);
}
