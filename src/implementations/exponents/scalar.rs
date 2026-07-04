// src/implementation/exponents/scalar.rs
use crate::core::integer::*;
use crate::core::undefined::*;
use crate::lut::SQRT_LUT;
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
    /// Squares this Scalar. Specialized for a² to avoid the double abnormal check, double inflate, sign comparison, and leading_ones branch of multiply(a, a). Result is always positive, so leading_zeros is the only normalization needed.
    pub fn square(&self) -> Self {
        if !self.is_normal() {
            // Edge cases delegate to multiplication (same tables, same logic).
            return self.scalar_multiply_scalar(self);
        }
        // AMBIG=0 native: cycle-position arithmetic with 2x widening via `cycle_widen`. Matches multiplication's pattern. `pa` is the operand's unsigned cycle position (0=AMBIG, 1..MAX_POS = normal). Direction-aware: `stored_pos > max_pos` → exploded, `< min_pos` → vanished (or AMBIG hit).
        let pa = self.exponent.cycle_widen();
        let bo = Self::binade_origin().cycle_widen();
        let max_pos = Self::max_exponent().cycle_widen();
        let min_pos = Self::min_exponent().cycle_widen();
        let w_one = E::one().cycle_widen();

        // NEG_ONE_NORMAL @ logical k represents -2^(k+1), so (val)² = 2^(2k+2). Cycle position: stored_pos = 2*pa - bo + 2.
        if self.fraction == Self::neg_one_normal() {
            let stored_pos = w_one.w_add(w_one).w_add(pa).w_add(pa).w_sub(bo);
            if stored_pos > max_pos {
                return Self {
                    fraction: Self::pos_one_exploded(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if stored_pos < min_pos {
                return Self {
                    fraction: Self::pos_one_vanished(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: Self::pos_one_normal(),
                exponent: stored_pos.deflate(),
            };
        }
        // Normal path: one inflate, one w_mul, positive result → leading_zeros.
        let inflated = self.fraction.inflate(true);
        let product = inflated.w_mul(inflated);
        let leading = product.w_leading_zeros();
        // Cycle position: stored_pos = 2*pa - bo + 1 - leading. `leading` from w_leading_zeros is non-negative; sign_extend and cycle_widen agree, use cycle_widen for symmetry with the cycle ops.
        let leading_e: E = leading.as_();
        let w_leading = leading_e.cycle_widen();
        let stored_pos = w_one.w_add(pa).w_add(pa).w_sub(bo).w_sub(w_leading);
        if stored_pos > max_pos {
            // Exponent overflow → exploded. When the squared wide wraps negative (the signed wide is one bit short for boundary-adjacent significands), leading == 0 and the shl(-1) would panic under overflow checks; the true product is positive read UNSIGNED, so shift right logically instead.
            let shl_amount = leading.wrapping_sub(1);
            let shifted = if shl_amount >= 0 {
                product.w_shl(shl_amount)
            } else {
                product.w_shr_logical(shl_amount.wrapping_neg())
            };
            let fraction = shifted.w_shr(Self::fraction_bits()).deflate();
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if stored_pos < min_pos {
            // Exponent underflow → vanished. Squaring always yields a positive result, so emit the canonical pos_one_vanished bit pattern rather than the leading-2-extracted fraction (which can wrap into N3 = undefined for products that overflowed the signed wide).
            return Self {
                fraction: Self::pos_one_vanished(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        // Normal: bounds check has guaranteed stored_pos is in [min_pos, max_pos], i.e. a representable non-AMBIG exponent.
        let exponent: E = stored_pos.deflate();
        let fraction = product
            .w_shl(leading)
            .w_shr(Self::fraction_bits())
            .deflate();
        Self { fraction, exponent }
    }
    /// Square root — restoring binary, bit-exact floor. Subtractive method on inflated unsigned value, 1 bit per iteration. Matches hardware spirix_sqrt_iter. Double-wide types only, no multiply.
    pub fn sqrt(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() || self.is_uniform() {
                return *self;
            }
            if self.vanished() {
                return Self {
                    fraction: SQRT_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: SQRT_EXPLODED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Sign is inverse of MSB
        if !self.fraction.is_negative() {
            return Self {
                fraction: SQRT_NEGATIVE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // AMBIG=0: stored = logical ^ binade_origin (E::MIN). Halving must operate on the logical exponent — arithmetic-right-shift floors toward -inf, matching the v0.1 ruler that wants floor(k/2) for both parities. odd is XOR-invariant on bit 0 so it can be read from stored directly.
        let odd: usize = (self.exponent & E::one()).as_();
        let logical_exp = self.exponent ^ Self::binade_origin();
        let result_logical = logical_exp >> 1isize;
        let result_exp = result_logical ^ Self::binade_origin();

        // Subtractive restoring binary sqrt on inflated unsigned value. v0.1 ruler: radicand shift is (FRAC-1)+odd (was FRAC+odd under old). Bit pairs extracted on the fly from eff — keeps everything double-wide.
        let fraction = match Self::fraction_bits() {
            8 => {
                let s: i8 = self.fraction.as_();
                let eff: u16 = s.inflate(true) as u16;
                let shift: usize = 7 + odd;
                let mut rem: u16 = 0;
                let mut root: u16 = 0;
                for i in (0..=8).rev() {
                    let bit_lo = i << 1;
                    let pair = if bit_lo >= shift {
                        (eff >> bit_lo.wrapping_sub(shift)) & 3
                    } else if (bit_lo | 1) >= shift {
                        ((eff >> (bit_lo | 1).wrapping_sub(shift)) & 1) << 1
                    } else {
                        0
                    };
                    rem = (rem << 2) | pair;
                    let trial = (root << 2) | 1;
                    if rem >= trial {
                        rem = rem.wrapping_sub(trial);
                        root = (root << 1) | 1;
                    } else {
                        root <<= 1;
                    }
                }
                // v0.1 ruler: shift = (FRAC-1)+odd, root already at FRAC-bit scale, no halving.
                let _ = odd;
                (root as u8 as i8).as_()
            }
            16 => {
                let s: i16 = self.fraction.as_();
                let eff: u32 = s.inflate(true) as u32;
                let shift: usize = 15 + odd;
                let mut rem: u32 = 0;
                let mut root: u32 = 0;
                for i in (0..=16).rev() {
                    let bit_lo = i << 1;
                    let pair = if bit_lo >= shift {
                        (eff >> bit_lo.wrapping_sub(shift)) & 3
                    } else if (bit_lo | 1) >= shift {
                        ((eff >> (bit_lo | 1).wrapping_sub(shift)) & 1) << 1
                    } else {
                        0
                    };
                    rem = (rem << 2) | pair;
                    let trial = (root << 2) | 1;
                    if rem >= trial {
                        rem = rem.wrapping_sub(trial);
                        root = (root << 1) | 1;
                    } else {
                        root <<= 1;
                    }
                }
                // v0.1 ruler: shift = (FRAC-1)+odd, root already at FRAC-bit scale, no halving.
                let _ = odd;
                (root as u16 as i16).as_()
            }
            32 => {
                let s: i32 = self.fraction.as_();
                let eff: u64 = s.inflate(true) as u64;
                let shift: usize = 31 + odd;
                let mut rem: u64 = 0;
                let mut root: u64 = 0;
                for i in (0..=32).rev() {
                    let bit_lo = i << 1;
                    let pair = if bit_lo >= shift {
                        (eff >> bit_lo.wrapping_sub(shift)) & 3
                    } else if (bit_lo | 1) >= shift {
                        ((eff >> (bit_lo | 1).wrapping_sub(shift)) & 1) << 1
                    } else {
                        0
                    };
                    rem = (rem << 2) | pair;
                    let trial = (root << 2) | 1;
                    if rem >= trial {
                        rem = rem.wrapping_sub(trial);
                        root = (root << 1) | 1;
                    } else {
                        root <<= 1;
                    }
                }
                // v0.1 ruler: shift = (FRAC-1)+odd, root already at FRAC-bit scale, no halving.
                let _ = odd;
                (root as u32 as i32).as_()
            }
            64 => {
                let s: i64 = self.fraction.as_();
                let eff: u128 = s.inflate(true) as u128;
                let shift: usize = 63 + odd;
                let mut rem: u128 = 0;
                let mut root: u128 = 0;
                for i in (0..=64).rev() {
                    let bit_lo = i << 1;
                    let pair = if bit_lo >= shift {
                        (eff >> bit_lo.wrapping_sub(shift)) & 3
                    } else if (bit_lo | 1) >= shift {
                        ((eff >> (bit_lo | 1).wrapping_sub(shift)) & 1) << 1
                    } else {
                        0
                    };
                    rem = (rem << 2) | pair;
                    let trial = (root << 2) | 1;
                    if rem >= trial {
                        rem = rem.wrapping_sub(trial);
                        root = (root << 1) | 1;
                    } else {
                        root <<= 1;
                    }
                }
                // v0.1 ruler: shift = (FRAC-1)+odd, root already at FRAC-bit scale, no halving.
                let _ = odd;
                (root as u64 as i64).as_()
            }
            128 => {
                let s: i128 = self.fraction.as_();
                let eff: I256 = s.inflate(true);
                let shift: usize = 127 + odd;
                let zero = I256::from(0i128);
                let one = I256::from(1i128);
                let three = I256::from(3i128);
                let mut rem = zero;
                let mut root = zero;
                for i in (0..=128).rev() {
                    let bit_lo = i << 1;
                    let pair = if bit_lo >= shift {
                        (eff >> bit_lo.wrapping_sub(shift)) & three
                    } else if (bit_lo | 1) >= shift {
                        ((eff >> (bit_lo | 1).wrapping_sub(shift)) & one) << 1usize
                    } else {
                        zero
                    };
                    rem = (rem << 2usize) | pair;
                    let trial = (root << 2usize) | one;
                    if rem >= trial {
                        rem = rem.wrapping_sub(trial);
                        root = (root << 1usize) | one;
                    } else {
                        root = root << 1usize;
                    }
                }
                // v0.1 ruler: shift = (FRAC-1)+odd absorbs the odd-bit, root already at FRAC-bit scale — no halving (matches the 8/16/32/64-bit branches above; the previous `if odd != 0 { root >>= 1 }` was a stale half-bit correction from the old ruler that produced negative N0 fractions for sqrts in the odd-binade tail). Extract low 128 bits of I256 via byte reassembly.
                let bytes = root.to_le_bytes();
                let r_low = i128::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                    bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                    bytes[15],
                ]);
                r_low.as_()
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        };

        Self {
            fraction,
            exponent: result_exp,
        }
    }

    /// Square root via LUT-seeded Newton-Raphson — within ±1 ULP of the floor. SQRT_LUT seeds 8 bits, Newton doubles per step (~2 iters for F5E3).
    ///
    /// Unlike [`sqrt`](Self::sqrt), which uses subtractive restoring and is the canonical bit-exact floor, this routine can drift by 1 ULP at last-bit boundaries. Newton can't self-correct because Spirix's multiply itself floors, so verifying `candidate² ≤ self` at ULP precision isn't reliable without double-wide arithmetic. Prefer `sqrt()` when bit-exact floor matters; use this for the LUT-warm-start performance trade-off.
    #[doc(hidden)] // Alternative sqrt used by benchmarks/comparisons — sqrt() is the stable API.
    pub fn sqrt_newton(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() || self.is_uniform() {
                return *self;
            }
            if self.vanished() {
                return Self {
                    fraction: SQRT_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: SQRT_EXPLODED.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        if self.is_negative() {
            return Self {
                fraction: SQRT_NEGATIVE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // LUT seed: top 8 bits of stored fraction (= top 8 of inflate, XOR is above FRAC).
        let idx: usize = self.fraction.sa::<u8>() as usize;
        let guess_frac: F = SQRT_LUT[idx].sa();

        // AMBIG=0: halve the *logical* exponent (stored = logical ^ E::MIN, so a raw `>> 1` of stored gives the wrong binade). The Newton seed at result_exp=0 sat at the wrong scale, so every input was being projected to a fixed point unrelated to its real square root.
        let odd: E = self.exponent & E::one();
        let logical_k = self.exponent ^ Self::binade_origin();
        let result_logical = (logical_k >> 1usize) + odd;
        let result_exp = result_logical ^ Self::binade_origin();
        let mut guess = Self {
            fraction: guess_frac,
            exponent: result_exp,
        };

        // Newton: x_{n+1} = (x_n + S/x_n) / 2. Converges monotonically, but can oscillate between floor and ceil at the last ULP. Track previous to detect the 2-cycle.
        let mut prev = guess;
        loop {
            let next = (guess + *self / guess) >> 1u8;
            if next.fraction == guess.fraction {
                break;
            }
            if next.fraction == prev.fraction {
                // 2-cycle: pick whichever neighbour the bitwise sqrt would. Spirix's multiply floors, so comparing squares with `<=` is unreliable at the last ULP; defer to the canonical floor.
                break;
            }
            prev = guess;
            guess = next;
        }
        guess
    }

    pub fn ln(&self) -> Self {
        let binary_log = self.lb();
        binary_log * Self::LN_TWO
    }

    pub fn lb(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }

            if self.is_zero() || self.is_infinite() {
                return Self::INFINITY;
            }
            if self.exploded() {
                return Self {
                    fraction: TRANSFINITE_LOG.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: NEGLIGIBLE_LOG.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        if self.is_negative() {
            return Self {
                fraction: NEGATIVE_LOG.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // x in normal form has magnitude m in [1, 2) at the +1.0 binade, so value = m * 2^logical_k, lb(x) = lb(m) + logical_k, lb(m) in [0, 1), floor(lb(x)) = logical_k. The characteristic is the logical exponent; recover it via stored ^ binade_origin.
        let characteristic = self.exponent ^ Self::binade_origin();

        // Normalize x to [1, 2), i.e. anchor at the +1.0 binade.
        let mut x = *self;
        x.exponent = Self::binade_origin();

        // Build fractional bits directly into u128 — each iteration is ~1 OR + 1 shift. Bit (FRAC-1) = 0.5 contribution, bit (FRAC-2) = 0.25, etc.
        let mut raw_frac: u128 = 0;
        let mut rotor: u128 = 1u128 << (Self::fraction_bits().wrapping_sub(1) as u32);
        while rotor != 0 {
            x = x.square();
            // x in [2, 4) after squaring means it left the +1.0 binade upward — unsigned cycle position > binade_origin position.
            if x.exponent.into_unsigned() > Self::binade_origin().into_unsigned() {
                raw_frac |= rotor;
                x.exponent = x.exponent.wrapping_sub(&E::one());
            }
            rotor >>= 1;
        }

        // Convert raw_frac (FRAC-bit fractional fixed-point) to a normalized Scalar. raw_frac has meaningful bits in positions 0..FRAC-1. Normalize by shifting the highest set bit up to position FRAC-1; the shift amount becomes -exponent.
        let fractional = if raw_frac == 0 {
            Self::ZERO
        } else {
            let frac_bits = Self::fraction_bits();
            // leading_zeros in the FRAC-bit view of raw_frac (raw_frac fits in FRAC bits)
            let lz = (raw_frac.leading_zeros() as isize).wrapping_sub(128 - frac_bits);
            let normalized_u = raw_frac << lz;
            let stored: F = match frac_bits {
                8 => (normalized_u as u8 as i8).as_(),
                16 => (normalized_u as u16 as i16).as_(),
                32 => (normalized_u as u32 as i32).as_(),
                64 => (normalized_u as u64 as i64).as_(),
                128 => (normalized_u as i128).as_(),
                _ => unreachable!(),
            };
            // raw_frac has bit (FRAC-1) = 0.5 contribution; the logical exponent for this Scalar is -lz - 1. Convert that logical k to AMBIG=0 stored form via ^ binade_origin.
            let logical_exp_e: E = (0isize).wrapping_sub(lz).wrapping_sub(1).as_();
            Self {
                fraction: stored,
                exponent: logical_exp_e ^ Self::binade_origin(),
            }
        };

        // Convert characteristic (E type) to Scalar and combine.
        let characteristic_scalar = match Self::exponent_bits() {
            8 => {
                let e: i8 = characteristic.as_();
                Self::from(e)
            }
            16 => {
                let e: i16 = characteristic.as_();
                Self::from(e)
            }
            32 => {
                let e: i32 = characteristic.as_();
                Self::from(e)
            }
            64 => {
                let e: i64 = characteristic.as_();
                Self::from(e)
            }
            128 => {
                let e: i128 = characteristic.as_();
                Self::from(e)
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                }
            }
        };

        characteristic_scalar + fractional
    }

    /// Computes e raised to the power of this Scalar value (e^x)
    ///
    /// # Description
    ///
    /// Calculates the exponential function e^x thru a combination of range reduction and Taylor series expansion. This implementation uses argument reduction by separating integer and fractional parts to improve convergence speed and numerical stability.
    ///
    /// Calculation process:
    /// 0. Checks for ambiguous values (undefined, exploded, vanished, Zero) and handles accordinly
    /// 1. Splits input x into integer and fractional parts (x = int + frac)
    /// 2. Computes e^frac using Taylor series: 1 + frac + frac²/2! + frac³/3! + ...
    /// 3. Uses binary exponentiation (square-and-multiply algorithm) for e^int
    /// 4. Combines results as e^x = e^int × e^frac
    /// 5. Detects convergence thru value stabilization
    ///
    /// # Returns
    ///
    /// - `[℘]` ➔ `[℘]` Undefined values remain undefined
    /// - `[0]` ➔ `[1]` e^0 = 1 (identity property)
    /// - `[↑+]` ➔ `[↑ e^+∞]` Positive infinity yields positive infinity
    /// - `[↑-]` ➔ `[0]` Negative infinity yields zero
    /// - `[↓-]` ➔ `[1-]` Tiny negative yields slightly less than 1
    /// - `[↓+]` ➔ `[1]` Tiny positive yields exactly 1
    /// - `[#]` ➔ `[#]` or `[↑]` or `[0]` A finite, exploded, vanished or zero Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // Basic exponential function: e^0 = 1 let zero = ScalarF6E4::ZERO; assert!(zero.exp() == 1_i64);
    ///
    /// // Undefined input stays undefined let undef: ScalarF6E4 = ScalarF6E4::ZERO / 0_i64; assert!(undef.exp().is_undefined());
    ///
    /// // Tiny values: e^tiny ≈ 1 let tiny_pos: ScalarF6E4 = ScalarF6E4::MIN_POS / 10_i64; assert!(tiny_pos.vanished() && tiny_pos.exp() == 1_i64);
    /// ```
    pub fn exp(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.is_zero() {
                return Self::ONE;
            }
            if self.is_infinite() {
                return *self;
            }
            if self.exploded() {
                if self.is_negative() {
                    return Self::ZERO;
                } else {
                    return Self {
                        fraction: POWER_TRANSFINITE.prefix.sa(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
            }
            // Vanished values
            if self.is_negative() {
                // e^(tiny negative) = a smidge less than 1
                return Self::EFFECTIVELY_POS_ONE;
            } else {
                // e^(tiny positive) = 1
                return Self::ONE;
            }
        }

        let integer_part = self.floor();

        let fractional_part = self - integer_part;

        let mut current_sum = Self::ONE;
        let mut previous_sum = Self::ZERO;
        let mut term_factorial = Self::ONE;
        let mut term_power = Self::ONE;

        let mut term_index: isize = 1;
        while current_sum.is_normal() {
            previous_sum = current_sum;
            term_power *= fractional_part;
            term_factorial *= term_index;
            current_sum += term_power / term_factorial;
            if current_sum == previous_sum {
                break;
            }

            term_index += 1;
        }

        let mut integer_result = Self::ONE;
        let mut current_power = Self::E;

        let mut remaining_exponent = integer_part.magnitude();

        // Walk every set bit of the integer part.
        // The bound is the FRACTION width, not the exponent width: `remaining_exponent` is a magnitude living in the fraction, so it can carry far more bits than the exponent type (e.g. 2^1108.5 needs e^768, and 768 is 10 bits wide in an i8-exponent Scalar).
        // The old `exponent_bits()` cap silently dropped the high bits, collapsing large results back to a bogus normal.
        for _bit in 0..Self::fraction_bits() {
            if (remaining_exponent & Self::ONE) == 1 {
                integer_result *= current_power;
            }
            remaining_exponent = remaining_exponent >> 1;
            if remaining_exponent.vanished() || remaining_exponent.is_zero() {
                break; // Exit when done processing bits
            }
            // Once the running product has escaped it can never return to normal range (multiply carries [^] x [#] = [^]), so the result is already settled and further squaring is wasted work.
            if !integer_result.is_normal() {
                break;
            }
            // Square for the next set bit.
            // If this escapes (e.g. E^128 in an i8 exponent), we must NOT stop on the square alone: a higher set bit still needs to multiply this escaped power into the result so the answer escapes too.
            // reciprocal() later turns a negative integer_part into the matching vanished/zero.
            current_power = current_power.square();
        }

        if integer_part.is_negative() {
            integer_result = integer_result.reciprocal();
        }

        previous_sum * integer_result
    }

    /// Computes 2 raised to the power of this Scalar value (2^x)
    ///
    /// # Description
    ///
    /// Calculates the binary exponential function 2^x by leveraging the natural exponential function. Uses the mathematical identity: 2^x = e^(x * ln(2))
    ///
    /// Computation process:
    /// 1. Multiplies input by ln(2) (natural logarithm of 2)
    /// 2. Applies the exponential function to the result
    ///
    /// # Returns
    ///
    /// Returns identical special cases as the exp() function, but for base 2 instead of base e:
    /// - `[℘]` ➔ `[℘]` Undefined values remain undefined
    /// - `[0]` ➔ `[1]` 2^0 = 1 (identity property)
    /// - `[↑+]` ➔ `[↑ 2^+∞]` Positive infinity yields positive infinity
    /// - `[↑-]` ➔ `[0]` Negative infinity yields zero
    /// - `[↓-]` ➔ `[1-]` Tiny negative yields slightly less than 1
    /// - `[↓+]` ➔ `[1]` Tiny positive yields exactly 1
    /// - `[#]` ➔ `[#]` or `[↑]` or `[0]` A finite, exploded, or zero Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // 2^0 = 1 let zero = ScalarF6E4::ZERO; assert!(zero.powb() == 1_i64);
    ///
    /// // Undefined input → undefined output let undef: ScalarF6E4 = ScalarF6E4::ZERO / 0_i64; assert!(undef.powb().is_undefined());
    /// ```
    pub fn powb(&self) -> Self {
        (self * Self::LN_TWO).exp()
    }
}
