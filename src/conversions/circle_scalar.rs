use crate::{core::integer::FullInt, Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// # Decompose a Complex Number into Components
///
/// Converts a `Circle` complex number into its real and imaginary `Scalar` components.
///
/// This function maintains all special states (Zero, Infinity, normal, exploded, vanished, undefined) during the conversion, ensuring the returned `Scalar` components exactly represent the original complex number.
///
/// ## Example
///
/// ```rust
/// use spirix::{Circle, CircleF5E3};
/// use spirix::conversions::circle_scalar::IntoScalars;
///
/// // Create a complex number
/// let z = CircleF5E3::from((3.5, -2.7));  // 3.5 - 2.7i
///
/// // Decompose into real and imaginary parts
/// let (real, imaginary) = z.into_scalars();
///
/// // Verify the components
/// assert_eq!(real, 3.5);
/// assert_eq!(imaginary, -2.7);
///
/// // Special states are preserved
/// let exploded = CircleF5E3::MAX * 5;  // Creates an exploded Circle with no imaginary component
/// let (exploded_real, zero_imaginary) = exploded.into_scalars();
///
/// assert!(exploded_real.exploded());
/// assert!(zero_imaginary.is_zero());
/// ```
///
/// Decomposing a Circle allows you to perform operations on its components:
///
/// ```rust
/// use spirix::{Circle, CircleF5E3, ScalarF5E3};
/// use spirix::conversions::circle_scalar::IntoScalars;
///
/// let z = CircleF5E3::from((4, 3));     // 4 + 3i
/// let (real, imaginary) = z.into_scalars();
///
/// // Calculate magnitude using the Pythagorean formula
/// let magnitude = (real.square() + imaginary.square()).sqrt();
/// assert_eq!(magnitude, 5);
/// ```
pub trait IntoScalars<F: Integer, E: Integer> {
    /// Creates a tuple of Scalars from a Circle
    ///
    /// # Returns
    ///
    /// A tuple of Scalars `(real, imaginary)` containing the real
    /// and imaginary components
    fn into_scalars(&self) -> (Scalar<F, E>, Scalar<F, E>);
}

impl<F: Integer, E: Integer> IntoScalars<F, E> for Circle<F, E>
where
    F: Integer
        + FullInt
        + core::ops::Shl<isize, Output = F>
        + core::ops::Shr<isize, Output = F>
        + core::ops::Shl<F, Output = F>
        + core::ops::Shr<F, Output = F>
        + core::ops::Shl<E, Output = F>
        + core::ops::Shr<E, Output = F>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    E: Integer
        + FullInt
        + core::ops::Shl<isize, Output = E>
        + core::ops::Shr<isize, Output = E>
        + core::ops::Shl<E, Output = E>
        + core::ops::Shr<E, Output = E>
        + core::ops::Shl<F, Output = E>
        + core::ops::Shr<F, Output = E>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    Circle<F, E>: CircleConstants,
    Scalar<F, E>: ScalarConstants,
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
    I256: From<F>,
    I256: From<E>,
{
    fn into_scalars(&self) -> (Scalar<F, E>, Scalar<F, E>) {
        let mut real = Scalar {
            fraction: self.real,
            exponent: self.exponent,
        };
        let mut imaginary = Scalar {
            fraction: self.imaginary,
            exponent: self.exponent,
        };
        if self.is_normal() {
            real.normalize();
            imaginary.normalize();
        } else if self.exploded() {
            real.normalize_exploded();
            imaginary.normalize_exploded();
        } else if self.vanished() {
            real.normalize_vanished();
            imaginary.normalize_vanished();
        }

        (real, imaginary)
    }
}

/// Implementation for mutable Circle references
///
/// Allows converting a mutable reference to a Circle into its
/// real and imaginary Scalar components without consuming the original.
impl<F: Integer, E: Integer> IntoScalars<F, E> for &mut Circle<F, E>
where
    F: Integer
        + FullInt
        + core::ops::Shl<isize, Output = F>
        + core::ops::Shr<isize, Output = F>
        + core::ops::Shl<F, Output = F>
        + core::ops::Shr<F, Output = F>
        + core::ops::Shl<E, Output = F>
        + core::ops::Shr<E, Output = F>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    E: Integer
        + FullInt
        + core::ops::Shl<isize, Output = E>
        + core::ops::Shr<isize, Output = E>
        + core::ops::Shl<E, Output = E>
        + core::ops::Shr<E, Output = E>
        + core::ops::Shl<F, Output = E>
        + core::ops::Shr<F, Output = E>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    Circle<F, E>: CircleConstants,
    Scalar<F, E>: ScalarConstants,
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
    I256: From<F>,
    I256: From<E>,
{
    fn into_scalars(&self) -> (Scalar<F, E>, Scalar<F, E>) {
        let mut real = Scalar {
            fraction: self.real,
            exponent: self.exponent,
        };
        let mut imaginary = Scalar {
            fraction: self.imaginary,
            exponent: self.exponent,
        };
        if self.is_normal() {
            real.normalize();
            imaginary.normalize();
        } else if self.exploded() {
            real.normalize_exploded();
            imaginary.normalize_exploded();
        } else if self.vanished() {
            real.normalize_vanished();
            imaginary.normalize_vanished();
        }

        (real, imaginary)
    }
}

/// Implementation for immutable Circle references
///
/// Allows converting an immutable reference to a Circle into its
/// real and imaginary Scalar components without consuming the original.
impl<F: Integer, E: Integer> IntoScalars<F, E> for &Circle<F, E>
where
    F: Integer
        + FullInt
        + core::ops::Shl<isize, Output = F>
        + core::ops::Shr<isize, Output = F>
        + core::ops::Shl<F, Output = F>
        + core::ops::Shr<F, Output = F>
        + core::ops::Shl<E, Output = F>
        + core::ops::Shr<E, Output = F>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    E: Integer
        + FullInt
        + core::ops::Shl<isize, Output = E>
        + core::ops::Shr<isize, Output = E>
        + core::ops::Shl<E, Output = E>
        + core::ops::Shr<E, Output = E>
        + core::ops::Shl<F, Output = E>
        + core::ops::Shr<F, Output = E>
        + WrappingNeg
        + WrappingAdd
        + WrappingMul
        + WrappingSub,
    Circle<F, E>: CircleConstants,
    Scalar<F, E>: ScalarConstants,
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
    I256: From<F>,
    I256: From<E>,
{
    fn into_scalars(&self) -> (Scalar<F, E>, Scalar<F, E>) {
        let mut real = Scalar {
            fraction: self.real,
            exponent: self.exponent,
        };
        let mut imaginary = Scalar {
            fraction: self.imaginary,
            exponent: self.exponent,
        };
        if self.is_normal() {
            real.normalize();
            imaginary.normalize();
        } else if self.exploded() {
            real.normalize_exploded();
            imaginary.normalize_exploded();
        } else if self.vanished() {
            real.normalize_vanished();
            imaginary.normalize_vanished();
        }

        (real, imaginary)
    }
}
