use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
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
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Self {
                        fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
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
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            // Escape paths (van/normal, normal/van, exp/normal, normal/exp, van/exp, exp/van): compute (n << FRAC-1) / d in wide form using sign_extend for escapes (raw stored bits) and inflate for normals (~MSB convention), then normalize toward N-1 (exploded) or N-2 (vanished) shape, then >> FRAC and deflate to extract the stored pattern — matches the multiplication escape path.
            let n = self.fraction.inflate(self.is_normal());
            let d = other.fraction.inflate(other.is_normal());
            let fb = Self::fraction_bits();
            let q = n.w_shl(fb.wrapping_sub(1)).w_div(d);
            let leading = q.leading_same();
            let result_vanished = self.vanished() || other.exploded();
            let target_n: isize = if result_vanished { 2 } else { 1 };
            let shift = leading.wrapping_sub(target_n);
            let fraction = if shift >= 0 {
                q.w_shl(shift).w_shr(fb).deflate()
            } else {
                q.w_shr(fb.wrapping_sub(shift)).deflate()
            };
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Main path: both normal. Inflated N0 fractions reach magnitude 2^FRAC at the ±1.0 boundary, so shifting the numerator left by FRAC bits would need 2*FRAC+1 signed bits — one more than our 2*FRAC-bit Wide can hold. Pre-shifting by FRAC-1 keeps the intermediate inside the wide and pushes the missing factor of 2 into the exponent term (+1 below).
        let n = self.fraction.inflate(true);
        let d = other.fraction.inflate(true);
        let fb = Self::fraction_bits();
        let q = n.w_shl(fb.wrapping_sub(1)).w_div(d);
        let leading = q.leading_same();
        // Normalize so |q'| ~ 2^FRAC (storable inflated range). |q| is in [2^(FRAC-2), 2^FRAC], giving leading_same ∈ [FRAC, FRAC+2] and shift ∈ [0, 2]. Always non-negative, so no conditional shift needed.
        let shift = leading.wrapping_sub(fb);
        debug_assert!(shift >= 0);
        let fraction = q.w_shl(shift).deflate();

        // AMBIG=0 native: stored_pos = pa - pb + bo - shift. The +bo is the division's bias correction (subtracting two stored exps cancels both binade_origin offsets, so we add one back). `cycle_widen` zero-extends each exponent into <E as Inflate>::Wide for clean wrap detection at any E width.
        let pa = self.exponent.cycle_widen();
        let pb = other.exponent.cycle_widen();
        let bo = Self::binade_origin().cycle_widen();
        let max_pos = Self::max_exponent().cycle_widen();
        let min_pos = Self::min_exponent().cycle_widen();
        let shift_e: E = shift.as_();
        let w_shift = shift_e.cycle_widen();
        let stored_pos = pa.w_sub(pb).w_add(bo).w_sub(w_shift);
        let result_neg = q.w_is_negative();
        if stored_pos > max_pos {
            return Self {
                fraction: if result_neg {
                    Self::neg_one_exploded()
                } else {
                    Self::pos_one_exploded()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        if stored_pos < min_pos {
            return Self {
                fraction: if result_neg {
                    Self::neg_one_vanished()
                } else {
                    Self::pos_one_vanished()
                },
                exponent: Self::ambiguous_exponent(),
            };
        }
        let exponent: E = stored_pos.deflate();
        Self { fraction, exponent }
    }

    pub fn reciprocal(&self) -> Self {
        Self::ONE / self
    }
}
