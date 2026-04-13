use crate::core::integer::{Inflate, FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// Reciprocal lookup table for 16-bit normalized fractions
/// Index = bits[13:6] of denominator fraction (8 bits, 256 entries)
/// Returns normalized reciprocal fraction for range [1.0, 2.0)
/// Scaled by 2^14 (not 2^15) to fit in signed i16
const RECIPROCAL_LUT_16: [i16; 256] = [
    0x4000, 0x3fc0, 0x3f80, 0x3f42, 0x3f03, 0x3ec6, 0x3e88, 0x3e4b, 0x3e0f, 0x3dd3, 0x3d98, 0x3d5d,
    0x3d22, 0x3ce8, 0x3cae, 0x3c75, 0x3c3c, 0x3c03, 0x3bcb, 0x3b94, 0x3b5c, 0x3b25, 0x3aef, 0x3ab9,
    0x3a83, 0x3a4e, 0x3a19, 0x39e4, 0x39b0, 0x397c, 0x3949, 0x3916, 0x38e3, 0x38b1, 0x387f, 0x384d,
    0x381c, 0x37eb, 0x37ba, 0x3789, 0x3759, 0x372a, 0x36fa, 0x36cb, 0x369d, 0x366e, 0x3640, 0x3612,
    0x35e5, 0x35b7, 0x358a, 0x355e, 0x3531, 0x3505, 0x34da, 0x34ae, 0x3483, 0x3458, 0x342d, 0x3403,
    0x33d9, 0x33af, 0x3385, 0x335c, 0x3333, 0x330a, 0x32e1, 0x32b9, 0x3291, 0x3269, 0x3241, 0x321a,
    0x31f3, 0x31cc, 0x31a6, 0x317f, 0x3159, 0x3133, 0x310d, 0x30e8, 0x30c3, 0x309e, 0x3079, 0x3054,
    0x3030, 0x300c, 0x2fe8, 0x2fc4, 0x2fa0, 0x2f7d, 0x2f5a, 0x2f37, 0x2f14, 0x2ef2, 0x2ecf, 0x2ead,
    0x2e8b, 0x2e69, 0x2e48, 0x2e26, 0x2e05, 0x2de4, 0x2dc3, 0x2da3, 0x2d82, 0x2d62, 0x2d42, 0x2d22,
    0x2d02, 0x2ce3, 0x2cc3, 0x2ca4, 0x2c85, 0x2c66, 0x2c47, 0x2c29, 0x2c0b, 0x2bec, 0x2bce, 0x2bb0,
    0x2b93, 0x2b75, 0x2b58, 0x2b3a, 0x2b1d, 0x2b00, 0x2ae3, 0x2ac7, 0x2aaa, 0x2a8e, 0x2a72, 0x2a55,
    0x2a3a, 0x2a1e, 0x2a02, 0x29e7, 0x29cb, 0x29b0, 0x2995, 0x297a, 0x295f, 0x2944, 0x292a, 0x2910,
    0x28f5, 0x28db, 0x28c1, 0x28a7, 0x288d, 0x2874, 0x285a, 0x2841, 0x2828, 0x280f, 0x27f6, 0x27dd,
    0x27c4, 0x27ab, 0x2793, 0x277a, 0x2762, 0x274a, 0x2732, 0x271a, 0x2702, 0x26ea, 0x26d3, 0x26bb,
    0x26a4, 0x268c, 0x2675, 0x265e, 0x2647, 0x2630, 0x261a, 0x2603, 0x25ed, 0x25d6, 0x25c0, 0x25aa,
    0x2593, 0x257d, 0x2568, 0x2552, 0x253c, 0x2526, 0x2511, 0x24fb, 0x24e6, 0x24d1, 0x24bc, 0x24a7,
    0x2492, 0x247d, 0x2468, 0x2454, 0x243f, 0x242a, 0x2416, 0x2402, 0x23ee, 0x23d9, 0x23c5, 0x23b1,
    0x239e, 0x238a, 0x2376, 0x2362, 0x234f, 0x233c, 0x2328, 0x2315, 0x2302, 0x22ef, 0x22dc, 0x22c9,
    0x22b6, 0x22a3, 0x2290, 0x227e, 0x226b, 0x2259, 0x2246, 0x2234, 0x2222, 0x220f, 0x21fd, 0x21eb,
    0x21d9, 0x21c8, 0x21b6, 0x21a4, 0x2192, 0x2181, 0x216f, 0x215e, 0x214d, 0x213b, 0x212a, 0x2119,
    0x2108, 0x20f7, 0x20e6, 0x20d5, 0x20c4, 0x20b3, 0x20a3, 0x2092, 0x2082, 0x2071, 0x2061, 0x2050,
    0x2040, 0x2030, 0x2020, 0x2010,
];

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
    pub fn scalar_divide_scalar_newton(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if other.is_zero() {
                if self.is_zero() {
                    return Self {
                        fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Self {
                        fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_infinite() {
                return Self::ZERO;
            }
            if self.exploded() && other.exploded() {
                return Self {
                    fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let fraction = match F::FRACTION_BITS {
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if other.exploded() {
                let fraction = match F::FRACTION_BITS {
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    _ => return self.scalar_divide_scalar(other),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let fraction = match F::FRACTION_BITS {
                16 => {
                    let numerator: i32 = self.fraction.as_();
                    let denominator: i32 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                _ => return self.scalar_divide_scalar(other),
            };
            return Self {
                fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let (fraction, expo_adjust) = match F::FRACTION_BITS {
            8 => {
                let numerator: i16 = self.fraction.as_();
                let denominator: i16 = other.fraction.as_();

                // Newton-Raphson for 8-bit
                let abs_denom = denominator.abs();

                // Extract top 8 bits for LUT index (bits 7-0, but normalize to 6-bit range)
                // For 8-bit normalized fractions, we only have 6 bits of fraction data
                let index = ((abs_denom >> 0) & 0x3F) as usize; // Use 64 entries
                let mut recip = RECIPROCAL_LUT_16[index * 4] as i16; // Subsample LUT

                // Scale from 2^14 to 2^6 scale
                recip = recip >> 8;

                // Newton-Raphson iterations (1 iteration for 8-bit)
                for _ in 0..1 {
                    let prod = (abs_denom * recip) >> 6;
                    let error = 0x80 - prod; // 2.0 in 2^6 scale
                    recip = ((recip as i32 * error as i32) >> 6) as i16;
                }

                // Final multiplication
                let sign_adjust = if denominator < 0 { -1 } else { 1 };
                let quotient = ((numerator as i32 * recip as i32 * sign_adjust) >> 4) as i16;

                // Normalize
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                let normalized = quotient << shift;

                (
                    (normalized >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            16 => {
                let numerator: i32 = self.fraction.as_();
                let denominator: i32 = other.fraction.as_();

                // Newton-Raphson reciprocal division
                // Step 1: Get absolute value for LUT lookup
                let abs_denom = denominator.abs();

                // Step 2: LUT lookup - index with bits [13:6] (top 8 bits after sign+norm)
                let index = ((abs_denom >> 6) & 0xFF) as usize;
                let mut recip = RECIPROCAL_LUT_16[index] as i32;

                // Step 3: Newton-Raphson refinement
                // Formula: y_new = y * (2 - b*y)
                // In fixed-point with both denom and recip in 2^14 scale:
                //   prod = (denom * recip) >> 14 gives 2^14 scale
                //   error = 0x8000 - prod (2.0 in 2^14 scale)
                //   recip = (recip * error) >> 14 (both 2^14, result 2^14)
                for _ in 0..2 {
                    let prod = (abs_denom * recip) >> 14;
                    let error = 0x8000 - prod;
                    recip = ((recip as i64 * error as i64) >> 14) as i32;
                }

                // Step 4: Divide = multiply by reciprocal
                // Numerator is scaled by 2^14, reciprocal LUT is scaled by 2^14
                // Product: (n * 2^14) * ((1/d) * 2^14) = (n/d) * 2^28
                // Shift by 12 gives closest match to Euclidean division
                let sign_adjust = if denominator < 0 { -1 } else { 1 };
                let quotient = ((numerator as i64 * recip as i64 * sign_adjust) >> 12) as i32;

                // Step 5: Normalize result
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                let normalized = quotient << shift;

                (
                    (normalized >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            32 => {
                let numerator: i64 = self.fraction.as_();
                let denominator: i64 = other.fraction.as_();

                // Newton-Raphson for 32-bit
                let abs_denom = denominator.abs();

                // Extract top 8 bits for LUT index (bits 31-24)
                let index = ((abs_denom >> 24) & 0xFF) as usize;
                let mut recip = RECIPROCAL_LUT_16[index] as i64;

                // Scale from 2^14 to 2^30 (for 32-bit)
                recip = recip << 16;

                // Newton-Raphson iterations (3 iterations for 32-bit)
                for _ in 0..3 {
                    let prod = (abs_denom * recip) >> 30;
                    let error = (1i64 << 31) - prod; // 2.0 in 2^30 scale
                    recip = ((recip as i128 * error as i128) >> 30) as i64;
                }

                // Final multiplication
                let sign_adjust = if denominator < 0 { -1 } else { 1 };
                let quotient = ((numerator as i128 * recip as i128 * sign_adjust) >> 28) as i64;

                // Normalize
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                let normalized = quotient << shift;

                (
                    (normalized >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            64 => {
                let numerator: i128 = self.fraction.as_();
                let denominator: i128 = other.fraction.as_();

                // Newton-Raphson for 64-bit
                let abs_denom = denominator.abs();

                // Extract top 8 bits for LUT index (bits 63-56)
                let index = ((abs_denom >> 56) & 0xFF) as usize;
                let mut recip = RECIPROCAL_LUT_16[index] as i128;

                // Scale from 2^14 to 2^62 (for 64-bit)
                recip = recip << 48;

                // Newton-Raphson iterations (4 iterations for 64-bit)
                for _ in 0..4 {
                    let prod = (abs_denom * recip) >> 62;
                    let error = (1i128 << 63) - prod; // 2.0 in 2^62 scale
                    recip = ((recip as i128 * error as i128) >> 62) as i128;
                }

                // Final multiplication
                let sign_adjust = if denominator < 0 { -1 } else { 1 };
                let quotient = ((numerator as i128 * recip as i128 * sign_adjust) >> 60) as i128;

                // Normalize
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                let normalized = quotient << shift;

                (
                    (normalized >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            128 => {
                let numerator: i256::I256 = self.fraction.into();
                let denominator: i256::I256 = other.fraction.into();

                // Newton-Raphson for 128-bit
                // Extract top 8 bits for LUT index (bits 127-120)
                let zero = i256::I256::from(0);
                let one = i256::I256::from(1);
                let abs_denom = if denominator < zero {
                    zero - denominator
                } else {
                    denominator
                };

                // Shift right by 120 to get top 8 bits
                let index_val: i256::I256 = abs_denom >> 120;
                let index = (index_val.as_i32() & 0xFF) as usize;
                let mut recip = i256::I256::from(RECIPROCAL_LUT_16[index]);

                // Scale LUT value from 2^14 to 2^126 (for 128-bit)
                recip = recip << 112; // 126 - 14 = 112

                // Newton-Raphson iterations (need more for 128-bit)
                for _ in 0..6 {
                    let prod = (abs_denom * recip) >> 126;
                    let error = (one << 127) - prod; // 2.0 in 2^126 scale
                    recip = (recip * error) >> 126;
                }

                // Final multiplication
                let sign_adjust = if denominator < zero { -one } else { one };
                let quotient: i256::I256 = (numerator * recip * sign_adjust) >> 124; // Shift to match Euclidean format

                // Normalize
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                let normalized: i256::I256 = quotient << shift;

                (
                    (normalized >> F::FRACTION_BITS).as_i128().as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            _ => {
                // For other bit widths, use the original Euclidean division
                return self.scalar_divide_scalar(other);
            }
        };

        match E::EXPONENT_BITS {
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let other_exponent: i32 = other.exponent.as_();
                let upcast_exponent: i32 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i32);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
    pub(crate) fn scalar_divide_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if other.is_zero() {
                if self.is_zero() {
                    return Self {
                        fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_infinite() {
                if other.is_infinite() {
                    return Self {
                        fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_infinite() {
                return Self::ZERO;
            }
            if self.exploded() && other.exploded() {
                return Self {
                    fraction: TRANSFINITE_DIVIDE_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && other.vanished() {
                return Self {
                    fraction: NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let fraction = match F::FRACTION_BITS {
                    8 => {
                        let numerator: i16 = self.fraction.as_();
                        let denominator: i16 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    32 => {
                        let numerator: i64 = self.fraction.as_();
                        let denominator: i64 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    64 => {
                        let numerator: i128 = self.fraction.as_();
                        let denominator: i128 = other.fraction.as_();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    128 => {
                        let numerator: I256 = self.fraction.into();
                        let denominator: I256 = other.fraction.into();
                        let mut quotient = (numerator << (F::FRACTION_BITS.wrapping_add(1)))
                            .div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_i128().as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if other.exploded() {
                let fraction = match F::FRACTION_BITS {
                    8 => {
                        let numerator: i16 = self.fraction.as_();
                        let denominator: i16 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    16 => {
                        let numerator: i32 = self.fraction.as_();
                        let denominator: i32 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    32 => {
                        let numerator: i64 = self.fraction.as_();
                        let denominator: i64 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    64 => {
                        let numerator: i128 = self.fraction.as_();
                        let denominator: i128 = other.fraction.as_();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_()
                    }
                    128 => {
                        let numerator: I256 = self.fraction.into();
                        let denominator: I256 = other.fraction.into();
                        let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                        let shift = quotient
                            .leading_ones()
                            .max(quotient.leading_zeros())
                            .wrapping_sub(2);
                        quotient <<= shift;
                        (quotient >> F::FRACTION_BITS).as_i128().as_()
                    }
                    _ => GENERAL.prefix.sa(),
                };
                return Self {
                    fraction,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let fraction = match F::FRACTION_BITS {
                8 => {
                    let numerator: i16 = self.fraction.as_();
                    let denominator: i16 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                16 => {
                    let numerator: i32 = self.fraction.as_();
                    let denominator: i32 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                32 => {
                    let numerator: i64 = self.fraction.as_();
                    let denominator: i64 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                64 => {
                    let numerator: i128 = self.fraction.as_();
                    let denominator: i128 = other.fraction.as_();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_()
                }
                128 => {
                    let numerator: I256 = self.fraction.into();
                    let denominator: I256 = other.fraction.into();
                    let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                    let shift = quotient
                        .leading_ones()
                        .max(quotient.leading_zeros())
                        .wrapping_sub(1);
                    quotient <<= shift;
                    (quotient >> F::FRACTION_BITS).as_i128().as_()
                }
                _ => GENERAL.prefix.sa(),
            };
            return Self {
                fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let (fraction, expo_adjust) = match F::FRACTION_BITS {
            8 => {
                let numerator: i16 = self.fraction.as_();
                let denominator: i16 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            16 => {
                let numerator: i32 = self.fraction.as_();
                let denominator: i32 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            32 => {
                let numerator: i64 = self.fraction.as_();
                let denominator: i64 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            64 => {
                let numerator: i128 = self.fraction.as_();
                let denominator: i128 = other.fraction.as_();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            128 => {
                let numerator: I256 = self.fraction.into();
                let denominator: I256 = other.fraction.into();
                let mut quotient = (numerator << F::FRACTION_BITS).div_euclid(denominator);
                let shift = (quotient.leading_ones().max(quotient.leading_zeros()) as isize)
                    .wrapping_sub(1);
                quotient <<= shift;
                (
                    (quotient >> F::FRACTION_BITS).as_i128().as_(),
                    shift.wrapping_sub(F::FRACTION_BITS.wrapping_sub(1)),
                )
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        };

        match E::EXPONENT_BITS {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let other_exponent: i16 = other.exponent.as_();
                let upcast_exponent: i16 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i16);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let other_exponent: i32 = other.exponent.as_();
                let upcast_exponent: i32 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i32);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            32 => {
                let self_exponent: i64 = self.exponent.as_();
                let other_exponent: i64 = other.exponent.as_();
                let upcast_exponent: i64 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i64);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            64 => {
                let self_exponent: i128 = self.exponent.as_();
                let other_exponent: i128 = other.exponent.as_();
                let upcast_exponent: i128 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(expo_adjust as i128);

                if upcast_exponent > E::MAX_EXPONENT.as_() {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.as_() {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            128 => {
                let self_exponent: I256 = self.exponent.into();
                let other_exponent: I256 = other.exponent.into();
                let exp_adj: I256 = (expo_adjust as i128).into();
                let upcast_exponent: I256 = self_exponent
                    .wrapping_sub(other_exponent)
                    .wrapping_sub(exp_adj);
                let max_e: I256 = E::MAX_EXPONENT.into();
                let min_e: I256 = E::MIN_EXPONENT.into();
                if upcast_exponent > max_e {
                    return Self {
                        fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Self {
                        fraction: fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Self {
                        fraction,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
    pub fn reciprocal(&self) -> Self {
        Self::ONE / self
    }
}
