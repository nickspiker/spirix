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
    /// Calculates the mathematical modulus of this Scalar
    ///
    /// # Description
    ///
    /// Returns the unique value `r` such that `numerator ≡ r (mod period)`, where `r` is the canonical representative of the numerator's congruence class with sign following the period.
    ///
    /// `⬆` denotes *transfinite* (either exploded `[↑]` or infinite `[∞]`).
    /// Undefined prefixes collapse both into one tag since the distinction
    /// isn't preserved in the stored undefined class.
    ///
    /// # Special Cases (first matching rule wins, in order):
    ///
    /// 1. `[℘?]` numerator or period → the first undefined encountered
    /// 2. `[0]` numerator or period → `[0]`
    /// 3. Transfinite numerator (`[↑]` or `[∞]`):
    ///    - transfinite period → `[℘⬆%⬆]`
    ///    - otherwise → `[℘⬆%]`
    /// 4. Vanished period `[↓]`:
    ///    - vanished numerator → `[℘↓%↓]`
    ///    - otherwise → `[℘%↓]`
    /// 5. Infinite period `[∞]` → `[℘%⬆]` (signless period, no sign to floor against)
    /// 6. Exploded period `[↑]`:
    ///    - signs match → numerator (preserved)
    ///    - signs differ → `[℘%⬆]`
    /// 7. Both finite (normal or vanished with normal period) → integer floored modulus
    ///
    /// # Returns
    ///
    /// - `[#] % [#]` ➔ `[0]`, `[↓]`, or `[#]`
    /// - `[#] % [↓]` ➔ `[℘%↓]`
    /// - `[↓] % [↓]` ➔ `[℘↓%↓]`
    /// - `[#] % [↑]` signs match ➔ `[#]`
    /// - `[#] % [↑]` signs differ ➔ `[℘%⬆]`
    /// - `[↓] % [↑]` signs match ➔ `[↓]`
    /// - `[↓] % [↑]` signs differ ➔ `[℘%⬆]`
    /// - `[?] % [∞]` ➔ `[℘%⬆]` (signless period)
    /// - `[↑] % [#]` or `[↑] % [↓]` or `[∞] % [#]` or `[∞] % [↓]` ➔ `[℘⬆%]`
    /// - `[↑] % [↑]` or `[↑] % [∞]` or `[∞] % [↑]` or `[∞] % [∞]` ➔ `[℘⬆%⬆]`
    /// - `[0] % [?]` or `[?] % [0]` ➔ `[0]`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    ///
    /// // Basic modulus
    /// let a = ScalarF5E3::from(7);
    /// let b = ScalarF5E3::from(3);
    /// assert!(a % b == 1);  // 7 % 3 = 1
    ///
    /// // Result takes sign of period
    /// let neg_a = ScalarF5E3::from(-7);
    /// assert!(neg_a % b == 2);   // -7 % 3 = 2
    /// let neg_b = ScalarF5E3::from(-3);
    /// assert!(a % neg_b == -2);  // 7 % -3 = -2
    ///
    /// // Exploded period, matching signs: numerator preserved
    /// let exploded = ScalarF5E3::MAX * 2;
    /// assert!((ScalarF5E3::PI % exploded) == ScalarF5E3::PI);
    ///
    /// // Exploded period, differing signs: undefined
    /// assert!((ScalarF5E3::PI % -exploded).is_undefined());
    ///
    /// // Vanished period: undefined
    /// let vanished = ScalarF5E3::MIN_POS / 19;
    /// assert!((ScalarF5E3::from(42) % vanished).is_undefined());
    ///
    /// // Zero cases
    /// assert!((ScalarF5E3::ZERO % ScalarF5E3::PI).is_zero());
    /// assert!((ScalarF5E3::PI % ScalarF5E3::ZERO).is_zero());
    /// ```
    pub(crate) fn scalar_modulus_scalar(&self, modulus: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !modulus.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if modulus.is_undefined() {
                return *modulus;
            }
            if self.is_zero() || modulus.is_zero() {
                return Self::ZERO;
            }
            // Rule 3: Transfinite numerator ([↑] or [∞])
            if self.is_transfinite() {
                if modulus.is_transfinite() {
                    return Self {
                        fraction: TRANSFINITE_MODULUS_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    fraction: TRANSFINITE_MODULUS.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Rule 4: Vanished period
            if modulus.vanished() {
                if self.vanished() {
                    return Self {
                        fraction: VANISHED_MODULUS_VANISHED.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    fraction: MODULUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Rule 5: Infinite period (signless — no sign to floor against)
            if modulus.is_infinite() {
                return Self {
                    fraction: MODULUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Rule 6: Exploded period
            if modulus.exploded() {
                if self.is_negative() == modulus.is_negative() {
                    return *self;
                }
                if self.vanished() {
                    return *modulus;
                }
                return Self {
                    fraction: MODULUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Vanished numerator with normal period: |↓| < |#| always
            if self.vanished() {
                if self.is_negative() == modulus.is_negative() {
                    return *self;
                }
                return *modulus;
            }
            return *self;
        }

        // Both operands are normal. Compute floored modulus using proper restoring-divider style remainder: align fractions by exponent, do integer modulo on inflated wide values, apply floored sign rule.
        //
        // Floored mod: result has the sign of the divisor.
        //   same signs:    result = a_mag mod b_mag, signed like a/b
        //   diff signs:    result = b_mag - (a_mag mod b_mag), signed like b
        //
        // |a| < |b| short-circuit:
        //   same signs:    result = a (already in [0, b) magnitude)
        //   diff signs:    result = a + b (one b-step over to land on b's side)

        let a_neg = self.is_negative();
        let b_neg = modulus.is_negative();
        let signs_differ = a_neg != b_neg;

        // |a| < |b|: shortcut with sign correction.
        if self.exponent < modulus.exponent {
            if !signs_differ {
                // Same sign: a is already the remainder.
                return *self;
            }
            // Diff signs: result = a + b. Inlined add (skips abnormal checks — we already know both are normal — and the result can't underflow to zero: |a+b| = |b|-|a| > 0 since |a|<|b|).
            let exp_diff_ba = modulus.exponent.wrapping_sub(&self.exponent);
            let shift_ba: isize = exp_diff_ba.saturate();
            if shift_ba >= Self::fraction_bits() {
                // a is negligible at b's precision: result = b.
                return *modulus;
            }
            let mut big_f = modulus.fraction.inflate(true);
            big_f.w_shl_assign(shift_ba);
            let sum = big_f.w_add(self.fraction.inflate(true));
            let leading = sum.leading_same();
            let offset = self
                .exponent
                .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
            if modulus.exponent.is_negative() && !offset.is_negative() {
                return Self {
                    fraction: sum
                        .w_shl(leading.wrapping_sub(1))
                        .w_shr(Self::fraction_bits())
                        .deflate(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: sum.w_shl(leading).w_shr(Self::fraction_bits()).deflate(),
                exponent: offset,
            };
        }

        let exp_diff_e = self.exponent.wrapping_sub(&modulus.exponent);
        let exp_diff: isize = exp_diff_e.saturate();

        // Exponent gap too wide for an exact remainder in 2*FRAC bits. For very large a relative to b, the remainder isn't representable without a wider intermediate. Match FPGA semantics: return ZERO. (Limit is FRAC-1 to avoid signed overflow on the left shift below.)
        if exp_diff >= Self::fraction_bits() {
            return Self::ZERO;
        }

        // Inflate both fractions to wide effective values, take magnitudes.
        let a_wide = self.fraction.inflate(true);
        let b_wide = modulus.fraction.inflate(true);
        let a_mag = if a_neg { a_wide.w_neg() } else { a_wide };
        let b_mag = if b_neg { b_wide.w_neg() } else { b_wide };

        // Align a's magnitude to b's exponent by shifting left.
        let a_aligned = a_mag.w_shl(exp_diff);

        // Integer modulo via unsigned arithmetic on the bit pattern. (After abs+shift, both magnitudes are positive; unsigned interpretation gives the correct remainder even if signed view overflows.)
        let r_mag = a_aligned.w_rem_unsigned(b_mag);

        if r_mag.w_is_zero() {
            return Self::ZERO;
        }

        // Floored sign correction.
        let result_wide = if signs_differ {
            // result magnitude = |b| - r_mag, sign of b
            let mag = b_mag.w_sub(r_mag);
            if b_neg {
                mag.w_neg()
            } else {
                mag
            }
        } else {
            // result magnitude = r_mag, sign of b (= sign of a)
            if b_neg {
                r_mag.w_neg()
            } else {
                r_mag
            }
        };

        if result_wide.w_is_zero() {
            return Self::ZERO;
        }

        // Normalize result at b's exponent. Same pattern as scalar_add_scalar.
        let leading = result_wide.leading_same();
        let offset = modulus
            .exponent
            .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

        if modulus.exponent.is_negative() && !offset.is_negative() {
            return Self {
                fraction: result_wide
                    .w_shl(leading.wrapping_sub(1))
                    .w_shr(Self::fraction_bits())
                    .deflate(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        Self {
            fraction: result_wide
                .w_shl(leading)
                .w_shr(Self::fraction_bits())
                .deflate(),
            exponent: offset,
        }
    }
}
