use crate::core::integer::{FullInt, IntConvert};
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
    /// Calculates the mathematical modulus of this Scalar with respect to another Scalar
    ///
    /// # Description
    ///
    /// The mathematical modulus operation finds the remainder after division, where the result takes the sign of the divisor
    ///
    /// For normal values, this follows the mathematical definition: `a % b = a - (⌊a/b⌋ × b)`, where the result has the same sign as the divisor.
    ///
    /// # Special Cases
    ///
    /// - For undefined states `[℘]`: Returns the first undefined state encountered
    /// - For exploded numerator `[↑]`: Returns an undefined state with `TRANSFINITE_MODULUS` pattern
    /// - For vanished denominator `[↓]`: Returns an undefined state with `MODULUS_VANISHED` pattern
    /// - For escaped values with differing signs: Returns an undefined state with `MODULUS_TRANSFINITE` pattern
    /// - For escaped values with matching signs: Returns the numerator
    /// - For Zero denominator or numerator: Returns Zero
    ///
    /// # Returns
    ///
    /// - `[#] % [#]` ➔ `[#]` A finite Scalar following mathematical modulus definition
    /// - `[↑] % [#]` ➔ `[℘ ↑%]` Undefined modulus with exploded numerator
    /// - `[#] % [↓]` ➔ `[℘ %↓]` Undefined modulus with vanished denominator
    /// - `[#] % [↑]` and signs match  ➔ `[#]` Original numerator (preserved)
    /// - `[#] % [↑]` and signs differ ➔ `[℘ %↑]` Undefined modulus with exploded denominator
    /// - `[↓] % [#]` and signs match  ➔ `[↓]` Original vanished value
    /// - `[↓] % [#]` and signs differ ➔ `[℘ %↑]` Undefined modulus with exploded denominator
    /// - `[0] % [?]` or `[?] % [0]`   ➔ `[0]` Zero (excluding undefined states)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Basic modulus operation for normal values
    /// let a = Scalar::<i32, i8>::from(7);
    /// let b = ScalarF5E3::from(3);
    /// assert!(a % b == 1);  // 7 % 3 = 1
    ///
    /// // Modulus preserves sign of divisor
    /// let neg_a = ScalarF5E3::from(-7);
    /// assert!(neg_a % b == 2);  // -7 % 3 = 2 (not -1)
    /// let neg_b = ScalarF5E3::from(-3);
    /// assert!(a % neg_b == -2);  // 7 % -3 = -2
    ///
    /// // Modulus with exploded values and matching signs
    /// let exploded = ScalarF5E3::MAX * 2;
    /// assert!((ScalarF5E3::PI % exploded) == ScalarF5E3::PI);  // π % +huge = π
    ///
    /// // Modulus with exploded values and differing signs produces undefined result
    /// assert!((ScalarF5E3::PI % -exploded).is_undefined());  // π % -huge = undefined
    ///
    /// // Modulus with vanished values as denominator is undefined
    /// let vanished = ScalarF5E3::MIN_POS / 19;
    /// assert!((ScalarF5E3::from(42) % vanished).is_undefined());
    ///
    /// // Zero cases
    /// assert!((0 % ScalarF5E3::PI).is_zero());
    /// assert!((ScalarF5E3::PI % 0).is_zero());
    /// assert!((ScalarF5E3::ZERO % 0).is_zero());
    /// assert!((exploded % 0).is_zero());
    /// assert!((vanished % 0).is_zero());
    /// assert!((0 % exploded).is_zero());
    /// assert!((0 % vanished).is_zero());
    /// ```
    pub(crate) fn scalar_modulus_scalar(&self, denominator: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if denominator.is_undefined() {
                return *denominator;
            }
            if self.is_zero() || denominator.is_zero() {
                return Self::ZERO;
            }
            if self.is_transfinite() {
                return Self {
                    fraction: TRANSFINITE_MODULUS.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if denominator.is_infinite() {
                return *self;
            }
            if denominator.vanished() {
                return Self {
                    fraction: MODULUS_VANISHED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.fraction.is_negative() == denominator.fraction.is_negative() {
                return *self;
            } else {
                // Signs differ, return undefined state (magnitude is indeterminate)
                return Self {
                    fraction: MODULUS_EXPLODED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        if self.is_zero() || denominator.is_zero() {
            return Self::ZERO;
        }

        let quotient = self / denominator;

        // Use the numerically stable algorithm for all cases:
        // remainder = self - floor(quotient) * denominator
        let product = quotient.floor() * denominator;
        self - product
    }
}
