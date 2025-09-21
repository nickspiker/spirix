// src/core/scalar.rs
use crate::Integer;
/// # Scalar
///
/// The `Scalar<F, E>` type represents real numbers using two's complement fraction and exponent components with customizable precision.
///
/// ## Type Parameters
///
/// - `F`: Fraction - Any sized Rust signed integer type (i8, i16, i32, i64, i128)
/// - `E`: Exponent - Any sized Rust signed integer type (i8, i16, i32, i64, i128)
///
/// ## Representation
///
/// Scalars use a normalized representation where the value is calculated as:
/// `fraction * 2^exponent` for normal numbers, with specific bit patterns for:
///
/// - Normal finite numbers `[#]` (positive and negative)
/// - Exploded values `[↑]` (numbers too large to represent)
/// - Vanished values `[↓]` (numbers too small to represent)
/// - Actual Zero `[0]`
/// - Singular Infinity `[∞]`
/// - Undefined states `[℘]`
///
/// ## Normalization Levels
///
/// The bit patterns in the fraction follow these normalization levels:
///
/// ```txt
/// Position: 01234567...
///
/// N0: Zero and Infinity
/// □□□□□□□□  Zero [0]
/// ■■■■■■■■  Infinity [∞]
///
/// N1: Normal and Exploded Values
/// □■xxxxxx  Positive normal [+#] or exploded [+↑] (with AMBIGUOUS_EXPONENT)
/// ■□xxxxxx  Negative normal [-#] or exploded [-↑] (with AMBIGUOUS_EXPONENT)
///
/// N2: Vanishing Values
/// □□■xxxxx  Positive vanished [+↓] (approaching but not equal to 0)
/// ■■□xxxxx  Negative vanished [-↓] (approaching but not equal to 0)
///
/// N3+: Undefined States
/// □□□xxxxx | ■■■xxxxx  Specific undefined states
/// ```
///
/// See `undefined.rs` for the complete catalog of undefined patterns.
///
/// ## Examples
///
/// ```rust
/// use spirix::{Scalar, ScalarF5E3};
///
/// // Create a Scalar with 32-bit fraction and 8-bit exponent
/// let a = Scalar::<i32, i8>::from(42);
///
/// // Using a type alias for the same size
/// let mut b = ScalarF5E3::from(-1);
/// b /= 12; // Divide -1 by 12 and assign to b
///
/// // Track undefined states while preserving first cause
/// let zero_div_zero = (a - 42) / 0;
/// assert!(zero_div_zero.is_undefined());
/// let still_undefined_zero_div_zero = (zero_div_zero + b).pow(-5.71).log(-0.005);  // first cause is preserved
///
/// // Escaped values preserve phase
/// let exploded = ScalarF5E3::MAX * 2;
/// assert!(exploded.exploded() && exploded.is_positive());
///
/// // Vanished values preserve phase too!
/// let vanished = ScalarF5E3::MAX_NEG / 3;
/// assert!(vanished.vanished() && vanished.is_negative());
/// // Absolute operations can be applied to escaped values
/// assert!(vanished.square().is_positive());
/// ```
#[derive(Clone, Copy)]
pub struct Scalar<F: Integer, E: Integer> {
    /// The normalized fraction component representing the significand.
    /// The fraction size determines the precision of the value.
    /// The prefix bit pattern determines the number's state (normal, Zero, Infinity, exploded, vanished, undefined).
    pub fraction: F,

    /// The exponent component determining the scale of the value.
    /// The exponent size determines the range of the value.
    /// When equal to AMBIGUOUS_EXPONENT (0b1000000...), indicates an abnormal state (Infinity, Zero, exploded, vanished, or undefined).
    pub exponent: E,
}
