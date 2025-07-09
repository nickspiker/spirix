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
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Circle::<F, E>::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Circle {
                        real: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        imaginary: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && other.vanished() {
                return Circle {
                    real: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    imaginary: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let n_level = if self.vanished() || other.exploded() {
                -2
            } else {
                -1
            };
            let (real, imaginary) = match F::FRACTION_BITS {
                8 => {
                    let a: i16 = self.fraction.as_();
                    let c: i16 = other.real.as_();
                    let d: i16 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc + dd) as u16;
                    let reciprocal =
                        ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i16;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = -ad;
                    let mut imaginary_wide =
                        (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = leading_r.min(leading_i) as isize + n_level;

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> F::FRACTION_BITS).as_(),
                        (imaginary_wide >> F::FRACTION_BITS).as_(),
                    )
                }
                16 => {
                    let a: i32 = self.fraction.as_();
                    let c: i32 = other.real.as_();
                    let d: i32 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc + dd) as u16;
                    let reciprocal =
                        ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i32;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = -ad;
                    let mut imaginary_wide =
                        (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = leading_r.min(leading_i) as isize + n_level;

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> F::FRACTION_BITS).as_(),
                        (imaginary_wide >> F::FRACTION_BITS).as_(),
                    )
                }
                32 => {
                    let a: i64 = self.fraction.as_();
                    let c: i64 = other.real.as_();
                    let d: i64 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc + dd) as u16;
                    let reciprocal =
                        ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i64;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = -ad;
                    let mut imaginary_wide =
                        (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = leading_r.min(leading_i) as isize + n_level;

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> F::FRACTION_BITS).as_(),
                        (imaginary_wide >> F::FRACTION_BITS).as_(),
                    )
                }
                64 => {
                    let a: i128 = self.fraction.as_();
                    let c: i128 = other.real.as_();
                    let d: i128 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc + dd) as u16;
                    let reciprocal =
                        ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i128;

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = -ad;
                    let mut imaginary_wide =
                        (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = leading_r.min(leading_i) as isize + n_level;

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> F::FRACTION_BITS).as_(),
                        (imaginary_wide >> F::FRACTION_BITS).as_(),
                    )
                }
                128 => {
                    let a: I256 = self.fraction.into();
                    let c: I256 = other.real.into();
                    let d: I256 = other.imaginary.into();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc + dd).as_unsigned();
                    let one: I256 = 1.into();
                    let one = one.as_unsigned();
                    let reciprocal = ((one << (F::FRACTION_BITS * 2 - 2))
                        / (mag_sq >> F::FRACTION_BITS))
                        .as_signed();

                    let ac = a.wrapping_mul(c);
                    let real_numerator = ac;
                    let mut real_wide =
                        (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let imaginary_numerator = -ad;
                    let mut imaginary_wide =
                        (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                    let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                    let leading_i = imaginary_wide
                        .leading_ones()
                        .max(imaginary_wide.leading_zeros());
                    let shift = leading_r.min(leading_i) as isize + n_level;

                    real_wide <<= shift;
                    imaginary_wide <<= shift;

                    (
                        (real_wide >> F::FRACTION_BITS).as_i128().as_(),
                        (imaginary_wide >> F::FRACTION_BITS).as_i128().as_(),
                    )
                }
                _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
            };
            return Circle {
                real,
                imaginary,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let (real, imaginary, expo_adjust) = match F::FRACTION_BITS {
            8 => {
                let a: i16 = self.fraction.as_();
                let c: i16 = other.real.as_();
                let d: i16 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc + dd) as u16;
                let reciprocal =
                    ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i16;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide = (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = -ad;
                let mut imaginary_wide =
                    (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = leading_r.min(leading_i) as isize - 1;

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> F::FRACTION_BITS).as_(),
                    (imaginary_wide >> F::FRACTION_BITS).as_(),
                    shift - 1,
                )
            }
            16 => {
                let a: i32 = self.fraction.as_();
                let c: i32 = other.real.as_();
                let d: i32 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc + dd) as u32;
                let reciprocal =
                    ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i32;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide = (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = -ad;
                let mut imaginary_wide =
                    (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = leading_r.min(leading_i) as isize - 1;

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> F::FRACTION_BITS).as_(),
                    (imaginary_wide >> F::FRACTION_BITS).as_(),
                    shift - 1,
                )
            }
            32 => {
                let a: i64 = self.fraction.as_();
                let c: i64 = other.real.as_();
                let d: i64 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc + dd) as u64;

                let reciprocal =
                    ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i64;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide = (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = -ad;
                let mut imaginary_wide =
                    (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = leading_r.min(leading_i) as isize - 1;

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> F::FRACTION_BITS).as_(),
                    (imaginary_wide >> F::FRACTION_BITS).as_(),
                    shift - 1,
                )
            }
            64 => {
                let a: i128 = self.fraction.as_();
                let c: i128 = other.real.as_();
                let d: i128 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc + dd) as u128;
                let reciprocal =
                    ((1 << (F::FRACTION_BITS * 2 - 2)) / (mag_sq >> F::FRACTION_BITS)) as i128;

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide = (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = -ad;
                let mut imaginary_wide =
                    (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = leading_r.min(leading_i) as isize - 1;

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> F::FRACTION_BITS).as_(),
                    (imaginary_wide >> F::FRACTION_BITS).as_(),
                    shift - 1,
                )
            }
            128 => {
                let a: I256 = self.fraction.into();
                let c: I256 = other.real.into();
                let d: I256 = other.imaginary.into();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc + dd).as_unsigned();
                let one: I256 = 1.into();
                let one = one.as_unsigned();
                let reciprocal = ((one << (F::FRACTION_BITS * 2 - 2))
                    / (mag_sq >> F::FRACTION_BITS))
                    .as_signed();

                let ac = a.wrapping_mul(c);
                let real_numerator = ac;
                let mut real_wide = (real_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let imaginary_numerator = -ad;
                let mut imaginary_wide =
                    (imaginary_numerator >> F::FRACTION_BITS).wrapping_mul(reciprocal);

                let leading_r = real_wide.leading_ones().max(real_wide.leading_zeros());
                let leading_i = imaginary_wide
                    .leading_ones()
                    .max(imaginary_wide.leading_zeros());
                let shift = leading_r.min(leading_i) as isize - 1;

                real_wide <<= shift;
                imaginary_wide <<= shift;

                (
                    (real_wide >> F::FRACTION_BITS).as_i128().as_(),
                    (imaginary_wide >> F::FRACTION_BITS).as_i128().as_(),
                    shift - 1,
                )
            }
            _ => {
                return Circle {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        };

        match E::EXPONENT_BITS {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let other_exponent: i16 = other.exponent.as_();
                let upcast_exponent: i16 = self_exponent - other_exponent - expo_adjust as i16;

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Circle {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Circle {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Circle {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let other_exponent: i32 = other.exponent.as_();
                let upcast_exponent: i32 = self_exponent - other_exponent - expo_adjust as i32;

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Circle {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Circle {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Circle {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            32 => {
                let self_exponent: i64 = self.exponent.as_();
                let other_exponent: i64 = other.exponent.as_();
                let upcast_exponent: i64 = self_exponent - other_exponent - expo_adjust as i64;

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Circle {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Circle {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Circle {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            64 => {
                let self_exponent: i128 = self.exponent.as_();
                let other_exponent: i128 = other.exponent.as_();
                let upcast_exponent: i128 = self_exponent - other_exponent - expo_adjust as i128;

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Circle {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Circle {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Circle {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            128 => {
                let self_exponent: I256 = self.exponent.into();
                let other_exponent: I256 = other.exponent.into();
                let exp_adj: I256 = (expo_adjust as i128).into();
                let upcast_exponent: I256 = self_exponent - other_exponent - exp_adj;
                let max_e: I256 = E::MAX_EXPONENT.into();
                let min_e: I256 = E::MIN_EXPONENT.into();
                if upcast_exponent > max_e {
                    return Circle {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Circle {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Circle {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Circle {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
}
