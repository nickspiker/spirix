use spirix::ScalarF4E4;

#[test]
fn test_sqrt_25() {
    let val = ScalarF4E4::from(25u8);
    let result = val.sqrt();
    let result_f32: f32 = result.into();
    println!("sqrt(25) = {}", result_f32);
    assert!((result_f32 - 5.0).abs() < 0.1, "sqrt(25) should be close to 5, got {}", result_f32);
}

#[test]
fn test_sqrt_9() {
    let val = ScalarF4E4::from(9u8);
    let result = val.sqrt();
    let result_f32: f32 = result.into();
    println!("sqrt(9) = {}", result_f32);
    assert!((result_f32 - 3.0).abs() < 0.1, "sqrt(9) should be close to 3, got {}", result_f32);
}

#[test]
fn test_sqrt_16() {
    let val = ScalarF4E4::from(16u8);
    let result = val.sqrt();
    let result_f32: f32 = result.into();
    println!("sqrt(16) = {}", result_f32);
    assert!((result_f32 - 4.0).abs() < 0.1, "sqrt(16) should be close to 4, got {}", result_f32);
}
