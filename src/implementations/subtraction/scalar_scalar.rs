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
    /// Subtracts another Scalar from this Scalar
    ///
    /// # Description
    ///
    /// Performs subtraction between two Scalars, handling special cases according to mathematical principles.
    /// Returns a finite Scalar unless the result exceeds representable range, in which case it may return an exploded or vanished Scalar.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped Scalars (vanished, exploded or undefined) and handles these cases
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the aligned values
    /// 4. Normalizes the result and adjusts exponent accordingly
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` - `[↑]` ➔ `[℘ ↑-↑]` Undefined exploded minus exploded state
    /// - `[↑]` - `[#]` ➔ `[℘ ↑-]` Undefined exploded minus finite state
    /// - `[#]` - `[↑]` ➔ `[℘ -↑]` Undefined finite minus exploded state
    /// - `[↓]` - `[↓]` ➔ `[℘ ↓-↓]` Undefined vanished minus vanished state
    /// - `[↓]` - `[#]` ➔ `[-#]` Negative of the finite Scalar
    /// - `[#]` - `[↓]` ➔ `[#]` The finite Scalar
    /// - `[0]` - `[#]` ➔ `[-#]` Negative of the Scalar
    /// - `[#]` - `[0]` ➔ `[#]` The Scalar
    /// - `[0]` - `[0]` ➔ `[0]` Zero
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Subtracting finite Scalars
    /// let a = Scalar::<i32, i8>::from(42);
    /// let b = ScalarF5E3::from(12.5);
    /// let diff = a - b;
    /// assert!(diff == 29.5);
    ///
    /// // Subtracting with Zero
    /// assert!(a - 0 == a);
    /// assert!(0 - a == -a);
    ///
    /// // Subtracting Scalars with different exponents
    /// let small = ScalarF5E3::from(0.25);
    /// let large = ScalarF5E3::from(256);
    /// assert!(large - small == 255.75);
    ///
    /// // Subtraction that produces Zero
    /// let pos = ScalarF5E3::from(1.125);
    /// assert!((pos - pos).is_zero());
    ///
    /// // Subtraction with vanished Scalars
    /// let tiny = ScalarF5E3::MIN_POS / 4;
    /// assert!(tiny.vanished());
    /// assert!(a - tiny == a); // Vanished value treated as Zero
    /// assert!(tiny - a == -a); // Subtraction from vanished returns negative
    ///
    /// // Subtraction with exploded Scalars
    /// let huge = ScalarF5E3::MAX * 4;
    /// assert!(huge.exploded());
    /// assert!((a - huge).is_undefined());
    ///
    /// // Subtracting two exploded Scalars
    /// assert!((huge - huge).is_undefined());
    /// ```
    pub(crate) fn scalar_subtract_scalar(&self, scalar: &Self) -> Self {
        if !self.is_normal() || !scalar.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if scalar.is_undefined() {
                return *scalar;
            }
            if self.is_transfinite() && scalar.is_transfinite() {
                return Self {
                    fraction: TRANSFINITE_MINUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && scalar.vanished() {
                return Self {
                    fraction: VANISHED_MINUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_transfinite() {
                return Self {
                    fraction: TRANSFINITE_MINUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if scalar.is_transfinite() {
                return Self {
                    fraction: FINITE_MINUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() {
                return -scalar;
            }
            if scalar.is_zero() {
                return *self;
            }
            if self.vanished() {
                return -scalar;
            }
            if scalar.vanished() {
                return *self;
            }
            return *self;
        }

        let (big, small) = if self.exponent > scalar.exponent {
            (self, scalar)
        } else {
            (scalar, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            return *self;
        }
        let shift: isize = exp_diff.saturate();
        if shift >= Self::fraction_bits() {
            return *self;
        }

        let mut self_f = self.fraction.inflate(true);
        let mut scalar_f = scalar.fraction.inflate(true);
        if self.exponent > scalar.exponent {
            self_f.w_shl_assign(shift);
        } else {
            scalar_f.w_shl_assign(shift);
        }
        let result = self_f.w_sub(scalar_f);
        if result.w_is_zero() {
            return Self {
                fraction: F::zero(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        let leading = result.leading_same();
        let offset = small
            .exponent
            .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
        if big.exponent.is_negative() && !offset.is_negative() {
            return Self {
                fraction: result
                    .w_shl(leading.wrapping_sub(1))
                    .w_shr(Self::fraction_bits())
                    .deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        Self {
            fraction: result.w_shl(leading).w_shr(Self::fraction_bits()).deflate(),
            exponent: offset,
        }
    }
}
