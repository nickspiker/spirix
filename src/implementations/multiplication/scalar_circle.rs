use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
#[allow(private_bounds)]
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
    > Scalar<F, E>
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
    /// Multiplies a Scalar by a Circle
    ///
    /// # Description
    ///
    /// Performs multiplication between a Scalar and a Circle, scaling both real and imaginary
    /// components of the Circle by the Scalar value while preserving the Circle's orientation.
    /// This follows the standard scalar-complex multiplication formula: s·(a+b·i) = (s·a) + (s·b)·i.
    ///
    /// The operation scales the magnitude of the complex number while preserving its angle in the
    /// complex plane, equivalent to multiplying the magnitude by the absolute value of the Scalar
    /// and adjusting the angle if the Scalar is negative (rotation by π radians).
    ///
    /// # Special Cases
    ///
    /// - If either value is undefined, returns the first undefined state encountered
    /// - If either value is zero, returns zero (multiplicative annihilation)
    /// - If a positive exploded Scalar multiplies a positive exploded Circle, returns an exploded Circle
    /// - If a vanished Scalar multiplies an exploded Circle (or vice versa), returns an undefined state
    ///   due to the magnitude being indeterminate
    ///
    /// # Returns
    ///
    /// - For normal values: A new Circle with scaled components
    /// - For special cases:
    ///   - `[℘ ]` → `[℘ ]` First undefined state encountered
    ///   - `[0]` × `[#]` or `[#]` × `[0]` → `[0]` Zero (multiplicative annihilation)
    ///   - `[↑]` × `[↓]` or `[↓]` × `[↑]` → `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    ///   - `[↑]` × `[#]` or `[#]` × `[↑]` → `[↑]` Exploded Circle with orientation following the multiplication rule
    ///   - `[↓]` × `[#]` or `[#]` × `[↓]` → `[↓]` Vanished Circle with orientation following the multiplication rule
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, Circle, ScalarF5E3, CircleF5E3};
    ///
    /// // Basic multiplication - scales both components
    /// let s = ScalarF5E3::from(2);
    /// let z = CircleF5E3::from((3, 4));
    /// let result = s * z;
    /// assert!(result.r() == 6);
    /// assert!(result.i() == 8);
    ///
    /// // Multiplying by negative Scalar negates the Circle
    /// let neg = ScalarF5E3::from(-1);
    /// let negated = neg * z;
    /// assert!(negated.r() == -3);
    /// assert!(negated.i() == -4);
    ///
    /// // Multiplying by Zero produces Zero
    /// let zero = ScalarF5E3::ZERO;
    /// assert!((zero * z).is_zero());
    ///
    /// // Multiplying with special states follows predictable rules
    /// let tiny = ScalarF5E3::MIN_POS / 10;
    /// let tiny_result = tiny * z;
    /// assert!(tiny_result.vanished());
    /// assert!(tiny_result.is_positive());
    ///
    /// // Fractional scaling preserves orientation
    /// let unit_circle = CircleF5E3::from((0.6, 0.8)); // magnitude = 1
    /// let half = ScalarF5E3::from(0.5);
    /// let half_circle = half * unit_circle;
    /// assert!(half_circle.magnitude() == 0.5);
    /// // Direction remains the same
    /// assert!(half_circle.r() / half_circle.magnitude() == unit_circle.r());
    /// assert!(half_circle.i() / half_circle.magnitude() == unit_circle.i());
    /// ```
    pub(crate) fn scalar_multiply_circle(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            } else if other.is_undefined() {
                return *other;
            } else if self.is_infinite() && other.is_zero() {
                return Circle {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_zero() && other.is_infinite() {
                return Circle {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_zero() || other.is_zero() {
                return Circle::<F, E>::ZERO;
            } else if self.exploded() && other.vanished() {
                return Circle {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.vanished() && other.exploded() {
                return Circle {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else {
                let n_level: isize = if self.exploded() || other.exploded() {
                    -1
                } else {
                    -2
                };
                let (real, imaginary) = match Self::fraction_bits() {
                    8 => {
                        let multiplier_r: i16 = other.real.as_();
                        let multiplier_i: i16 = other.imaginary.as_();
                        let multiplicand: i16 = self.fraction.as_();
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
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    16 => {
                        let multiplier_r: i32 = other.real.as_();
                        let multiplier_i: i32 = other.imaginary.as_();
                        let multiplicand: i32 = self.fraction.as_();
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
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    32 => {
                        let multiplier_r: i64 = other.real.as_();
                        let multiplier_i: i64 = other.imaginary.as_();
                        let multiplicand: i64 = self.fraction.as_();
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
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    64 => {
                        let multiplier_r: i128 = other.real.as_();
                        let multiplier_i: i128 = other.imaginary.as_();
                        let multiplicand: i128 = self.fraction.as_();
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
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    128 => {
                        let multiplier_r: I256 = other.real.into();
                        let multiplier_i: I256 = other.imaginary.into();
                        let multiplicand: I256 = self.fraction.into();
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
                            (normalized_wide_r >> Self::fraction_bits()).as_i128().as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_i128().as_(),
                        )
                    }
                    _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
                };

                return Circle::<F, E> {
                    real,
                    imaginary,
                    exponent: Self::ambiguous_exponent(),
                };
            }
        } else {
            let real;
            let imaginary;
            let expo_adjust: isize;
            match Self::fraction_bits() {
                8 => {
                    let multiplier_r: i16 = other.real.as_();
                    let multiplier_i: i16 = other.imaginary.as_();
                    let multiplicand: i16 = self.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Circle::<F, E>::ZERO;
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
                    real = (normalized_wide_r >> Self::fraction_bits()).as_();
                    imaginary = (normalized_wide_i >> Self::fraction_bits()).as_();
                }
                16 => {
                    let multiplier_r: i32 = other.real.as_();
                    let multiplier_i: i32 = other.imaginary.as_();
                    let multiplicand: i32 = self.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Circle::<F, E>::ZERO;
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
                    real = (normalized_wide_r >> Self::fraction_bits()).as_();
                    imaginary = (normalized_wide_i >> Self::fraction_bits()).as_();
                }
                32 => {
                    let multiplier_r: i64 = other.real.as_();
                    let multiplier_i: i64 = other.imaginary.as_();
                    let multiplicand: i64 = self.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Circle::<F, E>::ZERO;
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
                    real = (normalized_wide_r >> Self::fraction_bits()).as_();
                    imaginary = (normalized_wide_i >> Self::fraction_bits()).as_();
                }
                64 => {
                    let multiplier_r: i128 = other.real.as_();
                    let multiplier_i: i128 = other.imaginary.as_();
                    let multiplicand: i128 = self.fraction.as_();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0 && product_wide_i == 0 {
                        return Circle::<F, E>::ZERO;
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
                    real = (normalized_wide_r >> Self::fraction_bits()).as_();
                    imaginary = (normalized_wide_i >> Self::fraction_bits()).as_();
                }
                128 => {
                    let multiplier_r: I256 = other.real.into();
                    let multiplier_i: I256 = other.imaginary.into();
                    let multiplicand: I256 = self.fraction.into();
                    let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                    let product_wide_i = multiplier_i.wrapping_mul(multiplicand);
                    if product_wide_r == 0.into() && product_wide_i == 0.into() {
                        return Circle::<F, E>::ZERO;
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
                    real = (normalized_wide_r >> Self::fraction_bits()).as_i128().as_();
                    imaginary = (normalized_wide_i >> Self::fraction_bits()).as_i128().as_();
                }
                _ => {
                    return Circle::<F, E> {
                        real: GENERAL.prefix.sa(),
                        imaginary: GENERAL.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
            }

            match Self::exponent_bits() {
                8 => {
                    let self_exponent: i16 = self.exponent.as_();
                    let other_exponent: i16 = other.exponent.as_();
                    let upcast_exponent: i16 = self_exponent
                        .wrapping_add(other_exponent)
                        .wrapping_sub(expo_adjust as i16);

                    if upcast_exponent > Self::max_exponent().as_() {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else if upcast_exponent < Self::min_exponent().as_() {
                        return Circle::<F, E> {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else {
                        return Circle::<F, E> {
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

                    if upcast_exponent > Self::max_exponent().as_() {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else if upcast_exponent < Self::min_exponent().as_() {
                        return Circle::<F, E> {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else {
                        return Circle::<F, E> {
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

                    if upcast_exponent > Self::max_exponent().as_() {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else if upcast_exponent < Self::min_exponent().as_() {
                        return Circle::<F, E> {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else {
                        return Circle::<F, E> {
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

                    if upcast_exponent > Self::max_exponent().as_() {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else if upcast_exponent < Self::min_exponent().as_() {
                        return Circle::<F, E> {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else {
                        return Circle::<F, E> {
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

                    if upcast_exponent > Self::max_exponent().into() {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else if upcast_exponent < Self::min_exponent().into() {
                        return Circle::<F, E> {
                            real: real >> 1isize,
                            imaginary: imaginary >> 1isize,
                            exponent: Self::ambiguous_exponent(),
                        };
                    } else {
                        return Circle::<F, E> {
                            real,
                            imaginary,
                            exponent: upcast_exponent.as_i128().as_(),
                        };
                    }
                }
                _ => {
                    return Circle {
                        real: GENERAL.prefix.sa(),
                        imaginary: GENERAL.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
            }
        }
    }
}
