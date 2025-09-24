use spirix::*;

fn main() {
    // Test step by step to see where it breaks
    println!("=== CircleF3E5 (failing) ===");
    let complex_neg = CircleF3E5::from((-4i16, 0f32));
    println!("Input: {:?}", complex_neg);

    let magnitude = complex_neg.magnitude();
    println!("Magnitude: {:?}", magnitude);

    let real_sum = magnitude + complex_neg.r();
    println!("magnitude + real: {:?}", real_sum);

    let mut real_half = real_sum;
    real_half = real_half >> 1;
    println!("(magnitude + real) >> 1: {:?}", real_half);

    let real_sqrt = real_half.sqrt();
    println!("sqrt((magnitude + real) >> 1): {:?}", real_sqrt);
    println!("  is_normal: {}, vanished: {}, undefined: {}",
             real_sqrt.is_normal(), real_sqrt.vanished(), real_sqrt.is_undefined());

    let imag_diff = magnitude - complex_neg.r();
    println!("magnitude - real: {:?}", imag_diff);

    let mut imag_half = imag_diff;
    imag_half = imag_half >> 1;
    println!("(magnitude - real) >> 1: {:?}", imag_half);

    let imag_sqrt = imag_half.sqrt();
    println!("sqrt((magnitude - real) >> 1): {:?}", imag_sqrt);
    println!("  is_normal: {}, vanished: {}, undefined: {}",
             imag_sqrt.is_normal(), imag_sqrt.vanished(), imag_sqrt.is_undefined());

    println!("\n=== CircleF3E4 (passing) ===");
    let complex_neg_e4 = CircleF3E4::from((-4i16, 0f32));
    println!("Input: {:?}", complex_neg_e4);

    let magnitude_e4 = complex_neg_e4.magnitude();
    println!("Magnitude: {:?}", magnitude_e4);

    let real_sum_e4 = magnitude_e4 + complex_neg_e4.r();
    println!("magnitude + real: {:?}", real_sum_e4);

    let mut real_half_e4 = real_sum_e4;
    real_half_e4 = real_half_e4 >> 1;
    println!("(magnitude + real) >> 1: {:?}", real_half_e4);

    let real_sqrt_e4 = real_half_e4.sqrt();
    println!("sqrt((magnitude + real) >> 1): {:?}", real_sqrt_e4);

    let imag_diff_e4 = magnitude_e4 - complex_neg_e4.r();
    println!("magnitude - real: {:?}", imag_diff_e4);

    let mut imag_half_e4 = imag_diff_e4;
    imag_half_e4 = imag_half_e4 >> 1;
    println!("(magnitude - real) >> 1: {:?}", imag_half_e4);

    let imag_sqrt_e4 = imag_half_e4.sqrt();
    println!("sqrt((magnitude - real) >> 1): {:?}", imag_sqrt_e4);
}