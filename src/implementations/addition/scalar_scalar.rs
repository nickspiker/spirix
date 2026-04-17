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
    /// Adds this Scalar to another Scalar
    ///
    /// # Description
    ///
    /// Performs addition between two Scalars according to mathematical principles. Returns a finite Scalar unless the result exceeds representable range, in which case it will return an exploded or vanished Scalar.
    ///
    /// Addition process:
    /// - Checks for any abnormal Scalars (Zero, vanished, exploded, Infinity, or undefined) and handles these cases
    /// - Aligns fractions by shifting the larger value left based on exponent difference
    /// - Adds the aligned values
    /// - Normalizes the result and adjusts exponent accordingly
    /// - If result is closer to Zero than the smallest representable value, a vanished Scalar is returned
    /// - If result is further away from Zero than the largest representable value, an exploded Scalar is returned
    ///
    /// # Return Truth Table
    ///
    /// | + | `[0]` Zero | `[↓]` Vanished | `[#]` Normal | `[↑]` Exploded | `[∞]` Infinity | `[℘?]` Undefined |
    /// |-|-|-|-|-|-|-|
    /// | `[0]` Zero | `[0]` | `[↓]` | `[#]` | `[℘⬆+]` | `[℘⬆+]` | `[℘?]` |
    /// | `[↓]` Vanished | `[↓]` | `[⬇+⬇]` | `[#]` | `[℘⬆+]` | `[℘⬆+]` | `[℘?]` |
    /// | `[#]` Normal | `[#]` | `[#]` | `[0]`,`[↓]`,`[#]`,`[↑]` | `[℘⬆+]` | `[℘⬆+]` | `[℘?]` |
    /// | `[↑]` Exploded | `[℘+⬆]` | `[℘+⬆]` | `[℘+⬆]` | `[℘⬆+⬆]` | `[℘⬆+⬆]` | `[℘?]` |
    /// | `[∞]` Infinity | `[℘+⬆]` | `[℘+⬆]` | `[℘+⬆]` | `[℘⬆+⬆]` | `[℘⬆+⬆]` | `[℘?]` |
    /// | `[℘?]` Undefined | `[℘?]` | `[℘?]` | `[℘?]` | `[℘?]` | `[℘?]` | `[℘?]` |
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Adding finite Scalars
    /// let a = Scalar::<i32, i8>::from(42);
    /// let b = ScalarF5E3::from(6.75);
    /// let sum = a + b;
    /// assert!(sum == 48.75);
    ///
    /// // Adding with Zero
    /// assert!(a + 0 == a);
    /// assert!(0 + a == a);
    ///
    /// // Adding with Infinity is undefined
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!((infinity + a).is_undefined()); // Returns [℘ ⬆+] (transfinite plus finite)
    /// assert!((a + infinity).is_undefined()); // Returns [℘ +⬆] (finite plus transfinite)
    /// assert!((infinity + infinity).is_undefined()); // Returns [℘ ⬆+⬆] (transfinite plus transfinite)
    ///
    /// // Adding Scalars with different exponents
    /// let small = ScalarF5E3::from(0.25);
    /// let large = ScalarF5E3::from(256);
    /// assert!(small + large == 256.25);
    ///
    /// // Adding Scalars that produce Zero
    /// let pos = ScalarF5E3::from(1.125);
    /// let neg = ScalarF5E3::from(-1.125);
    /// assert!((pos + neg).is_zero());
    ///
    /// // Addition with vanished Scalars
    /// let tiny = ScalarF5E3::MIN_POS / 3;
    /// assert!(tiny.vanished());
    /// assert!(a + tiny == a); // Vanished value treated as Zero
    ///
    /// // Addition with exploded Scalars
    /// let huge = ScalarF5E3::MAX * 3;
    /// assert!(huge.exploded());
    /// assert!((a + huge).is_undefined());
    ///
    /// // Adding two exploded Scalars
    /// assert!((huge + huge).is_undefined());
    /// ```
    pub(crate) fn scalar_add_scalar(&self, scalar: &Self) -> Self {
        if self.is_normal() && scalar.is_normal() {
            let (big, small) = if self.exponent > scalar.exponent {
                (self, scalar)
            } else {
                (scalar, self)
            };
            let exp_diff = big.exponent.wrapping_sub(&small.exponent);
            if exp_diff.is_negative() {
                return *big;
            }

            let shift: isize = exp_diff.saturate();
            if shift >= Self::fraction_bits() {
                return *big;
            }
            let mut big_f = big.fraction.inflate(true);
            big_f.w_shl_assign(shift);
            let result = big_f.w_add(small.fraction.inflate(true));
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
            return Self {
                fraction: result.w_shl(leading).w_shr(Self::fraction_bits()).deflate(),
                exponent: offset,
            };
        }
        if self.is_undefined() {
            return *self;
        }
        if scalar.is_undefined() {
            return *scalar;
        }
        if self.is_transfinite() && scalar.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.vanished() && scalar.vanished() {
            return Self {
                fraction: VANISHED_PLUS_VANISHED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if scalar.is_transfinite() {
            return Self {
                fraction: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.is_zero() {
            return *scalar;
        }
        if scalar.is_zero() {
            return *self;
        }
        if self.vanished() {
            return *scalar;
        }
        if scalar.vanished() {
            return *self;
        }
        return *self;
    }
}
