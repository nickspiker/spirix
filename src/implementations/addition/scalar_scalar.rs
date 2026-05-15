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
            // Magnitude comparison on the cyclic unsigned-stored exponent: convert both
            // operands to v0.1 form (XOR with E::MIN) so that signed `>` gives the
            // correct cyclic-magnitude ordering. The XOR cancels in the subtraction
            // below (since (a^M) - (b^M) ≡ a - b mod 2^N), so exp_diff is convention-
            // independent.
            let (big, small) = if self.v01_exp() > scalar.v01_exp() {
                (self, scalar)
            } else {
                (scalar, self)
            };
            let exp_diff = big.exponent.wrapping_sub(&small.exponent);
            if exp_diff.is_negative() {
                return *big;
            }

            let shift: isize = exp_diff.saturate();
            // x86 implementation note: Spirix's signless design wants to drop small only when shift ≥ FRAC. But Wide = 2*FRAC bits, and the inflated form reaches 2^FRAC magnitude at the ±1.0 boundary, so big_f<<shift + small_f needs 2*FRAC+2 bits in the worst case — one bit more than Rust/x86 provides at any power-of-2 width. Tightening the threshold to FRAC-1 guarantees the sum fits in the signed 2*FRAC-bit intermediate, letting us use pure two's-complement arithmetic with no sign branching. Cost: dropping small at shift == FRAC-1 loses ≤1 ULP of its contribution. In Verilog this tightening is unnecessary — we just declare a 2*FRAC+2-bit intermediate and keep full precision.
            if shift >= Self::fraction_bits().wrapping_sub(1) {
                return *big;
            }
            let big_f = big.fraction.inflate(true).w_shl(shift);
            let small_f = small.fraction.inflate(true);
            let result = big_f.w_add(small_f);
            if result.w_is_zero() {
                return Self {
                    fraction: F::zero(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            let leading = result.leading_same();
            let fb = Self::fraction_bits();
            let delta: isize = fb.wrapping_sub(leading);
            // v0.1 ruler arithmetic: compute the exponent offset in a wider signed type
            // (isize), then convert to stored (new) form at the end via from_v01_exp.
            // small.v01_exp() gives the v0.1-form interpretation of the stored exp; the
            // range bounds use the v0.1-form helpers so the comparisons remain meaningful.
            // Bounded by |delta| ≤ FRAC so isize never overflows.
            let small_exp_wide: isize = small.v01_exp().saturate();
            let offset_wide: isize = small_exp_wide.wrapping_add(delta);
            let max_exp_wide: isize = Self::v01_max_exponent().saturate();
            let min_exp_wide: isize = Self::v01_min_exponent().saturate();
            if offset_wide > max_exp_wide {
                return Self {
                    fraction: result.w_shl(leading.wrapping_sub(1)).w_shr(fb).deflate(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if offset_wide < min_exp_wide {
                return Self {
                    fraction: result.w_shl(leading.wrapping_sub(2)).w_shr(fb).deflate(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            let offset: E = Self::from_v01_exp(offset_wide.as_());
            // Main path: `result << L >> FRAC` composed as a net shift of L-FRAC. Written directly to sidestep Rust's shift-overflow semantics when L == wide_bits (result is all sign bits).
            let shl_amount = leading.wrapping_sub(fb);
            let canonical = if shl_amount >= 0 {
                result.w_shl(shl_amount)
            } else {
                result.w_shr(shl_amount.wrapping_neg())
            };
            return Self {
                fraction: canonical.deflate(),
                exponent: offset,
            };
        }
        if self.is_undefined() {
            return *self;
        }
        if scalar.is_undefined() {
            return *scalar;
        }
        // Infinity absorbs everything. [∞] is the signless Riemann-sphere point reached only by n/0; −∞ is a no-op so [∞]−[∞] = [∞] too. Checked before zero-identity and before any exploded/vanished branches so the [∞] row/column of the truth table absorbs uniformly.
        if self.is_infinite() || scalar.is_infinite() {
            return Self::INFINITY;
        }
        // Zero is the additive identity for every remaining class.
        if self.is_zero() {
            return *scalar;
        }
        if scalar.is_zero() {
            return *self;
        }
        // [↑]+[↑] indeterminate: opposing phases could partially cancel back into normal range.
        if self.exploded() && scalar.exploded() {
            return Self {
                fraction: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // [↓]+[↓] indeterminate: same-magnitude vanished collisions are non-recoverable.
        if self.vanished() && scalar.vanished() {
            return Self {
                fraction: VANISHED_PLUS_VANISHED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // [↑] vs normal/vanished: vanished is negligible so [↑]+[↓] = [↑]; normal could cancel so [↑]+[#] = ℘.
        if self.exploded() {
            if scalar.vanished() {
                return *self;
            }
            return Self {
                fraction: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if scalar.exploded() {
            if self.vanished() {
                return *scalar;
            }
            return Self {
                fraction: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Remaining: vanished + normal → normal (vanished negligible).
        if self.vanished() {
            return *scalar;
        }
        if scalar.vanished() {
            return *self;
        }
        return *self;
    }
}
