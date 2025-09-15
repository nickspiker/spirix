use spirix::*;

#[test]
fn test_debug_division_by_zero() {
    let one = ScalarF5E3::from(1);
    let zero = ScalarF5E3::ZERO;
    let result = one / zero;

    println!("1 / 0 = {:?}", result);
    println!("is_undefined: {}", result.is_undefined());
    println!("is_normal: {}", result.is_normal());
    println!("is_zero: {}", result.is_zero());
    println!("exploded: {}", result.exploded());
    println!("vanished: {}", result.vanished());
    println!("is_finite: {}", result.is_finite());
}

#[test]
fn test_debug_bitwise() {
    let a = ScalarF5E3::from(0b1010); // 10
    let b = ScalarF5E3::from(0b1100); // 12
    let and_result = a & b;

    println!("a = {:?}", a);
    println!("b = {:?}", b);
    println!("a & b = {:?}", and_result);

    let and_val: i32 = and_result.into();
    println!("a & b as i32 = {}", and_val);
    println!("Expected: 8 (0b1000)");
}

#[test]
fn test_debug_integer_properties() {
    let neg_int = ScalarF5E3::from(-17);
    println!("ScalarF5E3::from(-17).is_integer() = {}", neg_int.is_integer());

    let pos_int = ScalarF5E3::from(42);
    println!("ScalarF5E3::from(42).is_integer() = {}", pos_int.is_integer());
}

#[test]
fn test_debug_complex_arithmetic() {
    let z1 = CircleF5E3::from((3.0, 4.0)); // 3 + 4i
    let z2 = CircleF5E3::from((1.0, -2.0)); // 1 - 2i

    println!("z1 = {:?}", z1);
    println!("z2 = {:?}", z2);

    let product = z1 * z2;
    println!("z1 * z2 = {:?}", product);

    let prod_real: f32 = product.r().into();
    let prod_imag: f32 = product.i().into();
    println!("Product real: {}, imaginary: {}", prod_real, prod_imag);
    println!("Expected real: 11.0, imaginary: -2.0");
    // (3+4i) * (1-2i) = 3 - 6i + 4i - 8i^2 = 3 - 2i + 8 = 11 - 2i
}