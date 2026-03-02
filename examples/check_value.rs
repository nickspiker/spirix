use spirix::ScalarF4E4;

fn main() {
    let three = ScalarF4E4::from(3u8);
    println!("3 as ScalarF4E4:");
    println!("  fraction = 0x{:04x} = {}", three.fraction, three.fraction);
    println!("  exponent = {}", three.exponent);

    let nine = ScalarF4E4::from(9u8);
    println!("\n9 as ScalarF4E4:");
    println!("  fraction = 0x{:04x} = {}", nine.fraction, nine.fraction);
    println!("  exponent = {}", nine.exponent);

    let result = nine.sqrt();
    println!("\nsqrt(9):");
    println!(
        "  fraction = 0x{:04x} = {}",
        result.fraction, result.fraction
    );
    println!("  exponent = {}", result.exponent);

    println!(
        "\nDoes sqrt(9) == 3? {}",
        result.fraction == three.fraction && result.exponent == three.exponent
    );

    // Convert to f64 properly
    let result_f64 = (result.fraction as f64) * 2f64.powi(result.exponent as i32 - 15);
    let three_f64 = (three.fraction as f64) * 2f64.powi(three.exponent as i32 - 15);
    println!("\nAs f64:");
    println!("  sqrt(9) ≈ {}", result_f64);
    println!("  3 = {}", three_f64);
}
