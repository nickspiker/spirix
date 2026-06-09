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
    /// Subtracts a Scalar from this Circle
    ///
    /// # Description
    ///
    /// Performs subtraction between a Circle and a Scalar, subtracting the Scalar value from the real component of the Circle while leaving the imaginary component unchanged (except for normalization). Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) Circles and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the Scalar from the real component of the Circle
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
    /// - `[↓]` - `[#]` ➔ `[-#, 0i]` Negative of the Scalar as real part, zero imaginary
    /// - `[#]` - `[↓]` ➔ `[#]` The Circle unchanged
    /// - `[0]` - `[#]` ➔ `[-#, 0i]` Negative of the Scalar as real part, zero imaginary
    /// - `[#]` - `[0]` ➔ `[#]` The Circle unchanged
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3, Scalar, ScalarF5E3};
    ///
    /// // Subtracting a Scalar from a Circle let a = CircleF5E3::from((7_i32, 4_i32)); let b = ScalarF5E3::from(3_i32); let diff = a - b; assert!(diff.r() == 4_i32); assert!(diff.i() == 4_i32);
    ///
    /// // Subtracting with Zero assert!(a - ScalarF5E3::ZERO == a);
    ///
    /// // Subtraction with vanished values let tiny: ScalarF5E3 = ScalarF5E3::MIN_POS / 4_i32; assert!(tiny.vanished()); assert!((a - tiny) == a);
    ///
    /// // Subtraction with exploded values let huge: ScalarF5E3 = ScalarF5E3::MAX * 4_i32; assert!(huge.exploded()); assert!((a - huge).is_undefined());
    /// ```
    pub(crate) fn circle_subtract_scalar(&self, scalar: &Scalar<F, E>) -> Self {
        if self.is_normal() && scalar.is_normal() {
            // AMBIG=0 unified pipeline. Circle - Scalar: track self_is_big for subtraction order. Scalar's N0 frac converted N1 via (s>>1)^F::MIN; scalar.imag = 0.
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
                    let neg_scalar = -*scalar;
                    Circle {
                        real: (neg_scalar.fraction >> 1isize) ^ F::min_value(),
                        imaginary: F::zero(),
                        exponent: neg_scalar.exponent,
                    }
                };
            }

            let shift: isize = exp_diff.saturate();
            let scalar_n1 = (scalar.fraction >> 1isize) ^ F::min_value();
            // self - scalar: big_aligned components - small_native components.
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
            let (result_r, result_i) = if self_is_big {
                (big_r.w_sub(small_r), big_i.w_sub(small_i))
            } else {
                (small_r.w_sub(big_r), small_i.w_sub(big_i))
            };

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

        // Escape-class handling (at least one operand is non-normal). Mirrors scalar_subtract_scalar's order: undefined → INFINITY-absorbs → zero identity → escape combinations. The earlier version used the combined `is_transfinite()` (inf OR exploded) which caused `inf - X` to return TRANSFINITE_MINUS_FINITE (= undefined) instead of inf.
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
            // [∞] is signless so [∞]-X and X-[∞] both yield [∞].
            if self.is_infinite() || scalar.is_infinite() {
                return Circle::<F, E>::INFINITY;
            }
            // Zero identity: self - 0 = self; 0 - scalar = -scalar (as Circle).
            if scalar.is_zero() {
                return *self;
            }
            if self.is_zero() {
                return Circle::<F, E>::from_ri(-*scalar, Scalar::<F, E>::ZERO);
            }
            if self.exploded() && scalar.exploded() {
                let prefix: F = TRANSFINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && scalar.vanished() {
                let prefix: F = VANISHED_MINUS_VANISHED.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Exploded - vanished = exploded (vanished negligible); exploded - normal = [℘].
            if self.exploded() {
                if scalar.vanished() {
                    return *self;
                }
                let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Vanished - exploded = -exploded; normal - exploded = [℘].
            if scalar.exploded() {
                if self.vanished() {
                    return Circle::<F, E>::from_ri(-*scalar, Scalar::<F, E>::ZERO);
                }
                let prefix: F = FINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Single vanished (other is normal here — zero/exploded/inf/undef already handled).
            if self.vanished() {
                return Circle::<F, E>::from_ri(-*scalar, Scalar::<F, E>::ZERO);
            }
            if scalar.vanished() {
                return *self;
            }
            *self
        }
    }
}
