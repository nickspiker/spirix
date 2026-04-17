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
        if !self.is_normal() || !other.is_normal() {
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
            let n_level = if self.vanished() || other.exploded() {
                -2
            } else {
                -1
            };
            let (real, imaginary) = match Self::fraction_bits() {
                8 => {
                    let a: i16 = self.real.as_();
                    let b: i16 = self.imaginary.as_();
                    let c: i16 = other.real.as_();
                    let d: i16 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u16;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i16;

                    let ac = a.wrapping_mul(c);
                    let bd = b.wrapping_mul(d);
                    let real_numerator = ac.wrapping_add(bd);
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let bc = b.wrapping_mul(c);
                    let imaginary_numerator = bc.wrapping_sub(ad);
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
                    let a: i32 = self.real.as_();
                    let b: i32 = self.imaginary.as_();
                    let c: i32 = other.real.as_();
                    let d: i32 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u32;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i32;

                    let ac = a.wrapping_mul(c);
                    let bd = b.wrapping_mul(d);
                    let real_numerator = ac.wrapping_add(bd);
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let bc = b.wrapping_mul(c);
                    let imaginary_numerator = bc.wrapping_sub(ad);
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
                    let a: i64 = self.real.as_();
                    let b: i64 = self.imaginary.as_();
                    let c: i64 = other.real.as_();
                    let d: i64 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u64;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i64;

                    let ac = a.wrapping_mul(c);
                    let bd = b.wrapping_mul(d);
                    let real_numerator = ac.wrapping_add(bd);
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let bc = b.wrapping_mul(c);
                    let imaginary_numerator = bc.wrapping_sub(ad);
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
                    let a: i128 = self.real.as_();
                    let b: i128 = self.imaginary.as_();
                    let c: i128 = other.real.as_();
                    let d: i128 = other.imaginary.as_();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq = (cc.wrapping_add(dd)) as u128;
                    let reciprocal =
                        ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                            / (mag_sq >> Self::fraction_bits())) as i128;

                    let ac = a.wrapping_mul(c);
                    let bd = b.wrapping_mul(d);
                    let real_numerator = ac.wrapping_add(bd);
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let bc = b.wrapping_mul(c);
                    let imaginary_numerator = bc.wrapping_sub(ad);
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
                    let a: I256 = self.real.into();
                    let b: I256 = self.imaginary.into();
                    let c: I256 = other.real.into();
                    let d: I256 = other.imaginary.into();

                    let cc = c.wrapping_mul(c);
                    let dd = d.wrapping_mul(d);
                    let mag_sq: U256 = (cc.wrapping_add(dd)).as_unsigned();
                    let one: I256 = 1.into();
                    let one: U256 = one.as_unsigned();
                    let reciprocal = (one
                        << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                        / (mag_sq >> Self::fraction_bits());
                    let reciprocal = reciprocal.as_signed();

                    let ac = a.wrapping_mul(c);
                    let bd = b.wrapping_mul(d);
                    let real_numerator = ac.wrapping_add(bd);
                    let mut real_wide =
                        (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                    let ad = a.wrapping_mul(d);
                    let bc = b.wrapping_mul(c);
                    let imaginary_numerator = bc.wrapping_sub(ad);
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
            return Self {
                real,
                imaginary,
                exponent: Self::ambiguous_exponent(),
            };
        }

        let (real, imaginary, expo_adjust) = match Self::fraction_bits() {
            8 => {
                let a: i16 = self.real.as_();
                let b: i16 = self.imaginary.as_();
                let c: i16 = other.real.as_();
                let d: i16 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u16;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i16;

                let ac = a.wrapping_mul(c);
                let bd = b.wrapping_mul(d);
                let real_numerator = ac.wrapping_add(bd);
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let bc = b.wrapping_mul(c);
                let imaginary_numerator = bc.wrapping_sub(ad);
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
                let a: i32 = self.real.as_();
                let b: i32 = self.imaginary.as_();
                let c: i32 = other.real.as_();
                let d: i32 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u32;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i32;

                let ac = a.wrapping_mul(c);
                let bd = b.wrapping_mul(d);
                let real_numerator = ac.wrapping_add(bd);
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let bc = b.wrapping_mul(c);
                let imaginary_numerator = bc.wrapping_sub(ad);
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
                let a: i64 = self.real.as_();
                let b: i64 = self.imaginary.as_();
                let c: i64 = other.real.as_();
                let d: i64 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u64;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i64;

                let ac = a.wrapping_mul(c);
                let bd = b.wrapping_mul(d);
                let real_numerator = ac.wrapping_add(bd);
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let bc = b.wrapping_mul(c);
                let imaginary_numerator = bc.wrapping_sub(ad);
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
                let a: i128 = self.real.as_();
                let b: i128 = self.imaginary.as_();
                let c: i128 = other.real.as_();
                let d: i128 = other.imaginary.as_();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq = (cc.wrapping_add(dd)) as u128;
                let reciprocal = ((1 << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits())) as i128;

                let ac = a.wrapping_mul(c);
                let bd = b.wrapping_mul(d);
                let real_numerator = ac.wrapping_add(bd);
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let bc = b.wrapping_mul(c);
                let imaginary_numerator = bc.wrapping_sub(ad);
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
                let a: I256 = self.real.into();
                let b: I256 = self.imaginary.into();
                let c: I256 = other.real.into();
                let d: I256 = other.imaginary.into();

                let cc = c.wrapping_mul(c);
                let dd = d.wrapping_mul(d);
                let mag_sq: U256 = (cc.wrapping_add(dd)).as_unsigned();
                let one: I256 = 1.into();
                let one: U256 = one.as_unsigned();
                let reciprocal = (one << (Self::fraction_bits().wrapping_shl(1).wrapping_sub(2)))
                    / (mag_sq >> Self::fraction_bits());
                let reciprocal = reciprocal.as_signed();

                let ac = a.wrapping_mul(c);
                let bd = b.wrapping_mul(d);
                let real_numerator = ac.wrapping_add(bd);
                let mut real_wide =
                    (real_numerator >> Self::fraction_bits()).wrapping_mul(reciprocal);
                let ad = a.wrapping_mul(d);
                let bc = b.wrapping_mul(c);
                let imaginary_numerator = bc.wrapping_sub(ad);
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
                return Self {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        };

        match Self::exponent_bits() {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let other_exponent: i16 = other.exponent.as_();
                let upcast_exponent: i16 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i16);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let other_exponent: i32 = other.exponent.as_();
                let upcast_exponent: i32 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i32);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            32 => {
                let self_exponent: i64 = self.exponent.as_();
                let other_exponent: i64 = other.exponent.as_();
                let upcast_exponent: i64 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i64);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            64 => {
                let self_exponent: i128 = self.exponent.as_();
                let other_exponent: i128 = other.exponent.as_();
                let upcast_exponent: i128 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i128);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
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
                let upcast_exponent: I256 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(exp_adj);
                let max_e: I256 = Self::max_exponent().into();
                let min_e: I256 = Self::min_exponent().into();
                if upcast_exponent > max_e {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < min_e {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Self {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
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
