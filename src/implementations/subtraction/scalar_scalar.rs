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

        let big_is_self = self.exponent > scalar.exponent;
        let (big, small) = if big_is_self {
            (self, scalar)
        } else {
            (scalar, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        // x86 implementation note: Spirix's signless design wants to drop small only when shift ≥ FRAC. But Wide = 2*FRAC bits, and the inflated form reaches 2^FRAC magnitude at the ±1.0 boundary, so big_f<<shift ± small_f needs 2*FRAC+2 bits — one more than Rust/x86 provides at any power-of-2 width. Tightening the threshold to FRAC-1 keeps the sub in signed 2*FRAC-bit arithmetic with no sign branching. Cost: ≤1 ULP when shift == FRAC-1. In Verilog this tightening is unnecessary.
        let small_lost = if big_is_self { *self } else { -scalar };
        if exp_diff.is_negative() {
            return small_lost;
        }
        let shift: isize = exp_diff.saturate();
        if shift >= Self::fraction_bits().wrapping_sub(1) {
            return small_lost;
        }

        let big_f = big.fraction.inflate(true).w_shl(shift);
        let small_f = small.fraction.inflate(true);
        // Pure two's complement subtract. With shift<=FRAC-2 both operands fit
        // with room to spare, so signed wrap cannot occur regardless of sign.
        let result = if big_is_self {
            big_f.w_sub(small_f)
        } else {
            small_f.w_sub(big_f)
        };
        if result.w_is_zero() {
            return Self {
                fraction: F::zero(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        let leading = result.leading_same();
        let fb = Self::fraction_bits();
        let delta: isize = fb.wrapping_sub(leading);
        let delta_e: E = delta.as_();
        let offset = small.exponent.wrapping_add(&delta_e);
        let one_e: E = 1u8.as_();
        // Underflow: cancellation shrank magnitude past MIN_EXP → vanished (N-2).
        let underflowed = delta.is_negative() && offset.wrapping_sub(&one_e) > small.exponent;
        if underflowed {
            return Self {
                fraction: result.w_shl(leading.wrapping_sub(2)).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Overflow: borrow bumped magnitude past MAX_EXP → exploded (N-1).
        let overflowed = delta > 0 && offset < small.exponent;
        if overflowed {
            return Self {
                fraction: result.w_shl(leading.wrapping_sub(1)).w_shr(fb).deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Main path extraction: `result << L >> FRAC` composed as a net shift of (L - FRAC). Writing it directly avoids the Rust shift-overflow semantics that bite when L == wide_bits (result = -1, full sign extension).
        let shl_amount = leading.wrapping_sub(fb);
        let canonical = if shl_amount >= 0 {
            result.w_shl(shl_amount)
        } else {
            result.w_shr(shl_amount.wrapping_neg())
        };
        Self {
            fraction: canonical.deflate(),
            exponent: offset,
        }
    }
}
