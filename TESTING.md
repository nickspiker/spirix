# Testing Guide for Spirix

This document explains how to use the test suite for the Spirix library.

## Test Structure

The tests are organized into several categories:

### Unit Tests

#### Scalar Tests (`tests/scalar_tests.rs`)
- **Basic Operations**: Addition, subtraction, multiplication, division
- **Special Values**: Zero, infinity, exploded, vanished, and undefined states
- **Mathematical Functions**: Trigonometry, logarithms, exponentials, powers
- **Comparison Operations**: Min, max, clamp, equality
- **Bitwise Operations**: AND, OR, XOR, shifts
- **State Checking**: Normal, finite, escaped values
- **Random Generation**: Uniform and Gaussian distributions
- **Precision Configurations**: Different F×E combinations

#### Circle Tests (`tests/circle_tests.rs`)
- **Complex Arithmetic**: Addition, subtraction, multiplication, division
- **Complex-Specific Operations**: Conjugate, magnitude, unit vectors
- **Circle-Scalar Interactions**: Mixed operations with real numbers
- **Complex Mathematical Functions**: Powers, exponentials, logarithms
- **Modular Operations**: Component-wise remainder operations
- **State Preservation**: Phase information thru operations

### Property-Based Tests (`tests/properties.rs`)
Uses the `proptest` crate to verify mathematical properties with random inputs:
- **Arithmetic Properties**: Commutativity, associativity, distributivity
- **Identity Elements**: Additive and multiplicative identities
- **Inverse Operations**: Additive and multiplicative inverses
- **Function Relationships**: exp/ln, sin²+cos²=1, sqrt/square
- **Complex Properties**: Conjugate properties, magnitude relationships
- **State Invariants**: Consistency of internal state representation

### Integration Tests (`tests/integration.rs`)
- **Cross-Precision Operations**: Interactions between different precision types
- **Mixed-Type Operations**: Scalar-Circle interactions, Rust primitive compatibility
- **Chained Operations**: Complex calculation chains preserving correctness
- **Real-World Scenarios**: Numerical integration, signal processing, financial calculations
- **Precision Boundaries**: Behavior near overflow/underflow limits

### Performance Benchmarks (`benches/benchmarks.rs`)
Uses the `criterion` crate to compare Spirix performance against standard floating-point:
- **Arithmetic Operations**: Basic operations vs f32/f64
- **Transcendental Functions**: sin, cos, exp, ln performance comparison
- **Complex Numbers**: Spirix Circle vs num-complex library
- **State Checking**: Overhead of state tracking
- **Conversion Costs**: Spirix ↔ Rust primitive conversion overhead

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Suites
```bash
# Run only scalar tests
cargo test --test scalar_tests

# Run only circle tests
cargo test --test circle_tests

# Run only property-based tests
cargo test --test properties

# Run only integration tests
cargo test --test integration
```

### Individual Test Functions
```bash
# Run a specific test function
cargo test test_scalar_creation_and_conversion

# Run tests matching a pattern
cargo test trigonometric
```

### Property-Based Test Configuration
Property-based tests use random inputs. You can control the test behavior:

```bash
# Run with more test cases (default is usually 256)
PROPTEST_CASES=1000 cargo test

# Run with a specific seed for reproducibility
PROPTEST_SEED=12345 cargo test

# Disable shrinking (faster but less detailed failures)
PROPTEST_DISABLE_SHRINK=1 cargo test
```

## Running Benchmarks

### All Benchmarks
```bash
cargo bench
```

### Specific Benchmark Groups
```bash
# Run only arithmetic benchmarks
cargo bench -- scalar_arithmetic

# Run only complex number benchmarks
cargo bench -- complex_arithmetic

# Generate HTML reports (requires criterion HTML feature)
cargo bench -- --output-format html
```

Benchmark results will be saved in `target/criterion/` with detailed HTML reports.

## Test Configuration

### Precision Levels Tested
The tests cover various precision configurations:
- `ScalarF3E3` (8-bit fraction, 8-bit exponent) - Low precision, small range
- 'ScalarF#E#' (x-bit fraction, y-bit exponent) - Where x and y are powers of twe between 3 and 7
- `ScalarF7E7` (128-bit fraction, 128-bit exponent) - High precision, large range

## Understanding Test Results

### Normal Test Failures
If tests fail, the output will show:
- Which assertion failed
- Expected vs actual values
- The specific test function and line

### Property-Based Test Failures
Property tests will show:
- The failing input values that triggered the bug
- A "shrunk" minimal example that still fails
- The property that was violated

Example:
```
thread 'properties::scalar_properties::test_scalar_addition_commutativity' panicked at 'Test failed: a=3.7, b=-2.1. Left side: 1.6000000, Right side: 1.6000002'
```

### Performance Regression Detection
Benchmarks will warn if performance significantly degrades between runs. Look for:
- Thruput changes (operations per second)
- Mean execution time changes
- Performance comparison vs standard floating-point

## Adding New Tests

### Unit Tests
Add new test functions using the `#[test]` attribute:

```rust
#[test]
fn test_my_new_feature() {
    let input = ScalarF5E3::from(42.0);
    let result = input.my_new_operation();
    assert!(result.is_normal());

    let value: f32 = result.into();
    assert_relative_eq!(value, expected, epsilon = 1e-5);
}
```

### Property-Based Tests
Add new property tests using `proptest!`:

```rust
proptest! {
    #[test]
    fn test_my_property(
        a in -1000.0f32..1000.0f32,
        b in -1000.0f32..1000.0f32
    ) {
        let sa = ScalarF5E3::from(a);
        let sb = ScalarF5E3::from(b);

        // Test some mathematical property
        let left = my_operation(sa, sb);
        let right = expected_equivalent(sa, sb);

        if left.is_normal() && right.is_normal() {
            let left_val: f32 = left.into();
            let right_val: f32 = right.into();
            assert_relative_eq!(left_val, right_val, epsilon = 1e-4);
        }
    }
}
```

### Benchmarks
Add new benchmarks by extending `benches/benchmarks.rs`:

```rust
fn benchmark_my_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_operation");

    let spirix_val = ScalarF5E3::from(42.0);
    let std_val = 42.0f32;

    group.bench_function("spirix_my_op", |b| {
        b.iter(|| {
            let result = black_box(spirix_val).my_operation();
            hint_black_box(result)
        })
    });

    group.bench_function("std_my_op", |b| {
        b.iter(|| {
            let result = black_box(std_val).my_std_equivalent();
            hint_black_box(result)
        })
    });

    group.finish();
}
```

## Continuous Integration

These tests are designed to run in CI environments:
- All tests should complete within reasonable time limits
- Property-based tests use deterministic seeds in CI
- Benchmarks can be run separately from tests
- The test suite is compatible with `--release` builds for performance testing

## Troubleshooting

### Common Issues

1. **Type Annotation Errors**: Spirix sometimes needs explicit type annotations
   ```rust
   let value: ScalarF5E3 = operation_result;
   ```

2. **Precision Mismatches**: When mixing precisions, convert explicitly
   ```rust
   let high_prec = ScalarF6E4::from(low_prec_value.into(): f32);
   ```

3. **Escaped Value Handling**: Check for exploded/vanished states
   ```rust
   if result.is_normal() {
       // Safe to extract value
       let val: f32 = result.into();
   }
   ```

4. **Random Test Flakiness**: Set explicit seeds for reproducible tests
   ```bash
   PROPTEST_SEED=42 cargo test
   ```

This comprehensive test suite ensures that Spirix maintains mathematical correctness, performance characteristics, and API compatibility across all supported operations and precision levels.