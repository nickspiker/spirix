use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
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
    pub fn reciprocal(&self) -> Self {
        Self::ONE / self
    }

    pub(crate) fn scalar_divide_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if other.is_zero() {
                if self.is_zero() {
                    return Self {
                        fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Self {
                        fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_infinite() {
                return Self::ZERO;
            }
            if self.exploded() && other.exploded() {
                return Self {
                    fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Escaped / escaped or escaped / normal: signed div is safe (escaped fractions are small)
            let self_wide = self.fraction.inflate(self.is_normal());
            let other_wide = other.fraction.inflate(other.is_normal());
            let quotient = self_wide.w_shl(Self::fraction_bits()).w_div(other_wide);
            let result_exploded = self.exploded() || other.vanished();
            let result_vanished = self.vanished() || other.exploded();
            let leading = quotient.leading_same();
            let n: isize = if result_exploded {
                1
            } else if result_vanished {
                2
            } else {
                1
            };
            let fraction = quotient
                .w_shl(leading.wrapping_sub(n))
                .w_shr(Self::fraction_bits())
                .deflate();
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Power-of-two fractions (stored=0 = NEG_ONE, stored=MIN = POS_ONE) overflow
        // left_hand_load. Division involving these is just exponent subtraction ± negate.
        let self_pot =
            self.fraction == Self::neg_one_normal() || self.fraction == Self::pos_one_normal();
        let other_pot =
            other.fraction == Self::neg_one_normal() || other.fraction == Self::pos_one_normal();
        if self_pot || other_pot {
            // Power-of-two / anything or anything / power-of-two.
            // For pot numerator: self = ±(0.5 or 1.0) * 2^exp. Division = reciprocal(other) * self.
            // For pot denominator: other = ±(0.5 or 1.0) * 2^exp. Division = self * reciprocal_pot.
            // Both pot: result is exact power of two.
            // Use multiply infrastructure: negate + exponent math for pot, reciprocal for non-pot.
            if self_pot && other_pot {
                let self_neg1 = self.fraction == Self::neg_one_normal();
                let other_neg1 = other.fraction == Self::neg_one_normal();
                let (frac, exp_adj): (F, E) = match (self_neg1, other_neg1) {
                    (false, false) => (Self::pos_one_normal(), E::one()), // +0.5/+0.5 = +1
                    (false, true) => (Self::neg_one_normal(), E::zero().wrapping_sub(&E::one())), // +0.5/-1 = -0.5
                    (true, false) => (Self::neg_one_normal(), E::one()), // -1/+0.5 = -2
                    (true, true) => (Self::pos_one_normal(), E::one()),  // -1/-1 = +1
                };
                let exponent = self
                    .exponent
                    .wrapping_sub(&other.exponent)
                    .wrapping_add(&exp_adj);
                // TODO: exponent overflow/underflow checks
                return Self {
                    fraction: frac,
                    exponent,
                };
            }
            // One is pot, other is not. abs + shift + unsigned div.
            let num = self.fraction.inflate(true);
            let den = other.fraction.inflate(true);
            let expect_neg = self.is_negative() != other.is_negative();
            let num_abs = if num.w_is_negative() {
                num.w_neg()
            } else {
                num
            };
            let den_abs = if den.w_is_negative() {
                den.w_neg()
            } else {
                den
            };
            let (shifted_num, num_shift_adj) = if self_pot {
                (num_abs.w_shl(Self::fraction_bits() - 1), 1isize)
            } else {
                (num_abs.w_shl(Self::fraction_bits()), 0isize)
            };
            let quotient = shifted_num.w_div_unsigned(den_abs);
            let leading = quotient.w_leading_zeros();
            let stored_pos = quotient
                .w_shl(leading)
                .w_shr_logical(Self::fraction_bits())
                .deflate();
            let (fraction, neg_extra) = if expect_neg {
                if stored_pos == Self::pos_one_normal() {
                    (Self::neg_one_normal(), 1isize)
                } else {
                    (stored_pos.wrapping_neg(), 0isize)
                }
            } else {
                (stored_pos, 0isize)
            };
            let diff = self.exponent.wrapping_sub(&other.exponent);
            let adj: E = (leading
                .wrapping_sub(Self::fraction_bits())
                .wrapping_sub(num_shift_adj)
                .wrapping_add(neg_extra))
            .as_();
            let exponent = diff.wrapping_sub(&adj);
            // TODO: exponent overflow/underflow checks
            return Self { fraction, exponent };
        }

        // General case: neither operand is power-of-two.
        // abs → shift → unsigned div → normalize positive → apply sign on stored.
        let num = self.fraction.inflate(true);
        let den = other.fraction.inflate(true);
        let expect_negative = self.is_negative() != other.is_negative();
        let num_abs = if num.w_is_negative() {
            num.w_neg()
        } else {
            num
        };
        let den_abs = if den.w_is_negative() {
            den.w_neg()
        } else {
            den
        };
        let numerator = num_abs.w_shl(Self::fraction_bits());
        let quotient = numerator.w_div_unsigned(den_abs);
        let leading = quotient.w_leading_zeros();
        let stored_pos = quotient
            .w_shl(leading)
            .w_shr_logical(Self::fraction_bits())
            .deflate();
        let (fraction, neg_extra) = if expect_negative {
            if stored_pos == Self::pos_one_normal() {
                (Self::neg_one_normal(), 1isize)
            } else {
                (stored_pos.wrapping_neg(), 0isize)
            }
        } else {
            (stored_pos, 0isize)
        };

        let diff = self.exponent.wrapping_sub(&other.exponent);
        if !self.exponent.is_negative() && other.exponent.is_negative() && diff.is_negative() {
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.exponent.is_negative() && !other.exponent.is_negative() && !diff.is_negative() {
            return Self {
                fraction: fraction >> 1isize,
                exponent: Self::ambiguous_exponent(),
            };
        }
        let adj: E = (leading
            .wrapping_sub(Self::fraction_bits())
            .wrapping_add(neg_extra))
        .as_();
        let exponent = diff.wrapping_sub(&adj);
        if exponent == Self::ambiguous_exponent() {
            Self {
                fraction: fraction >> 1isize,
                exponent: Self::ambiguous_exponent(),
            }
        } else {
            Self { fraction, exponent }
        }
    }
}
