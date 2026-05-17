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

            let leading_r = real_quotient.leading_same();
            let leading_i = imag_quotient.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_quotient.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_quotient.w_shl(shift).w_shr(fb).deflate();

            // Div: stored_pos = pa - pb + binade_origin (no expo_adjust because the canonical-N1 shift already lands the fraction correctly; no -1 because cross-type div doesn't have the doubled bias of Circle*Circle's reciprocal-times-numerator).
            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let w_bo = Self::binade_origin().cycle_widen();
            let stored_pos = pa.w_sub(pb).w_add(w_bo);
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
            let n_level = if self.vanished() || other.exploded() {
                -2
            } else {
                -1
            };
            let (real, imaginary) = match Self::fraction_bits() {
                8 => {
                    let numerator_r: i16 = self.real.as_();
                    let numerator_i: i16 = self.imaginary.as_();
                    let denominator: i16 = ((other.fraction >> 1isize) ^ F::min_value()).as_();

                    let mut quotient_r =
                        (numerator_r << Self::fraction_bits()).div_euclid(denominator);
                    let mut quotient_i =
                        (numerator_i << Self::fraction_bits()).div_euclid(denominator);

                    let leading_r = quotient_r.leading_ones().max(quotient_r.leading_zeros());
                    let leading_i = quotient_i.leading_ones().max(quotient_i.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    quotient_r <<= shift;
                    quotient_i <<= shift;

                    (
                        (quotient_r >> Self::fraction_bits()).as_(),
                        (quotient_i >> Self::fraction_bits()).as_(),
                    )
                }
                16 => {
                    let numerator_r: i32 = self.real.as_();
                    let numerator_i: i32 = self.imaginary.as_();
                    let denominator: i32 = ((other.fraction >> 1isize) ^ F::min_value()).as_();

                    let mut quotient_r =
                        (numerator_r << Self::fraction_bits()).div_euclid(denominator);
                    let mut quotient_i =
                        (numerator_i << Self::fraction_bits()).div_euclid(denominator);

                    let leading_r = quotient_r.leading_ones().max(quotient_r.leading_zeros());
                    let leading_i = quotient_i.leading_ones().max(quotient_i.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    quotient_r <<= shift;
                    quotient_i <<= shift;

                    (
                        (quotient_r >> Self::fraction_bits()).as_(),
                        (quotient_i >> Self::fraction_bits()).as_(),
                    )
                }
                32 => {
                    let numerator_r: i64 = self.real.as_();
                    let numerator_i: i64 = self.imaginary.as_();
                    let denominator: i64 = ((other.fraction >> 1isize) ^ F::min_value()).as_();

                    let mut quotient_r =
                        (numerator_r << Self::fraction_bits()).div_euclid(denominator);
                    let mut quotient_i =
                        (numerator_i << Self::fraction_bits()).div_euclid(denominator);

                    let leading_r = quotient_r.leading_ones().max(quotient_r.leading_zeros());
                    let leading_i = quotient_i.leading_ones().max(quotient_i.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    quotient_r <<= shift;
                    quotient_i <<= shift;

                    (
                        (quotient_r >> Self::fraction_bits()).as_(),
                        (quotient_i >> Self::fraction_bits()).as_(),
                    )
                }
                64 => {
                    let numerator_r: i128 = self.real.as_();
                    let numerator_i: i128 = self.imaginary.as_();
                    let denominator: i128 = ((other.fraction >> 1isize) ^ F::min_value()).as_();

                    let mut quotient_r =
                        (numerator_r << Self::fraction_bits()).div_euclid(denominator);
                    let mut quotient_i =
                        (numerator_i << Self::fraction_bits()).div_euclid(denominator);

                    let leading_r = quotient_r.leading_ones().max(quotient_r.leading_zeros());
                    let leading_i = quotient_i.leading_ones().max(quotient_i.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    quotient_r <<= shift;
                    quotient_i <<= shift;

                    (
                        (quotient_r >> Self::fraction_bits()).as_(),
                        (quotient_i >> Self::fraction_bits()).as_(),
                    )
                }
                128 => {
                    let numerator_r: I256 = self.real.into();
                    let numerator_i: I256 = self.imaginary.into();
                    let denominator: I256 = ((other.fraction >> 1isize) ^ F::min_value()).into();

                    let mut quotient_r =
                        (numerator_r << Self::fraction_bits()).div_euclid(denominator);
                    let mut quotient_i =
                        (numerator_i << Self::fraction_bits()).div_euclid(denominator);

                    let leading_r = quotient_r.leading_ones().max(quotient_r.leading_zeros());
                    let leading_i = quotient_i.leading_ones().max(quotient_i.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    quotient_r <<= shift;
                    quotient_i <<= shift;

                    (
                        (quotient_r >> Self::fraction_bits()).as_i128().as_(),
                        (quotient_i >> Self::fraction_bits()).as_i128().as_(),
                    )
                }
                _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
            };
            return Self {
                real,
                imaginary,
                exponent: Self::ambiguous_exponent(),
            };
        }
    }
}
