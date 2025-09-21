use spirix::*;
use std::time::{Duration, Instant};

/// Performance regression tests to ensure Spirix maintains reasonable performance characteristics
/// These tests verify that operations complete within expected time bounds

const PERFORMANCE_ITERATIONS: usize = 10000;
const MAX_OPERATION_NANOS: u128 = 10000; // 10 microseconds per operation max

#[test]
fn test_scalar_arithmetic_performance() {
    let values: Vec<ScalarF5E3> = (0..1000)
        .map(|i| ScalarF5E3::from(i as f32 * 0.1))
        .collect();

    // Test addition performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = values[42] + values[43];
        std::hint::black_box(result); // Prevent optimization
    }
    let addition_time = start.elapsed();
    assert!(
        addition_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS,
        "Addition too slow: {} ns per operation",
        addition_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );

    // Test multiplication performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = values[42] * values[43];
        std::hint::black_box(result);
    }
    let mult_time = start.elapsed();
    assert!(
        mult_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS,
        "Multiplication too slow: {} ns per operation",
        mult_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );

    // Test division performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = values[42] / values[43];
        std::hint::black_box(result);
    }
    let div_time = start.elapsed();
    assert!(
        div_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS * 5,
        "Division too slow: {} ns per operation",
        div_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );
}

#[test]
fn test_circle_arithmetic_performance() {
    let circles: Vec<CircleF5E3> = (0..1000)
        .map(|i| {
            CircleF5E3::new(
                ScalarF5E3::from(i as f32 * 0.1),
                ScalarF5E3::from(i as f32 * 0.2),
            )
        })
        .collect();

    // Test complex addition performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = circles[42] + circles[43];
        std::hint::black_box(result);
    }
    let addition_time = start.elapsed();
    assert!(
        addition_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS * 2,
        "Complex addition too slow: {} ns per operation",
        addition_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );

    // Test complex multiplication performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = circles[42] * circles[43];
        std::hint::black_box(result);
    }
    let mult_time = start.elapsed();
    assert!(
        mult_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS * 8,
        "Complex multiplication too slow: {} ns per operation",
        mult_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );
}

#[test]
fn test_transcendental_function_performance() {
    let values: Vec<ScalarF5E3> = (1..1000)
        .map(|i| ScalarF5E3::from(i as f32 * 0.01))
        .collect();

    // Test sin performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS / 10 {
        // Fewer iterations for expensive operations
        let result = values[42].sin();
        std::hint::black_box(result);
    }
    let sin_time = start.elapsed();
    assert!(
        sin_time.as_nanos() / ((PERFORMANCE_ITERATIONS / 10) as u128) < MAX_OPERATION_NANOS * 100,
        "Sin too slow: {} ns per operation",
        sin_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 / 10)
    );

    // Test exp performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS / 10 {
        let result = values[42].exp();
        std::hint::black_box(result);
    }
    let exp_time = start.elapsed();
    assert!(
        exp_time.as_nanos() / ((PERFORMANCE_ITERATIONS / 10) as u128) < MAX_OPERATION_NANOS * 100,
        "Exp too slow: {} ns per operation",
        exp_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 / 10)
    );

    // Test ln performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS / 10 {
        let result = values[42].ln();
        std::hint::black_box(result);
    }
    let ln_time = start.elapsed();
    assert!(
        ln_time.as_nanos() / ((PERFORMANCE_ITERATIONS / 10) as u128) < MAX_OPERATION_NANOS * 100,
        "Ln too slow: {} ns per operation",
        ln_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 / 10)
    );
}

#[test]
fn test_conversion_performance() {
    let f32_values: Vec<f32> = (0..1000).map(|i| i as f32 * 0.1).collect();
    let scalar_values: Vec<ScalarF5E3> = f32_values.iter().map(|&f| ScalarF5E3::from(f)).collect();

    // Test f32 -> Scalar conversion performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result = ScalarF5E3::from(f32_values[42]);
        std::hint::black_box(result);
    }
    let from_f32_time = start.elapsed();
    assert!(
        from_f32_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS,
        "f32 -> Scalar conversion too slow: {} ns per operation",
        from_f32_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );

    // Test Scalar -> f32 conversion performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let result: f32 = scalar_values[42].into();
        std::hint::black_box(result);
    }
    let to_f32_time = start.elapsed();
    assert!(
        to_f32_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128) < MAX_OPERATION_NANOS,
        "Scalar -> f32 conversion too slow: {} ns per operation",
        to_f32_time.as_nanos() / PERFORMANCE_ITERATIONS as u128
    );
}

#[test]
fn test_state_checking_performance() {
    let values: Vec<ScalarF5E3> = vec![
        ScalarF5E3::from(42.0),
        ScalarF5E3::ZERO,
        ScalarF5E3::INFINITY,
        ScalarF5E3::MAX * ScalarF5E3::from(2.0), // exploded
        ScalarF5E3::MIN_POSITIVE / ScalarF5E3::from(2.0), // vanished
        ScalarF5E3::ZERO / ScalarF5E3::ZERO,     // undefined
    ];

    // Test is_normal performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        for value in &values {
            let result = value.is_normal();
            std::hint::black_box(result);
        }
    }
    let normal_check_time = start.elapsed();
    assert!(
        normal_check_time.as_nanos() / ((PERFORMANCE_ITERATIONS * values.len()) as u128) < 100,
        "is_normal too slow: {} ns per operation",
        normal_check_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 * values.len() as u128)
    );

    // Test is_zero performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        for value in &values {
            let result = value.is_zero();
            std::hint::black_box(result);
        }
    }
    let zero_check_time = start.elapsed();
    assert!(
        zero_check_time.as_nanos() / ((PERFORMANCE_ITERATIONS * values.len()) as u128) < 100,
        "is_zero too slow: {} ns per operation",
        zero_check_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 * values.len() as u128)
    );

    // Test normalization_level performance
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        for value in &values {
            let result = value.normalization_level();
            std::hint::black_box(result);
        }
    }
    let level_check_time = start.elapsed();
    assert!(
        level_check_time.as_nanos() / ((PERFORMANCE_ITERATIONS * values.len()) as u128) < 1000,
        "normalization_level too slow: {} ns per operation",
        level_check_time.as_nanos() / (PERFORMANCE_ITERATIONS as u128 * values.len() as u128)
    );
}

#[test]
fn test_precision_scaling_performance() {
    // Test that higher precision doesn't cause exponential slowdown
    let value_f32 = 42.0f32;

    // Low precision timing
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let a = ScalarF3E3::from(value_f32);
        let b = ScalarF3E3::from(value_f32 * 2.0);
        let result = a + b * a / b;
        std::hint::black_box(result);
    }
    let low_precision_time = start.elapsed();

    // High precision timing
    let start = Instant::now();
    for _ in 0..PERFORMANCE_ITERATIONS {
        let a = ScalarF7E7::from(value_f32);
        let b = ScalarF7E7::from(value_f32 * 2.0);
        let result = a + b * a / b;
        std::hint::black_box(result);
    }
    let high_precision_time = start.elapsed();

    // High precision should not be more than 100x slower than low precision
    let slowdown_factor =
        high_precision_time.as_nanos() as f64 / low_precision_time.as_nanos() as f64;
    assert!(
        slowdown_factor < 100.0,
        "High precision is {}x slower than low precision (should be < 100x)",
        slowdown_factor
    );
}

#[test]
fn test_bulk_operation_performance() {
    // Test performance with arrays of values (common use case)
    let size = 1000;
    let values1: Vec<ScalarF5E3> = (0..size).map(|i| ScalarF5E3::from(i as f32)).collect();
    let values2: Vec<ScalarF5E3> = (0..size)
        .map(|i| ScalarF5E3::from((i + 1) as f32))
        .collect();

    // Test bulk addition
    let start = Instant::now();
    let mut results = Vec::with_capacity(size);
    for i in 0..size {
        results.push(values1[i] + values2[i]);
    }
    let bulk_add_time = start.elapsed();
    std::hint::black_box(results);

    assert!(
        bulk_add_time.as_nanos() / (size as u128) < MAX_OPERATION_NANOS * 2,
        "Bulk addition too slow: {} ns per operation",
        bulk_add_time.as_nanos() / size as u128
    );

    // Test bulk multiplication
    let start = Instant::now();
    let mut results = Vec::with_capacity(size);
    for i in 0..size {
        results.push(values1[i] * values2[i]);
    }
    let bulk_mult_time = start.elapsed();
    std::hint::black_box(results);

    assert!(
        bulk_mult_time.as_nanos() / (size as u128) < MAX_OPERATION_NANOS * 2,
        "Bulk multiplication too slow: {} ns per operation",
        bulk_mult_time.as_nanos() / size as u128
    );
}

#[test]
fn test_memory_allocation_performance() {
    // Test that Spirix types don't cause excessive allocations
    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        // Create many temporary values
        let mut sum = ScalarF5E3::ZERO;
        for i in 0..100 {
            let val = ScalarF5E3::from(i as f32);
            sum = sum + val;
        }
        std::hint::black_box(sum);
    }
    let allocation_time = start.elapsed();

    // This should complete quickly since Spirix types are stack-allocated
    assert!(
        allocation_time.as_millis() < 100,
        "Memory allocation pattern too slow: {} ms for {} iterations",
        allocation_time.as_millis(),
        iterations
    );
}

#[test]
fn test_comparison_with_native_floats() {
    // Compare Spirix performance with native f32 operations
    let iterations = PERFORMANCE_ITERATIONS;
    let values: Vec<f32> = (0..1000).map(|i| i as f32 * 0.1).collect();

    // Native f32 addition
    let start = Instant::now();
    for _ in 0..iterations {
        let result = values[42] + values[43];
        std::hint::black_box(result);
    }
    let native_add_time = start.elapsed();

    // Spirix addition
    let spirix_values: Vec<ScalarF5E3> = values.iter().map(|&f| ScalarF5E3::from(f)).collect();
    let start = Instant::now();
    for _ in 0..iterations {
        let result = spirix_values[42] + spirix_values[43];
        std::hint::black_box(result);
    }
    let spirix_add_time = start.elapsed();

    // Spirix should be at most 50x slower than native (allowing for additional features)
    let slowdown = spirix_add_time.as_nanos() as f64 / native_add_time.as_nanos() as f64;
    assert!(
        slowdown < 50.0,
        "Spirix addition is {}x slower than native f32 (should be < 50x)",
        slowdown
    );

    println!(
        "Performance comparison - Spirix is {:.1}x slower than native f32",
        slowdown
    );
}
