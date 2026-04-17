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
    /// Performs bitwise AND on two Scalars after aligning their fractions by exponent.
    ///
    /// Operates in inflated (effective) value space so the standard two's complement
    /// AND semantics apply: negative operands have leading ones, positive operands
    /// have leading zeros, after inflate.
    pub(crate) fn aligned_and(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
                || self.is_infinite()
                || other.is_infinite()
            {
                return Self {
                    fraction: AND.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.vanished() {
                if self.is_negative() {
                    return *other;
                }
                return Self::ZERO;
            }
            if other.vanished() {
                if other.is_negative() {
                    return *self;
                }
                return Self::ZERO;
            }
            if self.is_normal() {
                if self.is_negative() {
                    return *other;
                }
                return Self::ZERO;
            }
            if other.is_normal() {
                if other.is_negative() {
                    return *self;
                }
                return Self::ZERO;
            }
            return Self {
                fraction: AND.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
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
            if self.is_infinite() || other.is_infinite() || self.exploded() && other.exploded() {
                return Self {
                    fraction: OR.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.exploded() {
                if other.is_negative() {
                    return *other;
                }
                return *self;
            }
            if other.exploded() {
                if self.is_negative() {
                    return *self;
                }
                return *other;
            }
            if self.is_normal() {
                if other.is_negative() {
                    return *other;
                }
                return *self;
            }
            if other.is_normal() {
                if self.is_negative() {
                    return *self;
                }
                return *other;
            }
            return Self {
                fraction: OR.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        self.bitwise_normal(other, BitwiseOp::Or)
    }

    /// Performs bitwise XOR on two Scalars after aligning their fractions by exponent.
    pub(crate) fn aligned_xor(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite()
                || other.is_infinite()
                || (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
            {
                return Self {
                    fraction: XOR.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.vanished() {
                if self.is_negative() {
                    return other.not_scalar();
                }
                return *other;
            }
            if other.vanished() {
                if other.is_negative() {
                    return self.not_scalar();
                }
                return *self;
            }
            if self.is_normal() {
                if self.is_negative() {
                    return other.not_scalar();
                }
                return *other;
            }
            if other.is_normal() {
                if other.is_negative() {
                    return self.not_scalar();
                }
                return *self;
            }
            return Self {
                fraction: XOR.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        self.bitwise_normal(other, BitwiseOp::Xor)
    }

    /// Generic normal-path bitwise operation. Inflates both operands, aligns by exponent,
    /// applies the op in wide effective space, then normalizes and deflates.
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
        let offset = small
            .exponent
            .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
        if big.exponent.is_negative() && !offset.is_negative() {
            return Self {
                fraction: result
                    .w_shl(leading.wrapping_sub(1))
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

    /// Bitwise result when operands don't overlap after alignment. Sign of `small`
    /// determines whether the high bits are all-ones (negative) or all-zeros (positive).
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
        if !shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                fraction: self.fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if shift.is_negative()
            && self.exponent.is_negative()
            && !(new_exp.wrapping_sub(&E::one())).is_negative()
        {
            return Self {
                fraction: self.fraction >> 1isize,
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
        if shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                fraction: self.fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if !shift.is_negative()
            && self.exponent.is_negative()
            && !(new_exp.wrapping_sub(&E::one())).is_negative()
        {
            return Self {
                fraction: self.fraction >> 1isize,
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
