#![no_std]
// src/ │ ├── core/                    Core types and traits │   ├── circle.rs            Circle<F, E> complex number type │   ├── circle_aliases.rs    Circle type aliases (F3E3-F7E7) │   ├── integer.rs           Public Integer and internal numeric traits │   ├── mod.rs               Module exports │   ├── scalar.rs            Scalar<F, E> real number type │   ├── scalar_aliases.rs    Scalar type aliases (F3E3-F7E7) │   └── undefined.rs         Undefined value prefixes and patterns │ ├── conversions/             Type conversion implementations │   ├── circle_circle.rs     Circle conversion traits │   ├── circle_rust.rs       Circle to primitive conversions │   ├── circle_scalar.rs     Circle to Scalar conversions │   ├── mod.rs               Conversion exports │   ├── rust_circle.rs       Primitive to Circle conversions │   ├── rust_scalar.rs       Primitive to Scalar conversions │   ├── scalar_circle.rs     Scalar to Circle conversions │   ├── scalar_rust.rs       Scalar to primitive conversions │   └── scalar_scalar.rs     Scalar to Scalar conversions │ ├── constants/               Constant values and implementations │   ├── circle.rs            Circle constants (i, π, roots) │   ├── exponent.rs          Exponent component constants │   ├── fraction.rs          Fraction component constants │   ├── mod.rs               Constants exports │   └── scalar.rs            Scalar constants (π, e, roots) │ ├── implementations/         Mathematical operations │   ├── addition/            Addition operations │   │   ├── circle_circle.rs Circle + Circle │   │   ├── circle_scalar.rs Circle + Scalar │   │   ├── mod.rs           Addition exports │   │   ├── scalar_circle.rs Scalar + Circle │   │   └── scalar_scalar.rs Scalar + Scalar │   │ │   ├── subtraction/         Subtraction operations │   │   ├── circle_circle.rs Circle - Circle │   │   ├── circle_scalar.rs Circle - Scalar │   │   ├── mod.rs           Subtraction exports │   │   ├── scalar_circle.rs Scalar - Circle │   │   └── scalar_scalar.rs Scalar - Scalar │   │ │   ├── multiplication/      Multiplication operations │   │   ├── circle_circle.rs Circle * Circle │   │   ├── circle_scalar.rs Circle * Scalar │   │   ├── mod.rs           Multiplication exports │   │   ├── scalar_circle.rs Scalar * Circle │   │   └── scalar_scalar.rs Scalar * Scalar │   │ │   ├── division/            Division operations │   │   ├── circle_circle.rs Circle / Circle │   │   ├── circle_scalar.rs Circle / Scalar │   │   ├── mod.rs           Division exports │   │   ├── scalar_circle.rs Scalar / Circle │   │   └── scalar_scalar.rs Scalar / Scalar │   │ │   ├── modular/             Modular arithmetic operations │   │   ├── circle_circle.rs Circle % Circle │   │   ├── circle_scalar.rs Circle % Scalar │   │   ├── mod.rs           Modular operations exports │   │   ├── scalar_circle.rs Scalar % Circle │   │   └── scalar_scalar.rs Scalar % Scalar │   │ │   ├── exponents/           Power and exponent operations │   │   ├── circle_circle.rs Circle^Circle powers │   │   ├── circle.rs        Circle exponent operations │   │   ├── circle_scalar.rs Circle^Scalar powers │   │   ├── mod.rs           Exponent operations exports │   │   ├── scalar_circle.rs Scalar^Circle powers │   │   ├── scalar.rs        Scalar exponent operations │   │   └── scalar_scalar.rs Scalar^Scalar powers │   │ │   ├── trigonometry/        Trig functions │   │   ├── circle.rs        Circle trigonometry │   │   ├── mod.rs           Trigonometry exports │   │   └── scalar.rs        Scalar trigonometry │   │ │   ├── bitwise/             Aligned bitwise operations │   │   ├── circle.rs        Circle bitwise operations │   │   ├── mod.rs           Bitwise operations exports │   │   └── scalar.rs        Scalar bitwise operations │   │ │   ├── formatting/          Display implementations │   │   ├── circle.rs        Circle formatting │   │   ├── colours.rs       ANSI colour support │   │   ├── mod.rs           Formatting exports │   │   └── scalar.rs        Scalar formatting │   │ │   ├── basic_circle.rs      Core Circle operations │   ├── basic_scalar.rs      Core Scalar operations │   ├── comparison.rs        Ordering operations (<, >, ==) │   ├── mod.rs               Implementation exports │   └── random.rs            Random number generation │ ├── operators/               Type interaction traits │   ├── circle.rs            Circle-specific operations │   ├── circle_circle.rs     Circle-Circle operations │   ├── circle_rust.rs       Circle-to-primitive ops │   ├── circle_scalar.rs     Circle-Scalar operations │   ├── mod.rs               Operator exports │   ├── rust_circle.rs       Primitive-to-Circle ops │   ├── rust_scalar.rs       Primitive-to-Scalar ops │   ├── scalar.rs            Scalar-specific operations │   ├── scalar_circle.rs     Scalar-Circle operations │   ├── scalar_rust.rs       Scalar-to-primitive ops │   └── scalar_scalar.rs     Scalar-Scalar operations │ ├── tensor/                  Tensor operations and neural networks │   ├── autograd.rs          Automatic differentiation │   ├── mod.rs               Tensor exports │   ├── nn.rs                Neural network layers │   ├── ops.rs               Tensor operations │   ├── optim.rs             Optimization algorithms │   └── tensor.rs            Core Tensor type │ └── lib.rs                   Library root and exports (this file)

//! # Spirix
//!
//! A high-performance two's complement floating-point arithmetic library with customizable precision. Spirix allows you to choose both fraction and exponent sizes independently.
//!
//! ## Core Types
//!
//! Spirix provides two primary numeric types:
//!
//! - `Scalar<F, E>`: Real numbers with fraction `F` and exponent `E`
//! - `Circle<F, E>`: Complex numbers with fractions `F` and shared exponent `E`
//!
//! Both types use Rust's native signed integer types (i8, i16, i32, i64, i128) for their components, allowing you to select precision and range based on your application's needs.
//!
//! ### Type Aliases for Valid Configurations
//!
//! For convenience, Spirix provides type aliases for all valid Rust fraction and exponent combinations.
//!
//! Here are some examples:
//! ```rust
//! use spirix::{ScalarF5E3, ScalarF6E4, ScalarF5E5, ScalarF6E6, ScalarF7E3}; // Format: F#E# = 2^# fraction bits, 2^# exponent bits. // F5E3 = 32-bit fraction, 8-bit exponent  (similar to f32) // F6E4 = 64-bit fraction, 16-bit exponent (similar to f64) // F5E5 = 32-bit fraction, 32-bit exponent // F6E6 = 64-bit fraction, 64-bit exponent // F7E3 = 128-bit fraction, 8-bit exponent let _ = (ScalarF5E3::ONE, ScalarF6E4::ONE, ScalarF5E5::ONE, ScalarF6E6::ONE, ScalarF7E3::ONE);
//! ```
//!
//! Similarly for complex numbers with `CircleF5E3`, `CircleF6E4`, etc.
//!
//! ```rust
//! use spirix::{Power, Scalar, ScalarConstants}; // High-precision fraction with small range. Note: this uses Rust f64 as an intermediary! let _precise_near_one = Scalar::<i128, i8>::from(9.8696044010893586188344909998761511353_f64);
//!
//! // Small fraction with huge range. let _large_magnitude_low_precision = Scalar::<i8, i128>::from(6000_i16).pow(6000_i16);
//! ```
//!
//! ## Key Features and Benefits
//!
//! ### 0. Escaped Values
//!
//! When numbers grow too large or small to maintain magnitude, Spirix creates ambiguous "escaped" values that:
//!
//! - Preserve orientation information, even tho magnitude is unknown
//! - Can still participate in absolute operations, i.e. * and /
//! - Track whether they escaped toward Infinity (`exploded`) or toward Zero (`vanished`)
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF4E7};
//!
//! // Create a value so large it loses its magnitude. let huge = ScalarF4E7::MAX * 2_i8; assert!(huge.exploded()); assert!(huge.is_positive());
//!
//! // Absolute operations can continue with escaped values (* / .square() etc.) let still_exploded = huge * 3_i8; assert!(still_exploded.exploded());
//!
//! // Division works too! let neg_huge = huge / -5_i8; assert!(!neg_huge.is_undefined()); // Still defined! assert!(neg_huge.is_negative());
//! ```
//!
//! ### 1. Infinity and Zero Identities
//!
//! Spirix implements mathematical identities in accordance with Riemann sphere principles, with proper handling of Infinity and Zero:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! // Division by Zero produces Infinity. let infinity = ScalarF5E3::from(42_i8) / 0_i8; assert!(infinity.is_infinite());
//!
//! // Multiplicative identities with Infinity. let still_infinite = infinity * 7_i8;          // ∞ × n = ∞ (for any non-Zero n) assert!(still_infinite.is_infinite());
//!
//! // Infinity times Zero is undefined. let inf_times_zero = infinity * 0_i8;          // ∞ × 0 = undefined assert!(inf_times_zero.is_undefined());
//!
//! // Zero is the multiplicative absorber (except with Infinity). let still_zero = ScalarF5E3::ZERO * 42_i8;     // 0 × n = 0 assert!(still_zero.is_zero()); let exploded = ScalarF5E3::MAX.square(); let also_zero = 0_i8 * exploded; assert!(also_zero.is_zero()); // You can use the built-in Infinity constant. let undefined_multiply = ScalarF5E3::INFINITY * still_zero; assert!(undefined_multiply.is_undefined());
//!
//! // Reciprocal relationships. let zero = ScalarF5E3::ONE / infinity;         // 1/∞ = 0 assert!(zero.is_zero());
//!
//! // Division by Infinity. let also_zero = ScalarF5E3::from(42_i8) / infinity;  // n/∞ = 0 (for any non-infinite n) assert!(also_zero.is_zero());
//!
//! // Infinity divided by Infinity is undefined. let inf_div_inf = infinity / infinity;         // ∞/∞ = undefined assert!(inf_div_inf.is_undefined());
//!
//! // Addition and subtraction with Infinity return Infinity (Riemann sphere: ∞ absorbs). let inf_add = infinity + 5_i8;                 // ∞ + n = ∞ let inf_sub = infinity - infinity;             // ∞ − ∞ = ∞ assert!(inf_add.is_infinite() && inf_sub.is_infinite());
//!
//! // Infinity absorbs Zero under addition too. let inf_plus_zero = infinity + 0_i8;           // ∞ + 0 = ∞ assert!(inf_plus_zero.is_infinite());
//! ```
//!
//! Infinity in Spirix represents the directionless "point at Infinity" on the Riemann sphere, a singularity that conceptually unifies various approaches to Infinity and zero.
//!
//! ### 2. Vanished Values and Their Identities
//!
//! Spirix uniquely handles infinitesimal values that approach but never equal Zero:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! // Create a vanished value. let tiny = ScalarF5E3::MIN_POS / 42_i8; assert!(tiny.vanished() && tiny.is_positive());
//!
//! // Adding a vanished value to a normal value is like adding Zero. let normal = ScalarF5E3::from(42_i8); let sum = normal + tiny; assert!(sum == normal);
//!
//! // But adding two vanished values is undefined, as the magnitude is unknown. let tiny2 = ScalarF5E3::MIN_POS / 17_i8; let undef_sum = tiny + tiny2; assert!(undef_sum.is_undefined());
//!
//! // Division by a vanished value produces an exploded result. let huge = ScalarF5E3::ONE / tiny; assert!(huge.exploded());
//!
//! // Multiplying vanished returns vanished. let even_smaller = tiny * tiny2; assert!(even_smaller.vanished());
//! ```
//!
//! ### 3. Tracking Undefined States
//!
//! Spirix provides undefined states that preserve the initial cause:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! // Division by Zero produces Infinity. let infinity = ScalarF5E3::ONE / 0_i8; assert!(infinity.is_infinite());
//!
//! // But indeterminate forms are undefined. let undefined = ScalarF5E3::ZERO / 0_i8;       // 0/0 is undefined assert!(undefined.is_undefined());
//!
//! // Operations with undefined states propagate, preserving the original cause. let still_undefined = undefined + 42_i8; assert!(still_undefined.is_undefined());
//!
//! // Specific undefined cases are tracked. let undefined_inf_div_inf = infinity / infinity;    // ∞/∞ is undefined assert!(undefined_inf_div_inf.is_undefined());
//! ```
//!
//! ### 4. Continuous Mathematical Functions
//!
//! Spirix attempts to maintain mathematical continuity across the entire number space, even beyond exponent magnitude:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E5};
//!
//! // Vanished values retain phase information. let tiny_positive = ScalarF5E5::MIN_POS.square(); assert!(tiny_positive.vanished()); assert!(tiny_positive.is_positive()); let tiny_negative = tiny_positive * -2.4_f32; assert!(tiny_negative.is_negative()); // Multiplication and divison don't truncate to Zero. let supa_tiny = tiny_positive.square(); assert!(supa_tiny != 0_i8);
//!
//! // Trigonometric functions work with vanished values. let sin_tiny = tiny_positive.sin(); assert!(sin_tiny.vanished() && sin_tiny.is_positive());
//! ```
//!
//! ### 5. Predictable Comparisons
//!
//! Comparing values is intuitive and follows mathematical principles:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! let pos_normal = ScalarF5E3::from(42_i8); let neg_normal = ScalarF5E3::from(-42_i8); let pos_exploded = ScalarF5E3::MAX * 2_i8; let neg_exploded = ScalarF5E3::MIN * 2_i8; let pos_vanished = ScalarF5E3::MIN_POS / 2_i8; let neg_vanished = pos_vanished * -1_i8; let infinity = ScalarF5E3::ONE / 0_i8;
//!
//! // Escaped and Zero values maintain ordering. assert!(pos_exploded > pos_normal); assert!(pos_normal > pos_vanished); assert!(pos_vanished > 0_i8); assert!(neg_exploded < neg_normal); assert!(neg_normal < neg_vanished); assert!(neg_vanished < 0_i8);
//!
//! // Infinity is not comparable to anything. assert!(!(infinity > pos_normal)); assert!(!(infinity < pos_normal));
//!
//! // Undefined states don't compare with anything either, including themselves. let undefined = ScalarF5E3::ZERO / 0_i8;       // 0/0 assert!(!(undefined == undefined)); assert!(!(undefined < pos_normal)); assert!(!(undefined > pos_normal)); assert!(!(undefined < infinity)); assert!(!(undefined > infinity));
//! ```
//!
//! ### 6. Modulus and Modulo Operations
//!
//! Spirix provides two distinct remainder operations:
//!
//! #### 0. Mathematical Modulus (%)
//!
//! The standard `%` operator calculates the mathematical remainder after division:
//!
//! ```rust
//! use spirix::{CircleF5E3, ScalarF5E3};
//!
//! // Scalar modulus — remainder after division. let a = ScalarF5E3::from(7.5_f32); let b = ScalarF5E3::from(3_i8); let remainder = a % b; assert!(remainder == 1.5_f32);
//!
//! // Circle-Circle modulus — derived from complex division. let z1 = CircleF5E3::from((7_i8, 4_i8)); let z2 = CircleF5E3::from((3.8_f32, 2.2_f32)); let _complex_remainder = z1 % z2;
//! ```
//! For Circle-Circle operations, this implements:
//!
//! (a + b*i) % (c + d*i) = ((a*c + b*d) + (b*c − a*d)*i) % (c² + d²)
//!
//! ```rust
//! use spirix::{CircleF5E3, ScalarF5E3}; let z1 = CircleF5E3::from((7_i8, 4_i8));
//!
//! // Circle-Scalar modulus — based on magnitude. let s = ScalarF5E3::from(2_i8); let _magnitude_remainder = z1 % s; // Or via a Rust primitive directly — primitives are always treated as Scalars // and ALL ops work with ALL primitives :) let _magnitude_remainder = z1 % 2_i8;
//! ```
//!
//! #### 1. Component-wise Modulo (.modulo())
//!
//! The `.modulo()` method performs modulo on each component separately:
//!
//! ```rust
//! use spirix::{CircleF5E3, ScalarF5E3};
//!
//! // Circle-Circle component-wise modulo — acts on each part independently. let z1 = CircleF5E3::from((7_i8, 4_i8));   // 7 + 4i let z2 = CircleF5E3::from((3_i8, 2_i8));   // 3 + 2i let _component_remainder = z1.modulo(z2);  // (7 % 3) + (4 % 2)i = 1 + 0i
//!
//! // Circle-Scalar component-wise modulo — both components modulo Scalar. let s = ScalarF5E3::from(2_i8); let _component_scalar_mod = z1.modulo(s);  // (7 % 2) + (4 % 2)i = 1 + 0i
//! ```
//!
//! The component-wise formula for Circle values is:
//! ```text
//! (a + b*i).modulo(c + d*i) = (a % c) + (b % d)*i
//! ```
//!
//! Both operations handle special cases (undefined, exploded, vanished) appropriately, preserving mathematical consistency thruout numerical space.
//!
//! ### 7. Circles!
//!
//! Spirix provides complex number support with the `Circle` type:
//!
//! ```rust
//! use spirix::{CircleConstants, CircleF6E4};
//!
//! // Create a complex number — note the float real, integer imag mix. let z = CircleF6E4::from((1.5_f32, 2_i8));
//!
//! // Access real and imaginary parts. assert!(z.r() == 1.5_f32); let imag = z.i(); assert!(imag == 2_i8);
//!
//! // Calculate magnitude. let mag = z.magnitude(); assert!(mag == 2.5_f32); // Or magnitude squared. let mag_sq = z.magnitude_squared(); assert!(mag_sq == 6.25_f32);
//!
//! // Complex conjugate. let _conj = z.conjugate();
//!
//! // Complex multiplication. let w = CircleF6E4::from((3_i8, -4.1_f32)); let _product = z * w * 5_i8;
//!
//! // Complex constants. let i = CircleF6E4::POS_I; assert!(i.square() == -1_i8);
//! ```
//!
//! ## Getting Started
//!
//! ### Basic Usage
//!
//! ```rust
//! use spirix::{Circle, Scalar, ScalarF5E3};
//!
//! // Create Scalar values. let a = Scalar::<i32, i8>::from(7_i8); // F5E3 alias = 32-bit (2^5) fraction, 8-bit (2^3) exponent. let b = ScalarF5E3::from(1.2020569_f32);
//!
//! // Convert between types — only same-sized Scalars and Circles are interoperable. let misfit: Scalar<i16, i16> = 42_i8.into(); let _c: Scalar<i32, i8> = misfit.into(); // Cross-width conversion.
//!
//! // Basic arithmetic. let _sum = a + b; let product = a * b; let _quotient = a / b; let _reciprocal = b / a;                 // Reciprocal via division.
//!
//! // Modulo operations. let _remainder = a % b;                  // Mathematical modulus — remainder after division.
//!
//! // Transcendental functions. let _sin_a = a.sin(); let _exp_b = b.exp(); let _natural_log_product = product.ln();
//!
//! // Create a complex number. let z = Circle::<i32, i8>::from((1.5_f32, 11_i8)); let _conjugate = z.conjugate();
//!
//! // Complex modulo operations. let z1 = Circle::<i32, i8>::from((5_i8, 3.6_f32));    // 5 + 3.6i let z2 = Circle::<i32, i8>::from((2.4_f32, -1_i8));   // 2.4 − i let _complex_remainder = z1 % z2;          // Complex mathematical modulus. let _component_remainder = z1.modulo(z2);  // Component-wise modulo.
//! ```
//!
//! ## Choosing Precision
//!
//! When selecting fraction and exponent sizes, consider:
//!
//! - **Fraction bits**: Determines precision (significant digits)
//! - **Exponent bits**: Determines range (how large/small values can be)
//!
//! ```rust
//! use spirix::Scalar;
//!
//! // High precision with limited range. type HighPrecisionNearOne = Scalar<i128, i8>;
//!
//! // Medium precision with large range. type ScientificNotation = Scalar<i32, i64>;
//!
//! // Extreme range with limited precision. type RoughApproximation = Scalar<i8, i128>;
//! # let _ = (HighPrecisionNearOne::ONE, ScientificNotation::ONE, RoughApproximation::ONE);
//! # use spirix::ScalarConstants;
//! ```
//!
//! | Type   | Precision (decimal digits) | Range                    |
//! |--------|----------------------------|--------------------------|
//! | F3E3   | 2.1 digits                 | 10^±38.5                 |
//! | F4E4   | 4.5 digits                 | 10^±9860                 |
//! | F5E5   | 9.3 digits                 | 10^(10^8.81)             |
//! | F6E6   | 18.9 digits                | 10^(10^18.4)             |
//! | F7E7   | 38.2 digits                | 10^(10^37.7)             |
//!
//! ## Extra Features!
//!
//! ### Bit Manipulation
//!
//! Spirix supports native bit-level operations that maintain alignment and mathematical consistancy:
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! let a = ScalarF5E3::from(42_i8);
//!
//! // Bitwise operations align fractions and return expected results. let _b = a & ScalarF5E3::from(15_i8);  // Aligned bitwise AND let _c = a | ScalarF5E3::from(3_i8);   // Aligned bitwise OR let _d = a ^ ScalarF5E3::from(21_i8);  // Aligned bitwise XOR
//!
//! // Bit shifts apply checked exponent adjustments directly. let _doubled = a << 1_u8;  // Multiply by 2 let _halved = a >> 3_u8;   // Divide by 8 // Spirix recognises an escape and returns a vanished Scalar. let _vanished = ScalarF5E3::MIN_POS >> 1_u8;
//! ```
//!
//! ### Mathematical Truncation
//!
//! Spirix provides various mathematical truncation functions:
//!
//! ```rust
//! use spirix::ScalarF5E3;
//!
//! let x = ScalarF5E3::from(3.7_f32);
//!
//! let _floor_x = x.floor();    // 3 let _ceil_x = x.ceil();      // 4 let _round_x = x.round();    // 4 let _frac_x = x.frac();      // 0.7
//! ```
//!
//! ## Application Examples
//!
//! ### Complex Number Grid
//!
//! ```rust
//! use spirix::CircleF5E3;
//!
//! // Wrapping a point to an 8×8 grid cell. fn wrap_to_grid(point: &CircleF5E3, grid_size: &CircleF5E3) -> CircleF5E3 { point.modulo(*grid_size) }
//!
//! let point = CircleF5E3::from((37.54_f32, -12.3_f32)); let grid = CircleF5E3::from((8_i8, 8_i8)); let _wrapped = wrap_to_grid(&point, &grid);
//! ```
//!
//! ### Periodic Function Mapping
//!
//! ```rust
//! use spirix::{ScalarConstants, ScalarF5E3};
//!
//! // Map angle to [0, 2π) range. fn normalize_angle(angle: ScalarF5E3) -> ScalarF5E3 { angle % ScalarF5E3::TAU }
//!
//! let angle = ScalarF5E3::from(8.5_f32); let normalized = normalize_angle(angle); assert!(normalized >= ScalarF5E3::ZERO); assert!(normalized < ScalarF5E3::PI * 2_i8);
//! ```

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

pub mod constants;
pub mod conversions;
pub mod core;
pub mod implementations;
pub mod lut;
pub mod operators;
pub mod simd;

// Public core types and traits
pub use crate::core::{
    // Core types
    circle::Circle,
    // Type aliases
    circle_aliases::*,
    // Core traits
    integer::Integer,
    scalar::Scalar,
    scalar_aliases::*,
};

// Constants for components
pub use crate::constants::{CircleConstants, ScalarConstants};

// Operator traits for mixed-type operations
pub use crate::operators::{Clamp, Logarithm, Max, Min, Power};

/// Compile-time `ScalarF4E4` literal from an f32 expression.
///
/// `sf!(0.0031308)` expands to a `const ScalarF4E4` at compile time — no IEEE runtime ops in the binary. The argument must be a normal finite non-zero f32 literal or simple const expression.
#[macro_export]
macro_rules! sf {
    ($e:expr) => {
        $crate::ScalarF4E4::from_f32($e as f32)
    };
}

/// Compile-time `ScalarF4E4` literal from an f64 expression.
///
/// `sd!(1.0/3.0)` expands to a `const ScalarF4E4` at compile time — no IEEE runtime ops in the binary. Provides higher precision than `sf!` for constants with more than 7 significant digits. The argument must be a normal finite non-zero f64 literal or simple const expression.
#[macro_export]
macro_rules! sd {
    ($e:expr) => {
        $crate::ScalarF4E4::from_f64($e as f64)
    };
}

#[cfg(feature = "alloc")]
pub mod tensor;

#[cfg(feature = "alloc")]
pub use tensor::{
    linear_backward, matmul, mse_loss, mse_loss_grad, relu, relu_backward, scale, transpose,
    Linear, LinearGradients, SimpleNet, Tensor, SGD,
};
