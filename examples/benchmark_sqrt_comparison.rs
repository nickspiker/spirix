use spirix::ScalarF4E4;
use std::time::Instant;

fn main() {
    println!("sqrt() Performance Comparison");
    println!("========================================");
    println!("LUT-Newton vs Bitwise Non-Restoring\n");

    let test_values: Vec<u8> = vec![
        2, 3, 4, 5, 9, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196, 225, 255,
    ];

    // Warmup
    println!("Warming up...");
    for _ in 0..10000 {
        let val = ScalarF4E4::from(100u8);
        let _ = val.sqrt();
        let _ = val.sqrt_bb();
    }

    println!("Running benchmarks...\n");

    let mut total_lut_time = 0u128;
    let mut total_bitwise_time = 0u128;

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

        // Benchmark bitwise multiplication-based
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = input.sqrt_bb();
        }
        let bitwise_duration = start.elapsed();
        let bitwise_ns = bitwise_duration.as_nanos() / iterations as u128;

        total_lut_time += lut_ns;
        total_bitwise_time += bitwise_ns;

        let speedup = bitwise_ns as f64 / lut_ns as f64;

        println!(
            "sqrt({:3}): LUT={:3}ns  Bitwise={:3}ns  Speedup={:.2}×",
            val, lut_ns, bitwise_ns, speedup
        );

        // Verify results match
        let lut_result = input.sqrt();
        let bitwise_result = input.sqrt_bb();
        if lut_result.fraction != bitwise_result.fraction
            || lut_result.exponent != bitwise_result.exponent
        {
            println!(
                "  ⚠️  MISMATCH! LUT: {:?}, Bitwise: {:?}",
                lut_result, bitwise_result
            );
        }
    }

    let avg_lut = total_lut_time / test_values.len() as u128;
    let avg_bitwise = total_bitwise_time / test_values.len() as u128;
    let avg_speedup = avg_bitwise as f64 / avg_lut as f64;

    println!("\n========================================");
    println!("Average Results:");
    println!(
        "  LUT-Newton:  {} ns/op  ({:.2} M ops/sec)",
        avg_lut,
        1000.0 / avg_lut as f64
    );
    println!(
        "  Bitwise:     {} ns/op  ({:.2} M ops/sec)",
        avg_bitwise,
        1000.0 / avg_bitwise as f64
    );
    println!("  Speedup:     {:.2}×", avg_speedup);
    println!("========================================");
}
