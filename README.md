# Spirix

<div align="center">
  <img src="spirix.png" alt="Spirix Logo">
  <h3>Two's Complement Floating Point Arithmetic with Adjustable Precision</h3>
</div>

[![Crates.io](https://img.shields.io/crates/v/spirix.svg)](https://crates.io/crates/spirix)
[![Docs.rs](https://docs.rs/spirix/badge.svg)](https://docs.rs/spirix)
[![Website](https://img.shields.io/badge/website-holdmyoscilloscope.com-green.svg)](https://holdmyoscilloscope.com/spirix/)
[![License](https://img.shields.io/badge/license-Custom-blue.svg)](https://holdmyoscilloscope.com/spirix/license.html)

## ⚠️ Beta Warning

**This is early beta software under active development.** While the core arithmetic operations and many mathematical functions have been tested fairly extensively, this library is not ready for production use. Please do not use this library in critical systems or applications where incorrect calculations could cause harm.

Current status:
- ✅ Core arithmetic operations (addition, subtraction, multiplication, division)
- ✅ Basic mathematical functions (sqrt, power, etc.)
- ✅ Complex number support
- ⚠️ Advanced mathematical functions still under development
- ⚠️ API may change in future versions

Use at your own risk and always validate results independently for important maths.

> **Note:** Version 0.0.12 is the last release using the current representation format. The next major version will introduce an implicit sign bit for normal numbers, gaining one bit of precision at every width. This is a breaking change to the binary representation — existing stored values will not be compatible.

## Overview

Spirix is a high-performance numeric library that implements a fundamentally new approach to floating-point arithmetic by utilizing two's complement representation thruout the entire calculation pipeline. Unlike traditional floating-point implementations that use separate sign bit and magnitude representation, Spirix employs a continuous numeric representation with left-aligned normalized fractions and unbiased exponents.

This approach provides several key advantages:
- **Simplified arithmetic**: Eliminates many branches required by sign-bit implementations
- **Error tracking**: Preserves the origin of undefined operations (compared to a single generic NaN)
- **Continuous mathematics**: Values maintain orientation even beyond representation ranges
- **Customizable precision and range**: Independently choose fraction and exponent sizes
- **Complex number support**: Real and imaginary components share the same exponent for efficiency

## Architectural Foundations

### Two's Complement Representation

Spirix's core innovation is using two's complement representation for floating-point numbers, which enables:

- A continuous number line without discontinuity at zero
- Natural operation consistency across zero without special handling
- Elimination of sign-bit branches in arithmetic operations

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

| MSB/N-level | Bit Pattern | Description | Symbol |
|-------|------------|-------------|--------|
| N-0  | □□□□□□□□ | Zero | [0] |
| N-0  | ■■■■■■■■ | General undefined | [℘] |
| N-1  | □■xxxxxx | Positive normal or exploded | [+#], [+↑] |
| N-1  | ■□xxxxxx | Negative normal or exploded | [-#], [-↑] |
| N-2  | □□■xxxxx | Positive vanished | [+↓] |
| N-2  | ■■□xxxxx | Negative vanished | [-↓] |
| N-3+ | □□□xxxxx | Various undefined states | [℘ 'state'] |
| N-3+ | ■■■xxxxx | Various undefined states | [℘ 'state'] |

Note that Zero and general undefined are uniform patterns, both of which have an ambiguous exponent. Each N level is distinguished by checking LSB (leading same bits). Ambiguous values are marked by having an exponent set to the lowest possible value in the exponent, referred to as AMBIGUOUS_EXPONENT. Ambiguous values include Zero, Infinity, Undefined, Vanished and Exploded. Exploded have LSB of one N-1 (same as normal states but with AMBIGUOUS_EXPONENT). Vanished have LSB of two with AMBIGUOUS EXPONENT. Undefined contain three or more LSB with AMBIGUOUS_EXPONENT. Zero contains a fraction of all zeros, allowing fast determination of a value's normality or ambiguous state without unnecessary testing or branching.

### Escaped Values

When a value exceeds the normal exponent range, Spirix doesn't simply truncate to infinity or zero. Instead, it creates an "escaped" phase that preserves sign or orientation information:

- **Exploded** values (↑): Extremely large numbers that maintain their sign/orientation
- **Vanished** values (↓): Extremely small numbers that maintain their sign/orientation

These escaped values can continue to participate in absolute mathematical operations, like multiplication and division, allowing calculations to proceed even with results beyond the exponent range. Escaped Scalars maintain their sign, escaped Circles maintain their angle/complex sign thru absolute operations.

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

### Infinity [∞]

Infinity is represented with a unique bit pattern in the fraction:

```
■■■■■■■■... - Infinity
```
All ones. Infinity is ambiguous, thus has exponent equal to AMBIGIOUS_EXPONENT

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
Note that Rust primitives like f32, i8 are treated as Scalars. For convenience, Circles can be converted to/from Num::Complex::<f32> or <f64>, where circle.r() and circle.i() return normalized Scalars from each respective component. A Circle can be constructed from a single Scalar/Rust primitive CircleF3E3::from(3),CircleF3E3::from(Scalar), or from a tuple of two valid types like CircleF5E4::from((scalar, i8))

### Truth Tables

The following tables show how different value states interact during basic arithmetic operations:

#### Addition

| + | [0] | [↓] | [#] | [↑] | [∞] | [℘?] |
|---|-----|-----|-----|-----|-----|------|
| **[0]** | [0] | [↓] | [#] | [℘+⬆] | [℘+⬆] | [℘?] |
| **[↓]** | [↓] | [℘↓+↓] | [#] | [℘ +⬆] | [℘ +⬆] | [℘?] |
| **[#]** | [#] | [#] | [0], [#], [↓], [↑] | [℘ +⬆] | [℘ +⬆] | [℘?] |
| **[↑]** | [℘ ⬆+] | [℘ ⬆+] | [℘ ⬆+] | [℘ ⬆+⬆] | [℘ ⬆+⬆] | [℘?] |
| **[∞]** | [℘ +⬆] | [℘ +⬆] | [℘ +⬆] | [℘ ⬆+⬆] | [℘ ⬆+⬆] | [℘?] |
| **[℘?]** | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] |

#### Subtraction

| - | [0] | [↓] | [#] | [↑] | [∞] | [℘?] |
|---|-----|-----|-----|-----|-----|------|
| **[0]** | [0] | [↓] | [#] | [℘-⬆] | [℘-⬆] | [℘?] |
| **[↓]** | [↓] | [℘↓-↓] | [#] | [℘ -⬆] | [℘ -⬆] | [℘?] |
| **[#]** | [#], [↓], [↑] | [#], [↓], [↑] | [0], [#], [↓], [↑] | [℘ -⬆] | [℘ -⬆] | [℘?] |
| **[↑]** | [℘ ⬆-] | [℘ ⬆-] | [℘ ⬆-] | [℘ ⬆-⬆] | [℘ ⬆-⬆] | [℘?] |
| **[∞]** | [℘ -⬆] | [℘ -⬆] | [℘ -⬆] | [℘ ⬆-⬆] | [℘ ⬆-⬆] | [℘?] |
| **[℘?]** | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] |

#### Multiplication

| × | [0] | [↓] | [#] | [↑] | [∞] | [℘?] |
|---|-----|-----|-----|-----|-----|------|
| **[0]** | [0] | [0] | [0] | [0] | [℘⬆×⬇] | [℘?] |
| **[↓]** | [0] | [↓] | [↓] | [℘⬆×⬇] | [∞] | [℘?] |
| **[#]** | [0] | [↓] | [#], [↓], [↑] | [↑] | [∞] | [℘?] |
| **[↑]** | [0] | [℘⬇×⬆] | [↑] | [↑] | [∞] | [℘?] |
| **[∞]** | [℘⬆×⬇] | [∞] | [∞] | [∞] | [∞] | [℘?] |
| **[℘?]** | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] |

#### Division

| ÷ | [0] | [↓] | [#] | [↑] | [∞] | [℘?] |
|---|-----|-----|-----|-----|-----|------|
| **[0]** | [℘ ⬇/⬇] | [∞] | [∞] | [∞] | [∞] | [℘?] |
| **[↓]** | [0] | [℘ ⬇/⬇] | [↑] | [↑] | [∞] | [℘?] |
| **[#]** | [0] | [↓] | [#], [↓], [↑] | [↑] | [∞] | [℘?] |
| **[↑]** | [0] | [↓] | [↓] | [℘ ⬆/⬆] | [∞] | [℘?] |
| **[∞]** | [0] | [0] | [0] | [0] | [℘ ⬆/⬆] | [℘?] |
| **[℘?]** | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] | [℘?] |

Where:
- **[0]**: Zero
- **[↓]**: Vanished (extremely small)
- **[#]**: Normal values
- **[↑]**: Exploded (extremely large)
- **[∞]**: Infinity
- **[℘?]**: Undefined states

Spirix Rust native operations supported:

# Spirix Mathematical Operations

## Basic Arithmetic Operations
```rust
// Basic operations
let sum = a + b;      // Addition
let diff = a - b;     // Subtraction
let product = a * b;  // Multiplication
let quotient = a / b; // Division
let remainder = a % b;           // Mathematical remainder
let component_mod = a.modulo(b); // Component-wise remainder (for Circle)
let negated = -a;     // Negation
let reciprocal = a.reciprocal();  // 1/a
```

// Safe aligned bitwise operations
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
let gaussian = ScalarF6E4::random_gauss(); // Normal distribution

// For complex numbers
let complex_uniform = CircleF7E5::random();        // Uniform inside unit circle
let complex_gaussian = CircleF4E4::random_gauss(); // Normal distribution
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
| `℘ ⬆+⬆` | Transfinite value addition with transfinite value |
| `℘ ⬆-⬆` | Transfinite value subtraction with transfinite value |
| `℘ ↓+↓` | Vanished value addition with vanished value |
| `℘ ↓-↓` | Vanished value subtraction with vanished value |
| `℘ ⬆+` | Transfinite value addition with finite value |
| `℘ ⬆-` | Transfinite value subtraction with finite value |
| `℘ ⨅∞` | Fractional part of Infinity |
| `℘ ±∅` | Sign/direction of Zero or Infinity is indeterminate |
| `℘ ⊥⊙` | Indeterminate Scalar → Circle conversion |
| `℘ ∩` | Clamp with non-ordered ambiguous values |
| `℘ ⌈` | Maximum of non-ordered ambiguous values |
| `℘ ⌊` | Minimum of non-ordered ambiguous values |
| `℘ +⬆` | Finite value addition with transfinite value |
| `℘ -⬆` | Finite value subtraction with transfinite value |
| `℘ ⬆/⬆` | Transfinite value division by transfinite value |
| `℘ ⬇/⬇` | Negligible value division by negligible value |
| `℘ ⬆%` | Transfinite value modulus operation |
| `℘ ⬆‰` | Transfinite value modulo operation |
| `℘ %↓` | Finite value modulus with vanished value |
| `℘ ‰↓` | Finite value modulo with vanished value |
| `℘ %↑` | Modulus with exploded denominator and mismatched signs |
| `℘ ‰↑` | Modulo with exploded denominator and mismatched signs |
| `℘ &` | Logical AND with escaped value |
| `℘ \|` | Logical OR with escaped value |
| `℘ ⊻` | Logical XOR with escaped value |
| `℘ ⬇×⬆` | Negligible value multiplication with transfinite value |
| `℘ ⬆×⬇` | Transfinite value multiplication with negligible value |
| `℘ ⬆^` | Transfinite value raised to power |
| `℘ ⬇^` | Vanished value raised to power |
| `℘ ^⬆` | Value raised to transfinite power |
| `℘ ^⬇` | Value raised to vanished power |
| `℘ -^` | Negative value raised to irrational power |
| `℘ @1` | Logarithm base One |
| `℘ √-` | Square root of negative value |
| `℘ √↑` | Square root of transfinite value |
| `℘ √↓` | Square root of vanished value |
| `℘ ⬆@` | Logarithm of transfinite value |
| `℘ ⬇@` | Logarithm of negligible value |
| `℘ @⬆` | Logarithm with transfinite base |
| `℘ @⬇` | Logarithm with negligible base |
| `℘ -@` | Logarithm of negative value |
| `℘ @-` | Logarithm with negative base |
| `℘ s` | Sine of value with imprecise period position |
| `℘ c` | Cosine of value with imprecise period position |
| `℘ S` | Arcsine of value outside domain [-1,1] |
| `℘ C` | Arccosine of value outside domain [-1,1] |
| `℘ t` | Tangent of value with imprecise period position |
| `℘` | General undefined or unimplemented and extensions |

These undefined states propagate thru operations, preserving the first cause of the undefined condition.

## Basic Usage Examples

### Creating Scalar Values

```rust
use spirix::{Scalar, ScalarF5E3};

// Create a Scalar with explicitly specified type parameters
let a = Scalar::<i32, i8>::from(42);

// Create a Scalar using a type alias
let b = ScalarF5E3::from(3.14159);

// Convert from Rust literals
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

## Performance Considerations

Spirix's design emphasizes efficiency in several ways:

1. **Reduced branching**: The two's complement representation eliminates most sign-specific code paths, resulting in fewer branches and more predictable execution.

2. **Efficient state detection**: The normalization level system allows quick determination of value state without extensive testing.

3. **Branchless algorithms**: Many operations can be implemented with few or no conditional branches, making them ideal for SIMD processing.

4. **Parametric sizing**: The ability to choose fraction and exponent sizes allows applications to optimize for their specific precision and range needs without unnecessary overhead.

For maximum performance:

- Choose the smallest fraction and exponent sizes that meet your requirements
- Prefer Scalar over Circle when complex numbers aren't needed

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

## Mathematical Identity Preservation

Spirix maintains fundamental mathematical identities that IEEE-754 violates:

### Additive Identity: a - a = 0
Both IEEE-754 and Spirix preserve this identity:

```rust
let normal_ieee = 5.;
assert!(normal_ieee - normal_ieee == 0.);

let normal_scalar : ScalarF6E5 = 5.into();
assert!(normal_scalar - normal_scalar == 0);
```

### Multiplicative Identity: a × b = 0 iff a | b = 0
IEEE-754 violates this fundamental property, while Spirix preserves it:

```rust
let tiny_ieee = f64::MIN_POSITIVE * f64::MIN_POSITIVE;  // Underflows to 0
assert!(tiny_ieee.is_zero());

let tiny_scalar = ScalarF7E5::MIN_POS.square(); // Returns a vanished scalar, not Zero
assert!(!tiny_scalar.is_zero());
```

These differences make Spirix particularly well-suited for:
- Applications requiring strict error tracking
- Computations with complex numbers
- Algorithms needing bit-level floating-point manipulation
- Systems with custom precision/range requirements

## FPGA Implementation

Spirix includes a complete hardware implementation in plain Verilog targeting the Lattice ECP5-25F. The design is a 21-operation register-machine ALU with an 8x128-bit register file, 18-bit instructions, and runtime-selectable precision (8/16/32/64-bit fraction and exponent).

### Silicon-Verified Performance (Colorlight 5A-75B, ECP5-25F speed-6)

| Operation | Module | Fmax | LUT4 | DSP | Latency |
|-----------|--------|------|------|-----|---------|
| NEG/ABS/SIGN/SHL/SHR | basic | 231 MHz | 3,900 | 0 | 1 clk |
| MIN/MAX | minmax | 208 MHz | 2,127 | 0 | 1 clk |
| ADD/SUB/AND/OR/XOR | addbit_pipe | 188 MHz | 6,687 | 0 | 3 clk |
| FLOOR/CEIL/ROUND | round | 235 MHz | 2,194 | 0 | 1 clk |
| FRAC | micro-op | — | 0 | 0 | 1\|3 clk |
| MUL | multiply_pipe | 170 MHz | ~2,666 | 0 | 3 clk |
| DIV/SQRT/MOD | divmodsqrt | 188 MHz | 5,433 | 0 | FRAC+2..5 |
| RNG | random | 800+ MHz | ~487 | 0 | 4 clk |

Full core standalone: 15,820 LUT4, 16 DSP18, 96 DPR16x4.

All Fmax numbers are measured on real silicon using a CE-gated self-test protocol, not static timing estimates. Consistent 2-2.5x margin over nextpnr estimates observed across all modules.

### Spirix vs HardFloat vs FPnew (IEEE 754 Binary32)

Single-width IEEE f32 comparison on ECP5 (Yosys synth_ecp5, -nowidelut):

| Op | Spirix LUT4 | Spirix Fmax | HardFloat LUT4 | HardFloat Fmax | FPnew LUT4 | FPnew Fmax |
|----|-------------|-------------|-----------------|----------------|------------|------------|
| Add | 842 | 95 MHz | 1,050 | 88 MHz | 825 | 74 MHz |
| Mul | 227 (4 DSP) | 115 MHz | 786 (4 DSP) | 65 MHz | 574 (0 DSP) | 74 MHz |
| FMA | 1,472 (3 DSP) | 63 MHz | 2,057 (4 DSP) | 47 MHz | 2,850 (0 DSP) | 25 MHz |

Spirix wins silicon Fmax on every operation.

### Multi-Width Comparison

HardFloat cannot do runtime-selectable width; each IEEE precision requires a separate instantiation. A comparable multi-width HardFloat unit covering binary16/32/64/128 requires 4 parallel instances:

| Op | HardFloat (4 instances) | Spirix (1 datapath) |
|----|-------------------------|---------------------|
| Add | 10,106 LUT4 | ~6,700 LUT4 |
| Mul | 9,243 LUT4 | ~2,666 LUT4 (16 DSP) |
| Div/Sqrt | 9,240 LUT4 | 5,433 LUT4 |
| **Total** | **28,589 LUT4** | **~14,799 LUT4** |

Spirix is ~48% smaller with a single datapath handling 16 width combinations (4 frac x 4 exp) vs HardFloat's 4 fixed IEEE widths. Spirix also includes 15 additional operations HardFloat lacks (NEG, ABS, SIGN, SHL, SHR, MIN, MAX, AND, OR, XOR, FLOOR, CEIL, ROUND, FRAC, MOD) and a hardware TRNG.

See [fpga/cores/minimal/README.md](fpga/cores/minimal/README.md) for full architecture details.

## GPU Compute Kernels

Spirix provides production-ready GPU kernels for batch ScalarF4E4 operations via HIP (AMD) with a cross-platform WebGPU port.

### Performance (AMD RX 6800, 60 CUs)

| Operation | Throughput | Instructions | VGPRs | vs f32 |
|-----------|------------|--------------|-------|--------|
| Addition | 27.22 GOPS | 56 | 10 | 0.69x |
| Subtraction | ~27 GOPS | 56 | 10 | 0.69x |
| Multiplication | 6.96 GOPS | 56 | 10 | 0.18x |
| Division | **19.51 GOPS** | 93 | 12 | **2.24x faster** |
| Square Root | 13.25 GOPS | 102 | 16 | 0.33x |

Division outperforms multiply despite more instructions: Newton-Raphson iterations provide instruction-level parallelism that hides memory latency.

### WebGPU Cross-Platform

The HIP kernels port trivially to WGSL because Spirix already uses 32-bit integer arithmetic throughout. Performance: 85-87% of native HIP across all operations. Runs on any GPU (AMD, NVIDIA, Intel, Apple) via browser.

See [gpu/README.md](gpu/README.md) for kernel details, benchmarks, and API usage.