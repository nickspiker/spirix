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
    pub(crate) fn scalar_divide_circle(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 unified pipeline. Scalar / Circle: s / (c + di) = s*(c-di) / (c²+d²). The Scalar's N0 fraction is converted N1; reciprocal of mag_sq computed in fixed-point.
            // Canonicalize a non-canonical denominator pair (pub-field-constructible) so the reciprocal's divisor can't vanish; a zero-valued pair is x/0 = ∞.
            let (dr, di, s_den) = Circle::<F, E>::canonical_n1_pair(other.real, other.imaginary);
            if s_den < 0 {
                return Circle::<F, E>::INFINITY;
            }
            let a = ((self.fraction >> 1isize) ^ F::min_value()).sign_extend();
            let c = dr.sign_extend();
            let d = di.sign_extend();
            let fb = Self::fraction_bits();

            let mag_sq = c.w_mul(c).w_add(d.w_mul(d));
            let scale = F::one()
                .sign_extend()
                .w_shl(fb.wrapping_shl(1).wrapping_sub(2));
            let reciprocal = scale.w_div_unsigned(mag_sq.w_shr_logical(fb));

            // s * conj(other) = (s * c) + (-s * d)i
            let real_num = a.w_mul(c);
            let imag_num = F::zero().sign_extend().w_sub(a.w_mul(d));
            let real_wide = real_num.w_shr(fb).w_mul(reciprocal);
            let imag_wide = imag_num.w_shr(fb).w_mul(reciprocal);

            if real_wide.w_is_zero() && imag_wide.w_is_zero() {
                return Circle::<F, E>::ZERO;
            }

            let leading_r = real_wide.leading_same();
            let leading_i = imag_wide.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_wide.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_wide.w_shl(shift).w_shr(fb).deflate();

            // AMBIG=0 exp: same shape as Circle/Circle div — stored_pos = pa - pb + binade_origin - shift, where shift = leading - 1 captures the left-shift used to normalize the wide result to canonical N1. The denominator canonicalization multiplied its pair by 2^s_den → exponent − s_den.
            let pa = self.exponent.cycle_widen();
            let s_den_e: E = s_den.as_();
            let pb = other.exponent.cycle_widen().w_sub(s_den_e.cycle_widen());
            let shift_e: E = leading.wrapping_sub(1).as_();
            let w_shift = shift_e.sign_extend();
            let w_bo = Scalar::<F, E>::binade_origin().cycle_widen();
            let stored_pos = pa.w_sub(pb).w_add(w_bo).w_sub(w_shift);
            let max_pos = Scalar::<F, E>::max_exponent().cycle_widen();
            let min_pos = Scalar::<F, E>::min_exponent().cycle_widen();

            return if stored_pos > max_pos {
                Circle {
                    real,
                    imaginary,
                    exponent: Circle::<F, E>::ambiguous_exponent(),
                }
            } else if stored_pos < min_pos {
                Circle {
                    real: real >> 1isize,
                    imaginary: imaginary >> 1isize,
                    exponent: Circle::<F, E>::ambiguous_exponent(),
                }
            } else {
                Circle {
                    real,
                    imaginary,
                    exponent: stored_pos.deflate(),
                }
            };
        }

        // Escape-class handling (at least one operand non-normal).
        {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if other.is_undefined() {
                return *other;
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
                return Circle {
                    real: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.vanished() {
                return Circle {
                    real: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    imaginary: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Mixed escape: same shape as the normal path — reciprocal of |c|² × s × conj(c) — but renormalized to N-2 (numerator vanished or denom exploded) or N-1 at AMBIG exp. The N0→N1 conversion only applies to normal self; escape patterns share layout across N0 and N1.
            let n_level: isize = if self.vanished() || other.exploded() {
                -2
            } else {
                -1
            };
            let a_narrow: F = if self.is_normal() {
                (self.fraction >> 1isize) ^ F::min_value()
            } else {
                self.fraction
            };
            // Canonicalize a non-canonical NORMAL denominator (escaped pairs are canonical by class); exponents are discarded on this path. Zero-valued pair → ∞.
            let (dr, di, s_den) = Circle::<F, E>::canonical_n1_pair(other.real, other.imaginary);
            if s_den < 0 {
                return Circle::<F, E>::INFINITY;
            }
            let a = a_narrow.sign_extend();
            let c = dr.sign_extend();
            let d = di.sign_extend();
            let fb = Self::fraction_bits();

            let mag_sq = c.w_mul(c).w_add(d.w_mul(d));
            let scale = F::one()
                .sign_extend()
                .w_shl(fb.wrapping_shl(1).wrapping_sub(2));
            let reciprocal = scale.w_div_unsigned(mag_sq.w_shr_logical(fb));

            let real_num = a.w_mul(c);
            let imag_num = F::zero().sign_extend().w_sub(a.w_mul(d));
            let real_wide = real_num.w_shr(fb).w_mul(reciprocal);
            let imag_wide = imag_num.w_shr(fb).w_mul(reciprocal);

            let leading_r = real_wide.leading_same();
            let leading_i = imag_wide.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_add(n_level);
            return Circle {
                real: real_wide.w_shl(shift).w_shr(fb).deflate(),
                imaginary: imag_wide.w_shl(shift).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
    }
}
