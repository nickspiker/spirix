use spirix::ScalarF4E4;
use std::time::Instant;

fn main() {
    println!("sqrt() Performance Comparison");
    println!("========================================");
    println!("sqrt vs sqrt_newton\n");

    let test_values: Vec<u8> = vec![
        2, 3, 4, 5, 9, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196, 225, 255,
    ];

    // Warmup
    println!("Warming up...");
    for _ in 0..10000 {
        let val = ScalarF4E4::from(100u8);
        let _ = val.sqrt();
        let _ = val.sqrt_newton();
    }

    println!("Running benchmarks...\n");

    let mut total_lut_time = 0u128;
    let mut total_newton_time = 0u128;

    for &val in &test_values {
        let input = ScalarF4E4::from(val);
        let iterations = 1_000_000;

        // Benchmark LUT-seeded Newton-Raphson
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = input.sqrt();
        }
        let lut_duration = start.elapsed();
        let lut_ns = lut_duration.as_nanos() / iterations as u128;

        // Benchmark sqrt_newton
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = input.sqrt_newton();
        }
        let newton_duration = start.elapsed();
        let newton_ns = newton_duration.as_nanos() / iterations as u128;

        total_lut_time += lut_ns;
        total_newton_time += newton_ns;

        let speedup = newton_ns as f64 / lut_ns as f64;

        println!(
            "sqrt({:3}): sqrt={:3}ns  newton={:3}ns  ratio={:.2}×",
            val, lut_ns, newton_ns, speedup
        );

        // Verify results match
        let sqrt_result = input.sqrt();
        let newton_result = input.sqrt_newton();
        if sqrt_result.fraction != newton_result.fraction
            || sqrt_result.exponent != newton_result.exponent
        {
            println!(
                "  MISMATCH! sqrt: {:?}, newton: {:?}",
                sqrt_result, newton_result
            );
        }
    }

    let avg_sqrt = total_lut_time / test_values.len() as u128;
    let avg_newton = total_newton_time / test_values.len() as u128;
    let avg_speedup = avg_newton as f64 / avg_sqrt as f64;

    println!("\n========================================");
    println!("Average Results:");
    println!(
        "  sqrt:        {} ns/op  ({:.2} M ops/sec)",
        avg_sqrt,
        1000.0 / avg_sqrt as f64
    );
    println!(
        "  sqrt_newton: {} ns/op  ({:.2} M ops/sec)",
        avg_newton,
        1000.0 / avg_newton as f64
    );
    println!("  Ratio:       {:.2}×", avg_speedup);
    println!("========================================");
}
