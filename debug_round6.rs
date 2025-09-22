use spirix::*;

fn main() {
    let val = ScalarF5E3::from(2.5f32);

    println!("Original: {}", val.to_f64());

    let floored = val.floor();
    println!("Floored: {}", floored.to_f64());

    let frac = val - floored;
    println!("Frac: {}", frac.to_f64());

    let half = ScalarF5E3::HALF;
    println!("HALF: {}", half.to_f64());

    let diff = frac - half;
    println!("Frac - HALF: {}", diff.to_f64());
    println!("Magnitude: {}", diff.magnitude().to_f64());

    let min_pos = ScalarF5E3::MIN_POS;
    println!("MIN_POS: {}", min_pos.to_f64());

    let is_exactly_half = diff.magnitude() < min_pos;
    println!("Is exactly half? {}", is_exactly_half);

    if is_exactly_half {
        let two = ScalarF5E3::ONE + ScalarF5E3::ONE;
        let mod_result = floored % two;
        println!("Floor % 2: {}", mod_result.to_f64());
        println!("Magnitude of mod: {}", mod_result.magnitude().to_f64());
        let is_even = mod_result.magnitude() < min_pos;
        println!("Is even? {}", is_even);
    }
}