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
    /// Subtracts another Circle from this Circle
    ///
    /// # Description
    ///
    /// Performs subtraction between two Circles, handling special cases according to mathematical principles. The operation subtracts both the real and imaginary components separately while maintaining fractional alignment and adjusting exponent accordingly. Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) Circles and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the aligned values for real and imaginary components
    /// 4. Normalizes result and adjusts exponent
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` - `[↑]` ➔ `[℘ ↑-↑]` Undefined exploded minus exploded state
    /// - `[↑]` - `[#]` ➔ `[℘ ↑-]` Undefined exploded minus finite state
    /// - `[#]` - `[↑]` ➔ `[℘ -↑]` Undefined finite minus exploded state
    /// - `[↓]` - `[↓]` ➔ `[℘ ↓-↓]` Undefined vanished minus vanished state
    /// - `[↓]` - `[#]` ➔ `[-#]` Negative of the finite Circle
    /// - `[#]` - `[↓]` ➔ `[#]` The finite Circle
    /// - `[0]` - `[#]` ➔ `[-#]` Negative of the Circle
    /// - `[#]` - `[0]` ➔ `[#]` The Circle
    /// - `[0]` - `[0]` ➔ `[0]` Zero
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Subtracting finite Circles let a = Circle::<i32, i8>::from((3_i32, 4_i32)); let b = CircleF5E3::from((1_i32, 2_i32)); let diff = a - b; assert!(diff.r() == 2_i32); assert!(diff.i() == 2_i32);
    ///
    /// // Subtracting with Zero assert!(a - Circle::<i32, i8>::ZERO == a); let neg_a = -a; assert!(Circle::<i32, i8>::ZERO - a == neg_a);
    ///
    /// // Subtraction that produces Zero assert!((a - a).is_zero());
    ///
    /// // Subtraction with vanished Circles let tiny: CircleF5E3 = CircleF5E3::MIN_POS / 4_i32; assert!(tiny.vanished());
    ///
    /// // Subtraction with exploded Circles let huge: CircleF5E3 = CircleF5E3::MAX * 4_i32; assert!(huge.exploded()); assert!((a - huge).is_undefined());
    ///
    /// // Subtracting two exploded Circles assert!((huge - huge).is_undefined());
    /// ```
    pub(crate) fn circle_subtract_circle(&self, circle: &Circle<F, E>) -> Circle<F, E> {
        if self.is_normal() && circle.is_normal() {
            // AMBIG=0 native: dominance via unsigned-cyclic compare. self_is_big tracks the subtraction order so the wide sub produces the right sign.
            let self_is_big = self.exponent.into_unsigned() > circle.exponent.into_unsigned();
            let (big, small) = if self_is_big {
                (self, circle)
            } else {
                (circle, self)
            };

            let exp_diff = big.exponent.wrapping_sub(&small.exponent);
            let frac_bits_e: E = Self::fraction_bits().as_();
            if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
                return if self_is_big { *self } else { -circle };
            }

            let shift: isize = exp_diff.saturate();
            let big_r = big.real.sign_extend().w_shl(shift);
            let big_i = big.imaginary.sign_extend().w_shl(shift);
            let small_r = small.real.sign_extend();
            let small_i = small.imaginary.sign_extend();
            // self - circle: when self is big, big_aligned - small_native; when circle is big, small_native - big_aligned.
            let (result_r, result_i) = if self_is_big {
                (big_r.w_sub(small_r), big_i.w_sub(small_i))
            } else {
                (small_r.w_sub(big_r), small_i.w_sub(big_i))
            };

            if result_r.w_is_zero() && result_i.w_is_zero() {
                return Circle::<F, E>::ZERO;
            }

            let leading_r = result_r.leading_same();
            let leading_i = result_i.leading_same();
            let leading = leading_r.min(leading_i);

            // Result exp = small.exp + delta, delta bounded → cyclic wrap to AMBIG IS the exploded signal; Circle exploded encoding is canonical N1 at AMBIG exp, which is what the deflate produces, so no explicit overflow branch is needed.
            let fb = Self::fraction_bits();
            let delta: isize = fb.wrapping_sub(leading).wrapping_add(1);
            let delta_e: E = delta.as_();
            let offset = small.exponent.wrapping_add(&delta_e);

            let shl_amount = leading.wrapping_sub(fb).wrapping_sub(1);
            let canonical_r = if shl_amount >= 0 {
                result_r.w_shl(shl_amount)
            } else {
                result_r.w_shr(shl_amount.wrapping_neg())
            };
            let canonical_i = if shl_amount >= 0 {
                result_i.w_shl(shl_amount)
            } else {
                result_i.w_shr(shl_amount.wrapping_neg())
            };
            return Circle::<F, E> {
                real: canonical_r.deflate(),
                imaginary: canonical_i.deflate(),
                exponent: offset,
            };
        }

        // Escape-class handling (mirrors Scalar sub ordering).
        if self.is_undefined() {
            return *self;
        }
        if circle.is_undefined() {
            return *circle;
        }
        // Infinity absorbs everything; signless [∞] makes [∞]−X = X−[∞] = [∞]−[∞] = [∞].
        if self.is_infinite() || circle.is_infinite() {
            return Self::INFINITY;
        }
        if circle.is_zero() {
            return *self;
        }
        if self.is_zero() {
            return -circle;
        }
        if self.exploded() && circle.exploded() {
            let prefix: F = TRANSFINITE_MINUS_TRANSFINITE.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.vanished() && circle.vanished() {
            let prefix: F = VANISHED_MINUS_VANISHED.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.exploded() {
            if circle.vanished() {
                return *self;
            }
            let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if circle.exploded() {
            if self.vanished() {
                return -circle;
            }
            let prefix: F = FINITE_MINUS_TRANSFINITE.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.vanished() {
            return -circle;
        }
        if circle.vanished() {
            return *self;
        }
        *self
    }
}
