// Add this to the implementations/powers/circle_circle.rs file

use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    pub(crate) fn circle_power_circle(&self, exp: &Self) -> Self {
        if !self.is_normal() || !exp.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if exp.is_undefined() {
                return *exp;
            }
            if self.is_zero() {
                if exp.real.is_positive() {
                    return Self::ZERO;
                }
                let prefix: F = NEGLIGIBLE_POWER.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_POWER.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_POWER.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if exp.exploded() {
                let prefix: F = POWER_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let prefix: F = POWER_NEGLIGIBLE.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Check if exponent is real and integer for exact computation
        if exp.i().is_zero() && exp.r().is_integer() {
            return self.integer_power(&exp.r());
        }

        let ln_z = self.ln();
        let w_ln_z = exp * ln_z;
        w_ln_z.exp()
    }

    /// Computes the logarithm of a complex number with a complex base.
    ///
    /// Implements log_b(z) (logarithm of complex z with complex base b) using:
    /// log_b(z) = ln(z) / ln(b)
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return appropriate undefined states
    /// - Exploded or vanished values return appropriate undefined states
    /// - log_b(0) returns ZERO_LOG undefined state
    /// - log_0(z) returns LOG_ZERO undefined state
    ///
    /// # Parameters:
    /// - `self`: The complex number to take the logarithm of
    /// - `base`: The complex base of the logarithm
    ///
    /// # Returns:
    /// - A complex number representing log_b(z)
    pub(crate) fn circle_logarithm_circle(&self, base: &Self) -> Self {
        if !self.is_normal() || !base.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if base.is_undefined() {
                return *base;
            }
            if self.is_zero() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if base.is_zero() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if base.exploded() {
                let prefix: F = LOG_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if base.vanished() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        let ln_z = self.ln();
        let ln_base = base.ln();

        ln_z / ln_base
    }
}
