use spirix::*;

fn main() {
    println!("=== Testing CircleF3E5 (failing type) ===");
    let complex = CircleF3E5::from((-4i16, 0f32));
    println!("Input complex: {:?}", complex);

    let magnitude = complex.magnitude();
    println!("Magnitude: {:?}", magnitude);
    println!("Magnitude is_normal: {}", magnitude.is_normal());
    println!("Magnitude vanished: {}", magnitude.vanished());

    // Let's also check what magnitude_squared gives us
    let mag_sq = complex.magnitude_squared();
    println!("\nMagnitude squared: {:?}", mag_sq);
    println!("Magnitude squared is_normal: {}", mag_sq.is_normal());

    // And check the sqrt of magnitude squared
    let mag_sq_sqrt = mag_sq.sqrt();
    println!("\nSqrt of magnitude squared: {:?}", mag_sq_sqrt);
    println!("This should equal magnitude above!");

    println!("\n=== For comparison: CircleF3E4 (passing type) ===");
    let complex_e4 = CircleF3E4::from((-4i16, 0f32));
    let magnitude_e4 = complex_e4.magnitude();
    println!("Magnitude: {:?}", magnitude_e4);
    println!("Magnitude is_normal: {}", magnitude_e4.is_normal());
}