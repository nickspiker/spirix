# Spirix

<div align="center">
  <img src="spirix.png" alt="Spirix Logo">
  <h3>Two's Complement Floating Point Arithmetic with Adjustable Precision</h3>
</div>

[![Crates.io](https://img.shields.io/crates/v/spirix.svg)](https://crates.io/crates/spirix)
[![Docs.rs](https://docs.rs/spirix/badge.svg)](https://docs.rs/spirix)
[![License](https://img.shields.io/badge/license-Custom-blue.svg)](LICENSE)

## ⚠️ Beta Warning

**This is early beta software under active development.** While the core arithmetic operations and many mathematical functions are tested and working, this library is not ready for production use. Please do not use this library in critical systems or applications where incorrect calculations could cause harm.

Current status:
- ✅ Core arithmetic operations (addition, subtraction, multiplication, division)
- ✅ Basic mathematical functions (sqrt, power, etc.)
- ✅ Complex number support
- ⚠️ Advanced mathematical functions still under development
- ⚠️ API may change in future versions

Use at your own risk and always validate results independently for important calculations.

## Overview

Spirix is a high-performance numeric library that implements a fundamentally new approach to floating-point arithmetic by utilizing two's complement representation thruout the entire calculation pipeline. Unlike traditional floating-point implementations that use separate sign bit and magnitude representation, Spirix employs a continuous numeric representation with left-aligned normalized fractions and unbiased exponents.

This approach provides several key advantages:
- **Simplified arithmetic**: Eliminates many branches required by sign-bit implementations
- **Error tracking**: Preserves the origin of undefined operations (compared to a single generic NaN)
- **Continuous mathematics**: Values maintain orientation even beyond standard representation ranges
- **Customizable precision and range**: Independently choose fraction and exponent sizes
- **Complex number support**: Real and imaginary components share the same exponent for efficiency

## Architectural Foundations

### Two's Complement Representation

Spirix's core innovation is using two's complement representation for floating-point numbers, which enables:

- A continuous number line without discontinuity at zero
- Natural operation consistency across zero without special handling
- Elimination of sign-bit testing in arithmetic operations
- More efficient implementation with fewer branches

Where traditional floating-point has a sign bit followed by exponent and mantissa:
```
IEEE-754: [sign bit][biased exponent][mantissa]
```

Spirix uses left-aligned two's complement fractions with unbiased two's compliment exponents:
```
Spirix: [normalized fraction][unbiased exponent]
```

This representation allows for:
- Ambiguous states (non-normal values) to be instantly recognized with a single exponent only check
- Branchless processing in most normal cases
- Continuous arithmetic without sign-specific branching

### State Classification System

Spirix introduces a novel normalization level system that encodes the normal/ambiguous state thru patterns in the most significant bits:

| LSB/N-level | Bit Pattern | Description | Symbol |
|-------|------------|-------------|--------|
| N-0  | □□□□□□□□ | Zero | [0] |
| N-0  | ■■■■■■■■ | General undefined | [℘] |
| N-1  | □■xxxxxx | Positive normal or exploded | [+#], [+↑] |
| N-1  | ■□xxxxxx | Negative normal or exploded | [-#], [-↑] |
| N-2  | □□■xxxxx | Positive vanished | [+↓] |
| N-2  | ■■□xxxxx | Negative vanished | [-↓] |
| N-3+ | □□□xxxxx | Various undefined states | [℘ 'state'] |
| N-3+ | ■■■xxxxx | Various undefined states | [℘ 'state'] |

Note that Zero and general undefined are uniform patterns, both of which have an ambiguous exponent. Each N level is distinguished by checking LSB (leading same bits). Ambiguous values are maraked by having an exponent set to the lowest possible value in the exponent referred to as AMBIGUOUS_EXPONENT. Ambiguous values include Zero, Undefined, Vanished and Exploded. Exploded have LSB of one N-1 (same as normal states but with AMBIGUOUS_EXPONENT). Vanished have LSB of two with AMBIGUOUS EXPONENT. Undefined contain three or more LSB with AMBIGUOUS_EXPONENT. Zero contains a fraction of all zeros, allowing fast determination of a value's normality or ambiguous state without unnecessary testing or branching.

### Escaped Values

When a value exceeds the normal exponent range, Spirix doesn't simply truncate to infinity or zero. Instead, it creates an "escaped" phase that preserve sign or orientation information:

- **Exploded** values (↑): Extremely large numbers that maintain their sign/orientation
- **Vanished** values (↓): Extremely small numbers that maintain their sign/orientation

These escaped values can continue to participate in absolute mathematical operations, like multiplication and division, allowing calculations to proceed even with results beyond the exponent range. Escaped Scalars maintain their sign, escaped Circles maintain their angle or complex sign thru absolute operations.

## The Type System

### Core Types

Spirix provides two primary numeric types:

1. **Scalar<F, E>**: Real numbers with fraction `F` and exponent `E`
2. **Circle<F, E>**: Complex numbers with fractions `F` and shared exponent `E`

Both types are parameterized to allow independent selection of fraction and exponent sizes based on application needs.

### Configurable Precision and Range

Spirix provides a flexible configuration system allowing independent selection of fraction and exponent sizes. This enables applications to precisely tune numerical behavior based on their specific needs.


| | 2.2 digits | 4.6 digits | 9.4 digits | 19 digits | 38.3 digits |
|-------|-----------|-----------|-----------|-----------|-----------|
| 10^±2.1 | F3E3<br>⟨i8, i8⟩ | F4E3<br>⟨i16, i8⟩ | F5E3<br>⟨i32, i8⟩ | F6E3<br>⟨i64, i8⟩ | F7E3<br>⟨i128, i8⟩ |
| 10^±4.5 | F3E4<br>⟨i8, i16⟩ | F4E4<br>⟨i16, i16⟩ | F5E4<br>⟨i32, i16⟩ | F6E4<br>⟨i64, i16⟩ | F7E4<br>⟨i128, i16⟩ |
| 10^±9.3 | F3E5<br>⟨i8, i32⟩ | F4E5<br>⟨i16, i32⟩ | F5E5<br>⟨i32, i32⟩ | F6E5<br>⟨i64, i32⟩ | F7E5<br>⟨i128, i32⟩ |
| 10^±18.9 | F3E6<br>⟨i8, i64⟩ | F4E6<br>⟨i16, i64⟩ | F5E6<br>⟨i32, i64⟩ | F6E6<br>⟨i64, i64⟩ | F7E6<br>⟨i128, i64⟩ |
| 10^±38.2 | F3E7<br>⟨i8, i128⟩ | F4E7<br>⟨i16, i128⟩ | F5E7<br>⟨i32, i128⟩ | F6E7<br>⟨i64, i128⟩ | F7E7<br>⟨i128, i128⟩ |

Scalar values are calculated as:
```
value = fraction × 2^exponent
```

Circle values have real and imaginary phases sharing a common exponent:
```
value = (real + imaginary × i) × 2^exponent
```

## States and Representations

### Normal Values [+#], [-#]

Normal values have definite magnitudes and participate fully in all arithmetic operations:

```
□■xxxxxx... - Positive normal numbers  
■□xxxxxx... - Negative normal numbers  
```

Normal numbers have:
- A non-ambiguous exponent
- A normalized fraction in the N-1 level
- A definite magnitude

### Zero [0]

Zero is represented with a unique bit pattern in the fraction:

```
□□□□□□□□... - Zero
```
All zeros. Zero is ambiguous, thus has exponent equal to AMBIGIOUS_EXPONENT

### Escaped Values [↑], [↓]

When values exceed the representable range, they become "escaped" values:

- **Exploded** values [↑]: Numbers too large to represent with current exponent range
  ```
  □■xxxxxx... N-1 fraction with AMBIGUOUS_EXPONENT - Positive exploded [+↑]
  ■□xxxxxx... N-1 fraction with AMBIGUOUS_EXPONENT - Negative exploded [-↑]
  ```

- **Vanished** values [↓]: Numbers too small to represent with current exponent range
  ```
  □□■xxxxx... N-2 fraction with AMBIGUOUS_EXPONENT - Positive vanished [+↓]
  ■■□xxxxx... N-2 fraction with AMBIGUOUS_EXPONENT - Negative vanished [-↓]
  ```
Escaped values maintain mathematical continuity and can participate in absolute operations while preserving phase information.

### Undefined States [℘]

When operations produce mathematically undefined results, Spirix returns distinct undefined states that preserve the cause:

```
■■■■■■■■... with AMBIGUOUS_EXPONENT - General undefined
□□□■xxxx... with AMBIGUOUS_EXPONENT - Specific undefined states
■■■□xxxx... with AMBIGUOUS_EXPONENT - Specific undefined states
```

Undefined states propagate thru operations, preserving their cause.

## Mathematical Operations

### Arithmetic Operations
Note that Rust primitives like f32, i8 are treated as Scalars. For convenience, Circles can be converted to/from Num::Complex::<f32> or <f64>, where circle.r() and circle.i() return normalized Scalars from each respective component. A Circle can be constructed from a single Scalar/Rust primitive CircleF3E3::from(3),CircleF3E3::from(ScalarF),

Spirix Rust native operations supported:

# Spirix Mathematical Operations

## Basic Arithmetic Operations
```rust
// Basic operations
let sum = a + b;      // Addition
let diff = a - b;     // Subtraction
let product = a * b;  // Multiplication
let quotient = a / b; // Division
let negated = -a;     // Negation
let reciprocal = a.reciprocal();  // 1/a
```

## Modulo and Bit Operations
```rust
// Modulo operations
let remainder = a % b;           // Mathematical remainder
let component_mod = a.modulo(b); // Component-wise remainder (for Circle)

// Aligned bitwise operations
let bit_and = a & b;      // Bitwise AND
let bit_or = a | b;       // Bitwise OR
let bit_xor = a ^ b;      // Bitwise XOR
let bit_not = !a;         // Bitwise NOT
let left_shift = a << 2;  // Left shift by integer (multiply by 2^2)
let right_shift = a >> 1; // Right shift by integer (divide by 2^1)
```

## Complex Number Operations
```rust
// Complex-specific operations
let conj = z.conjugate();                // Complex conjugate
let mag = z.magnitude();                 // Distance from origin
let mag_squared = z.magnitude_squared(); // Squared magnitude (faster)
let unit = z.sign();                     // Unit vector in same direction
let components = (z.r(), z.i());         // Extract real and imaginary parts as Scalars
```

## Power and Logarithmic Functions
```rust
// Power operations
let squared = x.square(); // x*x
let root = x.sqrt();      // Square root
let result = x.pow(y);    // Power

// Logarithmic operations
let natural_log = x.ln();     // Natural logarithm (base e)
let binary_log = x.lb();      // Binary logarithm (base 2)
let custom_log = x.log(base); // Logarithm with custom base
let exponential = x.exp();    // e^x
let power_of_two = x.powb();  // 2^x
```

## Trigonometric Functions
```rust
// Standard trigonometric functions
let sine = x.sin();    // Sine
let cosine = x.cos();  // Cosine
let tangent = x.tan(); // Tangent

// Inverse trigonometric functions
let arcsine = x.asin();    // Arc sine
let arccosine = x.acos();  // Arc cosine
let arctangent = x.atan(); // Arc tangent
let atan2 = y.atan2(x);    // Two-argument arctangent

// Hyperbolic functions
let hyperbolic_sine = x.sinh();             // Hyperbolic sine
let hyperbolic_cosine = x.cosh();           // Hyperbolic cosine
let hyperbolic_tangent = x.tanh();          // Hyperbolic tangent
let inverse_hyperbolic_sine = x.asinh();    // Inverse hyperbolic sine
let inverse_hyperbolic_cosine = x.acosh();  // Inverse hyperbolic cosine
let inverse_hyperbolic_tangent = x.atanh(); // Inverse hyperbolic tangent
```

## Integer-Related Operations
```rust
// Integer functions
let floor_value = x.floor(); // Greatest integer ≤ x
let ceiling = x.ceil();      // Smallest integer ≥ x
let rounded = x.round();     // Nearest integer, ties to even
let fractional = x.frac();   // Part after decimal

// Number properties
let is_integer = x.is_integer();       // Checks if value is an integer
let is_contiguous = x.is_contiguous(); // Checks if in contiguous range
let is_prime = x.is_prime();           // Tests for primality
```

## Special Value Testing
```rust
// State testing
let normal = x.is_normal();         // Standard numeric value
let zero = x.is_zero();             // Actual Zero
let negligible = x.is_negligible(); // Zero or vanished
let tiny = x.vanished();            // Escaped small value
let huge = x.exploded();            // Escaped large value
let undefined = x.is_undefined();   // Undefined state
let finite = x.is_finite();         // Normal or Zero

// Sign testing
let positive = x.is_positive(); // Greater than Zero
let negative = x.is_negative(); // Less than Zero
```

## Random Number Generation
```rust
// Random values
let uniform = ScalarF5E3::random();        // Uniform -1 to 1 distribution
let gaussian = ScalarF5E3::random_gauss(); // Normal distribution

// For complex numbers
let complex_uniform = CircleF5E3::random();        // Uniform inside unit circle
let complex_gaussian = CircleF5E3::random_gauss(); // Normal distribution
```

## Value Comparison Operations
```rust
// Min, max, clamp
let minimum = a.min(b);              // Smaller value
let maximum = a.max(b);              // Larger value
let constrained = x.clamp(min, max); // Value within bounds
```

## Undefined State Catalog

Spirix tracks the cause of undefined operations with specific bit patterns:

| Undefined State | Description |
|-----------------|-------------|
| `℘ /0` | Division by zero |
| `℘ 0^0` | Zero raised to zero power |
| `℘ √-` | Square root of negative number |
| `℘ ↑+↑` | Exploded value addition with exploded value |
| `℘ ↑-↑` | Exploded value subtraction with exploded value |
| `℘ ↓+↓` | Vanished value addition with vanished value |
| `℘ ↓-↓` | Vanished value subtraction with vanished value |
| `℘ ↑×↓` | Exploded value multiplication with vanished value |
| `℘ ↓×↑` | Vanished value multiplication with exploded value |
| `℘ -@` | Logarithm of negative value |
| `℘ @0` | Logarithm with zero base |
| `℘ s` | Sine of exploded value (period position unknown) |
| `℘ c` | Cosine of exploded value (period position unknown) |
| `℘ t` | Tangent of exploded value or at asymptote |

These undefined states propagate thru operations, preserving the first cause of the undefined condition.

## Basic Usage Examples

### Creating Scalar Values

```rust
use spirix::{Scalar, ScalarF5E3};

// Create a Scalar with explicitly specified type parameters
let a = Scalar::<i32, i8>::from(42);

// Create a Scalar using a type alias (same as above)
let b = ScalarF5E3::from(3.14159);

// Convert from floating-point literals
let c: ScalarF5E3 = 2.71828.into();

// Create from constants
let pi = ScalarF5E3::PI;
let e = ScalarF5E3::E;
```

### Arithmetic Operations

```rust
use spirix::{Scalar, ScalarF6E4};

let a = ScalarF6E4::from(7);
let b = ScalarF6E4::from(3);

// Basic operations
let sum = a + b;        // 10
let difference = a - b; // 4
let product = a * b;    // 21
let quotient = a / b;   // 2.33333...

// Transcendental functions
let sin_a = a.sin();
let exp_b = b.exp();    // e^3
let log_ab = (a * b).ln(); // ln(21)
```

### Working with Complex Numbers

```rust
use spirix::{Circle, CircleF5E3};

// Create a complex number (real, imaginary)
let z = CircleF5E3::from((3.0, 4.0)); // 3 + 4i

// Access components
let real = z.r();      // 3.0
let imag = z.i();      // 4.0

// Complex arithmetic
let w = CircleF5E3::from((1.0, -2.0)); // 1 - 2i
let sum = z + w;      // 4 + 2i
let product = z * w;  // 11 - 2i

// Circle constants
let i = CircleF5E3::POS_I;  // 0 + 1i
let two_pi_i = CircleF5E3::TAU * i; // 0 + 2πi

// Complex-specific operations
let conj = z.conjugate();  // 3 - 4i
let mag = z.magnitude();   // 5
```

### Handling Special Values

```rust
use spirix::{Scalar, ScalarF5E3};

// Create escaped values
let huge = ScalarF5E3::MAX * 2;
assert!(huge.exploded());
assert!(huge.is_positive());

let tiny = ScalarF5E3::MIN_POS / 42;
assert!(tiny.vanished());
assert!(tiny.is_positive());

// Operations with escaped values
let still_exploded = huge * 3;  // Still exploded
let neg_huge = huge * -1;       // Negative exploded
let zero_like = huge * 0;       // Actual zero

// Undefined states
let div_by_zero = ScalarF5E3::ONE / 0;
assert!(div_by_zero.is_undefined());

// Checking value state
if value.is_normal() {
    // Process normal value
} else if value.vanished() {
    // Handle vanished value
} else if value.exploded() {
    // Handle exploded value
} else if value.is_undefined() {
    // Handle undefined state
} else if value.is_zero() {
    // Handle zero value
}
```

## Advanced Use Cases

### Numerical Integration with Adaptive Precision

```rust
use spirix::{Scalar, ScalarF7E3, ScalarF4E7};

// Use high precision for values near 1
fn integrate_high_precision(f: impl Fn(ScalarF7E3) -> ScalarF7E3, 
                            a: ScalarF7E3, 
                            b: ScalarF7E3, 
                            steps: usize) -> ScalarF7E3 {
    // Integration code...
    // High precision is maintained thruout calculation
}

// Use wide range for values that could be very large or small
fn integrate_wide_range(f: impl Fn(ScalarF4E7) -> ScalarF4E7, 
                        a: ScalarF4E7, 
                        b: ScalarF4E7, 
                        steps: usize) -> ScalarF4E7 {
    // Integration code...
    // Wide range is maintained thruout calculation
}
```

### Wrapping in Complex Space

```rust
use spirix::{Circle, CircleF5E3};

// Maps a complex number onto a periodic grid
fn wrap_to_lattice(z: CircleF5E3, cell_size: CircleF5E3) -> CircleF5E3 {
    z.modulo(cell_size)
}

// Example: map (37.2 + 45.6i) onto a 10×10 grid
let point = CircleF5E3::from((37.2, 45.6));
let grid = CircleF5E3::from((10.0, 10.0));
let wrapped = wrap_to_lattice(point, grid);
// Result: (7.2 + 5.6i)
```

### Strict Mathematical Error Handling

```rust
use spirix::{Scalar, ScalarF5E3};

fn safe_compute(value: ScalarF5E3) -> Result<ScalarF5E3, String> {
    let result = value.sqrt();
    
    if result.is_undefined() {
        // Check the specific undefined state
        if format!("{}", result).contains("℘ √-") {
            return Err("Cannot take square root of a negative number".to_string());
        } else {
            return Err(format!("Undefined result: {}", result));
        }
    }
    
    Ok(result)
}
```

## Performance Considerations

Spirix's design emphasizes efficiency in several ways:

1. **Reduced branching**: The two's complement representation eliminates most sign-specific code paths, resulting in fewer branches and more predictable execution.

2. **Efficient state detection**: The normalization level system allows quick determination of value state without extensive testing.

3. **Branchless algorithms**: Many operations can be implemented with few or no conditional branches, making them ideal for SIMD processing.

4. **Parametric sizing**: The ability to choose fraction and exponent sizes allows applications to optimize for their specific precision and range needs without unnecessary overhead.

For maximum performance:

- Choose the smallest fraction and exponent sizes that meet your requirements
- Prefer Scalar over Circle when complex numbers aren't needed
- Be aware that escaped values (exploded/vanished) may have lower performance in some operations

## Comparing with Traditional Floating-Point

Spirix differs from traditional floating-point implementations in several key ways:

| Feature | Traditional FP | Spirix |
|---------|----------------|--------|
| Sign representation | Separate sign bit | Two's complement thruout |
| Number line | Discontinuous at zero | Continuous thru entire range |
| Special values | Positive/negative infinity, NaN | Exploded, vanished, and specific undefined states |
| Denormal numbers | Gradual precision loss | Vanished values with sign preservation |
| Error information | Single NaN value | Multiple specific undefined states |
| Bit manipulation | Requires int conversion | Directly supported with alignment |
| Complex support | Separate real/imaginary | Unified Circle type with shared exponent |
| Precision/range | Fixed configurations | Independently configurable |

These differences make Spirix particularly well-suited for:
- Applications requiring strict error tracking
- Computations with complex numbers
- Algorithms needing bit-level floating-point manipulation
- Systems with custom precision/range requirements

## Conclusion

Spirix represents a fundamental reimagining of how floating-point arithmetic can be implemented in computing systems. By leveraging the natural properties of two's complement representation thruout the calculation pipeline, it achieves greater simplicity, improved error handling, and flexible precision while maintaining mathematical consistency across the entire numeric spectrum.

The library's design philosophy emphasizes the inherent mathematical continuity of the number line, preserving orientation information even when precise magnitude cannot be represented. This approach, combined with the rich state classification system and specific undefined tracking, provides a more mathematically coherent framework for numerical computation.

Whether you're developing high-precision scientific applications, complex number algorithms, or systems with custom numeric requirements, Spirix offers a powerful alternative to traditional floating-point implementations.