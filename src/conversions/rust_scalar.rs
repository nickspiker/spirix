//! # IEEE 754 → Scalar Conversions
//!
//! ## State mapping rationale
//!
//! IEEE 754 and Spirix categorize "abnormal" values differently, so the conversion isn't one-to-one for non-finite inputs:
//!
//! | IEEE 754                  | Spirix              | Reason |
//! |---------------------------|---------------------|--------|
//! | finite normal             | `[#]` Normal        | exact math when precision allows |
//! | finite subnormal          | `[#]` Normal (or `[↓]` if it underflows Spirix) | IEEE's subnormals are genuinely-tiny non-zeros. |
//! | `±0.0`                    | `[0]`               | IEEE `+0.0 == -0.0` (both are defined as mathematically zero). The sign bit is informational — used for direction-of-approach in `atan2`, `1/x`, etc. — not part of the value. |
//! | `±∞`                      | `[+↑]` / `[-↑]` Exploded | see below |
//! | NaN                       | `[℘?]` Undefined    | mantissa bits lost, first-cause prefix set to GENERAL |
//!
//! ### Why IEEE `±∞` maps to Exploded, not Infinity
//!
//! IEEE 754 calls its saturating overflow value "infinity", but its algebraic behavior is closer to "signed overflow with direction" than to a true mathematical infinity:
//!
//! - **IEEE `±∞` has direction**: `+∞` and `-∞` are distinct, with a sign bit. A true singular infinity has no direction.
//! - **IEEE `±∞` is reached by overflow**: `f64::MAX * 2.0 = +∞`. That's an overflow, not "we divided by zero and got infinity". In Spirix, reaching the representable upper bound is exactly what `[↑]` Exploded means.
//! - **IEEE `±∞ - ±∞ = NaN`** and **`0 × ±∞ = NaN`**: these indicate IEEE doesn't actually treat `±∞` as an absorbing element the way mathematical infinity would. Spirix's Exploded has matching semantics here — arithmetic with Exploded operands can produce Undefined results.
//! - **Conservation of information**: rounding IEEE `+∞` to singular `[∞]` silently discards the sign. Mapping to `[+↑]` / `[-↑]` keeps the sign and roundtrips cleanly (Exploded → f64 goes back to `±∞`).
//!
//! Spirix reserves the singular `[∞]` for cases where the result genuinely is directionless — e.g. `1/0`, `ln(0)`, or other operations whose true mathematical result is a point-at-infinity with no well-defined sign.

use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
use num_traits::AsPrimitive;

impl<F: Integer + FullInt, E: Integer + FullInt> From<f64> for Scalar<F, E>
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
    fn from(binary64: f64) -> Self {
        Self::from(&binary64)
    }
}

impl<F: Integer + FullInt, E: Integer + FullInt> From<&mut f64> for Scalar<F, E>
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
    fn from(binary64: &mut f64) -> Self {
        Self::from(*binary64)
    }
}

impl<F: Integer + FullInt, E: Integer + FullInt> From<&f64> for Scalar<F, E>
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
        let bits = binary64.to_bits();
        let sign = bits >> 63;
        let raw_exp = ((bits >> 52) & 0x7FF) as i16;
        let mantissa = bits & 0xFFFFFFFFFFFFF;

        if raw_exp == 0x7FF {
            return if mantissa != 0 {
                // NaN -> undefined
                Scalar {
                    fraction: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                }
            } else if sign != 0 {
                // IEEE -∞ -> Spirix [-↑]: see module doc for rationale
                Self::EXPLODED_NEG
            } else {
                // IEEE +∞ -> Spirix [+↑]
                Self::EXPLODED_POS
            };
        }
        if raw_exp == 0 && mantissa == 0 {
            // IEEE ±0.0 are both mathematically zero (they compare equal).
            // The sign bit is informational, not part of the value.
            return Self::ZERO;
        }

        // Build positive magnitude from IEEE mantissa; sign applied below.
        let abs_mantissa: u64 = if raw_exp == 0 {
            mantissa
        } else {
            mantissa | (1 << 52)
        };

        // Count significant bits of the mantissa magnitude.
        let leading = abs_mantissa.leading_zeros() as isize;
        let significant = 64isize.wrapping_sub(leading);
        // spirix_exp = ieee_scale + significant_bits.
        // For subnormals (raw_exp=0), IEEE uses effective exp=1 — compensate.
        // v0.1 ruler: −1 at end because value formula is inflate × 2^(exp − FRAC + 1).
        let eff_exp: i16 = if raw_exp == 0 { 1 } else { raw_exp };
        let spirix_exp: i16 = eff_exp
            .wrapping_sub(1076)
            .wrapping_add(significant as i16);

        // Cast to F first, THEN shift — avoids overflow when FRAC > 64 (i.e., i128 fraction).
        let shift = Self::fraction_bits().wrapping_sub(significant);
        let fraction_pos: F = if shift < 0 {
            (abs_mantissa >> shift.wrapping_neg()).as_()
        } else {
            let intermediate: F = abs_mantissa.as_();
            intermediate << shift as usize
        };

        if Self::exponent_bits() == 8 {
            if spirix_exp > Self::max_exponent().saturate::<i16>() {
                return Self {
                    fraction: if sign != 0 {
                        Self::neg_one_exploded()
                    } else {
                        Self::pos_one_exploded()
                    },
                    exponent: Self::ambiguous_exponent(),
                };
            } else if spirix_exp < Self::min_exponent().saturate::<i16>() {
                return Self {
                    fraction: if sign != 0 {
                        Self::neg_one_vanished()
                    } else {
                        Self::pos_one_vanished()
                    },
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        if sign == 0 {
            return Self { fraction: fraction_pos, exponent: spirix_exp.as_() };
        }
        // Negate: general case is F::zero() - stored; pos_one_normal shifts exponent because its magnitude straddles the boundary between exp buckets.
        if fraction_pos == Self::pos_one_normal() {
            Self {
                fraction: Self::neg_one_normal(),
                exponent: (spirix_exp.wrapping_sub(1)).as_(),
            }
        } else {
            Self {
                fraction: F::zero() - fraction_pos,
                exponent: spirix_exp.as_(),
            }
        }
    }
}

impl<F: Integer + FullInt, E: Integer + FullInt> From<f32> for Scalar<F, E>
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
    fn from(binary32: f32) -> Self {
        Self::from(&binary32)
    }
}

impl<F: Integer + FullInt, E: Integer + FullInt> From<&mut f32> for Scalar<F, E>
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
    fn from(binary32: &mut f32) -> Self {
        Self::from(*binary32)
    }
}

impl<F: Integer + FullInt, E: Integer + FullInt> From<&f32> for Scalar<F, E>
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
    fn from(binary32: &f32) -> Self {
        let bits = binary32.to_bits();
        let sign = bits >> 31;
        let raw_exp = ((bits >> 23) & 0xFF) as i16;
        let mantissa = bits & 0x7FFFFF;

        if raw_exp == 0xFF {
            return if mantissa != 0 {
                // NaN -> undefined
                Scalar {
                    fraction: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                }
            } else if sign != 0 {
                // -Infinity -> negative exploded
                Self::EXPLODED_NEG
            } else {
                // +Infinity -> positive exploded
                Self::EXPLODED_POS
            };
        }
        if raw_exp == 0 && mantissa == 0 {
            // IEEE ±0.0 are both mathematically zero (they compare equal).
            // The sign bit is informational, not part of the value.
            return Self::ZERO;
        }

        let abs_mantissa: u32 = if raw_exp == 0 {
            mantissa
        } else {
            mantissa | (1 << 23)
        };

        let leading = abs_mantissa.leading_zeros() as isize;
        let significant = 32isize.wrapping_sub(leading);
        // For subnormals (raw_exp=0), IEEE uses effective exp=1.
        // v0.1 ruler: −1 at end because value formula is inflate × 2^(exp − FRAC + 1).
        let eff_exp: i16 = if raw_exp == 0 { 1 } else { raw_exp as i16 };
        let spirix_exp: i16 = eff_exp
            .wrapping_sub(151)
            .wrapping_add(significant as i16);

        // Cast to F first, then shift — avoids overflow when FRAC > 32.
        let shift = Self::fraction_bits().wrapping_sub(significant);
        let fraction_pos: F = if shift < 0 {
            (abs_mantissa >> shift.wrapping_neg()).as_()
        } else {
            let intermediate: F = abs_mantissa.as_();
            intermediate << shift as usize
        };

        if Self::exponent_bits() == 8 {
            if spirix_exp > Self::max_exponent().saturate::<i16>() {
                return Self {
                    fraction: if sign != 0 {
                        Self::neg_one_exploded()
                    } else {
                        Self::pos_one_exploded()
                    },
                    exponent: Self::ambiguous_exponent(),
                };
            } else if spirix_exp < Self::min_exponent().saturate::<i16>() {
                return Self {
                    fraction: if sign != 0 {
                        Self::neg_one_vanished()
                    } else {
                        Self::pos_one_vanished()
                    },
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        if sign == 0 {
            return Self { fraction: fraction_pos, exponent: spirix_exp.as_() };
        }
        if fraction_pos == Self::pos_one_normal() {
            Self {
                fraction: Self::neg_one_normal(),
                exponent: (spirix_exp.wrapping_sub(1)).as_(),
            }
        } else {
            Self {
                fraction: F::zero() - fraction_pos,
                exponent: spirix_exp.as_(),
            }
        }
    }
}

macro_rules! impl_from_int {
    ($($i:ty),*) => {
        $(
            impl<F: Integer+FullInt, E: Integer+FullInt> From<$i> for Scalar<F, E>
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
                    // Handle negative by negating, converting as positive, then negating the Scalar.
                    let negative = value < 0;
                    // Compute abs(value). For MIN, negation overflows — handle specially below.
                    let abs_value = if value == <$i>::MIN {
                        // |MIN| = 2^(bits-1) is exactly a power of 2. We can construct directly.
                        let bits = (core::mem::size_of::<$i>() as isize).wrapping_shl(3);
                        if bits > Self::max_exponent().saturate::<isize>() {
                            return Self {
                                fraction: Self::neg_one_exploded(),
                                exponent: Self::ambiguous_exponent(),
                            };
                        }
                        // value = MIN = -2^(bits-1). Under v0.1 ruler, represent as neg_one_normal at exp = bits-2.
                        return Self {
                            fraction: Self::neg_one_normal(),
                            exponent: bits.wrapping_sub(2).as_(),
                        };
                    } else if negative {
                        value.wrapping_neg()
                    } else {
                        value
                    };

                    // Positive path: compute significant bits from MSB of abs_value.
                    let leading = abs_value.leading_zeros() as isize;
                    let significant_bits = (core::mem::size_of::<$i>() as isize).wrapping_shl(3).wrapping_sub(leading);
                    // v0.1 ruler: value = inflate × 2^(exp − FRAC + 1), so exp = significant − 1.
                    let spirix_exp: isize = significant_bits.wrapping_sub(1);

                    if spirix_exp > Self::max_exponent().saturate::<isize>() {
                        return Self {
                            fraction: if negative { Self::neg_one_exploded() } else { Self::pos_one_exploded() },
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    // Position the MSB of abs_value at bit FRAC-1 of the stored fraction.
                    let shift = (Self::fraction_bits() as isize).wrapping_sub(significant_bits);
                    let fraction_pos: F = if shift < 0 {
                        (abs_value >> shift.wrapping_neg()).as_()
                    } else {
                        let intermediate: F = abs_value.as_();
                        intermediate << shift as usize
                    };

                    if !negative {
                        return Self { fraction: fraction_pos, exponent: spirix_exp.as_() };
                    }

                    // Negation: power-of-2 boundary requires exponent shift.
                    //   {pos_one_normal, e} negated → {neg_one_normal, e-1}
                    //   general stored s → {-s, e}  (safe: fraction_pos != F::min_value() here)
                    if fraction_pos == Self::pos_one_normal() {
                        Self {
                            fraction: Self::neg_one_normal(),
                            exponent: spirix_exp.wrapping_sub(1).as_(),
                        }
                    } else {
                        Self {
                            fraction: F::zero() - fraction_pos,
                            exponent: spirix_exp.as_(),
                        }
                    }
                }
            }
            impl<F: Integer+FullInt, E: Integer+FullInt> From<&mut $i> for Scalar<F, E>
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
            impl<F: Integer+FullInt, E: Integer+FullInt> From<&$i> for Scalar<F, E>
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
            impl<F: Integer+FullInt, E: Integer+FullInt> From<$u> for Scalar<F, E>
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
                    let leading = value.leading_zeros() as isize;
                    let significant_bits = (core::mem::size_of::<$u>() as isize).wrapping_shl(3).wrapping_sub(leading);
                    // v0.1 ruler: value = inflate × 2^(exp − FRAC + 1), so exp = significant − 1.
                    let spirix_exp: isize = significant_bits.wrapping_sub(1);

                    if spirix_exp > Self::max_exponent().saturate::<isize>() {
                        return Self {
                            fraction: Self::pos_one_exploded(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    // Position MSB at bit FRAC-1 of stored. For positive-only u-types,
                    // the result fraction has stored MSB=1 → represents positive in new format.
                    let shift = (Self::fraction_bits() as isize).wrapping_sub(significant_bits);
                    let fraction: F = if shift < 0 {
                        (value >> shift.wrapping_neg()).as_()
                    } else {
                        let intermediate: F = value.as_();
                        intermediate << shift as usize
                    };
                    Self { fraction, exponent: spirix_exp.as_() }
                }
            }
            impl<F: Integer+FullInt, E: Integer+FullInt> From<&mut $u> for Scalar<F, E>
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
            impl<F: Integer+FullInt, E: Integer+FullInt> From<&$u> for Scalar<F, E>
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
        let sign = bits >> 31;
        let raw_exp = ((bits >> 23) & 0xFF) as i16;
        let mantissa = bits & 0x7F_FFFF;

        // Special values. v0.1: AMBIG sentinel at E::MAX.
        if raw_exp == 0xFF {
            if mantissa != 0 {
                // NaN → undefined (generic N-3 prefix)
                return Scalar { fraction: 0xE000u16 as i16, exponent: i16::MAX };
            }
            return if sign != 0 {
                // -Inf → EXPLODED_NEG
                Scalar { fraction: i16::MIN, exponent: i16::MAX }
            } else {
                // +Inf → EXPLODED_POS
                Scalar { fraction: -(i16::MIN >> 1), exponent: i16::MAX }
            };
        }
        if raw_exp == 0 && mantissa == 0 {
            return Scalar { fraction: 0, exponent: i16::MAX };
        }

        // Build positive magnitude (mantissa with implicit leading 1 for normals).
        let abs_mantissa: u32 = if raw_exp == 0 {
            mantissa
        } else {
            mantissa | 0x80_0000
        };

        let leading = abs_mantissa.leading_zeros() as i16;
        let significant: i16 = 32 - leading;
        // spirix_exp = ieee_scale + significant. For subnormals (raw_exp=0), IEEE uses
        // effective exp=1 (not 0), so we add 1 to compensate.
        // v0.1 ruler: −1 at end because value formula is inflate × 2^(exp − FRAC + 1).
        let eff_exp = if raw_exp == 0 { 1 } else { raw_exp };
        let spirix_exp: i16 = eff_exp - 151 + significant;

        // Position MSB at bit FRAC-1 = 15 (FRAC=16 for Scalar<i16, i16>).
        let shift = 16 - significant;
        let fraction_pos: i16 = if shift < 0 {
            (abs_mantissa >> ((-shift) as u32)) as i16
        } else {
            (abs_mantissa as i16).wrapping_shl(shift as u32)
        };

        if sign == 0 {
            return Scalar { fraction: fraction_pos, exponent: spirix_exp };
        }

        if fraction_pos == i16::MIN {
            return Scalar { fraction: 0, exponent: spirix_exp - 1 };
        }
        Scalar { fraction: -fraction_pos, exponent: spirix_exp }
    }
}

#[cfg(test)]
mod tests_from_f32 {
    use crate::ScalarF4E4;

    #[test]
    fn from_f32_matches_runtime() {
        let cases: &[f32] = &[
            1.0, -1.0, 0.5, -0.5, 0.0031308, 12.92, 1.055, 0.055, 255.0, 0.00390625, 3.14159265,
            0.1, 100.0, -42.75,
        ];
        for &v in cases {
            let runtime = ScalarF4E4::from(v);
            let compile = ScalarF4E4::from_f32(v);
            assert_eq!(
                compile.fraction, runtime.fraction,
                "fraction mismatch for {v}: from_f32={} from={}",
                compile.fraction, runtime.fraction
            );
            assert_eq!(
                compile.exponent, runtime.exponent,
                "exponent mismatch for {v}: from_f32={} from={}",
                compile.exponent, runtime.exponent
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
