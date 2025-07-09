use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
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
    /// Raises a complex number to a scalar power.
    ///
    /// Implements z^s (complex raised to scalar power) using:
    /// - Specialized algorithm for integer exponents
    /// - General formula z^s = exp(s * ln(z)) for non-integer exponents
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return appropriate undefined states
    /// - 0^0 returns the ZERO_POWER_ZERO undefined state
    /// - 0^s returns 0 if s is positive
    /// - 0^s returns ZERO_NEGATIVE_POWER undefined state if s is negative
    ///
    /// # Parameters:
    /// - `self`: The complex base
    /// - `exp`: The scalar exponent
    ///
    /// # Returns:
    /// - A complex number representing z^s
    pub(crate) fn circle_power_scalar(&self, exp: &Scalar<F, E>) -> Self {
        if !self.is_normal() || !exp.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if exp.is_undefined() {
                return Self {
                    real: exp.fraction,
                    imaginary: exp.fraction,
                    exponent: exp.exponent,
                };
            }
            if self.is_zero() {
                if exp.fraction.is_positive() {
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
        let ln_z = self.ln();
        let s_ln_z = ln_z * exp;
        s_ln_z.exp()
    }
    /// Computes the logarithm of a complex number with a scalar base.
    ///
    /// Implements c log s (logarithm of complex c with scalar base s) using:
    /// c log s = ln(c) / ln(s)
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return appropriate undefined states
    /// - Exploded or vanished values return appropriate undefined states
    /// - 0 log s returns ZERO_LOG undefined state
    /// - c log 0 returns LOG_ZERO undefined state
    /// - c log -# returns LOG_NEGATIVE undefined state
    ///
    /// # Parameters:
    /// - `self`: The complex number to take the logarithm of
    /// - `base`: The scalar base of the logarithm
    ///
    /// # Returns:
    /// - A complex number representing c log s
    pub(crate) fn circle_logarithm_scalar(&self, base: &Scalar<F, E>) -> Self {
        if !self.is_normal() || !base.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if base.is_undefined() {
                return Self {
                    real: base.fraction,
                    imaginary: 0.as_(),
                    exponent: base.exponent,
                };
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

        if base.fraction.is_negative() {
            let prefix: F = LOG_NEGATIVE.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let ln_z = self.ln();
        let ln_base = base.ln();

        ln_z / ln_base
    }
}
