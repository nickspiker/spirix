use spirix::ScalarF4E4;

fn main() {
    println!("Testing sqrt() vs sqrt_newton() for equality\n");

    let mut mismatches = 0;
    let mut total_tests = 0;

    // Test a range of values
    for val in 0u8..=255 {
        let input = ScalarF4E4::from(val);

        // Skip non-normal values
        if !input.is_normal() {
            continue;
        }

        let a = input.sqrt();
        let b = input.sqrt_newton();

        total_tests += 1;

        if a.fraction != b.fraction || a.exponent != b.exponent {
            mismatches += 1;
            println!("MISMATCH for input {}:", val);
            println!("  Input:        {:#?}", input);
            println!("  sqrt:         {:#?}", a);
            println!("  sqrt_newton:  {:#?}", b);

            // Show bit differences
            let frac_diff = (a.fraction as i32 - b.fraction as i32).unsigned_abs();
            let exp_diff = (a.exponent as i32 - b.exponent as i32).unsigned_abs();
            println!("  Frac diff: {}", frac_diff);
            println!("  Exp diff:  {}\n", exp_diff);
        }
    }

    println!("=======================================");
    println!("Total tests: {}", total_tests);
    println!("Mismatches: {}", mismatches);
    println!(
        "Match rate: {:.2}%",
        100.0 * (total_tests - mismatches) as f64 / total_tests as f64
    );

    if mismatches == 0 {
        println!("\nPerfect match! All results identical.");
    }
}
