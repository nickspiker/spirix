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
        // Escaped: sqrt halves the (lost) magnitude exponent, which can land back INSIDE normal range (sqrt of 2^200 = 2^100) — class indeterminate → ℘√↑ / ℘√↓ (was falling into the magnitude+add path below and leaking a ℘⬆+⬆ addition tag).
        if self.exploded() || self.vanished() {
            let prefix: F = if self.exploded() {
                SQRT_EXPLODED.prefix.sa()
            } else {
                SQRT_VANISHED.prefix.sa()
            };
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        let magnitude = self.magnitude();
        // |z| ≥ |Re(z)| is an identity, so any negative half-angle operand is floor-rounding error (worst case: pure-real input where |z| rounds a hair below |Re|, sending sqrt a tiny negative → spurious ℘√-). Clamp to zero.
        let mut real = magnitude + self.r();
        if real.is_negative() {
            real = Scalar::<F, E>::ZERO;
        }
        real = real >> 1;
        real = real.sqrt();
        let mut imaginary = magnitude - self.r();
        if imaginary.is_negative() {
            imaginary = Scalar::<F, E>::ZERO;
        }
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
            // ln(0) is unboundedly negative-real — the Scalar convention maps it to the singular ∞ ([0] → [∞] in the log table); ln(∞) likewise (was falling thru to the normal path and leaking a ℘⬆/⬆ division tag).
            if self.is_zero() || self.is_infinite() {
                return Self::INFINITY;
            }
            // Escaped: ln|z| is huge-with-magnitude-lost while iθ is finite and known — one unrepresentable component poisons the pair → ℘, tagged by which class.
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
            // e^(tiny·direction) = 1 + tiny ≈ 1 for any direction — definite, matching the Scalar table's [±↓] → ≈1.
            if self.vanished() {
                return Self::ONE;
            }
            // Exploded: |e^z| = e^Re(z), and Re(z)'s SIGN is stored in the orientation even tho the magnitude is lost. Re < 0 → e^(-huge) = 0, definite (Scalar parallel: exp(-↑) = 0). Re ≥ 0 → the result's angle Im(z) mod 2π is unknowable → ℘^⬆. Circle components are plain two's complement, so the raw sign test is correct here.
            if self.exploded() {
                if self.real.is_negative() {
                    return Self::ZERO;
                }
                let prefix: F = POWER_TRANSFINITE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Infinity: match the Scalar convention exp(∞) = ∞ (was falling thru to the normal path and leaking a ℘c cosine tag).
            if self.is_infinite() {
                return Self::INFINITY;
            }
        }
        Circle::from((
            self.r().exp() * self.i().cos(),
            self.r().exp() * self.i().sin(),
        ))
    }
}
