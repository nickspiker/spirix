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
    /// Multiplies a Scalar by a Circle
    ///
    /// # Description
    ///
    /// Performs multiplication between a Scalar and a Circle, scaling both real and imaginary components of the Circle by the Scalar value while preserving the Circle's orientation. This follows the standard scalar-complex multiplication formula: s·(a+b·i) = (s·a) + (s·b)·i.
    ///
    /// The operation scales the magnitude of the complex number while preserving its angle in the complex plane, equivalent to multiplying the magnitude by the absolute value of the Scalar and adjusting the angle if the Scalar is negative (rotation by π radians).
    ///
    /// # Special Cases
    ///
    /// - If either value is undefined, returns the first undefined state encountered
    /// - If either value is zero, returns zero (multiplicative annihilation)
    /// - If a positive exploded Scalar multiplies a positive exploded Circle, returns an exploded Circle
    /// - If a vanished Scalar multiplies an exploded Circle (or vice versa), returns an undefined state due to the magnitude being indeterminate
    ///
    /// # Returns
    ///
    /// - For normal values: A new Circle with scaled components
    /// - For special cases:
    ///   - `[℘ ]` → `[℘ ]` First undefined state encountered
    ///   - `[0]` × `[#]` or `[#]` × `[0]` → `[0]` Zero (multiplicative annihilation)
    ///   - `[↑]` × `[↓]` or `[↓]` × `[↑]` → `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    ///   - `[↑]` × `[#]` or `[#]` × `[↑]` → `[↑]` Exploded Circle with orientation following the multiplication rule
    ///   - `[↓]` × `[#]` or `[#]` × `[↓]` → `[↓]` Vanished Circle with orientation following the multiplication rule
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, Circle, ScalarF5E3, CircleF5E3};
    ///
    /// // Basic multiplication - scales both components let s = ScalarF5E3::from(2_i32); let z = CircleF5E3::from((3_i32, 4_i32)); let result = s * z; assert!(result.r() == 6_i32); assert!(result.i() == 8_i32);
    ///
    /// // Multiplying by negative Scalar negates the Circle let neg = ScalarF5E3::from(-1_i32); let negated = neg * z; assert!(negated.r() == -3_i32); assert!(negated.i() == -4_i32);
    ///
    /// // Multiplying by Zero produces Zero assert!((ScalarF5E3::ZERO * z).is_zero());
    ///
    /// // Multiplying with vanished Scalar let tiny: ScalarF5E3 = ScalarF5E3::MIN_POS / 10_i32; let tiny_result = tiny * z; assert!(tiny_result.vanished());
    /// ```
    pub(crate) fn scalar_multiply_circle(&self, other: &Circle<F, E>) -> Circle<F, E> {
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 unified pipeline. Scalar * Circle: scale each Circle component by the (N0→N1) Scalar fraction.
            let s = ((self.fraction >> 1isize) ^ F::min_value()).sign_extend();
            let c = other.real.sign_extend();
            let d = other.imaginary.sign_extend();
            let fb = Self::fraction_bits();

            let real_product = s.w_mul(c);
            let imag_product = s.w_mul(d);

            if real_product.w_is_zero() && imag_product.w_is_zero() {
                return Circle::<F, E>::ZERO;
            }

            let leading_r = real_product.leading_same();
            let leading_i = imag_product.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_product.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_product.w_shl(shift).w_shr(fb).deflate();

            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let expo_adjust_e: E = leading.wrapping_sub(2).as_();
            let w_adj = expo_adjust_e.sign_extend();
            let w_bo = Scalar::<F, E>::binade_origin().cycle_widen();
            let w_one = E::one().cycle_widen();
            let stored_pos = pa.w_add(pb).w_sub(w_adj).w_sub(w_bo).w_add(w_one);
            let max_pos = Scalar::<F, E>::max_exponent().cycle_widen();
            let min_pos = Scalar::<F, E>::min_exponent().cycle_widen();

            return if stored_pos > max_pos {
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
            };
        }

        // Escape-class handling (at least one operand non-normal).
        {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            } else if other.is_undefined() {
                return *other;
            } else if self.is_infinite() && other.is_zero() {
                return Circle {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_zero() && other.is_infinite() {
                return Circle {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_infinite() || other.is_infinite() {
                return Circle::<F, E>::INFINITY;
            } else if self.is_zero() || other.is_zero() {
                return Circle::<F, E>::ZERO;
            } else if self.exploded() && other.vanished() {
                return Circle {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.vanished() && other.exploded() {
                return Circle {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else {
                // Mixed escape: at least one of self/other is non-normal, and the early-return class checks above didn't apply. Compute the product directly with WideOps, then renormalize to the N-1 (one operand exploded) or N-2 (both vanished) shape at AMBIG exp. The N0→N1 conversion `(x>>1)^MIN` is only valid for normal-class fractions where the MSB implicitly encodes sign; vanished/exploded use top-3-bit (001/110) or top-2-bit (01/10) tags whose layout is identical between N0 and N1, so escape-class self uses its bit pattern directly.
                let n_level: isize = if self.exploded() || other.exploded() { -1 } else { -2 };
                let multiplicand_narrow: F = if self.is_normal() {
                    (self.fraction >> 1isize) ^ F::min_value()
                } else {
                    self.fraction
                };
                let m = multiplicand_narrow.sign_extend();
                let c = other.real.sign_extend();
                let d = other.imaginary.sign_extend();
                let product_r = c.w_mul(m);
                let product_i = d.w_mul(m);

                let leading_r = product_r.leading_same();
                let leading_i = product_i.leading_same();
                let leading = leading_r.min(leading_i);
                let shift = leading.wrapping_add(n_level);
                let fb = Self::fraction_bits();
                return Circle::<F, E> {
                    real: product_r.w_shl(shift).w_shr(fb).deflate(),
                    imaginary: product_i.w_shl(shift).w_shr(fb).deflate(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
    }
}
