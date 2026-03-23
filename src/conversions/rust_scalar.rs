use crate::constants::ScalarConstants;
use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar};
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

#[cfg(feature = "ieee")]
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + ExponentConstants
            + FullInt
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<f64> for Scalar<F, E>
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
{
    /// # Convert a Binary64 value to a Scalar
    ///
    /// Creates a Scalar from an f64 value, properly handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Normal conversion
    /// let s1 = ScalarF5E3::from(3.25);
    /// assert!(s1, 3.25);
    ///
    /// // Special values
    /// let infinity = ScalarF5E3::from(f64::INFINITY);
    /// assert!(infinity.is_infinite());
    ///
    /// let nan = ScalarF5E3::from(f64::NAN);
    /// assert!(nan.is_undefined());
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN is converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f64 values are properly scaled
    fn from(binary64: f64) -> Self {
        Self::from(&binary64)
    }
}

#[cfg(feature = "ieee")]
impl<
        F: Integer
            + FractionConstants
            + FullInt
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + ExponentConstants
            + FullInt
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > From<&mut f64> for Scalar<F, E>
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
{
    /// # Convert a Binary64 value to a Scalar
    ///
    /// Creates a Scalar from an f64 value, properly handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Normal conversion
    /// let s1 = ScalarF5E3::from(3.25);
    /// assert!(s1, 3.25);
    ///
    /// // Special values
    /// let infinity = ScalarF5E3::from(f64::INFINITY);
    /// assert!(infinity.is_infinite());
    ///
    /// let nan = ScalarF5E3::from(f64::NAN);
    /// assert!(nan.is_undefined());
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN is converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f64 values are properly scaled
    fn from(binary64: &mut f64) -> Self {
        Self::from(*binary64)
    }
}
#[cfg(feature = "ieee")]
impl<F: Integer + FractionConstants + FullInt, E: Integer + ExponentConstants + FullInt> From<&f64>
    for Scalar<F, E>
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
{
    fn from(binary64: &f64) -> Self {
        if binary64.is_nan() {
            return Scalar {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if binary64.is_infinite() {
            return Self::INFINITY;
        }
        if binary64 == &0. {
            return if binary64.is_sign_negative() {
                Self {
                    fraction: F::NEG_SMALL_FRACTION,
                    exponent: E::AMBIGUOUS_EXPONENT,
                }
            } else {
                Self::ZERO
            };
        }
        let bits = binary64.to_bits();
        let raw_exp = ((bits >> 52) & ((1 << 11).wrapping_sub(&1))) as i16;

        let mut fraction = if raw_exp == 0 {
            (bits & ((1 << 52).wrapping_sub(&1))) as i64
        } else {
            (bits & ((1 << 52).wrapping_sub(&1)) | (1 << 52)) as i64
        };
        if binary64.is_sign_negative() {
            fraction = fraction.wrapping_neg();
        }
        let mut intermediary = Scalar::<i64, i16>::new(fraction, raw_exp.wrapping_sub(1012));
        intermediary.normalize();
        let fraction = intermediary.fraction.sa();
        if E::EXPONENT_BITS == 8 {
            if intermediary.exponent > E::MAX_EXPONENT.as_() {
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if intermediary.exponent < E::MIN_EXPONENT.as_() {
                let fraction = fraction >> 1usize;
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
        let exponent = intermediary.exponent.as_();
        Self { fraction, exponent }
    }
}

#[cfg(feature = "ieee")]
impl<F: Integer + FractionConstants + FullInt, E: Integer + ExponentConstants + FullInt> From<f32>
    for Scalar<F, E>
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
{
    /// # Convert a Binary32 value to a Scalar
    ///
    /// Creates a Scalar from an f32 value, properly handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Normal conversion
    /// let s1 = ScalarF5E3::from(3.25f32);
    /// assert!(s1, 3.25);
    ///
    /// // Special values
    /// let infinity = ScalarF5E3::from(f32::INFINITY);
    /// assert!(infinity.is_infinite());
    ///
    /// let nan = ScalarF5E3::from(f32::NAN);
    /// assert!(nan.is_undefined());
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN is converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f32 values are properly scaled
    fn from(binary32: f32) -> Self {
        Self::from(&binary32)
    }
}

#[cfg(feature = "ieee")]
impl<F: Integer + FractionConstants + FullInt, E: Integer + ExponentConstants + FullInt>
    From<&mut f32> for Scalar<F, E>
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
{
    /// # Convert a Binary32 value to a Scalar
    ///
    /// Creates a Scalar from an f32 value, properly handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Normal conversion
    /// let s1 = ScalarF5E3::from(3.25f32);
    /// assert!(s1, 3.25);
    ///
    /// // Special values
    /// let infinity = ScalarF5E3::from(f32::INFINITY);
    /// assert!(infinity.is_infinite());
    ///
    /// let nan = ScalarF5E3::from(f32::NAN);
    /// assert!(nan.is_undefined());
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN is converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f32 values are properly scaled
    fn from(binary32: &mut f32) -> Self {
        Self::from(*binary32)
    }
}

#[cfg(feature = "ieee")]
impl<F: Integer + FractionConstants + FullInt, E: Integer + ExponentConstants + FullInt> From<&f32>
    for Scalar<F, E>
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
{
    /// # Convert a Binary32 value to a Scalar
    ///
    /// Creates a Scalar from an f32 value, properly handling IEEE-754 special values.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Normal conversion
    /// let s1 = ScalarF5E3::from(3.25f32);
    /// assert!(s1, 3.25);
    ///
    /// // Special values
    /// let infinity = ScalarF5E3::from(f32::INFINITY);
    /// assert!(infinity.is_infinite());
    ///
    /// let nan = ScalarF5E3::from(f32::NAN);
    /// assert!(nan.is_undefined());
    /// ```
    ///
    /// ## Special Cases
    ///
    /// - NaN is converted to a general undefined state
    /// - Infinities are coerced to infinity
    /// - Subnormal f32 values are properly scaled
    fn from(binary32: &f32) -> Self {
        if binary32.is_nan() {
            return Scalar {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if binary32.is_infinite() {
            return Self::INFINITY;
        }
        if binary32 == &0. {
            return if binary32.is_sign_negative() {
                Self {
                    fraction: F::NEG_SMALL_FRACTION,
                    exponent: E::AMBIGUOUS_EXPONENT,
                }
            } else {
                Self::ZERO
            };
        }
        let bits = binary32.to_bits();
        let raw_exp = ((bits >> 23) & ((1 << 8).wrapping_sub(&1))) as i16;

        let mut fraction = if raw_exp == 0 {
            (bits & ((1 << 23).wrapping_sub(&1))) as i32
        } else {
            (bits & ((1 << 23).wrapping_sub(&1)) | (1 << 23)) as i32
        };
        if binary32.is_sign_negative() {
            fraction = fraction.wrapping_neg();
        }
        let mut intermediary = Scalar::<i32, i16>::new(fraction, raw_exp.wrapping_sub(119));
        intermediary.normalize();
        let fraction = intermediary.fraction.sa();
        if E::EXPONENT_BITS == 8 {
            if intermediary.exponent > E::MAX_EXPONENT.as_() {
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if intermediary.exponent < E::MIN_EXPONENT.as_() {
                let fraction = fraction >> 1usize;
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
        let exponent = intermediary.exponent.as_();
        Self { fraction, exponent }
    }
}

macro_rules! impl_from_int {
    ($($i:ty),*) => {
        $(
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<$i> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: $i) -> Self {
                    if value == 0 {
                        return Self::ZERO;
                    }
                    let mut shift = value.leading_ones().max(value.leading_zeros()) as isize;
                    shift = (core::mem::size_of::<$i>() as isize).wrapping_mul(8).wrapping_sub(shift);
                    let exponent:E = shift.as_();
                    shift = (F::FRACTION_BITS as isize).wrapping_sub(shift).wrapping_sub(1);
                    let fraction: F = if shift < 0 {
                        (value >> shift.wrapping_neg()).as_()
                    } else {
                        let intermediate: F = value.as_();
                        intermediate << shift as usize
                    };
                    Self { fraction, exponent }
                }
            }
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<&mut $i> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: &mut $i) -> Self {
                    Self::from(*value)
                }
            }
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<&$i> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: &$i) -> Self {
                    Self::from(*value)
                }
            }
        )*
    }
}
impl_from_int!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_from_uint {
    ($($u:ty),*) => {
        $(
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<$u> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: $u) -> Self {
                    if value == 0 {
                        return Self::ZERO;
                    }
                    let mut shift = value.leading_zeros() as isize;
                    shift = (core::mem::size_of::<$u>() as isize).wrapping_mul(8).wrapping_sub(shift);
                    let exponent:E = shift.as_();
                    shift = (F::FRACTION_BITS as isize).wrapping_sub(shift).wrapping_sub(1);
                    let fraction: F = if shift < 0 {
                        (value >> shift.wrapping_neg()).as_()
                    } else {
                        let intermediate: F = value.as_();
                        intermediate << shift as usize
                    };
                    Self { fraction, exponent }
                }
            }
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<&mut $u> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: &mut $u) -> Self {
                    Self::from(*value)
                }
            }
            impl<F: Integer+FractionConstants+FullInt, E: Integer+ExponentConstants+FullInt> From<&$u> for Scalar<F, E>
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
            {
                /// # Convert Integers to Scalar
                ///
                /// The following implementations allow converting any Rust integer type to a Scalar.
                /// This includes:
                /// - Signed integers (i8, i16, i32, i64, i128, isize)
                /// - Unsigned integers (u8, u16, u32, u64, u128, usize)
                /// - References to these types
                ///
                /// ## Examples
                ///
                /// ```rust
                /// use spirix::{Scalar, ScalarF5E3};
                ///
                /// // Conversion from various integer types
                /// let from_i32 = ScalarF5E3::from(42);
                /// let from_i64 = ScalarF5E3::from(9223372036854775807i64);
                /// let from_u8 = ScalarF5E3::from(255u8);
                ///
                /// let neg = ScalarF5E3::from(-42);
                /// assert!(neg.is_negative());
                /// ```
                ///
                /// ## Conversion Process
                ///
                /// 0. Zero is converted directly to Scalar::ZERO
                /// 1. For non-zero values:
                ///    - The number of significant bits is determined
                ///    - The fraction is normalized accordingly
                ///    - The exponent is set
                fn from(value: &$u) -> Self {
                    Self::from(*value)
                }
            }
        )*
    }
}
impl_from_uint!(u8, u16, u32, u64, u128, usize);

impl Scalar<i16, i16> {
    /// Convert a normal (finite, non-zero, non-NaN) f32 literal to `Scalar<i16,i16>` at
    /// compile time.  Panics at compile time if called with NaN, infinity, or zero.
    ///
    /// Use this for compile-time constants — e.g. `const K: ScalarF4E4 = ScalarF4E4::from_f32(0.0031308)`.
    /// For runtime conversion of arbitrary values use `ScalarF4E4::from(v)`.
    #[inline(always)]
    pub const fn from_f32(v: f32) -> Self {
        // Decode IEEE 754 binary32 using pure integer ops (all const-stable).
        let bits = v.to_bits();
        let raw_exp = ((bits >> 23) & 0xFF) as i16;
        // fraction as i32 with implicit leading bit (subnormal: no leading 1)
        let frac_u = if raw_exp == 0 {
            bits & 0x7F_FFFF
        } else {
            (bits & 0x7F_FFFF) | 0x80_0000
        };
        // Apply sign: two's complement negate if negative
        let mut frac: i32 = frac_u as i32;
        if (bits >> 31) != 0 {
            frac = frac.wrapping_neg();
        }
        // Intermediary exponent: raw_exp - 119 brings binary32 bias (127) to Spirix normal (8)
        // Specifically: for a normalised f32 the value is frac * 2^(raw_exp - 127 - 23 + 16)
        //   = frac * 2^(raw_exp - 134 + 16) ... but Spirix normalises so MSB is at bit 1 (N1),
        //   meaning frac already has its MSB at bit 8 (of 32). That's raw_exp - 127 - 23 + (32-1) = raw_exp - 119.
        let mut exp: i16 = raw_exp.wrapping_sub(119);

        // Inline normalize for Scalar<i32, i16>:
        // FRACTION_BITS for i32 = 32, AMBIGUOUS_EXPONENT for i16 = i16::MIN
        let lo = frac.leading_ones();
        let lz = frac.leading_zeros();
        let shift = if lo > lz { lo } else { lz };
        if shift > 1 {
            let shift = shift as i32;
            let new_exp: i16;
            if shift == 32 {
                // fraction is all-zero or all-ones (only valid if negative = MIN_i32)
                if frac >= 0 {
                    // positive zero-fraction: vanished/undefined
                    return Scalar { fraction: i16::MIN >> 1, exponent: i16::MIN };
                }
                new_exp = exp.wrapping_sub((shift.wrapping_add(1)) as i16);
            } else {
                new_exp = exp.wrapping_sub(shift as i16);
            }
            if exp < 0 && new_exp >= 0 {
                exp = i16::MIN; // AMBIGUOUS_EXPONENT
                frac = frac << (shift.wrapping_sub(2) as u32);
            } else {
                exp = new_exp.wrapping_add(1);
                frac = frac << (shift.wrapping_sub(1) as u32);
            }
        }

        // Left-aligned cast i32 → i16: take the top 16 bits
        let fraction = (frac >> 16) as i16;
        Scalar { fraction, exponent: exp }
    }
}

#[cfg(test)]
mod tests_from_f32 {
    use crate::ScalarF4E4;

    #[test]
    fn from_f32_matches_runtime() {
        let cases: &[f32] = &[
            1.0, -1.0, 0.5, -0.5, 0.0031308, 12.92, 1.055, 0.055,
            255.0, 0.00390625, 3.14159265, 0.1, 100.0, -42.75,
        ];
        for &v in cases {
            let runtime = ScalarF4E4::from(v);
            let compile = ScalarF4E4::from_f32(v);
            assert_eq!(
                compile.fraction, runtime.fraction,
                "fraction mismatch for {v}: from_f32={} from={}", compile.fraction, runtime.fraction
            );
            assert_eq!(
                compile.exponent, runtime.exponent,
                "exponent mismatch for {v}: from_f32={} from={}", compile.exponent, runtime.exponent
            );
        }
    }

    #[test]
    fn from_f32_is_const() {
        const K: ScalarF4E4 = ScalarF4E4::from_f32(0.0031308);
        const ONE: ScalarF4E4 = ScalarF4E4::from_f32(1.0);
        assert_eq!(ONE, ScalarF4E4::ONE);
        let _ = K;
    }
}
