use crate::constants::{CircleConstants, ScalarConstants};
use crate::core::integer::FullInt;
use crate::{Circle, Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_complex::Complex;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// # Circle to `Complex<f64>` Conversion
///
/// Implements conversion from Circle to the standard library's `Complex<f64>` type, mapping Spirix's states to appropriate IEEE-754 floating-point equivalents.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f64>
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
    > Into<Complex<f64>> for &Circle<F, E>
where
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
    /// Converts a Circle to `Complex<f64>`
    ///
    /// This conversion maps a Circle to IEEE-754 floating-point, preserving mathematical properties where possible.
    ///
    /// # Special Cases
    ///
    /// - Undefined states (`[℘]`): Map to `(NaN, NaN)`
    /// - Vanished values (`[+↓]`, `[-↓]`): Convert to zero, if negative components, returns `-0.0` for each
    /// - Exploded values (`[+↑]`, `[-↑]`): Converted to + or - infinity, depending on sign for each component
    /// - Normal values (`[+#]`, `[-#]`): Calculated as `component * 2^exponent` for both real and imaginary parts
    /// - Infinite and Zero values are coerced to positive infinity and positive zero, respectively.
    /// # Notes
    ///
    /// The conversion process scales the fraction components by dividing by 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
    fn into(self) -> Complex<f64> {
        if !self.is_normal() {
            if self.is_undefined() {
                return Complex::new(f64::NAN, f64::NAN);
            }
            if self.is_infinite() {
                // Singular [∞] has no direction; IEEE can't represent it.
                return Complex::new(f64::NAN, f64::NAN);
            }
            if self.is_negligible() {
                return Complex::new(
                    if self.real.is_negative() { -0.0 } else { 0.0 },
                    if self.imaginary.is_negative() {
                        -0.0
                    } else {
                        0.0
                    },
                );
            }
            // Exploded: per-component sign is meaningful.
            return Complex::new(
                if self.real.is_negative() {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                },
                if self.imaginary.is_negative() {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                },
            );
        }

        // Normal path: delegate to the component extractors + the verified Scalar→f64 conversion. The previous hand-rolled exponent math predated the v0.1 ruler shift and read every value at exactly half (N1 alignment off by one).
        let re: f64 = self.r().into();
        let im: f64 = self.i().into();
        Complex::new(re, im)
    }
}

/// # Circle to `Complex<f32>` Conversion
///
/// Implements conversion from Circle to the standard library's `Complex<f32>` type, mapping Spirix's states to appropriate IEEE-754 floating-point equivalents.
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f32>
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
    > Into<Complex<f32>> for &Circle<F, E>
where
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
    /// Converts a Circle to `Complex<f32>`
    ///
    /// This conversion maps a Circle to IEEE-754 floating-point, preserving mathematical properties where possible.
    ///
    /// # Special Cases
    ///
    /// - Undefined states (`[℘]`): Map to `(NaN, NaN)`
    /// - Vanished values (`[+↓]`, `[-↓]`): Convert to zero, if negative components, returns `-0.0` for each
    /// - Exploded values (`[+↑]`, `[-↑]`): Converted to + or - infinity, depending on sign for each component
    /// - Normal values (`[+#]`, `[-#]`): Calculated as `component * 2^exponent` for both real and imaginary parts
    /// - Infinite and Zero values are coerced to positive infinity and positive zero, respectively.
    /// # Notes
    ///
    /// The conversion process scales the fraction components by dividing by 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
    fn into(self) -> Complex<f32> {
        if !self.is_normal() {
            if self.is_undefined() {
                return Complex::new(f32::NAN, f32::NAN);
            }
            if self.is_infinite() {
                // Singular [∞] has no direction.
                return Complex::new(f32::NAN, f32::NAN);
            }
            if self.is_negligible() {
                return Complex::new(
                    if self.real.is_negative() { -0.0 } else { 0.0 },
                    if self.imaginary.is_negative() {
                        -0.0
                    } else {
                        0.0
                    },
                );
            }
            // Exploded: per-component sign is meaningful.
            return Complex::new(
                if self.real.is_negative() {
                    f32::NEG_INFINITY
                } else {
                    f32::INFINITY
                },
                if self.imaginary.is_negative() {
                    f32::NEG_INFINITY
                } else {
                    f32::INFINITY
                },
            );
        }

        // Normal path: delegate to the component extractors + the verified Scalar→f32 conversion (same v0.1 ruler-shift off-by-one as the f64 version).
        let re: f32 = self.r().into();
        let im: f32 = self.i().into();
        Complex::new(re, im)
    }
}

// Implement owned versions that use the reference implementations
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f64>
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
    > Into<Complex<f64>> for Circle<F, E>
where
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
    /// Converts a Circle to `Complex<f64>`
    ///
    /// This conversion maps a Circle to IEEE-754 floating-point, preserving mathematical properties where possible.
    ///
    /// # Special Cases
    ///
    /// - Undefined states (`[℘]`): Map to `(NaN, NaN)`
    /// - Vanished values (`[+↓]`, `[-↓]`): Convert to zero, if negative components, returns `-0.0` for each
    /// - Exploded values (`[+↑]`, `[-↑]`): Converted to + or - infinity, depending on sign for each component
    /// - Normal values (`[+#]`, `[-#]`): Calculated as `component * 2^exponent` for both real and imaginary parts
    /// - Infinite and Zero values are coerced to positive infinity and positive zero, respectively.
    /// # Notes
    ///
    /// The conversion process scales the fraction components by dividing by 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
    fn into(self) -> Complex<f64> {
        (&self).into()
    }
}

impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f32>
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
    > Into<Complex<f32>> for Circle<F, E>
where
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
    /// Converts a Circle to `Complex<f32>`
    ///
    /// This conversion maps a Circle to IEEE-754 floating-point, preserving mathematical properties where possible.
    ///
    /// # Special Cases
    ///
    /// - Undefined states (`[℘]`): Map to `(NaN, NaN)`
    /// - Vanished values (`[+↓]`, `[-↓]`): Convert to zero, if negative components, returns `-0.0` for each
    /// - Exploded values (`[+↑]`, `[-↑]`): Converted to + or - infinity, depending on sign for each component
    /// - Normal values (`[+#]`, `[-#]`): Calculated as `component * 2^exponent` for both real and imaginary parts
    /// - Infinite and Zero values are coerced to positive infinity and positive zero, respectively.
    /// # Notes
    ///
    /// The conversion process scales the fraction components by dividing by 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
    fn into(self) -> Complex<f32> {
        (&self).into()
    }
}
