use crate::core::integer::FullInt;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, PrimInt};
use std::ops::*;

impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f64>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<f64> for &Scalar<F, E>
where
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
    fn into(self) -> f64 {
        if self.is_normal() {
            let i64: i64 = self.fraction.sa();
            let base = i64 as f64 / 2f64.powi(64 - 1);
            let exponent: i32 = self.exponent.saturate();

            base * 2f64.powi(exponent)
        } else {
            if self.is_undefined() {
                return f64::NAN;
            }

            if self.is_negligible() {
                return if self.fraction.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                return f64::INFINITY;
            }
            return if self.fraction.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
    }
}

impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f32>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<f32> for &Scalar<F, E>
where
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
    fn into(self) -> f32 {
        if self.is_normal() {
            let i32: i32 = self.fraction.sa();
            let base = i32 as f32 / 2f32.powi(32 - 1);
            let exponent: i32 = self.exponent.saturate();

            base * 2f32.powi(exponent)
        } else {
            if self.is_undefined() {
                return f32::NAN;
            }

            if self.is_negligible() {
                return if self.fraction.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                return f32::INFINITY;
            }
            return if self.fraction.is_negative() {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
        }
    }
}
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f32>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<f32> for Scalar<F, E>
where
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
    fn into(self) -> f32 {
        (&self).into()
    }
}
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f64>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<f64> for Scalar<F, E>
where
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
    fn into(self) -> f64 {
        (&self).into()
    }
}
macro_rules! impl_into_int {
    ($($i:ty),*) => {
        $(
            impl<
  F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<$i> for Scalar<F, E>
    where
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
                fn into(self) -> $i {
                    (&self).into()
                }
            }

            impl<
  F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<$i> for &Scalar<F, E>
    where
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
fn into(self) -> $i {
    if !self.exponent.is_positive() {
        if self.exploded() {
            if self.fraction.is_negative() {
                return <$i>::MIN;
            }
            return <$i>::MAX;
        }
        let number_bits = self.prefix() >> 5;
        if number_bits == number_bits.wrapping_shr(1) {
            return 0;
        }
        if self.fraction.is_negative() {
            return -1;
        }
        return 0;
    }

    let shift: usize = (self.exponent).saturate();
    if shift >= std::mem::size_of::<$i>() * 8 {
        if self.fraction.is_negative() {
            return <$i>::MIN;
        }
        return <$i>::MAX;
    }
    let mut value = self.fraction.sa();
    value = value >> (std::mem::size_of::<$i>() * 8 + 1 - shift);
    value
}
}
        )*
    }
}
impl_into_int!(i8, i16, i32, i64, i128, isize);
macro_rules! impl_into_uint {
    ($($u:ty),*) => {
        $(
            impl<
  F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<$u> for Scalar<F, E>
    where
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
    I256: From<E>, Scalar<F, E>: Into<f64>,
{
                fn into(self) -> $u {
                    (&self).into()
                }
            }

            impl<
  F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Into<$u> for &Scalar<F, E>
    where
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
    I256: From<E>, Scalar<F, E>: Into<f64>,
{
    fn into(self) -> $u {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return <$u>::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift > std::mem::size_of::<$u>() * 8 {
            return <$u>::MAX;
        }
        let mut value = (self.fraction<<1isize).sa();
        value = value >> (std::mem::size_of::<$u>() * 8 - shift);
        value
    }
}
        )*
    }
}
impl_into_uint!(u8, u16, u32, u64, u128, usize);
#[allow(private_bounds)]
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Scalar<F, E>
where
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
    /// Converts this Scalar to an i8 value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns i8::MIN, positive returns i8::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return i8::MIN or i8::MAX
    #[inline]
    pub fn to_i8(&self) -> i8 {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return i8::MIN;
                }
                return i8::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= 8 {
            if self.fraction.is_negative() {
                return i8::MIN;
            }
            return i8::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (7 - shift);
        value
    }

    /// Converts this Scalar to an i16 value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns i16::MIN, positive returns i16::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return i16::MIN or i16::MAX
    #[inline]
    pub fn to_i16(&self) -> i16 {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return i16::MIN;
                }
                return i16::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= 16 {
            if self.fraction.is_negative() {
                return i16::MIN;
            }
            return i16::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (15 - shift);
        value
    }

    /// Converts this Scalar to an i32 value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns i32::MIN, positive returns i32::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return i32::MIN or i32::MAX
    #[inline]
    pub fn to_i32(&self) -> i32 {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return i32::MIN;
                }
                return i32::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= 32 {
            if self.fraction.is_negative() {
                return i32::MIN;
            }
            return i32::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (31 - shift);
        value
    }

    /// Converts this Scalar to an i64 value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns i64::MIN, positive returns i64::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return i64::MIN or i64::MAX
    #[inline]
    pub fn to_i64(&self) -> i64 {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return i64::MIN;
                }
                return i64::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= 64 {
            if self.fraction.is_negative() {
                return i64::MIN;
            }
            return i64::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (63 - shift);
        value
    }

    /// Converts this Scalar to an i128 value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns i128::MIN, positive returns i128::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return i128::MIN or i128::MAX
    #[inline]
    pub fn to_i128(&self) -> i128 {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return i128::MIN;
                }
                return i128::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= 128 {
            if self.fraction.is_negative() {
                return i128::MIN;
            }
            return i128::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (127 - shift);
        value
    }

    /// Converts this Scalar to an isize value
    ///
    /// Handles special cases:
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Vanished negative returns -1, positive returns 0
    /// - Exploded negative returns isize::MIN, positive returns isize::MAX
    /// - Values with negative exponents round towards zero
    /// - Values outside the representable range return isize::MIN or isize::MAX
    #[inline]
    pub fn to_isize(&self) -> isize {
        if !self.exponent.is_positive() {
            if self.exploded() {
                if self.fraction.is_negative() {
                    return isize::MIN;
                }
                return isize::MAX;
            }
            let number_bits = self.prefix() >> 5;
            if number_bits == number_bits.wrapping_shr(1) {
                return 0;
            }
            if self.fraction.is_negative() {
                return -1;
            }
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= std::mem::size_of::<isize>() * 8 {
            if self.fraction.is_negative() {
                return isize::MIN;
            }
            return isize::MAX;
        }
        let mut value = self.fraction.sa();
        value = value >> (std::mem::size_of::<isize>() * 8 - 1 - shift);
        value
    }

    /// Converts this Scalar to a u8 value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Exploded positive returns u8::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return u8::MAX
    #[inline]
    pub fn to_u8(&self) -> u8 {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return u8::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();

        if shift > 8 {
            return u8::MAX;
        }

        let shifted_fraction = self.fraction << 1isize;

        let mut value = shifted_fraction.sa();

        value = value >> (8 - shift);

        value
    }

    /// Converts this Scalar to a u16 value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Exploded positive returns u16::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return u16::MAX
    #[inline]
    pub fn to_u16(&self) -> u16 {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return u16::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift > 16 {
            return u16::MAX;
        }
        let mut value = (self.fraction << 1isize).sa();
        value = value >> (16 - shift);
        value
    }

    /// Converts this Scalar to a u32 value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Exploded positive returns u32::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return u32::MAX
    #[inline]
    pub fn to_u32(&self) -> u32 {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return u32::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift > 32 {
            return u32::MAX;
        }
        let mut value = (self.fraction << 1isize).sa();
        value = value >> (32 - shift);
        value
    }

    /// Converts this Scalar to a u64 value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Exploded positive returns u64::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return u64::MAX
    #[inline]
    pub fn to_u64(&self) -> u64 {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return u64::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift > 64 {
            return u64::MAX;
        }
        let mut value = (self.fraction << 1isize).sa();
        value = value >> (64 - shift);
        value
    }

    /// Converts this Scalar to a u128 value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Infinity returns 0
    /// - Exploded positive returns u128::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return u128::MAX
    #[inline]
    pub fn to_u128(&self) -> u128 {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return u128::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift > 128 {
            return u128::MAX;
        }
        let mut value = (self.fraction << 1isize).sa();
        value = value >> (128 - shift);
        value
    }

    /// Converts this Scalar to a usize value
    ///
    /// Handles special cases:
    /// - Negative values return 0
    /// - Zero returns 0
    /// - Undefined returns 0
    /// - Exploded positive returns usize::MAX
    /// - Values with exponents < 1 return 0
    /// - Values outside the representable range return usize::MAX
    #[inline]
    pub fn to_usize(&self) -> usize {
        if self.fraction.is_negative() {
            return 0;
        }
        if self.exploded() {
            return usize::MAX;
        }
        if !self.exponent.is_positive() {
            return 0;
        }

        let shift: usize = (self.exponent).saturate();
        if shift >= std::mem::size_of::<usize>() * 8 {
            return usize::MAX;
        }
        let mut value = (self.fraction << 1isize).sa();
        value = value >> (std::mem::size_of::<usize>() * 8 - shift);
        value
    }

    /// Converts this Scalar to an f32 value
    ///
    /// Handles special cases:
    /// - Undefined returns NaN
    /// - Vanished negative returns -0.0, positive returns 0.0
    /// - Exploded negative returns -∞, positive returns ∞
    /// - Normal values convert via binary scaling
    #[inline]
    pub fn to_f32(&self) -> f32 {
        if self.is_normal() {
            let i32: i32 = self.fraction.sa();
            let base = i32 as f32 / 2f32.powi(32 - 1);
            let exponent: i32 = self.exponent.saturate();

            base * 2f32.powi(exponent)
        } else {
            if self.is_undefined() {
                return f32::NAN;
            }

            if self.is_negligible() {
                return if self.fraction.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                return f32::INFINITY;
            }
            return if self.fraction.is_negative() {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
        }
    }

    /// Converts this Scalar to an f64 value
    ///
    /// Handles special cases:
    /// - Undefined returns NaN
    /// - Vanished negative returns -0.0, positive returns 0.0
    /// - Exploded negative returns -∞, positive returns ∞
    /// - Normal values convert via binary scaling
    #[inline]
    pub fn to_f64(&self) -> f64 {
        if self.is_normal() {
            let i64: i64 = self.fraction.sa();
            let base = i64 as f64 / 2f64.powi(64 - 1);
            let exponent: i32 = self.exponent.saturate();

            base * 2f64.powi(exponent)
        } else {
            if self.is_undefined() {
                return f64::NAN;
            }
            if self.is_negligible() {
                return if self.fraction.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                return f64::INFINITY;
            }
            return if self.fraction.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
    }
}
fn _printey<T: std::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = std::mem::size_of::<T>() * 8;
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits - 1 && b % 8 == 7 {
            result.push(' ');
        }
        if b == bits / 2 - 1 {
            result.push(' '); // Extra space at center
        }
    }
    result
}
