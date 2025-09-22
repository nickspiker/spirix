use spirix::*;

fn main() {
    let half = ScalarF5E3::from(2.5f32);
    let half_rounded = half.round();

    println!("2.5 as float: {}", half.to_f64());
    println!("2.5 rounded: {}", half_rounded.to_f64());

    let two = ScalarF5E3::from(2u8);
    let three = ScalarF5E3::from(3u8);

    println!("2: {}", two.to_f64());
    println!("3: {}", three.to_f64());

    println!("2.5 -> 2? {}", half_rounded == two);
    println!("2.5 -> 3? {}", half_rounded == three);

    // Test 3.5 too (should round to 4, even)
    let three_half = ScalarF5E3::from(3.5f64);
    let three_half_rounded = three_half.round();
    let four = ScalarF5E3::from(4i16);

    println!("3.5 rounded: {}", three_half_rounded.to_f64());
    println!("3.5 -> 4? {}", three_half_rounded == four);
}