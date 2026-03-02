use spirix::ScalarF4E4;

fn main() {
    let input = ScalarF4E4::from(9u8);
    println!("Input: {:#?}", input);

    let result = input.sqrt();
    println!("Result: {:#?}", result);
    println!(
        "Result as f64: {}",
        f64::from(result.fraction) / 2f64.powi(16) * 2f64.powi(result.exponent as i32)
    );

    let expected = ScalarF4E4::from(3u8);
    println!("Expected: {:#?}", expected);

    println!(
        "Match: {}",
        result.fraction == expected.fraction && result.exponent == expected.exponent
    );
}
