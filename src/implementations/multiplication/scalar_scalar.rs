use crate::core::integer::{Inflate, FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + ExponentConstants
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
    /// Multiplies this Scalar by another Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between two Scalars, handling special cases according to mathematical principles.
    /// For normal values, this produces the expected mathematical product. Special states follow special rules to maintain mathematical continuity even when results exceed representable ranges.
    ///
    /// Multiplication process:
    /// 0. Checks for escaped values (undefined, exploded, vanished) and applies special case handling  
    /// 1. Uses wider integer types for fraction multiplication as product lands in the high half  
    /// ```txt
    ///    □□□□□□□ ■■■■■■■ = Multiplier
    ///    □□□□□□□ ■■■■■■■ = Multiplicand
    ///         ⤪⤪⤪⤪⤪⤪      Multiply!
    ///    ■■■■■■■ □□□□□□□ = Intermediate 2x-bit product space
    ///        ↘↘↘↘↘↘↘
    ///            ■■■■■■■ = High half kept, low bits discarded to floor
    /// 2. Calculates leading Zeros/Ones to determine normalization shift and exponent nudge  
    /// 3. Adds exponents and adjusts by normalization shift (exponent_result = self.exponent + other.exponent - shift)  
    /// 4. Handles special cases where exponent exceeds MAX_EXPONENT (explode) or falls below MIN_EXPONENT (vanish)  
    /// 5. For escaped values, preserves sign and phase information while following mathematical convention  
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // Multiplying finite Scalars
    /// let eight = Scalar::<i64, i16>::from(8);
    /// let eigth = ScalarF6E4::ONE / 8;
    /// assert!(eight * eigth == 1); // Restores unity thru multiplicative inverse
    ///
    /// // Multiplying near boundaries
    /// let large = ScalarF6E4::from(64) * ScalarF6E4::MAX_NEG;
    /// let small = ScalarF6E4::from(1) / large;
    /// assert!(large * small == 1);
    ///
    /// // Multiplication preserves sign according to mathematical rule
    /// let negative = ScalarF6E4::from(-1.5);
    /// assert!((eight * negative).is_negative()); // Positive × Negative = Negative
    /// assert!((negative * negative).is_positive()); // Negative × Negative = Positive
    ///
    /// // Vanished values maintain sign thru multiplication
    /// let tiny = ScalarF6E4::MIN_POS / ScalarF6E4::from(11);
    /// assert!(tiny.vanished());
    /// let neg_tiny = tiny * ScalarF6E4::NEG_ONE;
    /// assert!(neg_tiny.vanished() && neg_tiny.is_negative());
    ///
    /// // Exploded values interact consistently with finite values
    /// let huge = ScalarF6E4::MAX * 42;
    /// assert!(huge.exploded());
    /// assert!((1 / huge).vanished()); // Inverse of exploded is vanished
    /// assert!(((-1) / huge).is_negative()); // Signs are propogated following multiplication rule
    /// assert!((huge * ScalarF6E4::NEG_ONE).exploded());
    /// assert!((huge * -1).is_negative());
    ///
    /// // Multiplying by zero always produces zero, even with escaped values
    /// assert!((huge * 0).is_zero());
    /// assert!((tiny * 0).is_zero());
    ///
    /// // Vanished × Exploded yields an undefined state (magnitude indeterminate)
    /// let undefined_product = tiny * huge;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn scalar_multiply_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            } else if other.is_undefined() {
                return *other;
            } else if self.is_infinite() && other.is_zero() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.is_zero() && other.is_infinite() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.is_infinite() || other.is_infinite() {
                return Self::INFINITY;
            } else if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            } else if self.exploded() && other.vanished() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.vanished() && other.exploded() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else {
                let n_level: isize = if self.exploded() || other.exploded() {
                    -1
                } else {
                    -2
                };
                let fraction = match F::FRACTION_BITS {
                    8 => {
                        let multiplier: i16 = self.fraction.as_();
                        let multiplicand: i16 = other.fraction.as_();
                        let product_wide = multiplier.wrapping_mul(multiplicand);

                        let leading = product_wide
                            .leading_ones()
                            .max(product_wide.leading_zeros())
                            as isize;

                        let shift = leading.wrapping_add(n_level);
                        let normalized_wide = product_wide << shift;
                        (normalized_wide >> F::FRACTION_BITS).as_()
                    }
                    16 => {
                        let multiplier: i32 = self.fraction.as_();
                        let multiplicand: i32 = other.fraction.as_();
                        let product_wide = multiplier.wrapping_mul(multiplicand);

                        let leading = product_wide
                            .leading_ones()
                            .max(product_wide.leading_zeros())
                            as isize;

                        let shift = leading.wrapping_add(n_level);
                        let normalized_wide = product_wide << shift;
                        (normalized_wide >> F::FRACTION_BITS).as_()
                    }
                    32 => {
                        let multiplier: i64 = self.fraction.as_();
                        let multiplicand: i64 = other.fraction.as_();
                        let product_wide = multiplier.wrapping_mul(multiplicand);

                        let leading = product_wide
                            .leading_ones()
                            .max(product_wide.leading_zeros())
                            as isize;

                        let shift = leading.wrapping_add(n_level);
                        let normalized_wide = product_wide << shift;
                        (normalized_wide >> F::FRACTION_BITS).as_()
                    }
                    64 => {
                        let multiplier: i128 = self.fraction.as_();
                        let multiplicand: i128 = other.fraction.as_();
                        let product_wide = multiplier.wrapping_mul(multiplicand);

                        let leading = product_wide
                            .leading_ones()
                            .max(product_wide.leading_zeros())
                            as isize;

                        let shift = leading.wrapping_add(n_level);
                        let normalized_wide = product_wide << shift;
                        (normalized_wide >> F::FRACTION_BITS).as_()
                    }
                    128 => {
                        let multiplier: i128 = self.fraction.as_();
                        let multiplicand: i128 = other.fraction.as_();
                        let multiplier: I256 = multiplier.into();
                        let multiplicand: I256 = multiplicand.into();
                        let product_wide: I256 = multiplier.wrapping_mul(multiplicand);

                        let leading = product_wide
                            .leading_ones()
                            .max(product_wide.leading_zeros())
                            as isize;

                        let shift = leading.wrapping_add(n_level);
                        let normalized_wide = product_wide << shift;
                        (normalized_wide >> F::FRACTION_BITS).as_i128().as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };

                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        } else {
            let fraction;
            let expo_adjust: isize;
            match F::FRACTION_BITS {
                8 => {
                    let multiplier: i16 = self.fraction.as_();
                    let multiplicand: i16 = other.fraction.as_();
                    let product_wide = multiplier.wrapping_mul(multiplicand);
                    if product_wide == 0 {
                        return Self::ZERO;
                    }
                    expo_adjust = (product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize)
                        .wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide = product_wide << shift;
                    fraction = (normalized_wide >> F::FRACTION_BITS).as_();
                }
                16 => {
                    let multiplier: i32 = self.fraction.as_();
                    let multiplicand: i32 = other.fraction.as_();
                    let product_wide = multiplier.wrapping_mul(multiplicand);
                    if product_wide == 0 {
                        return Self::ZERO;
                    }
                    expo_adjust = (product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize)
                        .wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide = product_wide << shift;
                    fraction = (normalized_wide >> F::FRACTION_BITS).as_();
                }
                32 => {
                    let multiplier: i64 = self.fraction.as_();
                    let multiplicand: i64 = other.fraction.as_();
                    let product_wide = multiplier.wrapping_mul(multiplicand);
                    if product_wide == 0 {
                        return Self::ZERO;
                    }
                    expo_adjust = (product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize)
                        .wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide = product_wide << shift;
                    fraction = (normalized_wide >> F::FRACTION_BITS).as_();
                }
                64 => {
                    let multiplier: i128 = self.fraction.as_();
                    let multiplicand: i128 = other.fraction.as_();
                    let product_wide = multiplier.wrapping_mul(multiplicand);
                    if product_wide == 0 {
                        return Self::ZERO;
                    }
                    expo_adjust = (product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize)
                        .wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide = product_wide << shift;
                    fraction = (normalized_wide >> F::FRACTION_BITS).as_();
                }
                128 => {
                    let multiplier: i128 = self.fraction.as_();
                    let multiplicand: i128 = other.fraction.as_();
                    let multiplier: I256 = multiplier.into();
                    let multiplicand: I256 = multiplicand.into();
                    let product_wide: I256 = multiplier.wrapping_mul(multiplicand);
                    if product_wide == 0.into() {
                        return Self::ZERO;
                    }
                    expo_adjust = (product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize)
                        .wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide = product_wide << shift;
                    fraction = (normalized_wide >> F::FRACTION_BITS).as_i128().as_();
                }
                _ => {
                    return Self {
                        fraction: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }

            match E::EXPONENT_BITS {
                8 => {
                    let self_exponent: i16 = self.exponent.as_();
                    let other_exponent: i16 = other.exponent.as_();
                    let upcast_exponent: i16 = self_exponent
                        .wrapping_add(other_exponent)
                        .wrapping_sub(expo_adjust as i16);

                    if upcast_exponent > E::MAX_EXPONENT.as_() {
                        return Scalar {
                            fraction,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Scalar {
                            fraction: fraction >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Scalar {
                            fraction,
                            exponent: upcast_exponent.as_(),
                        };
                    }
                }
                16 => {
                    let self_exponent: i32 = self.exponent.as_();
                    let other_exponent: i32 = other.exponent.as_();
                    let upcast_exponent: i32 = self_exponent
                        .wrapping_add(other_exponent)
                        .wrapping_sub(expo_adjust as i32);

                    if upcast_exponent > E::MAX_EXPONENT.as_() {
                        return Scalar {
                            fraction,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Scalar {
                            fraction: fraction >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Scalar {
                            fraction,
                            exponent: upcast_exponent.as_(),
                        };
                    }
                }
                32 => {
                    let self_exponent: i64 = self.exponent.as_();
                    let other_exponent: i64 = other.exponent.as_();
                    let upcast_exponent: i64 = self_exponent
                        .wrapping_add(other_exponent)
                        .wrapping_sub(expo_adjust as i64);

                    if upcast_exponent > E::MAX_EXPONENT.as_() {
                        return Scalar {
                            fraction,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Scalar {
                            fraction: fraction >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Scalar {
                            fraction,
                            exponent: upcast_exponent.as_(),
                        };
                    }
                }
                64 => {
                    let self_exponent: i128 = self.exponent.as_();
                    let other_exponent: i128 = other.exponent.as_();
                    let upcast_exponent: i128 = self_exponent
                        .wrapping_add(other_exponent)
                        .wrapping_sub(expo_adjust as i128);

                    if upcast_exponent > E::MAX_EXPONENT.as_() {
                        return Scalar {
                            fraction,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Scalar {
                            fraction: fraction >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Scalar {
                            fraction,
                            exponent: upcast_exponent.as_(),
                        };
                    }
                }
                128 => {
                    let self_exponent: I256 = self.exponent.into();
                    let other_exponent: I256 = other.exponent.into();
                    let e: I256 = (expo_adjust as i128).into();
                    let upcast_exponent: I256 =
                        self_exponent.wrapping_add(other_exponent).wrapping_sub(e);

                    if upcast_exponent > E::MAX_EXPONENT.into() {
                        return Scalar {
                            fraction,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.into() {
                        return Scalar {
                            fraction: fraction >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Scalar {
                            fraction,
                            exponent: upcast_exponent.as_i128().as_(),
                        };
                    }
                }
                _ => {
                    return Scalar {
                        fraction: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        }
    }
}
