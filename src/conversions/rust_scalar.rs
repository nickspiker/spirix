use crate::constants::ScalarConstants;
use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar};
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

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
                    shift = (std::mem::size_of::<$i>() as isize).wrapping_mul(8).wrapping_sub(shift);
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
                    shift = (std::mem::size_of::<$u>() as isize).wrapping_mul(8).wrapping_sub(shift);
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
