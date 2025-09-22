use spirix::*;

fn main() {
    let val = ScalarF5E3::from(3.6);
    println!("Value: {:?}", val);
    println!("Is normal: {}", val.is_normal());
    println!("Exponent: {:?}", val.exponent);
    println!("Fraction: {:?}", val.fraction);

    let rounded = val.round();
    println!("Rounded: {:?}", rounded);
    println!("Rounded exponent: {:?}", rounded.exponent);
    println!("Rounded fraction: {:?}", rounded.fraction);

    // Test the components
    let floor_val = val.floor();
    println!("Floor: {:?}", floor_val);

    // Test 2.5 too
    let half_val = ScalarF5E3::from(2.5);
    println!("2.5: {:?}", half_val);
    let half_rounded = half_val.round();
    println!("2.5 rounded: {:?}", half_rounded);
}