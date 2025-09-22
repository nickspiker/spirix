use spirix::*;

fn main() {
    // Test negative banker's rounding
    let neg_half = ScalarF5E3::from(-2.5f32);
    println!("Original: {}", neg_half.to_f64());

    let result = neg_half.round();
    println!("Rounded: {}", result.to_f64());

    let neg_two = ScalarF5E3::from(-2isize);
    println!("-2: {}", neg_two.to_f64());

    println!("-2.5 -> -2? {}", result == neg_two);

    // Also test -3.5
    let neg_three_half = ScalarF5E3::from(-3.5);
    let neg_three_result = neg_three_half.round();
    let neg_four = ScalarF5E3::from(-4i128);

    println!("-3.5 rounded: {}", neg_three_result.to_f64());
    println!("-3.5 -> -4? {}", neg_three_result == neg_four);
}