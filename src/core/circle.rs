use crate::Integer;
/// # Circle
///
/// Circles represents complex numbers using two's complement real and imaginary components with a shared exponent and customizable precision/range.
///
/// ## Type Parameters
///
/// - `F`: Fraction - Any sized Rust signed integer type (i8, i16, i32, i64, i128)
/// - `E`: Exponent - Any sized Rust signed integer type (i8, i16, i32, i64, i128)
///
/// ## Representation
///
/// Circles use a normalized representation where components share the same exponent:
/// `(real + imaginary*i) * 2^exponent` for normal numbers, with specific bit patterns for:
///
/// - Normal finite numbers `[#]` (with positive and negative components)
/// - Exploded values `[↑]` (numbers too large to represent)
/// - Vanished values `[↓]` (numbers too small to represent)
/// - Actual Zero `[0]`
/// - Singular Infinity `[∞]`
/// - Undefined states `[℘]`
///
/// ## Normalization Levels
///
/// The bit patterns in the real and imaginary components follow these normalization levels:
///
/// ```txt
/// Position: 01234567...
///
/// N0: Zero and Infinity
/// □□□□□□□□  Zero [0]
/// ■■■■■■■■  Infinity [∞]
///
/// N1: Normal and Exploded Values
/// □■xxxxxx | ■□xxxxxx  Normal [#,#] or exploded [↑]
///
/// N2: Vanishing Values
/// □□■xxxxx | ■■□xxxxx Vanished [↓]
///
/// N3+: Undefined States
/// □□□xxxxx | ■■■xxxxx
/// ```
///
/// See `undefined.rs` for the complete catalog of undefined patterns.
///
/// ## Mathematical Properties
///
/// Circles maintain consistent behavior across operations:
///
/// - The shared exponent applies to both fractions
/// - When the exponent goes out of range, values "escape" to exploded (almost ∞) or vanished (almost 0) states while preserving angular orientation
/// - Undefined states stay fixed thru subsequent operations
/// - Complex operations like addition, multiplication, and division work with Circles, Scalars and Rust primitives
/// - Distinct modulus (%) and modulo (.modulo()) operations are provided for mathematical vs programming applications
///
/// ## Examples
///
/// ```rust
/// use spirix::{Circle, CircleF5E3};
///
/// // Create a Circle with 32-bit components and 8-bit exponent from a tuple (R,I)
/// let z = Circle::<i32, i8>::from((1.5, 2));
///
/// // Using a type alias for the same size with no real component
/// let w = CircleF5E3::from((0, -16.8));
///
/// // Using a single real component
/// let d = CircleF5E3::from(2.2);
///
/// // Basic arithmetic
/// let sum = z + w;
/// let product = d * w;
///
/// // Basic arithmetic with primitives
/// let sum = z + 42;
/// let product = w.pow(d) * -44.25;
/// let reciprocal = 1 / d;
///
/// // Complex-specific operations
/// let conj = sum.conjugate();
/// let magnitude = product.magnitude();
///
/// // Infinity is a singular entity in Spirix
/// let infinite = z / 0;
/// assert!(infinite.is_infinite());
/// ```
#[derive(Clone, Copy)]
pub struct Circle<F: Integer, E: Integer> {
    /// The real component representing the real part of the complex number.
    pub real: F,

    /// The imaginary component representing the imaginary part of the complex number.
    pub imaginary: F,

    /// The shared exponent component determining the scale of both components.
    /// When equal to AMBIGUOUS_EXPONENT (0b1000000...), indicates an abnormal state (Infinity, Zero, exploded, vanished, or undefined).
    pub exponent: E,
}
