use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::{I256, U256};
use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    pub(crate) fn circle_divide_circle(&self, other: &Self) -> Self {
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 native unified pipeline. Complex division: (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c² + d²). Reciprocal computed in fixed-point (scale = 2^(2*FRAC-2)) then multiplied with numerators.
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let c = other.real.sign_extend();
            let d = other.imaginary.sign_extend();
            let fb = Self::fraction_bits();

            let mag_sq = c.w_mul(c).w_add(d.w_mul(d));
            let scale = F::one().sign_extend().w_shl(fb.wrapping_shl(1).wrapping_sub(2));
            let reciprocal = scale.w_div_unsigned(mag_sq.w_shr_logical(fb));

            let real_num = a.w_mul(c).w_add(b.w_mul(d));
            let imag_num = b.w_mul(c).w_sub(a.w_mul(d));
            let real_wide = real_num.w_shr(fb).w_mul(reciprocal);
            let imag_wide = imag_num.w_shr(fb).w_mul(reciprocal);

            let leading_r = real_wide.leading_same();
            let leading_i = imag_wide.leading_same();
            let leading = leading_r.min(leading_i);
            // Canonical N1 result: shift = leading - 1.
            let shift = leading.wrapping_sub(1);
            let real = real_wide.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_wide.w_shl(shift).w_shr(fb).deflate();

            // AMBIG=0 exp: stored_pos = pa - pb - expo_adjust + binade_origin - 1. expo_adjust = leading - 3.
            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let expo_adjust_e: E = leading.wrapping_sub(3).as_();
            let w_adj = expo_adjust_e.sign_extend();
            let w_bo = Self::binade_origin().cycle_widen();
            let w_one = E::one().cycle_widen();
            let stored_pos = pa.w_sub(pb).w_sub(w_adj).w_add(w_bo).w_sub(w_one);
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

        // Escape-class handling.
        {
            if self.is_undefined() {
                return *self;
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
            // Mixed escape: compute the division anyway, output at AMBIG with the right N1/N2 shape via n_level.
            let n_level: isize = if self.vanished() || other.exploded() { -2 } else { -1 };
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let c = other.real.sign_extend();
            let d = other.imaginary.sign_extend();
            let fb = Self::fraction_bits();

            let mag_sq = c.w_mul(c).w_add(d.w_mul(d));
            let scale = F::one().sign_extend().w_shl(fb.wrapping_shl(1).wrapping_sub(2));
            let reciprocal = scale.w_div_unsigned(mag_sq.w_shr_logical(fb));

            let real_num = a.w_mul(c).w_add(b.w_mul(d));
            let imag_num = b.w_mul(c).w_sub(a.w_mul(d));
            let real_wide = real_num.w_shr(fb).w_mul(reciprocal);
            let imag_wide = imag_num.w_shr(fb).w_mul(reciprocal);

            let leading_r = real_wide.leading_same();
            let leading_i = imag_wide.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_add(n_level);
            return Self {
                real: real_wide.w_shl(shift).w_shr(fb).deflate(),
                imaginary: imag_wide.w_shl(shift).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
    }
}
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
#[allow(dead_code)]
fn _printey<T: core::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = core::mem::size_of::<T>() * 8;
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits - 1 && b % 8 == 7 {
            result.push(' ');
        }
        if b == bits / 2 - 1 {
            result.push(' '); // Extra space at center
        }
    }
    result
}
