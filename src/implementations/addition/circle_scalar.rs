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
    /// Adds a Scalar to this Circle
    ///
    /// # Description
    ///
    /// Performs addition between a Circle and a Scalar, adding the Scalar value to the real component of the Circle while leaving the imaginary component unchanged (except for normalization). Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Addition process:
    /// 0. Checks for any escaped (vanished, exploded, infinity, or undefined) values and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Adds the Scalar to the real component of the Circle
    /// 4. Normalizes result and adjusts exponent, considering both components
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[∞]` + `[#, #i]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#, #i]` + `[∞]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[∞]` + `[∞, #i]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[↑]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[↑]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↓]` + `[↓]` ➔ `[℘ ↓+↓]` Undefined vanished plus vanished state
    /// - `[↓]` + `[#]` ➔ `[#]` The finite Scalar as real part, zero imaginary
    /// - `[#]` + `[↓]` ➔ `[#]` The Circle unchanged
    /// - `[0]` + `[#]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[#]` + `[0]` ➔ `[#]` The Circle unchanged
    /// - `[#]` + `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3, Scalar, ScalarF5E3};
    ///
    /// // Adding a Scalar to a Circle
    /// let a = Circle::<i32, i8>::from((3, 4));
    /// let b = ScalarF5E3::from(5.25);
    /// let sum = a + b;
    /// assert!(sum.r() == 8.25);
    /// assert!(sum.i() == 4);
    ///
    /// // Adding with Zero
    /// assert!(a + ScalarF5E3::ZERO == a);
    /// assert!(CircleF5E3::ZERO + b == CircleF5E3::from((5.25, 0)));
    ///
    /// // Adding with infinity (mathematically undefined)
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!((a + infinity).is_undefined()); // Returns [℘ +↑] (finite plus exploded)
    /// assert!((infinity + a).is_undefined()); // Returns [℘ ↑+] (exploded plus finite)
    ///
    /// // Adding with different exponents
    /// let small_circle = CircleF5E3::from((0.125, 0.5));
    /// let large_scalar = ScalarF5E3::from(144);
    /// let result = small_circle + large_scalar;
    /// assert!(result.r() == 144.125);
    /// assert!(result.i() == 0.5);
    ///
    /// // Adding components that cancel
    /// let c = CircleF5E3::from((-2.5, 1.75));
    /// let d = ScalarF5E3::from(2.5);
    /// let result = c + d;
    /// assert!(result.r() == 0);
    /// assert!(result.i() == 1.75);
    ///
    /// // Addition with vanished values
    /// let tiny = ScalarF5E3::MIN_POS / 5;
    /// assert!(tiny.vanished());
    /// assert!((a + tiny) == a);
    ///
    /// // Addition with exploded values
    /// let huge = ScalarF5E3::MAX * 5;
    /// assert!(huge.exploded());
    /// assert!((a + huge).is_undefined());
    ///
    /// // Adding two exploded values
    /// let huge_circle = CircleF5E3::MAX * 3;
    /// assert!((huge_circle + huge).is_undefined());
    /// ```
    pub(crate) fn circle_add_scalar(&self, scalar: &Scalar<F, E>) -> Self {
        if self.is_normal() && scalar.is_normal() {
            // AMBIG=0 native unified pipeline. Scalar's N0 fraction → N1 via (s >> 1) ^ F::MIN before sign_extend; Scalar contributes 0 to the imaginary side.
            let self_is_big =
                self.exponent.into_unsigned() > scalar.exponent.into_unsigned();
            let (big_exp, small_exp) = if self_is_big {
                (self.exponent, scalar.exponent)
            } else {
                (scalar.exponent, self.exponent)
            };
            let exp_diff = big_exp.wrapping_sub(&small_exp);
            let frac_bits_e: E = Self::fraction_bits().as_();
            if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
                return if self_is_big {
                    *self
                } else {
                    Circle {
                        real: (scalar.fraction >> 1isize) ^ F::min_value(),
                        imaginary: F::zero(),
                        exponent: scalar.exponent,
                    }
                };
            }

            let shift: isize = exp_diff.saturate();
            let scalar_n1 = (scalar.fraction >> 1isize) ^ F::min_value();
            let (big_r, big_i, small_r, small_i) = if self_is_big {
                (
                    self.real.sign_extend().w_shl(shift),
                    self.imaginary.sign_extend().w_shl(shift),
                    scalar_n1.sign_extend(),
                    F::zero().sign_extend(),
                )
            } else {
                (
                    scalar_n1.sign_extend().w_shl(shift),
                    F::zero().sign_extend(),
                    self.real.sign_extend(),
                    self.imaginary.sign_extend(),
                )
            };
            let result_r = big_r.w_add(small_r);
            let result_i = big_i.w_add(small_i);

            if result_r.w_is_zero() && result_i.w_is_zero() {
                return Self::ZERO;
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
            return Self {
                real: canonical_r.deflate(),
                imaginary: canonical_i.deflate(),
                exponent: offset,
            };
        }

        // Escape-class handling (at least one operand is non-normal).
        {
            if self.is_undefined() {
                return *self;
            }
            if scalar.is_undefined() {
                return Circle {
                    real: scalar.fraction,
                    imaginary: scalar.fraction,
                    exponent: scalar.exponent,
                };
            }
            if self.is_transfinite() && scalar.is_transfinite() {
                return Self {
                    real: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && scalar.vanished() {
                return Self {
                    real: VANISHED_PLUS_VANISHED.prefix.sa(),
                    imaginary: VANISHED_PLUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_transfinite() {
                return Self {
                    real: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if scalar.is_transfinite() {
                return Self {
                    real: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return Circle {
                    real: (scalar.fraction >> 1isize) ^ F::min_value(),
                    imaginary: F::zero(),
                    exponent: scalar.exponent,
                };
            }
            if scalar.vanished() {
                return *self;
            }
            if self.is_zero() {
                return Circle {
                    real: (scalar.fraction >> 1isize) ^ F::min_value(),
                    imaginary: F::zero(),
                    exponent: scalar.exponent,
                };
            }
            *self
        }
    }
}
