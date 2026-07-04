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
            // z^0 = 1 for any finite z (mathematical identity, 0^0 = 1 included) — matches the Scalar convention.
            if exp.is_zero() {
                return Self::ONE;
            }
            // Zero base: only the exponent's SIGN matters — 0^(+) = 0, 0^(−) = ∞ — for any positive/negative exponent, normal or escaped.
            // NOTE: exp is an N0-convention Scalar, so use its is_negative(), never the raw fraction sign (stored MSB=1 reads POSITIVE, the opposite of a plain int).
            if self.is_zero() {
                return if exp.is_negative() {
                    Self::INFINITY
                } else {
                    Self::ZERO
                };
            }
            // Escaped base: the orientation θ is stored, so much more resolves than the Scalar case — see escaped_rotate_to_power for the non-integer rules.
            if self.exploded() || self.vanished() {
                let base_exploded = self.exploded();
                if exp.is_normal() {
                    if exp.is_integer() {
                        // Multiply chain: class, parity, and orientation all survive — and z^1 ≡ z.
                        return self.integer_power(exp);
                    }
                    if exp.magnitude() > Scalar::<F, E>::ONE {
                        // Non-integer |p| > 1: magnitude class is determinate (dominance; p < 0 inverts thru the reciprocal) and the direction rotates to p·θ, computable from the stored orientation.
                        return self.escaped_rotate_to_power(exp, base_exploded);
                    }
                    // Non-integer 0 < |p| < 1: tiny^p / huge^p can re-enter normal range — class indeterminate.
                    let prefix: F = if base_exploded {
                        TRANSFINITE_POWER.prefix.sa()
                    } else {
                        VANISHED_POWER.prefix.sa()
                    };
                    return Self {
                        real: prefix,
                        imaginary: prefix,
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                // Vanished exponent: p·lb|z| is a 0·∞ form → ℘^⬇. Anything transfinite (exploded/∞ exponent): the rotation p·θ mod 2π is unknowable → ℘^⬆.
                let prefix: F = if exp.vanished() {
                    POWER_VANISHED.prefix.sa()
                } else {
                    POWER_TRANSFINITE.prefix.sa()
                };
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Infinite base (the single unsigned point at infinity, no angle to lose): ∞^(+) = ∞, ∞^(−) = 0. (∞^0 = 1 returned above.)
            if self.is_infinite() {
                return if exp.is_negative() {
                    Self::ZERO
                } else {
                    Self::INFINITY
                };
            }
            // Normal base, non-normal exponent: transfinite exponents (exploded or ∞) can't commit a rotation → ℘^⬆; a vanished exponent is the 0·∞-adjacent form ℘^⬇.
            let prefix: F = if exp.vanished() {
                POWER_VANISHED.prefix.sa()
            } else {
                POWER_TRANSFINITE.prefix.sa()
            };
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Check if exponent is integer for exact computation
        if exp.is_integer() {
            return self.integer_power(exp);
        }

        let ln_z = self.ln();
        let s_ln_z = ln_z * exp;
        s_ln_z.exp()
    }

    /// Raise an escaped Circle to a non-integer real power with |p| > 1: the magnitude class is fixed by dominance (same class for p > 0, inverted thru the reciprocal for p < 0), and the direction rotates from the stored θ to p·θ.
    /// The stored escaped fractions ARE the unit direction (N1 shape for exploded, N2 for vanished), so rebuild them as a normal Circle, extract θ via atan2, rotate, and stamp the result class shape.
    /// If p·θ's period position exceeds precision, sin/cos return their imprecision undefineds — propagated honestly.
    fn escaped_rotate_to_power(&self, exp: &Scalar<F, E>, base_exploded: bool) -> Self {
        let n_shift: isize = if base_exploded { 0 } else { 1 };
        let dir = Self {
            real: self.real << n_shift,
            imaginary: self.imaginary << n_shift,
            exponent: Self::ONE.exponent,
        };
        let theta = dir.i().atan2(dir.r());
        let phi = theta * *exp;
        let unit = Self::from((phi.cos(), phi.sin()));
        if !unit.is_normal() {
            return unit;
        }
        let result_exploded = if exp.is_positive() {
            base_exploded
        } else {
            !base_exploded
        };
        if result_exploded {
            Self {
                real: unit.real,
                imaginary: unit.imaginary,
                exponent: Self::ambiguous_exponent(),
            }
        } else {
            Self {
                real: unit.real >> 1isize,
                imaginary: unit.imaginary >> 1isize,
                exponent: Self::ambiguous_exponent(),
            }
        }
    }

    pub(crate) fn integer_power(&self, n: &Scalar<F, E>) -> Self {
        if n.is_zero() {
            return Self::ONE;
        }

        let mut result = Self::ONE;
        let mut base = if n.is_negative() {
            Self::ONE / *self
        } else {
            *self
        };
        let mut exp = n.magnitude();
        // Two's-complement boundary: n = -2^MAX_EXP escapes under magnitude() and would hang the squaring loop (see the Scalar twin). For a Circle the result DIRECTION is n·θ with |n| = 2^MAX_EXP — a period position far beyond any stored precision — so the honest result is undefined, transfinite-power tagged.
        if !exp.is_normal() {
            if base.is_undefined() {
                return base;
            }
            let prefix: F = POWER_TRANSFINITE.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }

        while !exp.is_zero() {
            if (exp & Scalar::<F, E>::ONE) == 1 {
                result *= base;
            }
            base = base.square();
            exp = (exp >> 1u8).floor();
        }

        result
    }
    /// Computes the logarithm of a complex number with a scalar base.
    ///
    /// Implements c log s (logarithm of complex c with scalar base s) using: c log s = ln(c) / ln(s)
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
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if base.is_zero() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if base.exploded() {
                let prefix: F = LOG_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if base.vanished() {
                let prefix: F = LOG_NEGLIGIBLE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        if base.fraction.is_negative() {
            let prefix: F = LOG_NEGATIVE.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        let ln_z = self.ln();
        let ln_base = base.ln();

        ln_z / ln_base
    }
}
