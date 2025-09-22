use spirix::*;

fn main() {
    let val = ScalarF5E3::from(3.6);
    let rounded = val.round();
    let four = ScalarF5E3::from(4i128);

    println!("3.6 rounded: {:?}", rounded);
    println!("4i128: {:?}", four);
    println!("Equal? {}", rounded == four);

    // Try direct comparison
    println!("Rounded as f64: {}", rounded.to_f64());
    println!("Four as f64: {}", four.to_f64());

    // Test 2.5 banker's rounding
    let half = ScalarF5E3::from(2.5f32);
    let half_rounded = half.round();
    let two = ScalarF5E3::from(2u8);

    println!("2.5 rounded: {:?}", half_rounded);
    println!("2u8: {:?}", two);
    println!("2.5 -> 2? {}", half_rounded == two);
}