use spirix::*;
use spirix::core::integer::*;

fn main() {
    // Replicate what From<f32> for Circle does
    let scalar = ScalarF5E3::from(1.5f32);
    println!("scalar.fraction = 0x{:08x}", scalar.fraction as u32);
    println!("scalar.exponent = 0x{:02x}", scalar.exponent as u8);

    let is_normal = scalar.exponent != i8::min_value();
    println!("is_normal = {}", is_normal);

    let circle_real: i32 = if is_normal {
        scalar.fraction.inflate(true).w_shr(1isize).deflate()
    } else {
        scalar.fraction.inflate(false).w_shr(1isize).deflate()
    };
    println!("circle_real = 0x{:08x}", circle_real as u32);

    let c = CircleF5E3::from(1.5f32);
    println!("Actual c.real = 0x{:08x}", c.real as u32);
}
