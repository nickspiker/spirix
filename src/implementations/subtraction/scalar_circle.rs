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
    /// Subtracts a Circle from this Scalar
    ///
    /// # Description
    ///
    /// Performs subtraction between a Scalar and a Circle, with the Scalar value becoming the real component and the negative of the Circle's imaginary component becoming the imaginary component of the result. Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) values and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the Circle's real component from the Scalar and negates the Circle's imaginary component
    /// 4. Normalizes result and adjusts exponent, considering both components
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` - `[↑]` ➔ `[℘ ↑-↑]` Undefined exploded minus exploded state
    /// - `[↑]` - `[#]` ➔ `[℘ ↑-]` Undefined exploded minus finite state
    /// - `[#]` - `[↑]` ➔ `[℘ -↑]` Undefined finite minus exploded state
    /// - `[↓]` - `[↓]` ➔ `[℘ ↓-↓]` Undefined vanished minus vanished state
    /// - `[↓]` - `[#]` ➔ `[-#, -#i]` Negative of the Circle with negated imaginary component
    /// - `[#]` - `[↓]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[0]` - `[#]` ➔ `[-#r, -#i]` Negative of the Circle
    /// - `[#]` - `[0]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF6E4, Scalar, ScalarF6E4};
    ///
    /// // Subtracting a Circle from a Scalar
    /// let a = Scalar::<i64,i16>::from(15);
    /// let b = Circle::<i64,i16>::from((-5, 4.5));
    /// let diff = a - b;
    /// assert!(diff.r() == 20);
    /// assert!(diff.i() == -4.5);
    ///
    /// // Subtracting with Zero
    /// assert!(a - CircleF6E4::ZERO == CircleF6E4::from((a, 0)));
    /// assert!(ScalarF6E4::ZERO - b == -b;
    ///
    /// // Subtracting with different exponents
    /// let large_scalar = ScalarF6E4::from(72);
    /// let small_circle = CircleF6E4::from((1.5, 0.0625));
    /// let result = large_scalar - small_circle;
    /// assert!(result.r() == 70.5);
    /// assert!(result.i() == -0.0625);
    ///
    /// // Subtraction with vanished values
    /// let tiny = CircleF6E4::MIN_POS_REAL / 4;
    /// assert!(tiny.vanished());
    /// assert!((a - tiny).r() == 7);
    /// assert!((a - tiny).i() == 0);
    ///
    /// // Subtraction with exploded values
    /// let huge = CircleF6E4::MAX_REAL_CIRCLE * 4;
    /// assert!(huge.exploded());
    /// assert!((a - huge).is_undefined());
    /// ```
    pub(crate) fn scalar_subtract_circle(&self, circle: &Circle<F, E>) -> Circle<F, E> {
        if self.is_normal() && circle.is_normal() {
            // AMBIG=0 unified pipeline. Scalar - Circle: self is Scalar (N0→N1), other is Circle (N1).
            let self_is_big =
                self.exponent.into_unsigned() > circle.exponent.into_unsigned();
            let (big_exp, small_exp) = if self_is_big {
                (self.exponent, circle.exponent)
            } else {
                (circle.exponent, self.exponent)
            };
            let exp_diff = big_exp.wrapping_sub(&small_exp);
            let frac_bits_e: E = Self::fraction_bits().as_();
            if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
                return if self_is_big {
                    Circle {
                        real: (self.fraction >> 1isize) ^ F::min_value(),
                        imaginary: F::zero(),
                        exponent: self.exponent,
                    }
                } else {
                    -circle
                };
            }

            let shift: isize = exp_diff.saturate();
            let scalar_n1 = (self.fraction >> 1isize) ^ F::min_value();
            // self - circle: when self is big, big_aligned_scalar - small_circle; when circle is big, small_scalar_native - big_aligned_circle.
            let (big_r, big_i, small_r, small_i) = if self_is_big {
                (
                    scalar_n1.sign_extend().w_shl(shift),
                    F::zero().sign_extend(),
                    circle.real.sign_extend(),
                    circle.imaginary.sign_extend(),
                )
            } else {
                (
                    circle.real.sign_extend().w_shl(shift),
                    circle.imaginary.sign_extend().w_shl(shift),
                    scalar_n1.sign_extend(),
                    F::zero().sign_extend(),
                )
            };
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

            let fb = Self::fraction_bits();
            let delta: isize = fb.wrapping_sub(leading).wrapping_add(1);
            let delta_e: E = delta.as_();
            let offset = small_exp.wrapping_add(&delta_e);

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

        // Escape-class handling (at least one operand is non-normal).
        {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if circle.is_undefined() {
                return *circle;
            }
            if self.is_transfinite() && circle.is_transfinite() {
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
            if self.is_transfinite() {
                let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if circle.is_transfinite() {
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
                return Circle {
                    real: (self.fraction >> 1isize) ^ F::min_value(),
                    imaginary: F::zero(),
                    exponent: self.exponent,
                };
            }
            if self.is_zero() {
                return -circle;
            }
            return Circle {
                real: (self.fraction >> 1isize) ^ F::min_value(),
                imaginary: F::zero(),
                exponent: self.exponent,
            };
        }
    }
}
