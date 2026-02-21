use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use spirix::{ScalarF3E3, ScalarF4E4, ScalarF5E5, ScalarF6E6};

fn bench_sqrt_8bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqrt_8bit");

    let input = ScalarF3E3::from(100i8);

    group.bench_function("newton", |b| b.iter(|| black_box(input).sqrt()));

    group.bench_function("bitwise", |b| b.iter(|| black_box(input).sqrt_bb()));

    group.finish();
}

fn bench_sqrt_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqrt_16bit");

    let input = ScalarF4E4::from(100u8);

    group.bench_function("newton", |b| b.iter(|| black_box(input).sqrt()));

    group.bench_function("bitwise", |b| b.iter(|| black_box(input).sqrt_bb()));

    group.finish();
}

fn bench_sqrt_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqrt_32bit");

    let input = ScalarF5E5::from(10000u16);

    group.bench_function("newton", |b| b.iter(|| black_box(input).sqrt()));

    group.bench_function("bitwise", |b| b.iter(|| black_box(input).sqrt_bb()));

    group.finish();
}

fn bench_sqrt_64bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqrt_64bit");

    let input = ScalarF6E6::from(10000u16);

    group.bench_function("newton", |b| b.iter(|| black_box(input).sqrt()));

    group.bench_function("bitwise", |b| b.iter(|| black_box(input).sqrt_bb()));

    group.finish();
}

criterion_group!(
    benches,
    bench_sqrt_8bit,
    bench_sqrt_16bit,
    bench_sqrt_32bit,
    bench_sqrt_64bit
);
criterion_main!(benches);
