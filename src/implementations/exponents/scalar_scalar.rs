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
            // Zero base: 0^0 = 1 (handled above); otherwise only the exponent's SIGN matters — 0^(+) = 0 and 0^(−) = ∞ — for any positive/negative exponent, normal or escaped.
            // (Without this, zero falls into the `is_negligible` catch-all below and 0^2 comes back undefined; a zero base is definite where a vanished base is not.)
            if self.is_zero() {
                return if exp.is_negative() {
                    Self::INFINITY
                } else {
                    Self::ZERO
                };
            }

            if self.is_infinite() {
                if exp.is_negative() || exp.exponent.is_positive() {
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

            // Escaped BASE (vanished or exploded): an escaped value is m·2^E with the significand m (phase) known and the integer exponent E hidden past the range boundary. Resolve everything that stays determinate:
            //  - integer p:  (m·2^E)^p = m^p · 2^(pE) — pE is still an integer, still hidden, so class, two's-complement sign (parity), AND phase all survive. Route thru integer_power; the phase-preserving multiply chain does the work, and pow(x,n) ≡ x·…·x, x^1 ≡ x.
            //  - non-integer |p| > 1, positive base: class is determinate (tiny^p ≤ tiny, huge^p ≥ huge; p < 0 inverts thru the reciprocal), but the fractional part of pE bleeds into the significand with E unknown → canonical positive escaped (phase lost honestly).
            //  - non-integer 0 < |p| < 1: tiny^p can land back in normal range (2^-1000^0.01 = 2^-10) — class indeterminate → keep ℘⬇^ / ℘⬆^.
            //  - negative base, non-integer p: no real value, same as normals → ℘-^.
            //  - escaped exponent: magnitude dominance resolves a positive base (tiny^huge = tinier, etc.), but a transfinite exponent's parity is unknowable so a negative base can't commit a sign → ℘^⬆.
            //  - vanished exponent: p·lb(base) is a 0·∞ form → ℘^⬇. Infinite exponent: unsigned ∞ has no direction → ℘^⬆.
            if self.exploded() || self.vanished() {
                let base_exploded = self.exploded();
                if exp.is_normal() {
                    if exp.is_integer() {
                        return self.integer_power(exp);
                    }
                    if self.is_negative() {
                        return Self {
                            fraction: NEGATIVE_POWER.prefix.sa(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    if exp.magnitude() > Self::ONE {
                        let result_exploded = if exp.is_positive() {
                            base_exploded
                        } else {
                            !base_exploded
                        };
                        return Self {
                            fraction: if result_exploded {
                                Self::pos_one_exploded()
                            } else {
                                Self::pos_one_vanished()
                            },
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    return Self {
                        fraction: if base_exploded {
                            TRANSFINITE_POWER.prefix.sa()
                        } else {
                            VANISHED_POWER.prefix.sa()
                        },
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                if exp.vanished() {
                    return Self {
                        fraction: POWER_VANISHED.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                if exp.is_infinite() || self.is_negative() {
                    return Self {
                        fraction: POWER_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                // Positive escaped base, exploded exponent: pure magnitude dominance.
                let result_exploded = if exp.is_positive() {
                    base_exploded
                } else {
                    !base_exploded
                };
                return Self {
                    fraction: if result_exploded {
                        Self::pos_one_exploded()
                    } else {
                        Self::pos_one_vanished()
                    },
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Normal base with an escaped/infinite exponent, unchanged behavior:
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
        }
        // Integer exponent is defined for ANY base sign — exponentiation by squaring keeps the sign correct ((-3)^2 = 9, (-3)^3 = -27).
        // This MUST come before rejecting negative bases, otherwise x^2 goes undefined for every x < 0.
        if exp.is_integer() {
            return self.integer_power(exp);
        }

        // Only a NON-integer (fractional / irrational) exponent of a negative base has no real value — e.g. (-2)^0.5.
        if self.is_negative() {
            return Self {
                fraction: NEGATIVE_POWER.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Positive base, non-integer exponent: logarithmic method.
        (exp * self.ln()).exp()
    }
    pub(crate) fn scalar_logarithm_scalar(&self, base: &Self) -> Self {
        // Undefined operands propagate their cause.
        if self.is_undefined() {
            return *self;
        }
        if base.is_undefined() {
            return *base;
        }
        // Base One is singular (ln 1 = 0), so log base 1 is undefined for any value.
        if base == 1 {
            return Self {
                fraction: LOG_ONE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Domain violations, most-primary first: a non-positive VALUE is undefined regardless of base; then a negative BASE.
        // Escaped operands keep the magnitude-lost undefined the README specifies, but tagged with which operand (value vs base) escaped.
        if self.is_negative() {
            return Self {
                fraction: NEGATIVE_LOG.prefix.sa(), // ℘-@  log of negative value
                exponent: Self::ambiguous_exponent(),
            };
        }
        if base.is_negative() {
            return Self {
                fraction: LOG_NEGATIVE.prefix.sa(), // ℘@-  logarithm with negative base
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.exploded() {
            return Self {
                fraction: TRANSFINITE_LOG.prefix.sa(), // ℘⬆@  log of transfinite value
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.vanished() {
            return Self {
                fraction: NEGLIGIBLE_LOG.prefix.sa(), // ℘⬇@  log of negligible value
                exponent: Self::ambiguous_exponent(),
            };
        }
        if base.exploded() {
            return Self {
                fraction: LOG_TRANSFINITE.prefix.sa(), // ℘@⬆  logarithm with transfinite base
                exponent: Self::ambiguous_exponent(),
            };
        }
        if base.vanished() {
            return Self {
                fraction: LOG_NEGLIGIBLE.prefix.sa(), // ℘@⬇  logarithm with negligible base
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Remaining operands are Zero, positive Normal, or Infinity. Change of base log_b(a) = lb(a) / lb(b) now resolves the definite limits exactly, because lb(0) = lb(∞) = ∞:
        //   log(0)     = ∞/lb(b) = ∞      log(∞)     = ∞/lb(b) = ∞
        //   log base 0 = lb(a)/∞ = 0      log base ∞ = lb(a)/∞ = 0
        self.lb() / base.lb()
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
