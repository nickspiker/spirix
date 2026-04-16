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
    pub(crate) fn scalar_power_scalar(&self, exp: &Self) -> Self {
        if !self.is_normal() || !exp.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if exp.is_undefined() {
                return *exp;
            }
            // x^0 = 1 for any finite x (mathematical identity)
            if exp.is_zero() {
                return Self::ONE;
            }
            if self == 1 {
                return *self;
            }

            if self.is_infinite() {
                if exp.fraction.is_negative() || exp.exponent.is_positive() {
                    return Self::ZERO;
                }
                if exp.is_zero() {
                    return Self::ONE;
                }
                return *self;
            }
            if exp.is_infinite() {
                if self.is_zero() {
                    return Self::ZERO;
                }
            }

            if self.exploded() {
                return Self {
                    fraction: TRANSFINITE_POWER.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if exp.exploded() {
                return Self {
                    fraction: POWER_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_negative() {
                return Self {
                    fraction: NEGATIVE_POWER.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if exp.is_negligible() {
                return Self {
                    fraction: POWER_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_negligible() {
                return Self {
                    fraction: VANISHED_POWER.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
        if self.is_negative() {
            return Self {
                fraction: NEGATIVE_POWER.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Check if exponent is an integer
        if exp.is_integer() {
            // Use exponentiation by squaring for integer exponents
            return self.integer_power(exp);
        }

        // Fall back to logarithmic method for non-integer exponents
        (exp * self.ln()).exp()
    }
    pub(crate) fn scalar_logarithm_scalar(&self, base: &Self) -> Self {
        // Needs infinity handling!
        if !self.is_normal() || !base.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if base.is_undefined() {
                return *base;
            }
            if self.exploded() {
                return Self {
                    fraction: TRANSFINITE_LOG.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_LOG.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if base.exploded() {
                return Self {
                    fraction: LOG_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: LOG_NEGLIGIBLE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if base == 1 {
            return Self {
                fraction: LOG_ONE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        let self_lb = self.lb();
        let base_lb = base.lb();
        let result = self_lb / base_lb;
        result
    }

    pub(crate) fn integer_power(&self, n: &Self) -> Self {
        if n.is_zero() {
            return Self::ONE;
        }

        let mut result = Self::ONE;
        let mut base = if n.is_negative() {
            self.reciprocal()
        } else {
            *self
        };
        let mut exp = n.magnitude();

        while !exp.is_zero() {
            if (exp & Self::ONE) == 1 {
                result *= base;
            }
            base = base.square();
            exp = (exp >> 1u8).floor();
        }

        result
    }
}
