use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub, Zero};
use core::ops::*;
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

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return *big;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return *big;
                }
            }

            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: i16 = big.fraction.as_();
                    big_f <<= shift;
                    let small_f: i16 = small.fraction.as_();
                    let result = big_f.wrapping_add(small_f);
                    if result.is_zero() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: i32 = big.fraction.as_();
                    big_f <<= shift;
                    let small_f: i32 = small.fraction.as_();
                    let result = big_f.wrapping_add(small_f);
                    if result.is_zero() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: i64 = big.fraction.as_();
                    big_f <<= shift;
                    let small_f: i64 = small.fraction.as_();
                    let result = big_f.wrapping_add(small_f);
                    if result.is_zero() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: i128 = big.fraction.as_();
                    big_f <<= shift;
                    let small_f: i128 = small.fraction.as_();
                    let result = big_f.wrapping_add(small_f);
                    if result.is_zero() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: I256 = big.fraction.into();
                    big_f <<= shift;
                    let small_f: I256 = small.fraction.into();
                    let result = big_f.wrapping_add(small_f);
                    if result == 0.into() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading)).as_());
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {
                    return Self {
                        fraction: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    }
                }
            }
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
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() && scalar.vanished() {
            return Self {
                fraction: VANISHED_PLUS_VANISHED.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if scalar.is_transfinite() {
            return Self {
                fraction: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() {
            return *scalar;
        }
        if scalar.vanished() {
            return *self;
        }
        if self.is_zero() {
            return *scalar;
        }
        return *self;
    }
    pub fn scalar_add_scalar_closefar(&self, scalar: &Self) -> Self {
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

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return *big;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return *big;
                }
            }

            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    // Pre-shift both by 1 into i16 (guard bit, free wiring in Verilog)
                    // INT_BITS = FRAC + 2 = 10 (sign + guard + 8 frac bits)
                    let big_ext: i16 = {
                        let f: i8 = big.fraction.as_();
                        (f as i16) << 1
                    };
                    let small_ext: i16 = {
                        let f: i8 = small.fraction.as_();
                        (f as i16) << 1
                    };

                    if shift == 0 {
                        // === CLOSE PATH: exponents match ===
                        // No alignment needed. Full normalization (cancellation possible).
                        let sum: i16 = big_ext.wrapping_add(small_ext);

                        if sum == 0 {
                            return Self {
                                fraction: F::ZERO,
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }

                        let leading = sum.leading_ones().max(sum.leading_zeros()) as isize;
                        let normalized = sum << (leading - 1);
                        let out_frac = (normalized >> (16 - F::FRACTION_BITS)) as i8;
                        // out_exp = big_exp + (16 - FRAC_BITS) - leading
                        // (16 because we're in i16; in Verilog INT_BITS=FRAC+2 so it's +2)
                        let out_exp: E = big
                            .exponent
                            .wrapping_add(&((16 - F::FRACTION_BITS) as isize).as_())
                            .wrapping_sub(&(leading as isize).as_());

                        // Underflow: big_exp negative but out_exp-1 positive
                        if big.exponent.is_negative()
                            && !out_exp.wrapping_sub(&E::ONE).is_negative()
                        {
                            return Self {
                                fraction: ((sum << (leading - 2)) >> (16 - F::FRACTION_BITS)).as_(),
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }

                        return Self {
                            fraction: out_frac.as_(),
                            exponent: out_exp,
                        };
                    } else {
                        // === FAR PATH: exp_diff >= 1 ===
                        // Alignment barrel shift on small. Normalization at most ±1 bit.
                        // Banker's rounding: sticky = bits lost in alignment shift
                        let small_aligned: i16 = small_ext >> shift;
                        let sticky = (small_aligned << shift) != small_ext;
                        let sum: i16 = big_ext.wrapping_add(small_aligned);

                        if sum == 0 {
                            return Self {
                                fraction: F::ZERO,
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }
                        let leading = sum.leading_ones().max(sum.leading_zeros()) as isize;
                        let normalized = sum << (leading - 1);
                        let out_frac_raw = (normalized >> (16 - F::FRACTION_BITS)) as i8;

                        // Banker's rounding: guard is the bit just below the fraction
                        // In i16 after normalization, fraction is top 8 bits.
                        // Guard bit is bit (16 - FRAC - 1) = bit 7 of normalized.
                        let guard = (normalized >> (16 - F::FRACTION_BITS - 1)) & 1 != 0;
                        let lsb_odd = out_frac_raw & 1 != 0;
                        let round_up = guard && (sticky || lsb_odd);
                        let out_frac = if round_up {
                            out_frac_raw.wrapping_add(1)
                        } else {
                            out_frac_raw
                        };

                        // out_exp = big_exp + (16 - FRAC_BITS) - leading
                        let out_exp: E = big
                            .exponent
                            .wrapping_add(&((16 - F::FRACTION_BITS) as isize).as_())
                            .wrapping_sub(&(leading as isize).as_());

                        if big.exponent.is_negative()
                            && !out_exp.wrapping_sub(&E::ONE).is_negative()
                        {
                            return Self {
                                fraction: ((sum << (leading - 2)) >> (16 - F::FRACTION_BITS)).as_(),
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }

                        return Self {
                            fraction: out_frac.as_(),
                            exponent: out_exp,
                        };
                    }
                }
                25 => {
                    let shift: isize = exp_diff.as_();
                    let big_ext: i32 = {
                        let f: i32 = big.fraction.as_();
                        f << 1
                    };
                    let small_ext: i32 = {
                        let f: i32 = small.fraction.as_();
                        f << 1
                    };

                    if shift == 0 {
                        let sum: i32 = big_ext.wrapping_add(small_ext);
                        if sum == 0 {
                            return Self {
                                fraction: F::ZERO,
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }
                        let leading = sum.leading_ones().max(sum.leading_zeros()) as isize;
                        let normalized = sum << (leading - 1);
                        let out_frac = normalized >> 7;
                        let out_exp: E = big
                            .exponent
                            .wrapping_add(&(7isize).as_())
                            .wrapping_sub(&(leading as isize).as_());
                        if big.exponent.is_negative()
                            && !out_exp.wrapping_sub(&E::ONE).is_negative()
                        {
                            return Self {
                                fraction: ((sum << (leading - 2)) >> 7).as_(),
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }
                        return Self {
                            fraction: out_frac.as_(),
                            exponent: out_exp,
                        };
                    } else {
                        let small_aligned: i32 = small_ext >> shift;
                        let sticky = (small_aligned << shift) != small_ext;
                        let sum: i32 = big_ext.wrapping_add(small_aligned);
                        if sum == 0 {
                            return Self {
                                fraction: F::ZERO,
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }
                        let leading = sum.leading_ones().max(sum.leading_zeros()) as isize;
                        let normalized = sum << (leading - 1);
                        let out_frac_raw = normalized >> 7;
                        let guard = (normalized >> 6) & 1 != 0;
                        let lsb_odd = out_frac_raw & 1 != 0;
                        let round_up = guard && (sticky || lsb_odd);
                        let out_frac = if round_up {
                            out_frac_raw.wrapping_add(1)
                        } else {
                            out_frac_raw
                        };
                        let out_exp: E = big
                            .exponent
                            .wrapping_add(&(7isize).as_())
                            .wrapping_sub(&(leading as isize).as_());
                        if big.exponent.is_negative()
                            && !out_exp.wrapping_sub(&E::ONE).is_negative()
                        {
                            return Self {
                                fraction: ((sum << (leading - 2)) >> 7).as_(),
                                exponent: E::AMBIGUOUS_EXPONENT,
                            };
                        }
                        return Self {
                            fraction: out_frac.as_(),
                            exponent: out_exp,
                        };
                    }
                }
                _ => {
                    return Self {
                        fraction: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    }
                }
            }
        }
        // Non-normal cases: same as the other implementations
        if self.is_undefined() {
            return *self;
        }
        if scalar.is_undefined() {
            return *scalar;
        }
        if self.is_transfinite() && scalar.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() && scalar.vanished() {
            return Self {
                fraction: VANISHED_PLUS_VANISHED.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if scalar.is_transfinite() {
            return Self {
                fraction: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() {
            return *scalar;
        }
        if scalar.vanished() {
            return *self;
        }
        if self.is_zero() {
            return *scalar;
        }
        return *self;
    }

    pub fn scalar_add_scalar_verilog(&self, scalar: &Self) -> Self {
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

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return *big;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return *big;
                }
            }

            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_f: i16 = big.fraction.as_();
                    big_f <<= 7;
                    let mut small_f: i16 = small.fraction.as_();
                    small_f <<= 7;
                    let prev = small_f;
                    small_f >>= shift;
                    let sticky = small_f << shift != prev;
                    let result = big_f.wrapping_add(small_f) & (i16::MIN >> 10);
                    if result.is_zero() {
                        return Self {
                            fraction: F::ZERO,
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = small.exponent.wrapping_add(
                        &(F::FRACTION_BITS
                            .wrapping_sub(leading)
                            .wrapping_sub(7)
                            .wrapping_add(shift))
                        .as_(),
                    );
                    if big.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {
                    return Self {
                        fraction: GENERAL.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    }
                }
            }
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
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() && scalar.vanished() {
            return Self {
                fraction: VANISHED_PLUS_VANISHED.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.is_transfinite() {
            return Self {
                fraction: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if scalar.is_transfinite() {
            return Self {
                fraction: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if self.vanished() {
            return *scalar;
        }
        if scalar.vanished() {
            return *self;
        }
        if self.is_zero() {
            return *scalar;
        }
        return *self;
    }
}
