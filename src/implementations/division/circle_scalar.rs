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
    pub(crate) fn circle_divide_scalar(&self, other: &Scalar<F, E>) -> Self {
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 unified pipeline. Circle / Scalar: divide each Circle component by the (N0→N1) Scalar fraction. Use div_euclid pattern.
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let s = ((other.fraction >> 1isize) ^ F::min_value()).sign_extend();
            let fb = Self::fraction_bits();

            let real_quotient = a.w_shl(fb).w_div(s);
            let imag_quotient = b.w_shl(fb).w_div(s);

            if real_quotient.w_is_zero() && imag_quotient.w_is_zero() {
                return Self::ZERO;
            }

            // Cross-type div uses `a << fb / s`: a and s are both N1 wide (value × 2^(FRAC-2)), quotient scale is value × 2^FRAC. Shifting left by `leading-1` brings the (renormalized) dominant component to canonical wide N1 (leading_same=1, magnitude bit at 2*FRAC-2); the >> fb then lands at narrow N1 canonical (FRAC-2). Producing canonical form here is required — Circle's class detection in is_n1/is_n2 looks at the magnitude bit of either component, so sub-canonical fractions get misclassified as N-2/vanished.
            let leading_r = real_quotient.leading_same();
            let leading_i = imag_quotient.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_quotient.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_quotient.w_shl(shift).w_shr(fb).deflate();

            // Exp follows the fraction shift: the canonical-N1 normalization shifted the value by `leading - 1` bits; offset the binade by `FRAC - 1 - leading` so the final stored value matches reality. This collapses to `pa - pb + bo` when leading = FRAC-1 (value lands in [1, 2) naturally), and bumps the binade down for sub-canonical leading (value < 1 → shifted up to canonical → exp drops).
            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let w_bo = Self::binade_origin().cycle_widen();
            let delta_e: E = fb.wrapping_sub(1).wrapping_sub(leading).as_();
            let w_delta = delta_e.sign_extend();
            let stored_pos = pa.w_sub(pb).w_add(w_bo).w_add(w_delta);
            let max_pos = Self::max_exponent().cycle_widen();
            let min_pos = Self::min_exponent().cycle_widen();

            return if stored_pos > max_pos {
                Self {
                    real,
                    imaginary,
                    exponent: Self::ambiguous_exponent(),
                }
            } else if stored_pos < min_pos {
                Self {
                    real: real >> 1isize,
                    imaginary: imaginary >> 1isize,
                    exponent: Self::ambiguous_exponent(),
                }
            } else {
                Self {
                    real,
                    imaginary,
                    exponent: stored_pos.deflate(),
                }
            };
        }

        // Escape-class handling (at least one operand non-normal).
        {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return Circle {
                    real: other.fraction,
                    imaginary: other.fraction,
                    exponent: other.exponent,
                };
            }
            if other.is_zero() {
                if self.is_zero() {
                    return Circle {
                        real: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        imaginary: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Circle::<F, E>::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Circle {
                        real: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        imaginary: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Circle::<F, E>::INFINITY;
            }
            if self.is_zero() || other.is_infinite() {
                return Circle::<F, E>::ZERO;
            }
            if self.exploded() && other.exploded() {
                return Self {
                    real: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    real: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    imaginary: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Mixed escape: divide each Circle component by the (N0→N1 for normal, pass-through for escape) Scalar, renormalize to N-2 (numerator vanished or denom exploded) or N-1 (otherwise) at AMBIG exp.
            let n_level: isize = if self.vanished() || other.exploded() { -2 } else { -1 };
            let denom_narrow: F = if other.is_normal() {
                (other.fraction >> 1isize) ^ F::min_value()
            } else {
                other.fraction
            };
            let s = denom_narrow.sign_extend();
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let fb = Self::fraction_bits();
            let quotient_r = a.w_shl(fb).w_div(s);
            let quotient_i = b.w_shl(fb).w_div(s);

            let leading_r = quotient_r.leading_same();
            let leading_i = quotient_i.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_add(n_level);
            return Self {
                real: quotient_r.w_shl(shift).w_shr(fb).deflate(),
                imaginary: quotient_i.w_shl(shift).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
    }
}
