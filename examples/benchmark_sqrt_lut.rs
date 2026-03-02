use spirix::ScalarF4E4;
use std::time::Instant;

fn main() {
    println!("sqrt() Benchmark - LUT-seeded Newton-Raphson");
    println!("==============================================\n");

    // Test values covering the range
    let test_values: Vec<u8> = vec![
        2, 3, 4, 5, 9, 16, 25, 36, 49, 64, 81, 100, 121, 144, 169, 196, 225, 255,
    ];

    println!("Warmup...");
    for _ in 0..10000 {
        let val = ScalarF4E4::from(100u8);
        let _ = val.sqrt();
    }

    println!("Running benchmark...\n");

    // Benchmark each value
    for &val in &test_values {
        let input = ScalarF4E4::from(val);
        let iterations = 1_000_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = input.sqrt();
        }
        let duration = start.elapsed();

        let ns_per_op = duration.as_nanos() / iterations as u128;
        let result = input.sqrt();

        println!(
            "sqrt({:3}) = {:.3}  |  {} ns/op",
            val,
            (result.fraction as f64) * 2f64.powi(result.exponent as i32 - 15),
            ns_per_op
        );
    }

    println!("\n==============================================");
    println!("Average performance across test cases:");

    // Overall benchmark
    let iterations = 10_000_000;
    let start = Instant::now();
    for i in 0..iterations {
        let val = ((i % 254) + 1) as u8;
        let input = ScalarF4E4::from(val);
        let _ = input.sqrt();
    }
    let duration = start.elapsed();
    let ns_per_op = duration.as_nanos() / iterations as u128;

    println!("Average: {} ns/op", ns_per_op);
    println!("Throughput: {:.2} M ops/sec", 1000.0 / ns_per_op as f64);
}
