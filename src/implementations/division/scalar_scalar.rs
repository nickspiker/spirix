use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + ExponentConstants
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
    pub(crate) fn scalar_divide_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if other.is_zero() {
                if self.is_zero() {
                    return Self {
                        fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Self {
                        fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_infinite() {
                return Self::ZERO;
            }
            if self.exploded() && other.exploded() {
                return Self {
                    fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let fraction = match F::FRACTION_BITS {
                    8 => {
                        let numerator: i16 = self.fraction.as_();
                        let denominator: i16 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    32 => {
                        let numerator: i64 = self.fraction.as_();
                        let denominator: i64 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    64 => {
                        let numerator: i128 = self.fraction.as_();
                        let denominator: i128 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    128 => {
                        let numerator: I256 = self.fraction.into();
                        let denominator: I256 = other.fraction.into();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_i128().as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if other.exploded() {
                let fraction = match F::FRACTION_BITS {
                    8 => {
                        let numerator: i16 = self.fraction.as_();
                        let denominator: i16 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    32 => {
                        let numerator: i64 = self.fraction.as_();
                        let denominator: i64 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    64 => {
                        let numerator: i128 = self.fraction.as_();
                        let denominator: i128 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    128 => {
                        let numerator: I256 = self.fraction.into();
                        let denominator: I256 = other.fraction.into();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_i128().as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let fraction = match F::FRACTION_BITS {
                8 => {
                    let numerator: i16 = self.fraction.as_();
                    let denominator: i16 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                16 => {
                    let numerator: i32 = self.fraction.as_();
                    let denominator: i32 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                32 => {
                    let numerator: i64 = self.fraction.as_();
                    let denominator: i64 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                64 => {
                    let numerator: i128 = self.fraction.as_();
                    let denominator: i128 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                128 => {
                    let numerator: I256 = self.fraction.into();
                    let denominator: I256 = other.fraction.into();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_i128().as_()
                }
                _ => GENERAL.prefix.sa(),
            };
            return Self {
                fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let (fraction, expo_adjust) = match F::FRACTION_BITS {
            8 => {
                let numerator: i16 = self.fraction.as_();
                let denominator: i16 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            16 => {
                let numerator: i32 = self.fraction.as_();
                let denominator: i32 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            32 => {
                let numerator: i64 = self.fraction.as_();
                let denominator: i64 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            64 => {
                let numerator: i128 = self.fraction.as_();
                let denominator: i128 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            128 => {
                let numerator: I256 = self.fraction.into();
                let denominator: I256 = other.fraction.into();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_i128().as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        };

        match E::EXPONENT_BITS {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let other_exponent: i16 = other.exponent.as_();
                let upcast_exponent: i16 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i16);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
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

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
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

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
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

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
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
                let max_e: I256 = E::MAX_EXPONENT.into();
                let min_e: I256 = E::MIN_EXPONENT.into();
                if upcast_exponent > max_e {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
    pub fn reciprocal(&self) -> Self {
        Self::ONE / self
    }
}
