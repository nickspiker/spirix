use crate::core::integer::{Deflate, FullInt, Inflate, IntConvert, WideOps};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    /// Multiplies this Scalar by another Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between two Scalars, handling special cases according to mathematical principles.
    /// For normal values, this produces the expected mathematical product. Special states follow special rules to maintain mathematical continuity even when results exceed representable ranges.
    ///
    /// Multiplication process:
    /// 0. Checks for escaped values (undefined, exploded, vanished) and applies special case handling  
    /// 1. Uses wider integer types for fraction multiplication as product lands in the high half  
    /// ```txt
    ///    □□□□□□□ ■■■■■■■ = Multiplier
    ///    □□□□□□□ ■■■■■■■ = Multiplicand
    ///         ⤪⤪⤪⤪⤪⤪      Multiply!
    ///    ■■■■■■■ □□□□□□□ = Intermediate 2x-bit product space
    ///        ↘↘↘↘↘↘↘
    ///            ■■■■■■■ = High half kept, low bits discarded to floor
    /// 2. Calculates leading Zeros/Ones to determine normalization shift and exponent nudge  
    /// 3. Adds exponents and adjusts by normalization shift (exponent_result = self.exponent + other.exponent - shift)  
    /// 4. Handles special cases where exponent exceeds MAX_EXPONENT (explode) or falls below MIN_EXPONENT (vanish)  
    /// 5. For escaped values, preserves sign and phase information while following mathematical convention  
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // Multiplying finite Scalars
    /// let eight = Scalar::<i64, i16>::from(8);
    /// let eigth = ScalarF6E4::ONE / 8;
    /// assert!(eight * eigth == 1); // Restores unity thru multiplicative inverse
    ///
    /// // Multiplying near boundaries
    /// let large = ScalarF6E4::from(64) * ScalarF6E4::MAX_NEG;
    /// let small = ScalarF6E4::from(1) / large;
    /// assert!(large * small == 1);
    ///
    /// // Multiplication preserves sign according to mathematical rule
    /// let negative = ScalarF6E4::from(-1.5);
    /// assert!((eight * negative).is_negative()); // Positive × Negative = Negative
    /// assert!((negative * negative).is_positive()); // Negative × Negative = Positive
    ///
    /// // Vanished values maintain sign thru multiplication
    /// let tiny = ScalarF6E4::MIN_POS / ScalarF6E4::from(11);
    /// assert!(tiny.vanished());
    /// let neg_tiny = tiny * ScalarF6E4::NEG_ONE;
    /// assert!(neg_tiny.vanished() && neg_tiny.is_negative());
    ///
    /// // Exploded values interact consistently with finite values
    /// let huge = ScalarF6E4::MAX * 42;
    /// assert!(huge.exploded());
    /// assert!((1 / huge).vanished()); // Inverse of exploded is vanished
    /// assert!(((-1) / huge).is_negative()); // Signs are propogated following multiplication rule
    /// assert!((huge * ScalarF6E4::NEG_ONE).exploded());
    /// assert!((huge * -1).is_negative());
    ///
    /// // Multiplying by zero always produces zero, even with escaped values
    /// assert!((huge * 0).is_zero());
    /// assert!((tiny * 0).is_zero());
    ///
    /// // Vanished × Exploded yields an undefined state (magnitude indeterminate)
    /// let undefined_product = tiny * huge;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn scalar_multiply_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite() && other.is_zero() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() && other.is_infinite() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_infinite() || other.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.exploded() && other.vanished() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && other.exploded() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            // Escaped * escaped/normal: determine sign and escape level from inputs
            let result_negative = self.is_negative() != other.is_negative();
            let result_exploded = self.exploded() || other.exploded();
            let fraction = if result_negative {
                if result_exploded { F::NEG_ONE_EXPLODED_FRACTION } else { F::NEG_ONE_VANISHED_FRACTION }
            } else {
                if result_exploded { F::POS_ONE_EXPLODED_FRACTION } else { F::POS_ONE_VANISHED_FRACTION }
            };
            return Self { fraction, exponent: E::AMBIGUOUS_EXPONENT };
        }

        // Normal * Normal
        // Pre-check: stored=0 (NEG_ONE_NORMAL_FRACTION) is -2^exp, multiply is just negate + shift
        if self.fraction == F::NEG_ONE_NORMAL_FRACTION {
            let mut result = -other;
            result.exponent = self.exponent.wrapping_add(&result.exponent);
            return result;
        }
        if other.fraction == F::NEG_ONE_NORMAL_FRACTION {
            let mut result = -self;
            result.exponent = other.exponent.wrapping_add(&result.exponent);
            return result;
        }
        let product = self.fraction.inflate().w_mul(other.fraction.inflate());
        let expect_negative = self.is_negative() != other.is_negative();
        let leading = if expect_negative {
            product.w_leading_ones()
        } else {
            product.w_leading_zeros()
        };
        let fraction = product
            .w_shl(leading)
            .w_shr_logical(F::FRACTION_BITS)
            .deflate();

        let sum = self.exponent.wrapping_add(&other.exponent);
        if self.exponent.is_positive() && other.exponent.is_positive() && !sum.is_positive() {
            return Self { fraction, exponent: E::AMBIGUOUS_EXPONENT };
        }
        if self.exponent.is_negative() && other.exponent.is_negative() && !sum.is_negative() {
            return Self { fraction: fraction >> 1isize, exponent: E::AMBIGUOUS_EXPONENT };
        }
        let adj: E = (leading as isize).as_();
        let result_exp = sum.wrapping_sub(&adj);
        if result_exp > E::MAX_EXPONENT {
            Self { fraction, exponent: E::AMBIGUOUS_EXPONENT }
        } else if result_exp <= E::AMBIGUOUS_EXPONENT {
            Self { fraction: fraction >> 1isize, exponent: E::AMBIGUOUS_EXPONENT }
        } else {
            Self { fraction, exponent: result_exp }
        }
    }
}
