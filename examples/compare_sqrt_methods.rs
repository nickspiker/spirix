use spirix::ScalarF4E4;

fn main() {
    println!("Testing sqrt() vs sqrt_bb() for equality\n");

    let mut mismatches = 0;
    let mut total_tests = 0;

    // Test a range of values
    for val in 0u8..=255 {
        let input = ScalarF4E4::from(val);

        // Skip non-normal values
        if !input.is_normal() {
            continue;
        }

        let newton = input.sqrt();
        let bitwise = input.sqrt_bb();

        total_tests += 1;

        if newton.fraction != bitwise.fraction || newton.exponent != bitwise.exponent {
            mismatches += 1;
            println!("MISMATCH for input {}:", val);
            println!("  Input:    {:#?}", input);
            println!("  Newton:   {:#?}", newton);
            println!("  Bitwise:  {:#?}", bitwise);

            // Show bit differences
            let frac_diff = (newton.fraction as i32 - bitwise.fraction as i32).abs();
            let exp_diff = (newton.exponent as i32 - bitwise.exponent as i32).abs();
            println!("  Frac diff: {}", frac_diff);
            println!("  Exp diff:  {}\n", exp_diff);
        }
    }

    println!("═══════════════════════════════════════");
    println!("Total tests: {}", total_tests);
    println!("Mismatches: {}", mismatches);
    println!(
        "Match rate: {:.2}%",
        100.0 * (total_tests - mismatches) as f64 / total_tests as f64
    );

    if mismatches == 0 {
        println!("\n✓ Perfect match! All results identical.");
    }
}
