use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
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
    /// Raises a scalar to a complex power.
    ///
    /// Implements s^z (scalar raised to complex power) using the formula:
    /// s^z = exp(z * ln(s))
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return appropriate undefined states
    /// - 0^0 returns the ZERO_POWER_ZERO undefined state
    /// - 0^z returns 0 if z has a positive real part
    /// - 0^z returns ZERO_NEGATIVE_POWER undefined state if z has a negative real part
    /// - Negative scalar bases return NEGATIVE_POWER undefined state (for complex exponents)
    ///
    /// # Parameters:
    /// - `self`: The scalar base
    /// - `exp`: The complex exponent
    ///
    /// # Returns:
    /// - A complex number representing s^z
    pub(crate) fn scalar_power_circle(&self, exp: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !exp.is_normal() {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if exp.is_undefined() {
                return *exp;
            }
            if self.is_zero() {
                if exp.real.is_positive() {
                    return Circle::<F, E>::ZERO;
                }

                let prefix: F = NEGLIGIBLE_POWER.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_POWER.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_POWER.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if exp.exploded() {
                let prefix: F = POWER_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let prefix: F = POWER_NEGLIGIBLE.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        if self.fraction.is_negative() {
            let prefix: F = NEGATIVE_POWER.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Check if exponent is real and integer for exact computation
        if exp.i().is_zero() && exp.r().is_integer() {
            return Circle::from(self.integer_power(&exp.r()));
        }

        (exp * self.ln()).exp()
    }
    /// Computes the logarithm of a scalar with a complex base.
    ///
    /// Implements log_z(s) (logarithm of scalar s with complex base z) using:
    /// log_z(s) = ln(s) / ln(z)
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return appropriate undefined states
    /// - Exploded or vanished values return appropriate undefined states
    /// - log_z(0) returns ZERO_LOG undefined state
    /// - log_0(s) returns LOG_ZERO undefined state
    /// - log_z(negative) returns NEGATIVE_LOG undefined state
    ///
    /// # Parameters:
    /// - `self`: The scalar number to take the logarithm of
    /// - `base`: The complex base of the logarithm
    ///
    /// # Returns:
    /// - A complex number representing log_z(s)
    pub(crate) fn scalar_logarithm_circle(&self, base: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !base.is_normal() {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if base.is_undefined() {
                return *base;
            }

            if self.is_zero() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if base.is_zero() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_LOG.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if base.exploded() {
                let prefix: F = LOG_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if base.vanished() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        if self.fraction.is_negative() {
            let prefix: F = NEGATIVE_LOG.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        self.ln() / base.ln()
    }
}
