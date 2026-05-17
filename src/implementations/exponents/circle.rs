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
    pub fn sqrt(&self) -> Self {
        if self.is_undefined() || self.is_n0() {
            return *self;
        }
        let magnitude = self.magnitude();
        let mut real = magnitude + self.r();
        real = real >> 1;
        real = real.sqrt();
        let mut imaginary = magnitude - self.r();
        imaginary = imaginary >> 1;
        imaginary = imaginary.sqrt();
        if self.imaginary.is_negative() {
            imaginary = -imaginary;
        }
        Circle::from((real, imaginary))
    }
    pub fn square(&self) -> Self {
        if self.is_normal() {
            // AMBIG=0 native unified pipeline. (r+ii)² = (r²-i²) + 2ri·i. Per-product >> 1 keeps wide range fits.
            let r = self.real.sign_extend();
            let i = self.imaginary.sign_extend();
            let fb = Self::fraction_bits();
            let real_product = r.w_mul(r).w_shr(1).w_sub(i.w_mul(i).w_shr(1));
            let imag_product = r.w_mul(i);

            if real_product.w_is_zero() && imag_product.w_is_zero() {
                return Self::ZERO;
            }

            let leading_r = real_product.leading_same();
            let leading_i = imag_product.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_product.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_product.w_shl(shift).w_shr(fb).deflate();

            // Squaring: stored_pos = 2*pa - expo_adjust - bo + 1.
            let pa = self.exponent.cycle_widen();
            let expo_adjust_e: E = leading.wrapping_sub(3).as_();
            let w_adj = expo_adjust_e.sign_extend();
            let w_bo = Self::binade_origin().cycle_widen();
            let w_one = E::one().cycle_widen();
            let stored_pos = pa.w_add(pa).w_sub(w_adj).w_sub(w_bo).w_add(w_one);
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
        if self.is_undefined() || self.is_n0() {
            return *self;
        }

        let n_level: isize = if self.exploded() { -1 } else { -2 };
        let r = self.real.sign_extend();
        let i = self.imaginary.sign_extend();
        let fb = Self::fraction_bits();
        let real_product = r.w_mul(r).w_shr(1).w_sub(i.w_mul(i).w_shr(1));
        let imag_product = r.w_mul(i);

        let leading_r = real_product.leading_same();
        let leading_i = imag_product.leading_same();
        let leading = leading_r.min(leading_i);
        let shift = leading.wrapping_add(n_level);
        let product_real = real_product.w_shl(shift).w_shr(fb).deflate();
        let product_imaginary = imag_product.w_shl(shift).w_shr(fb).deflate();

        // Overflow of exploded positive-real squared can collapse to zero — preserve the original exploded magnitude in that case.
        if product_real == F::zero()
            && product_imaginary == F::zero()
            && self.exploded()
            && self.imaginary == F::zero()
        {
            return Self {
                real: self.real,
                imaginary: F::zero(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        return Self {
            real: product_real,
            imaginary: product_imaginary,
            exponent: Self::ambiguous_exponent(),
        };
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
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                let prefix: F = NEGLIGIBLE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_LOG.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
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
                let prefix: F = POWER_VANISHED.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                let prefix: F = POWER_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
        Circle::from((
            self.r().exp() * self.i().cos(),
            self.r().exp() * self.i().sin(),
        ))
    }
}
