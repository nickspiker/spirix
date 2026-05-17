use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub, Zero};
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
    /// Adds this Circle to another Circle
    ///
    /// # Description
    ///
    /// Performs addition between two Circles, handling special cases according to mathematical principles. Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Addition process:
    /// 0. Checks for any abnormal Circles (Zeros, vanished, exploded or undefined) and handles these cases
    /// 1. Aligns fractions by shifting the larger value left based on exponent difference
    /// 2. Adds the aligned values for both real and imaginary components simultaneously
    /// 3. Normalizes the result and adjusts exponent accordingly
    /// 4. Escapes for underflow if necessary (vanished), overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` + `[↑]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[↑]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↓]` + `[↓]` ➔ `[℘ ↓+↓]` Undefined vanished plus vanished state
    /// - `[↓]` + `[#]` or `[#]` + `[↓]` ➔ `[#]` The finite Circle
    /// - `[0]` + `[#]` or `[#]` + `[0]` ➔ `[#]` The non-Zero Circle
    /// - `[0]` + `[0]` ➔ `[0]` Zero
    /// - `[#]` + `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Adding finite Circles
    /// let a = Circle::<i32, i8>::from((3, 4.5));
    /// let b = CircleF5E3::from((1, 2));
    /// let sum = a + b;
    /// assert!(sum.r() == 4);
    /// assert!(sum.i() == 6.5);
    ///
    /// // Adding with Zero
    /// assert!(a + 0 == a);
    /// assert!(0 + a == a);
    ///
    /// // Adding Circles with different exponents
    /// let small = CircleF5E3::from((0.125, 0.25));
    /// let large = CircleF5E3::from((128, 64));
    /// let result = small + large;
    /// assert!(result.r() == 128.125);
    /// assert!(result.i() == 64.25);
    ///
    /// // Adding Circles that produce Zero
    /// let pos = CircleF5E3::from((2.5, 3.75));
    /// let neg = CircleF5E3::from((-2.5, -3.75));
    /// assert!((pos + neg).is_zero());
    /// assert!((neg + pos).is_zero());
    ///
    /// // Addition with vanished Circles
    /// let tiny = CircleF5E3::MIN_POS_REAL / 5;
    /// assert!(tiny.vanished());
    /// assert!((a + tiny).r() == a.r());
    /// assert!((a + tiny).i() == a.i());
    ///
    /// // Addition with exploded Circles
    /// let huge = CircleF5E3::MAX_REAL_CIRCLE * 5;
    /// assert!(huge.exploded());
    /// assert!((a + huge).is_undefined());
    ///
    /// // Adding two exploded Circles
    /// assert!((huge + huge).is_undefined());
    /// ```
    pub(crate) fn circle_add_circle(&self, circle: &Self) -> Self {
        if !self.is_normal() || !circle.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if circle.is_undefined() {
                return *circle;
            }
            // Infinity absorbs everything. [∞] is the signless Riemann-sphere point reached only by n/0; −∞ is a no-op so [∞]−[∞] = [∞] too.
            if self.is_infinite() || circle.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() {
                return *circle;
            }
            if circle.is_zero() {
                return *self;
            }
            if self.exploded() && circle.exploded() {
                return Self {
                    real: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && circle.vanished() {
                return Self {
                    real: VANISHED_PLUS_VANISHED.prefix.sa(),
                    imaginary: VANISHED_PLUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                if circle.vanished() {
                    return *self;
                }
                return Self {
                    real: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if circle.exploded() {
                if self.vanished() {
                    return *circle;
                }
                return Self {
                    real: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return *circle;
            }
            if circle.vanished() {
                return *self;
            }
            return *self;
        }

        // AMBIG=0 native: dominance via unsigned-cyclic compare on stored exp (matches Scalar add/sub).
        let (big, small) = if self.exponent.into_unsigned() > circle.exponent.into_unsigned() {
            (self, circle)
        } else {
            (circle, self)
        };

        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        // Under cmp_unsigned ordering, exp_diff (interpreted unsigned) is non-negative. If the unsigned diff exceeds FRAC_BITS, small is negligible against big.
        let frac_bits_e: E = Self::fraction_bits().as_();
        if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
            return *big;
        }

        // AMBIG=0 native unified pipeline (mirrors Scalar add). Circle's N1 inflation is just sign_extend — the explicit sign bit at MSB means widening preserves the signed value without needing the XOR mask Scalar's N0 inflate applies.
        let shift: isize = exp_diff.saturate();
        let big_r = big.real.sign_extend().w_shl(shift);
        let big_i = big.imaginary.sign_extend().w_shl(shift);
        let small_r = small.real.sign_extend();
        let small_i = small.imaginary.sign_extend();
        let result_r = big_r.w_add(small_r);
        let result_i = big_i.w_add(small_i);

        if result_r.w_is_zero() && result_i.w_is_zero() {
            return Self::ZERO;
        }

        let leading_r = result_r.leading_same();
        let leading_i = result_i.leading_same();
        let leading = leading_r.min(leading_i);

        let fb = Self::fraction_bits();
        let small_pos = small.exponent.cycle_widen();
        // N1 canonical wide leading = FRAC + 1 (vs Scalar N0's FRAC). The +1 accounts for the explicit sign bit's slot in the wide representation.
        let delta: isize = fb.wrapping_sub(leading).wrapping_add(1);
        let delta_e: E = delta.as_();
        let w_delta = delta_e.sign_extend();
        let offset_pos = small_pos.w_add(w_delta);
        let max_pos = Self::max_exponent().cycle_widen();
        let min_pos = Self::min_exponent().cycle_widen();

        if offset_pos > max_pos {
            // Exploded: N1 canonical shape (leading - 1 in narrow).
            return Self {
                real: result_r.w_shl(leading.wrapping_sub(1)).w_shr(fb).deflate(),
                imaginary: result_i.w_shl(leading.wrapping_sub(1)).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if offset_pos < min_pos {
            // Vanished: N2 canonical shape (leading - 2 in narrow).
            return Self {
                real: result_r.w_shl(leading.wrapping_sub(2)).w_shr(fb).deflate(),
                imaginary: result_i.w_shl(leading.wrapping_sub(2)).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Normal path: shift to canonical N1 (wide leading = FRAC + 1). Net shift relative to wide = leading - (FRAC + 1).
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
        Self {
            real: canonical_r.deflate(),
            imaginary: canonical_i.deflate(),
            exponent: offset_pos.deflate(),
        }
    }
}
