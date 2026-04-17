use crate::constants::{CircleConstants, ScalarConstants};
use crate::core::integer::FullInt;
use crate::{Circle, Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_complex::Complex;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// # Circle to `Complex<f64>` Conversion
///
/// Implements conversion from Circle to the standard library's `Complex<f64>` type,
/// mapping Spirix's states to appropriate IEEE-754 floating-point equivalents.
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
    /// The conversion process scales the fraction components by dividing by
    /// 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
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

        // Convert normalized fractions
        let mut base_real: f64 = self.real.as_();
        let mut base_imag: f64 = self.imaginary.as_();

        // Adjust for normal value normalization
        let frac_scale =
            f64::from_bits(((1023i64 + Circle::<F, E>::fraction_bits() as i64 - 1) as u64) << 52);
        base_real = base_real / frac_scale;
        base_imag = base_imag / frac_scale;

        // Apply exponent scaling
        let exponent: i32 = self.exponent.saturate();
        let scale = f64::from_bits(((1023i64 + exponent as i64) as u64) << 52);

        Complex::new(base_real * scale, base_imag * scale)
    }
}

/// # Circle to `Complex<f32>` Conversion
///
/// Implements conversion from Circle to the standard library's `Complex<f32>` type,
/// mapping Spirix's states to appropriate IEEE-754 floating-point equivalents.
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
    /// The conversion process scales the fraction components by dividing by
    /// 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
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

        // Convert normalized fractions
        let mut base_real: f32 = self.real.as_();
        let mut base_imag: f32 = self.imaginary.as_();

        // Adjust for fraction normalization
        let frac_scale =
            f32::from_bits(((127i32 + Circle::<F, E>::fraction_bits() as i32 - 1) as u32) << 23);
        base_real = base_real / frac_scale;
        base_imag = base_imag / frac_scale;

        // Apply exponent scaling
        let exponent: i32 = self.exponent.saturate();
        let scale = f32::from_bits(((127i32 + exponent) as u32) << 23);

        Complex::new(base_real * scale, base_imag * scale)
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
    /// The conversion process scales the fraction components by dividing by
    /// 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
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
    /// The conversion process scales the fraction components by dividing by
    /// 2^(FRACTION_BITS-1) before applying the exponent scaling to maintain normalization alignment.
    fn into(self) -> Complex<f32> {
        (&self).into()
    }
}
