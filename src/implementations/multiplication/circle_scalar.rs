use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use core::ops::*;
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
    > Circle<F, E>
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
    /// Multiplies this Circle by a Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between a Circle and a Scalar, scaling both real and imaginary components by the Scalar value while preserving the orientation of the Circle.
    /// For normal values, this multiplies each component of the Circle by the Scalar, following standard complex-real multiplication formula: (a+b·i)·c = (a·c) + (b·c)·i.
    ///
    /// Multiplication process:
    /// 0. Checks for and handle undefined states
    /// 1. Uses wider integer types for component multiplication:
    ///    ```text
    ///    a   →         ■■■■■■■ = self.real
    ///    c   →         ■■■■■■■ = scalar.fraction
    ///                  ⤪⤪⤪⤪⤪⤪ Multiply!
    ///    a·c → ■■■■■■■ □□□□□□□ = Intermediate product for real part
    ///              ↘↘↘↘↘↘↘
    ///    Real →        ■■■■■■■ = High bits kept for real component
    ///    
    ///    b   →         ■■■■■■■ = self.imaginary
    ///    c   →         ■■■■■■■ = scalar.fraction
    ///                  ⤪⤪⤪⤪⤪⤪ Multiply!
    ///    b·c → ■■■■■■■ □□□□□□□ = Intermediate product for imaginary part
    ///              ↘↘↘↘↘↘↘
    ///    Imaginary →   ■■■■■■■ = High bits kept for imaginary component
    ///    ```
    /// 2. Calculates leading Zeros/Ones for both components to determine normalization shift
    /// 3. If normal, adds exponents and adjusts by normalization shift (exponent_result = self.exponent + scalar.exponent - shift)
    /// 4. Checks for overflow/underflow and returns appropriate escaped values if necessary
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with orientation following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with orientation following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with orientation following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with orientation following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, Scalar, CircleF6E4, ScalarF6E4};
    ///
    /// // Basic multiplication - scales both components
    /// let z = Circle::<i64, i16>::from((3, 4));
    /// let s = ScalarF6E4::from(2);
    /// let result = z * s;
    /// assert!(result.r() == 6);
    /// assert!(result.i() == 8);
    ///
    /// // Multiplying by negative Scalar negates the Circle
    /// let neg = ScalarF6E4::from(-1);
    /// let negated = z * neg;
    /// assert!(negated.r() == -3);
    /// assert!(negated.i() == -4);
    ///
    /// // Multiplying by Zero produces Zero
    /// let Zero = CircleF6E4::ZERO;
    /// let scale = ScalarF6E4::from(10);
    /// assert!((Zero * scale).is_zero());
    /// assert!((z * ScalarF6E4::ZERO).is_zero());
    ///
    /// // Fractional scaling preserves orientation
    /// let unit_circle = CircleF6E4::from((0.6, 0.8)); // magnitude = 1
    /// let half = ScalarF6E4::from(0.5);
    /// let half_circle = unit_circle * half;
    /// assert!(half_circle.magnitude() == 0.5);
    /// // Direction remains the same
    /// assert!(half_circle.r() / half_circle.magnitude() == unit_circle.r());
    /// assert!(half_circle.i() / half_circle.magnitude() == unit_circle.i());
    ///
    /// // Vanished values maintain orientation through multiplication
    /// let tiny = CircleF6E4::MIN_POS_REAL / 10;
    /// let tiny_circle = CircleF6E4::from((tiny.r(), tiny.r()));
    /// assert!(tiny_circle.vanished());
    /// let scaled_tiny = tiny_circle * ScalarF6E4::from(3);
    /// assert!(scaled_tiny.vanished()); // Still vanished, preserves orientation
    ///
    /// // Exploded values interact consistently with Scalars
    /// let huge = CircleF6E4::MAX_REAL_CIRCLE * 10;
    /// assert!(huge.exploded());
    /// let neg_huge = huge * ScalarF6E4::NEG_ONE;
    /// assert!(neg_huge.exploded());
    /// assert!(neg_huge.r() < 0); // Orientation is flipped
    ///
    /// // Vanished × Exploded yields an undefined state
    /// let undefined_product = tiny_circle * ScalarF6E4::from(1) / 0;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn circle_multiply_scalar(&self, other: &Scalar<F, E>) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            } else if other.is_undefined() {
                return Self {
                    real: other.fraction,
                    imaginary: other.fraction,
                    exponent: other.exponent,
                };
            } else if self.is_infinite() && other.is_zero() {
                return Self {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.is_zero() && other.is_infinite() {
                return Self {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            } else if self.exploded() && other.vanished() {
                return Self {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else if self.vanished() && other.exploded() {
                return Self {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else {
                let n_level: isize = if self.exploded() || other.exploded() {
                    -1
                } else {
                    -2
                };
                let (real, imaginary) = match F::FRACTION_BITS {
                    8 => {
                        let multiplier_r: i16 = self.real.as_();
                        let multiplier_i: i16 = self.imaginary.as_();
                        let multiplicand: i16 = other.fraction.as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> F::FRACTION_BITS).as_(),
                            (normalized_wide_i >> F::FRACTION_BITS).as_(),
                        )
                    }
                    16 => {
                        let multiplier_r: i32 = self.real.as_();
                        let multiplier_i: i32 = self.imaginary.as_();
                        let multiplicand: i32 = other.fraction.as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> F::FRACTION_BITS).as_(),
                            (normalized_wide_i >> F::FRACTION_BITS).as_(),
                        )
                    }
                    32 => {
                        let multiplier_r: i64 = self.real.as_();
                        let multiplier_i: i64 = self.imaginary.as_();
                        let multiplicand: i64 = other.fraction.as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> F::FRACTION_BITS).as_(),
                            (normalized_wide_i >> F::FRACTION_BITS).as_(),
                        )
                    }
                    64 => {
                        let multiplier_r: i128 = self.real.as_();
                        let multiplier_i: i128 = self.imaginary.as_();
                        let multiplicand: i128 = other.fraction.as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> F::FRACTION_BITS).as_(),
                            (normalized_wide_i >> F::FRACTION_BITS).as_(),
                        )
                    }
                    128 => {
                        let multiplier_r: I256 = self.real.into();
                        let multiplier_i: I256 = self.imaginary.into();
                        let multiplicand: I256 = other.fraction.into();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> F::FRACTION_BITS).as_i128().as_(),
                            (normalized_wide_i >> F::FRACTION_BITS).as_i128().as_(),
                        )
                    }
                    _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
                };

                return Self {
                    real,
                    imaginary,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        } else {
            let real;
            let imaginary;
            let expo_adjust: isize;
            match F::FRACTION_BITS {
                8 => {
                    let multiplier_r: i16 = self.real.as_();
                    let multiplier_i: i16 = self.imaginary.as_();
                    let multiplicand: i16 = other.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Self::ZERO;
                    }
                    let leading_r = product_wide_r
                        .leading_ones()
                        .max(product_wide_r.leading_zeros());
                    let leading_i = product_wide_i
                        .leading_ones()
                        .max(product_wide_i.leading_zeros());
                    expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide_r = product_wide_r << shift;
                    let normalized_wide_i = product_wide_i << shift;
                    real = (normalized_wide_r >> F::FRACTION_BITS).as_();
                    imaginary = (normalized_wide_i >> F::FRACTION_BITS).as_();
                }
                16 => {
                    let multiplier_r: i32 = self.real.as_();
                    let multiplier_i: i32 = self.imaginary.as_();
                    let multiplicand: i32 = other.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Self::ZERO;
                    }
                    let leading_r = product_wide_r
                        .leading_ones()
                        .max(product_wide_r.leading_zeros());
                    let leading_i = product_wide_i
                        .leading_ones()
                        .max(product_wide_i.leading_zeros());
                    expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide_r = product_wide_r << shift;
                    let normalized_wide_i = product_wide_i << shift;
                    real = (normalized_wide_r >> F::FRACTION_BITS).as_();
                    imaginary = (normalized_wide_i >> F::FRACTION_BITS).as_();
                }
                32 => {
                    let multiplier_r: i64 = self.real.as_();
                    let multiplier_i: i64 = self.imaginary.as_();
                    let multiplicand: i64 = other.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Self::ZERO;
                    }
                    let leading_r = product_wide_r
                        .leading_ones()
                        .max(product_wide_r.leading_zeros());
                    let leading_i = product_wide_i
                        .leading_ones()
                        .max(product_wide_i.leading_zeros());
                    expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide_r = product_wide_r << shift;
                    let normalized_wide_i = product_wide_i << shift;
                    real = (normalized_wide_r >> F::FRACTION_BITS).as_();
                    imaginary = (normalized_wide_i >> F::FRACTION_BITS).as_();
                }
                64 => {
                    let multiplier_r: i128 = self.real.as_();
                    let multiplier_i: i128 = self.imaginary.as_();
                    let multiplicand: i128 = other.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Self::ZERO;
                    }
                    let leading_r = product_wide_r
                        .leading_ones()
                        .max(product_wide_r.leading_zeros());
                    let leading_i = product_wide_i
                        .leading_ones()
                        .max(product_wide_i.leading_zeros());
                    expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide_r = product_wide_r << shift;
                    let normalized_wide_i = product_wide_i << shift;
                    real = (normalized_wide_r >> F::FRACTION_BITS).as_();
                    imaginary = (normalized_wide_i >> F::FRACTION_BITS).as_();
                }
                128 => {
                    let multiplier_r: I256 = self.real.into();
                    let multiplier_i: I256 = self.imaginary.into();
                    let multiplicand: I256 = other.fraction.into();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0.into() && product_wide_i == 0.into() {
                        return Self::ZERO;
                    }
                    let leading_r = product_wide_r
                        .leading_ones()
                        .max(product_wide_r.leading_zeros());
                    let leading_i = product_wide_i
                        .leading_ones()
                        .max(product_wide_i.leading_zeros());
                    expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(2);
                    let shift = expo_adjust.wrapping_add(1);
                    let normalized_wide_r = product_wide_r << shift;
                    let normalized_wide_i = product_wide_i << shift;
                    real = (normalized_wide_r >> F::FRACTION_BITS).as_i128().as_();
                    imaginary = (normalized_wide_i >> F::FRACTION_BITS).as_i128().as_();
                }
                _ => {
                    return Self {
                        real: GENERAL.prefix.sa(),
                        imaginary: GENERAL.prefix.sa(),
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
                        return Self {
                            real,
                            imaginary,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Self {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Self {
                            real,
                            imaginary,
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
                        return Self {
                            real,
                            imaginary,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Self {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Self {
                            real,
                            imaginary,
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
                        return Self {
                            real,
                            imaginary,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Self {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Self {
                            real,
                            imaginary,
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
                        return Self {
                            real,
                            imaginary,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                        return Self {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Self {
                            real,
                            imaginary,
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
                        return Self {
                            real,
                            imaginary,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else if upcast_exponent < E::MIN_EXPONENT.into() {
                        return Self {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    } else {
                        return Self {
                            real,
                            imaginary,
                            exponent: upcast_exponent.as_i128().as_(),
                        };
                    }
                }
                _ => {
                    return Self {
                        real: GENERAL.prefix.sa(),
                        imaginary: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        }
    }
}
