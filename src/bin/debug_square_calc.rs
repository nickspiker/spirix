use spirix::*;
use num_traits::cast::AsPrimitive;

fn main() {
    let val = ScalarF3E5::from(-4);
    println!("Input: {:?}", val);

    // Manually trace through the square calculation
    let self_exponent: i64 = val.exponent.as_();
    println!("self_exponent: {}", self_exponent);

    // Calculate product (this is what happens in the normal path)
    let multiplier: i64 = val.fraction.as_();
    println!("fraction as i64: {}", multiplier);

    let product_wide = multiplier.wrapping_mul(multiplier);
    println!("product_wide: {}", product_wide);

    let expo_adjust = product_wide
        .leading_ones()
        .max(product_wide.leading_zeros())
        .wrapping_sub(2) as isize;
    println!("expo_adjust: {}", expo_adjust);

    let upcast_exponent = self_exponent
        .wrapping_mul(2)
        .wrapping_sub(expo_adjust as i64);
    println!("upcast_exponent: {}", upcast_exponent);

    let max_e: i64 = <i32 as spirix::ExponentConstants>::MAX_EXPONENT.as_();
    let min_e: i64 = <i32 as spirix::ExponentConstants>::MIN_EXPONENT.as_();

    println!("max_e: {}, min_e: {}", max_e, min_e);
    println!("upcast_exponent > max_e: {}", upcast_exponent > max_e);
    println!("upcast_exponent < min_e: {}", upcast_exponent < min_e);
}