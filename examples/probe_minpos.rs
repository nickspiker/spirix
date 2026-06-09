use spirix::*;
fn main() {
    let min_f7e7 = ScalarF7E7::MIN_POS;
    let temp: f32 = (&min_f7e7).into();
    let min_as_f3e3 = ScalarF3E3::from(temp);
    println!("MIN_POS<F7E7> = {}", min_f7e7);
    println!("  as f32 = {}", temp);
    println!("MIN_POS<F7E7> -> f32 -> F3E3 = {}", min_as_f3e3);
    println!("  is_normal:    {}", min_as_f3e3.is_normal());
    println!("  is_zero:      {}", min_as_f3e3.is_zero());
    println!("  vanished:     {}", min_as_f3e3.vanished());
    println!("  exploded:     {}", min_as_f3e3.exploded());
    println!("  is_undefined: {}", min_as_f3e3.is_undefined());
}
