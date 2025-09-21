use criterion::{black_box, criterion_group, criterion_main, Criterion};
use spirix::*;
use std::hint::black_box as hint_black_box;

// Performance benchmarks comparing Spirix operations with standard floating-point

fn benchmark_scalar_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_arithmetic");

    // Test data
    let spirix_a = ScalarF5E3::from(42.0);
    let spirix_b = ScalarF5E3::from(17.0);
    let std_a = 42.0f32;
    let std_b = 17.0f32;

    // Addition benchmarks
    group.bench_function("spirix_addition", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) + black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_addition", |b| {
        b.iter(|| {
            let result = black_box(std_a) + black_box(std_b);
            hint_black_box(result)
        })
    });

    // Multiplication benchmarks
    group.bench_function("spirix_multiplication", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) * black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_multiplication", |b| {
        b.iter(|| {
            let result = black_box(std_a) * black_box(std_b);
            hint_black_box(result)
        })
    });

    // Division benchmarks
    group.bench_function("spirix_division", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) / black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_division", |b| {
        b.iter(|| {
            let result = black_box(std_a) / black_box(std_b);
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_scalar_transcendental(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_transcendental");

    let spirix_val = ScalarF5E3::from(2.5);
    let std_val = 2.5f32;

    // Sine function
    group.bench_function("spirix_sin", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).sin();
            hint_black_box(result)
        })
    });

    group.bench_function("std_sin", |b| {
        b.iter(|| {
            let result = black_box(std_val).sin();
            hint_black_box(result)
        })
    });

    // Natural logarithm
    group.bench_function("spirix_ln", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).ln();
            hint_black_box(result)
        })
    });

    group.bench_function("std_ln", |b| {
        b.iter(|| {
            let result = black_box(std_val).ln();
            hint_black_box(result)
        })
    });

    // Exponential function
    group.bench_function("spirix_exp", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).exp();
            hint_black_box(result)
        })
    });

    group.bench_function("std_exp", |b| {
        b.iter(|| {
            let result = black_box(std_val).exp();
            hint_black_box(result)
        })
    });

    // Square root
    group.bench_function("spirix_sqrt", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).sqrt();
            hint_black_box(result)
        })
    });

    group.bench_function("std_sqrt", |b| {
        b.iter(|| {
            let result = black_box(std_val).sqrt();
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_complex_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_arithmetic");

    let spirix_a = CircleF5E3::from((3.0, 4.0));
    let spirix_b = CircleF5E3::from((2.0, -1.0));
    let std_a = num_complex::Complex::new(3.0f32, 4.0f32);
    let std_b = num_complex::Complex::new(2.0f32, -1.0f32);

    // Complex addition
    group.bench_function("spirix_complex_add", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) + black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_complex_add", |b| {
        b.iter(|| {
            let result = black_box(std_a) + black_box(std_b);
            hint_black_box(result)
        })
    });

    // Complex multiplication
    group.bench_function("spirix_complex_mul", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) * black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_complex_mul", |b| {
        b.iter(|| {
            let result = black_box(std_a) * black_box(std_b);
            hint_black_box(result)
        })
    });

    // Complex magnitude
    group.bench_function("spirix_complex_magnitude", |b| {
        b.iter(|| {
            let result = black_box(spirix_a).magnitude();
            hint_black_box(result)
        })
    });

    group.bench_function("std_complex_magnitude", |b| {
        b.iter(|| {
            let result = black_box(std_a).norm();
            hint_black_box(result)
        })
    });

    // Complex conjugate
    group.bench_function("spirix_complex_conjugate", |b| {
        b.iter(|| {
            let result = black_box(spirix_a).conjugate();
            hint_black_box(result)
        })
    });

    group.bench_function("std_complex_conjugate", |b| {
        b.iter(|| {
            let result = black_box(std_a).conj();
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_precision_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("precision_comparison");

    // Compare different Spirix precisions
    let low_prec = ScalarF3E3::from(42.0);
    let med_prec = ScalarF5E3::from(42.0);
    let high_prec = ScalarF7E4::from(42.0);

    group.bench_function("low_precision_mul", |b| {
        b.iter(|| {
            let result = black_box(low_prec) * black_box(low_prec);
            hint_black_box(result)
        })
    });

    group.bench_function("med_precision_mul", |b| {
        b.iter(|| {
            let result = black_box(med_prec) * black_box(med_prec);
            hint_black_box(result)
        })
    });

    group.bench_function("high_precision_mul", |b| {
        b.iter(|| {
            let result = black_box(high_prec) * black_box(high_prec);
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_conversion_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("conversion_overhead");

    let float_val = 42.5f32;
    let spirix_val = ScalarF5E3::from(42.5);

    // Conversion to Spirix
    group.bench_function("float_to_spirix", |b| {
        b.iter(|| {
            let result = ScalarF5E3::from(black_box(float_val));
            hint_black_box(result)
        })
    });

    // Conversion from Spirix
    group.bench_function("spirix_to_float", |b| {
        b.iter(|| {
            let result: f32 = black_box(spirix_val).into();
            hint_black_box(result)
        })
    });

    // Round-trip conversion
    group.bench_function("round_trip_conversion", |b| {
        b.iter(|| {
            let spirix = ScalarF5E3::from(black_box(float_val));
            let result: f32 = spirix.into();
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_state_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_checking");

    let normal = ScalarF5E3::from(42.0);
    let zero = ScalarF5E3::ZERO;
    let undefined = ScalarF5E3::from(1.0) / ScalarF5E3::ZERO;

    // Normal state checking
    group.bench_function("is_normal", |b| {
        b.iter(|| {
            let result = black_box(normal).is_normal();
            hint_black_box(result)
        })
    });

    group.bench_function("is_zero", |b| {
        b.iter(|| {
            let result = black_box(zero).is_zero();
            hint_black_box(result)
        })
    });

    group.bench_function("is_undefined", |b| {
        b.iter(|| {
            let result = black_box(undefined).is_undefined();
            hint_black_box(result)
        })
    });

    group.bench_function("is_finite", |b| {
        b.iter(|| {
            let result = black_box(normal).is_finite();
            hint_black_box(result)
        })
    });

    // Compare with standard float checks
    let std_normal = 42.0f32;
    let std_nan = f32::NAN;

    group.bench_function("std_is_normal", |b| {
        b.iter(|| {
            let result = black_box(std_normal).is_normal();
            hint_black_box(result)
        })
    });

    group.bench_function("std_is_nan", |b| {
        b.iter(|| {
            let result = black_box(std_nan).is_nan();
            hint_black_box(result)
        })
    });

    group.bench_function("std_is_finite", |b| {
        b.iter(|| {
            let result = black_box(std_normal).is_finite();
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_chained_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("chained_operations");

    let spirix_val = ScalarF5E3::from(2.5);
    let std_val = 2.5f32;

    // Chain of arithmetic operations
    group.bench_function("spirix_arithmetic_chain", |b| {
        b.iter(|| {
            let val = black_box(spirix_val);
            let result = val.square().sqrt().reciprocal().square();
            hint_black_box(result)
        })
    });

    group.bench_function("std_arithmetic_chain", |b| {
        b.iter(|| {
            let val = black_box(std_val);
            let result = val.powi(2).sqrt().recip().powi(2);
            hint_black_box(result)
        })
    });

    // Chain of transcendental operations
    group.bench_function("spirix_transcendental_chain", |b| {
        b.iter(|| {
            let val = black_box(spirix_val);
            let result = val.sin().exp().ln().cos();
            hint_black_box(result)
        })
    });

    group.bench_function("std_transcendental_chain", |b| {
        b.iter(|| {
            let val = black_box(std_val);
            let result = val.sin().exp().ln().cos();
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_random_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_generation");

    // Random scalar generation
    group.bench_function("spirix_random_scalar", |b| {
        b.iter(|| {
            let result = ScalarF5E3::random();
            hint_black_box(result)
        })
    });

    group.bench_function("spirix_random_gauss_scalar", |b| {
        b.iter(|| {
            let result = ScalarF5E3::random_gauss();
            hint_black_box(result)
        })
    });

    // Random complex generation
    group.bench_function("spirix_random_circle", |b| {
        b.iter(|| {
            let result = CircleF5E3::random();
            hint_black_box(result)
        })
    });

    group.bench_function("spirix_random_gauss_circle", |b| {
        b.iter(|| {
            let result = CircleF5E3::random_gauss();
            hint_black_box(result)
        })
    });

    // Compare with standard random
    use rand::Rng;
    group.bench_function("std_random_f32", |b| {
        b.iter(|| {
            let mut rng = rand::thread_rng();
            let result: f32 = rng.gen_range(-1.0..1.0);
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_bitwise_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitwise_operations");

    let spirix_a = ScalarF5E3::from(0b11110000);
    let spirix_b = ScalarF5E3::from(0b10101010);
    let int_a = 0b11110000u32;
    let int_b = 0b10101010u32;

    // Bitwise AND
    group.bench_function("spirix_bitwise_and", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) & black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_bitwise_and", |b| {
        b.iter(|| {
            let result = black_box(int_a) & black_box(int_b);
            hint_black_box(result)
        })
    });

    // Bitwise OR
    group.bench_function("spirix_bitwise_or", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) | black_box(spirix_b);
            hint_black_box(result)
        })
    });

    group.bench_function("std_bitwise_or", |b| {
        b.iter(|| {
            let result = black_box(int_a) | black_box(int_b);
            hint_black_box(result)
        })
    });

    // Left shift
    group.bench_function("spirix_left_shift", |b| {
        b.iter(|| {
            let result = black_box(spirix_a) << 2;
            hint_black_box(result)
        })
    });

    group.bench_function("std_left_shift", |b| {
        b.iter(|| {
            let result = black_box(int_a) << 2;
            hint_black_box(result)
        })
    });

    group.finish();
}

fn benchmark_mathematical_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("mathematical_functions");

    let spirix_val = ScalarF5E3::from(5.0);
    let std_val = 5.0f32;

    // Power function
    group.bench_function("spirix_power", |b| {
        b.iter(|| {
            let base = black_box(spirix_val);
            let exp = ScalarF5E3::from(2.5);
            let result = base.pow(exp);
            hint_black_box(result)
        })
    });

    group.bench_function("std_power", |b| {
        b.iter(|| {
            let result = black_box(std_val).powf(2.5);
            hint_black_box(result)
        })
    });

    // Hyperbolic functions
    group.bench_function("spirix_sinh", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).sinh();
            hint_black_box(result)
        })
    });

    group.bench_function("std_sinh", |b| {
        b.iter(|| {
            let result = black_box(std_val).sinh();
            hint_black_box(result)
        })
    });

    // Inverse trigonometric
    let small_val = ScalarF5E3::from(0.5);
    let small_std = 0.5f32;

    group.bench_function("spirix_asin", |b| {
        b.iter(|| {
            let result = black_box(small_val).asin();
            hint_black_box(result)
        })
    });

    group.bench_function("std_asin", |b| {
        b.iter(|| {
            let result = black_box(small_std).asin();
            hint_black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_scalar_arithmetic,
    benchmark_scalar_transcendental,
    benchmark_complex_arithmetic,
    benchmark_precision_comparison,
    benchmark_conversion_overhead,
    benchmark_state_checking,
    benchmark_chained_operations,
    benchmark_random_generation,
    benchmark_bitwise_operations,
    benchmark_mathematical_functions
);

criterion_main!(benches);
