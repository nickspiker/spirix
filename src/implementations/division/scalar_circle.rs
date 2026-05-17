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
        if !self.is_normal() || !other.is_normal() {
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
            let n_level = if self.vanished() || other.exploded() {
                -2
            } else {
                -1
            };
            let (real, imaginary) = match Self::fraction_bits() {
                8 => {
                    let a: i16 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                    let c: i16 = other.real.as_();
                    let d: i16 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u16;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i16;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = ad.wrapping_neg();
                    let mut imaginary_wide =
                        (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> Self::fraction_bits()).as_(),
                        (imaginary_wide >> Self::fraction_bits()).as_(),
                    )
                }
                16 => {
                    let a: i32 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                    let c: i32 = other.real.as_();
                    let d: i32 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u16;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i32;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = ad.wrapping_neg();
                    let mut imaginary_wide =
                        (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> Self::fraction_bits()).as_(),
                        (imaginary_wide >> Self::fraction_bits()).as_(),
                    )
                }
                32 => {
                    let a: i64 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                    let c: i64 = other.real.as_();
                    let d: i64 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u16;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i64;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = ad.wrapping_neg();
                    let mut imaginary_wide =
                        (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> Self::fraction_bits()).as_(),
                        (imaginary_wide >> Self::fraction_bits()).as_(),
                    )
                }
                64 => {
                    let a: i128 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                    let c: i128 = other.real.as_();
                    let d: i128 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u16;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i128;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = ad.wrapping_neg();
                    let mut imaginary_wide =
                        (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> Self::fraction_bits()).as_(),
                        (imaginary_wide >> Self::fraction_bits()).as_(),
                    )
                }
                128 => {
                    let a: I256 = ((self.fraction >> 1isize) ^ F::min_value()).into();
                    let c: I256 = other.real.into();
                    let d: I256 = other.imaginary.into();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)).as_unsigned();
                    let one: I256 = 1.into();
                    let one = one.as_unsigned();
                    let reciprocal = ((one
                        << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                        / (mag_sq >> Self::fraction_bits()))
                    .as_signed();

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = ad.wrapping_neg();
                    let mut imaginary_wide =
                        (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = (leading_r.min(leading_i) as isize).wrapping_add(n_level);

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> Self::fraction_bits()).as_i128().as_(),
                        (imaginary_wide >> Self::fraction_bits()).as_i128().as_(),
                    )
                }
                _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
            };
            return Circle {
                real,
                imaginary,
                exponent: Self::ambiguous_exponent(),
            };
        }

        let (real, imaginary, expo_adjust) = match Self::fraction_bits() {
            8 => {
                let a: i16 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                let c: i16 = other.real.as_();
                let d: i16 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u16;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i16;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = ad.wrapping_neg();
                let mut imaginary_wide =
                    (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = (leading_r.min(leading_i) as isize).wrapping_sub(1);

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> Self::fraction_bits()).as_(),
                    (imaginary_wide >> Self::fraction_bits()).as_(),
                    shift.wrapping_sub(1),
                )
            }
            16 => {
                let a: i32 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                let c: i32 = other.real.as_();
                let d: i32 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u32;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i32;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = ad.wrapping_neg();
                let mut imaginary_wide =
                    (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = (leading_r.min(leading_i) as isize).wrapping_sub(1);

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> Self::fraction_bits()).as_(),
                    (imaginary_wide >> Self::fraction_bits()).as_(),
                    shift.wrapping_sub(1),
                )
            }
            32 => {
                let a: i64 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                let c: i64 = other.real.as_();
                let d: i64 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u64;

                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i64;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = ad.wrapping_neg();
                let mut imaginary_wide =
                    (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = (leading_r.min(leading_i) as isize).wrapping_sub(1);

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> Self::fraction_bits()).as_(),
                    (imaginary_wide >> Self::fraction_bits()).as_(),
                    shift.wrapping_sub(1),
                )
            }
            64 => {
                let a: i128 = ((self.fraction >> 1isize) ^ F::min_value()).as_();
                let c: i128 = other.real.as_();
                let d: i128 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u128;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i128;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = ad.wrapping_neg();
                let mut imaginary_wide =
                    (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = (leading_r.min(leading_i) as isize).wrapping_sub(1);

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> Self::fraction_bits()).as_(),
                    (imaginary_wide >> Self::fraction_bits()).as_(),
                    shift.wrapping_sub(1),
                )
            }
            128 => {
                let a: I256 = ((self.fraction >> 1isize) ^ F::min_value()).into();
                let c: I256 = other.real.into();
                let d: I256 = other.imaginary.into();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)).as_unsigned();
                let one: I256 = 1.into();
                let one = one.as_unsigned();
                let reciprocal = ((one << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits()))
                .as_signed();

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = ad.wrapping_neg();
                let mut imaginary_wide =
                    (imaginary_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = (leading_r.min(leading_i) as isize).wrapping_sub(1);

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> Self::fraction_bits()).as_i128().as_(),
                    (imaginary_wide >> Self::fraction_bits()).as_i128().as_(),
                    shift.wrapping_sub(1),
                )
            }
            _ => {
                return Circle {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        };

        // AMBIG=0 native: scalar is converted N0→N1 at fraction load; -1 compensates for the inverted scale.
        let pa = self.exponent.cycle_widen();
        let pb = other.exponent.cycle_widen();
        let expo_adjust_e: E = expo_adjust.as_();
        let w_adj = expo_adjust_e.sign_extend();
        let w_bo = Scalar::<F, E>::binade_origin().cycle_widen();
        let w_one = E::one().cycle_widen();
        let stored_pos = pa.w_sub(pb).w_sub(w_adj).w_add(w_bo).w_sub(w_one);

        let max_pos = Scalar::<F, E>::max_exponent().cycle_widen();
        let min_pos = Scalar::<F, E>::min_exponent().cycle_widen();

        if stored_pos > max_pos {
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
        }
    }
}
