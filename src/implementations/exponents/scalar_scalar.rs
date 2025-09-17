use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::AsPrimitive;
use std::ops::*;
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if exp.exploded() {
                return Self {
                    fraction: POWER_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.fraction.is_negative() {
                return Self {
                    fraction: NEGATIVE_POWER.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if exp.is_negligible() {
                return Self {
                    fraction: POWER_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_negligible() {
                return Self {
                    fraction: NEGLIGIBLE_POWER.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
        if self.fraction.is_negative() {
            return Self {
                fraction: NEGATIVE_POWER.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Check if exponent is an integer
        if exp.is_integer() {
            // Use exponentiation by squaring for integer exponents
            let exp_int = exp.to_isize();
            return self.integer_power(exp_int);
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_LOG.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if base.exploded() {
                return Self {
                    fraction: LOG_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Self {
                fraction: LOG_NEGLIGIBLE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if base == 1 {
            return Self {
                fraction: LOG_ONE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let self_lb = self.lb();
        let base_lb = base.lb();
        let result = self_lb / base_lb;
        result
    }

    fn integer_power(&self, n: isize) -> Self {
        if n == 0 {
            return Self::ONE;
        }
        if n == 1 {
            return *self;
        }
        if n == -1 {
            return Self::ONE / *self;
        }

        let mut result = Self::ONE;
        let mut base = *self;
        let mut exp = n.abs() as usize;

        // Exponentiation by squaring
        while exp > 0 {
            if exp & 1 == 1 {
                result = result * base;
            }
            base = base.square();
            exp >>= 1;
        }

        if n < 0 {
            Self::ONE / result
        } else {
            result
        }
    }
}
