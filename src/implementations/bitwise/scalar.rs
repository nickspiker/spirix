use crate::core::integer::{Deflate, FullInt, IntConvert, WideOps};
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
    /// Bitwise AND of two Scalars, aligned at the binary point.
    ///
    /// Two's complement AND extended to numbers with exponents: line the operands up at their binary points, AND the bit patterns, renormalize. Class-level behaviour follows from the boolean identities of zero and infinity, which are alignment-independent:
    ///
    /// - `[0] & X = [0]` — zero is the absorber (all-zeros erases every bit).
    /// - `[∞] & X = X` — infinity is the identity (all-ones leaves bits alone).
    /// - `[℘?] & X = [℘?]` — undefined propagates first to preserve the error cause.
    ///
    /// Escape operands (`[↓]`, `[↑]`) paired with a normal can't align their ambiguous exponent with a real one, so those pairings resolve to `[℘&]`.
    pub(crate) fn aligned_and(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            // All-ones (infinity) is the identity for AND: [∞] & X = X.
            if self.is_infinite() {
                return *other;
            }
            if other.is_infinite() {
                return *self;
            }
            // All-zeros (zero) is the absorber for AND: [0] & X = [0].
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
                || self.is_normal()
                || other.is_normal()
            {
                return Self {
                    fraction: AND.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                if self.fraction.is_negative() {
                    return *other;
                }
                return Self::ZERO;
            }
            if other.fraction.is_negative() {
                return *self;
            }
            return Self::ZERO;
        }
        self.bitwise_normal(other, BitwiseOp::And)
    }

    /// Performs bitwise OR on two Scalars after aligning their fractions by exponent.
    pub(crate) fn aligned_or(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            // Infinity's fraction is all-ones; OR with anything stays all-ones.
            if self.is_infinite() || other.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
                || self.is_normal()
                || other.is_normal()
            {
                return Self {
                    fraction: OR.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                if other.is_negative() {
                    return *other;
                }
                return *self;
            }
            if self.is_negative() {
                return *self;
            }
            return *other;
        }
        self.bitwise_normal(other, BitwiseOp::Or)
    }

    /// Bitwise XOR of two Scalars, aligned at the binary point.
    ///
    /// `[0]` is the identity; `[∞]` inverts (NOT). Both are alignment-independent. Escape operands (`[↓]`, `[↑]`) paired with a normal produce `[℘⊻]` because ambiguous exponents can't align with real ones. At the shared ambiguous frame, `[↓] ⊻ [↑]` collapses to `[↑]` (opposite-rank bit patterns always XOR to N-1); same-class escape pairings are `[℘⊻]`.
    pub(crate) fn aligned_xor(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            // Undefined propagates first.
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            // Zero is the identity.
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            // Infinity (all-ones) inverts the other operand.
            if self.is_infinite() {
                return other.not_scalar();
            }
            if other.is_infinite() {
                return self.not_scalar();
            }
            // Escape-with-normal or same-class escapes: ambiguous → [℘⊻].
            if (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
                || self.is_normal()
                || other.is_normal()
            {
                return Self {
                    fraction: XOR.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Remaining case: one is vanished, the other is exploded. XOR always yields exploded with sign = sign(exploded) XOR sign(vanished).
            if self.exploded() {
                if other.is_negative() {
                    return self.not_scalar();
                }
                return *self;
            }
            if self.is_negative() {
                return other.not_scalar();
            }
            return *other;
        }
        self.bitwise_normal(other, BitwiseOp::Xor)
    }

    /// Generic normal-path bitwise operation. Inflates both operands, aligns by exponent, applies the op in wide effective space, then normalizes and deflates.
    fn bitwise_normal(&self, other: &Scalar<F, E>, op: BitwiseOp) -> Scalar<F, E> {
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            return Self::bitwise_no_overlap(big, small, op);
        }
        let shift: isize = exp_diff.saturate();
        if shift >= Self::fraction_bits() {
            return Self::bitwise_no_overlap(big, small, op);
        }
        let mut big_w = big.fraction.inflate(true);
        big_w.w_shl_assign(shift);
        let small_w = small.fraction.inflate(true);
        let result = match op {
            BitwiseOp::And => big_w.w_and(small_w),
            BitwiseOp::Or => big_w.w_or(small_w),
            BitwiseOp::Xor => big_w.w_xor(small_w),
        };
        if result.w_is_zero() {
            return Self {
                fraction: F::zero(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        let leading = result.leading_same();
        let delta: isize = Self::fraction_bits().wrapping_sub(leading);
        let delta_e: E = delta.as_();
        let offset = small.exponent.wrapping_add(&delta_e);
        // Underflow happens two ways: offset lands exactly on the AMBIGUOUS_EXPONENT slot (reserved), or it wraps past MIN through the negative/positive sign.
        let one_e: E = 1u8.as_();
        let underflowed = delta.is_negative() && offset.wrapping_sub(&one_e) > small.exponent;
        if underflowed {
            // Result magnitude smaller than MIN_EXP permits. Produce vanished (N-2) with the result's sign, not exploded (N-1).
            return Self {
                fraction: result
                    .w_shl(leading.wrapping_sub(2))
                    .w_shr(Self::fraction_bits())
                    .deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        Self {
            fraction: result.w_shl(leading).w_shr(Self::fraction_bits()).deflate(),
            exponent: offset,
        }
    }

    /// Bitwise result when operands don't overlap after alignment. Sign of `small` determines whether the high bits are all-ones (negative) or all-zeros (positive).
    fn bitwise_no_overlap(big: &Scalar<F, E>, small: &Scalar<F, E>, op: BitwiseOp) -> Scalar<F, E> {
        match op {
            BitwiseOp::And => {
                if small.is_negative() {
                    *big
                } else {
                    Self::ZERO
                }
            }
            BitwiseOp::Or => {
                if small.is_negative() {
                    *small
                } else {
                    *big
                }
            }
            BitwiseOp::Xor => {
                if small.is_negative() {
                    big.not_scalar()
                } else {
                    *big
                }
            }
        }
    }

    pub(crate) fn not_scalar(&self) -> Scalar<F, E> {
        if self.is_undefined() {
            return *self;
        }
        Self {
            fraction: !self.fraction,
            exponent: self.exponent,
        }
    }

    pub fn scalar_shl_integer(&self, shift: &E) -> Scalar<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_add(shift);
        // Input sign in N0 normal: stored MSB=1 → positive, MSB=0 → negative.
        let neg = !self.fraction.is_negative();
        // Overflow: both non-negative, sum wrapped negative → true sum > MAX_EXP → Exploded.
        let overflowed =
            !shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative();
        if overflowed {
            return Self {
                fraction: if neg {
                    Self::neg_one_exploded()
                } else {
                    Self::pos_one_exploded()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Underflow: either AMBIGUOUS landing (no wrap) or both-negative wrap to non-negative. Both mean true sum ≤ AMBIGUOUS → Vanished.
        let underflowed = new_exp == Self::ambiguous_exponent()
            || (shift.is_negative() && self.exponent.is_negative() && !new_exp.is_negative());
        if underflowed {
            return Self {
                fraction: if neg {
                    Self::neg_one_vanished()
                } else {
                    Self::pos_one_vanished()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        Self {
            fraction: self.fraction,
            exponent: new_exp,
        }
    }

    pub fn scalar_shr_integer(&self, shift: &E) -> Scalar<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_sub(shift);
        let neg = !self.fraction.is_negative();
        // Overflow: shift negative, self.exp non-negative, diff wrapped negative → true diff > MAX_EXP → Exploded.
        let overflowed =
            shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative();
        if overflowed {
            return Self {
                fraction: if neg {
                    Self::neg_one_exploded()
                } else {
                    Self::pos_one_exploded()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Underflow: either AMBIGUOUS landing or shift≥0/self<0 wrap.
        let underflowed = new_exp == Self::ambiguous_exponent()
            || (!shift.is_negative() && self.exponent.is_negative() && !new_exp.is_negative());
        if underflowed {
            return Self {
                fraction: if neg {
                    Self::neg_one_vanished()
                } else {
                    Self::pos_one_vanished()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        Self {
            fraction: self.fraction,
            exponent: new_exp,
        }
    }
}

#[derive(Clone, Copy)]
enum BitwiseOp {
    And,
    Or,
    Xor,
}
