use crate::core::integer::*;
use crate::core::undefined::*;
use crate::ScalarConstants;
use crate::{Circle, CircleConstants, Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_complex::Complex;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// # Convert Values to Complex Numbers
///
/// The `IntoCircle` trait provides a consistent way to convert various types into Spirix's `Circle<F, E>` complex number type.
///
/// ## What This Trait Does
///
/// This trait defines the interface for converting different numeric types (both real and complex) into Spirix's two-component complex number representation with customizable precision.
///
/// ## Implemented For
///
/// - Primitive numeric types (i8, i16, i32, i64, i128, f32, f64, etc.)
/// - `std::num::Complex<f32>` and `std::num::Complex<f64>`
/// - Tuples representing complex numbers (real, imaginary)
/// - References to these types
///
/// ## Example
///
/// ```rust
/// use spirix::{Circle, CircleF5E3};
///
/// // 0. Converting a primitive (creates a complex number with Zero imaginary part) let from_integer = CircleF5E3::from(42_i32); assert_eq!(from_integer.r(), 42_i32); assert_eq!(from_integer.i(), 0_i32);
///
/// // 1. Converting a tuple of (real, imaginary) components let from_tuple = CircleF5E3::from((3_i32, -2_i32)); assert_eq!(from_tuple.r(), 3_i32); assert_eq!(from_tuple.i(), -2_i32);
/// ```
///
/// ## Precision Handling
///
/// When converting from IEEE-754 floating point types to Spirix's `Circle`, special care is taken to properly handle:
///
/// - NaN values (converted to generic undefined)
/// - Infinities (coerced to singular infinity)
/// - Subnormal numbers (properly scaled)
///
/// ## Conversion Between Number Systems
///
/// Converting from standard IEEE-754 floating point to Spirix's number system involves several steps:
///
/// 0. For real primitives:
///    - The real part is converted to a `Scalar`
///    - A Circle is created from the Scalar with imaginary part set to Zero
///
/// 1. For complex tuples or `std::num::Complex`:
///    - Both components are independently converted to `Scalar` values
///    - The components are then aligned and combined
///    - A shared exponent is assigned to both components
pub trait IntoCircle<F: Integer, E: Integer> {
    /// Converts the value into a Circle
    ///
    /// # Returns
    ///
    /// A Circle instance representing this value
    fn into_circle(self) -> Circle<F, E>;
}

/// Implementation for converting primitive types to Circle
///
/// This allows any type R that can be converted to a Scalar to also be converted to a Circle. The resulting Circle will have a Zero imaginary component.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
        R,
    > From<R> for Circle<F, E>
where
    Scalar<F, E>: From<R>,
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
    R: Copy,
{
    /// # Convert a Real Number to a Complex Number
    ///
    /// Creates a Circle with the given value as real component and zero imaginary component.
    ///
    /// This conversion is useful when you need to use a real number in context where  a complex number is expected.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // From integer let z1 = CircleF5E3::from(42_i32); assert_eq!(z1.r(), 42_i32); assert_eq!(z1.i(), 0_i32);
    ///
    /// // From floating point (3.75 = 15/4, exactly representable) let z2 = CircleF5E3::from(3.75_f32); assert_eq!(z2.r(), 3.75_f32); assert_eq!(z2.i(), 0_i32);
    /// ```
    fn from(value: R) -> Self {
        let scalar = Scalar::<F, E>::from(value);
        // Translate Scalar format (implicit sign, FRAC bits) to Circle format (explicit sign, FRAC-1 bits): Circle stored = scalar_inflate >> 1. AMBIG=0: normal class is stored exponent != 0; the old `!= E::min_value()` check predated AMBIG=0 (when E::MIN was the sentinel) and silently broke `Circle::from(1.5)` once E::MIN became the +1.0 binade.
        let circle_real: F = if scalar.exponent != E::zero() {
            scalar.fraction.inflate(true).w_shr(1isize).deflate()
        } else {
            // Escape classes: prefix bit patterns differ between formats. ZERO, INFINITY: same bit pattern (all 0s / all 1s). Others: translation via sign_extend >> 1 works (prefix shifts down by 1).
            scalar.fraction.inflate(false).w_shr(1isize).deflate()
        };
        Self {
            real: circle_real,
            imaginary: 0.as_(),
            exponent: scalar.exponent,
        }
    }
}

/// Implementation for converting `Complex<f64>` to Circle
///
/// This conversion handles IEEE-754 special values like NaN by converting them to appropriate undefined states in the Spirix number system.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<Complex<f64>> for Circle<F, E>
where
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
{
    /// # Convert a Binary64 Complex pair to a Circle
    ///
    /// Creates a Circle from a `std::num::Complex<f64>` pair, handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3}; use num_complex::Complex;
    ///
    /// // Create a `std::num::Complex<f64>` let complex = Complex::new(1.5, -2.7);
    ///
    /// // Convert to Circle let z = CircleF5E3::from(complex);
    ///
    /// assert_eq!(z.r(), 1.5); assert_eq!(z.i(), -2.7);
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN values are converted to undefined states with the `GENERAL` prefix
    /// - Infinities are coerced to singular infinity
    /// - Zero components are preserved as exact zeros
    fn from(complex: Complex<f64>) -> Self {
        let real_scalar = Scalar::<F, E>::from(complex.re);
        let imag_scalar = Scalar::<F, E>::from(complex.im);
        if real_scalar.is_undefined() || imag_scalar.is_undefined() {
            let prefix: F = GENERAL.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Scalar::<F, E>::ambiguous_exponent(),
            };
        }
        Self::from_ri(real_scalar, imag_scalar)
    }
}

/// Implementation for converting `Complex<f32>` to Circle
///
/// Similar to the f64 implementation, this handles IEEE-754 special values appropriately when converting to the Spirix number system.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<Complex<f32>> for Circle<F, E>
where
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
{
    /// # Convert a Complex Binary32 pair to a Circle
    ///
    /// Creates a Circle from a `std::num::Complex<f32>` value, handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3}; use num_complex::Complex;
    ///
    /// // Create a `std::num::Complex<f32>` with exactly-representable values let complex = Complex::new(1.5f32, -2.5f32);
    ///
    /// // Convert to Circle let z = CircleF5E3::from(complex);
    ///
    /// assert_eq!(z.r(), 1.5_f32); assert_eq!(z.i(), -2.5_f32);
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN values are converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f32 values are properly scaled during conversion
    fn from(complex: Complex<f32>) -> Self {
        let real_scalar = Scalar::<F, E>::from(complex.re);
        let imag_scalar = Scalar::<F, E>::from(complex.im);
        if real_scalar.is_undefined() || imag_scalar.is_undefined() {
            let prefix: F = GENERAL.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Scalar::<F, E>::ambiguous_exponent(),
            };
        }
        Self::from_ri(real_scalar, imag_scalar)
    }
}

/// Implementation for converting tuples to Circle
///
/// This allows creating a Circle from a tuple (real, imaginary) where both components can be independently converted to Scalars.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        R,
        I,
    > From<(R, I)> for Circle<F, E>
where
    Scalar<F, E>: From<R> + From<I>,
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
    R: Copy,
    I: Copy,
{
    /// # Create a Circle from a pair
    ///
    /// Creates a Circle from a tuple of (real, imaginary) components, allowing different types for each component.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Same types let z1 = CircleF5E3::from((3, 4)); assert_eq!(z1.r(), 3); assert_eq!(z1.i(), 4);
    ///
    /// // Mixed types let z2 = CircleF5E3::from((5, -1.5)); assert_eq!(z2.r(), 5); assert_eq!(z2.i(), -1.5);
    /// ```
    ///
    /// ## Conversion Process
    ///
    /// 1. Each component is independently converted to a Scalar
    /// 2. Components are aligned
    /// 3. A shared exponent is determined for both components
    /// 4. Any special cases (infinities, zeros, etc.) are handled
    fn from(pair: (R, I)) -> Self {
        let (real, imag) = pair;
        let real_scalar = Scalar::<F, E>::from(real);
        let imag_scalar = Scalar::<F, E>::from(imag);
        Self::from_ri(real_scalar, imag_scalar)
    }
}

/// Implementation for converting references to `Complex<f64>` to Circle
///
/// This provides a convenient way to convert a reference to a `Complex<f64>` without taking ownership.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<&Complex<f64>> for Circle<F, E>
where
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
{
    /// # Convert Binary64 Complex pair reference to a Circle
    ///
    /// Creates a Circle from a reference to a `std::num::Complex<f64>`, allowing conversion without taking ownership of the source value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3}; use num_complex::Complex;
    ///
    /// // Create a `std::num::Complex<f64>` let complex = Complex::new(1.5, -2.7);
    ///
    /// // Convert from reference without moving the original let z = CircleF5E3::from(&complex);
    ///
    /// // Original complex value is still available assert_eq!(complex.re, 1.5); assert_eq!(z.r(), 1.5); assert_eq!(z.i(), -2.7);
    /// ```
    ///
    /// This implementation delegates to the `From<Complex<f64>>` implementation after dereferencing.
    fn from(complex: &Complex<f64>) -> Self {
        Self::from(*complex)
    }
}

/// Implementation for converting references to `Complex<f32>` to Circle
///
/// This provides a convenient way to convert a reference to a `Complex<f32>` without taking ownership.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<&Complex<f32>> for Circle<F, E>
where
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
{
    /// # Convert Binary32 Complex pair reference to a Circle
    ///
    /// Creates a Circle from a reference to a `std::num::Complex<f32>`, allowing conversion without taking ownership of the source value.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3}; use num_complex::Complex;
    ///
    /// // Create a `std::num::Complex<f32>` with exactly-representable values let complex = Complex::new(1.5f32, -2.5f32);
    ///
    /// // Convert from reference without moving the original let z = CircleF5E3::from(&complex);
    ///
    /// // Original complex value is still available assert_eq!(complex.re, 1.5f32); assert_eq!(z.r(), 1.5_f32); assert_eq!(z.i(), -2.5_f32);
    /// ```
    ///
    /// This implementation delegates to the `From<Complex<f32>>` implementation after dereferencing.
    fn from(complex: &Complex<f32>) -> Self {
        Self::from(*complex)
    }
}
