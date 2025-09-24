use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
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
    pub fn sqrt(&self) -> Self {
        if self.is_undefined() || self.is_n0() {
            return *self;
        }
        let magnitude = self.magnitude();
        let mut real = magnitude + self.r();
        real = real / 2;
        real = real.sqrt();
        let mut imaginary = magnitude - self.r();
        imaginary = imaginary / 2;
        imaginary = imaginary.sqrt();
        if self.imaginary.is_negative() {
            imaginary = -imaginary;
        }
        Circle::from((real, imaginary))
    }
    pub fn square(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() || self.is_n0() {
                return *self;
            } else {
                let n_level: isize = if self.exploded() { -1 } else { -2 };

                let (product_real, product_imaginary) = match F::FRACTION_BITS {
                    8 => {
                        let r: i16 = self.real.as_();
                        let i: i16 = self.imaginary.as_();

                        let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                        let imag_product = r.wrapping_mul(i);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift + n_level;
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> F::FRACTION_BITS).as_(),
                            (normalized_imag >> F::FRACTION_BITS).as_(),
                        )
                    }
                    16 => {
                        let r: i32 = self.real.as_();
                        let i: i32 = self.imaginary.as_();

                        let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                        let imag_product = r.wrapping_mul(i);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift + n_level;
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> F::FRACTION_BITS).as_(),
                            (normalized_imag >> F::FRACTION_BITS).as_(),
                        )
                    }
                    32 => {
                        let r: i64 = self.real.as_();
                        let i: i64 = self.imaginary.as_();

                        let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                        let imag_product = r.wrapping_mul(i);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift + n_level;
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> F::FRACTION_BITS).as_(),
                            (normalized_imag >> F::FRACTION_BITS).as_(),
                        )
                    }
                    64 => {
                        let r: i128 = self.real.as_();
                        let i: i128 = self.imaginary.as_();

                        let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                        let imag_product = r.wrapping_mul(i);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift + n_level;
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> F::FRACTION_BITS).as_(),
                            (normalized_imag >> F::FRACTION_BITS).as_(),
                        )
                    }
                    128 => {
                        let r: i128 = self.real.as_();
                        let i: i128 = self.imaginary.as_();

                        let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                        let imag_product = r.wrapping_mul(i);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift + n_level;
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> F::FRACTION_BITS).as_(),
                            (normalized_imag >> F::FRACTION_BITS).as_(),
                        )
                    }
                    _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
                };

                // Check if the calculation resulted in zero due to overflow
                if product_real == F::ZERO && product_imaginary == F::ZERO {
                    if self.exploded() && self.imaginary == F::ZERO {
                        return Self {
                            real: self.real,  // Keep exploded magnitude
                            imaginary: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                }

                return Self {
                    real: product_real,
                    imaginary: product_imaginary,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        let real;
        let imaginary;
        let expo_adjust: isize;

        match F::FRACTION_BITS {
            8 => {
                let r: i16 = self.real.as_();
                let i: i16 = self.imaginary.as_();

                let real_product = (r.wrapping_mul(r) >> 1).wrapping_sub(i.wrapping_mul(i) >> 1);
                let imag_product = r.wrapping_mul(i);

                if real_product == 0 && imag_product == 0 {
                    // Check if this zero is from overflow of exploded positive real
                    if self.exploded() && self.imaginary == F::ZERO {
                        return Self {
                            real: self.real,  // Keep the exploded magnitude
                            imaginary: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> F::FRACTION_BITS).as_();
                imaginary = (normalized_imag >> F::FRACTION_BITS).as_();
            }
            16 => {
                let r: i32 = self.real.as_();
                let i: i32 = self.imaginary.as_();

                let real_product = (r.wrapping_mul(r) >> 1) - (i.wrapping_mul(i) >> 1);
                let imag_product = r.wrapping_mul(i);

                if real_product == 0 && imag_product == 0 {
                    // Check if this zero is from overflow of exploded positive real
                    if self.exploded() && self.imaginary == F::ZERO {
                        return Self {
                            real: self.real,  // Keep the exploded magnitude
                            imaginary: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> F::FRACTION_BITS).as_();
                imaginary = (normalized_imag >> F::FRACTION_BITS).as_();
            }
            32 => {
                let r: i64 = self.real.as_();
                let i: i64 = self.imaginary.as_();

                let real_product = (r.wrapping_mul(r) >> 1) - (i.wrapping_mul(i) >> 1);
                let imag_product = r.wrapping_mul(i);

                if real_product == 0 && imag_product == 0 {
                    // Check if this zero is from overflow of exploded positive real
                    if self.exploded() && self.imaginary == F::ZERO {
                        return Self {
                            real: self.real,  // Keep the exploded magnitude
                            imaginary: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> F::FRACTION_BITS).as_();
                imaginary = (normalized_imag >> F::FRACTION_BITS).as_();
            }
            64 => {
                let r: i128 = self.real.as_();
                let i: i128 = self.imaginary.as_();

                let real_product = (r.wrapping_mul(r) >> 1) - (i.wrapping_mul(i) >> 1);
                let imag_product = r.wrapping_mul(i);

                if real_product == 0 && imag_product == 0 {
                    // Check if this zero is from overflow of exploded positive real
                    if self.exploded() && self.imaginary == F::ZERO {
                        return Self {
                            real: self.real,  // Keep the exploded magnitude
                            imaginary: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> F::FRACTION_BITS).as_();
                imaginary = (normalized_imag >> F::FRACTION_BITS).as_();
            }
            128 => {
                let r: I256 = self.real.into();
                let i: I256 = self.imaginary.into();

                let real_product: I256 = (r.wrapping_mul(r) >> 1) - (i.wrapping_mul(i) >> 1);
                let imag_product: I256 = r.wrapping_mul(i);

                if real_product == 0.into() && imag_product == 0.into() {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> F::FRACTION_BITS).as_i128().as_();
                imaginary = (normalized_imag >> F::FRACTION_BITS).as_i128().as_();
            }
            _ => {
                return Self {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        match E::EXPONENT_BITS {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let upcast_exponent: i16 = (self_exponent << 1).wrapping_sub(expo_adjust as i16);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                let upcast_exponent: i32 = (self_exponent << 1).wrapping_sub(expo_adjust as i32);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                let upcast_exponent: i64 = (self_exponent << 1).wrapping_sub(expo_adjust as i64);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                let upcast_exponent: i128 = (self_exponent << 1).wrapping_sub(expo_adjust as i128);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                let e: I256 = (expo_adjust as i128).into();
                let upcast_exponent: I256 = (self_exponent << 1usize).wrapping_sub(e);

                if upcast_exponent > E::MAX_EXPONENT.into() {
                    return Self {
                        real,
                        imaginary,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.into() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
    pub fn ln(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }

            if self.is_zero() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        Self::from((self.magnitude().ln(), self.i().atan2(self.r())))
    }

    pub fn exp(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.is_zero() {
                return Self::ONE;
            }
            if self.vanished() {
                let prefix: F = POWER_NEGLIGIBLE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = POWER_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
        Circle::from((
            self.r().exp() * self.i().cos(),
            self.r().exp() * self.i().sin(),
        ))
    }
}
