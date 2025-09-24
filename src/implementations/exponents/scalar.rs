// src/implementation/exponents/scalar.rs
use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use std::ops::*;

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
    pub fn square(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() || self.is_n0() {
                // Undefined, Zeros and Infinities stay the same
                return *self;
            }

            let shift_adjust: isize = if self.exploded() { 1 } else { 2 };
            let fraction = match F::FRACTION_BITS {
                8 => {
                    let fraction: i16 = self.fraction.as_();
                    let product_wide = fraction.wrapping_mul(fraction);

                    let normalize_shift = product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize;

                    let shift_amount = normalize_shift.wrapping_sub(shift_adjust);
                    let normalized_wide = product_wide << shift_amount;
                    (normalized_wide >> 8).as_()
                }
                16 => {
                    let fraction: i32 = self.fraction.as_();
                    let product_wide = fraction.wrapping_mul(fraction);

                    let normalize_shift = product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize;

                    let shift_amount = normalize_shift.wrapping_sub(shift_adjust);
                    let normalized_wide = product_wide << shift_amount;
                    (normalized_wide >> 16).as_()
                }
                32 => {
                    let fraction: i64 = self.fraction.as_();
                    let product_wide = fraction.wrapping_mul(fraction);

                    let normalize_shift = product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize;

                    let shift_amount = normalize_shift.wrapping_sub(shift_adjust);
                    let normalized_wide = product_wide << shift_amount;
                    (normalized_wide >> 32).as_()
                }
                64 => {
                    let fraction: i128 = self.fraction.as_();
                    let product_wide = fraction.wrapping_mul(fraction);

                    let normalize_shift = product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize;

                    let shift_amount = normalize_shift.wrapping_sub(shift_adjust);
                    let normalized_wide = product_wide << shift_amount;
                    (normalized_wide >> 64).as_()
                }
                128 => {
                    let fraction: i128 = self.fraction.as_();
                    let multiplier: I256 = fraction.into();
                    let product_wide: I256 = multiplier.wrapping_mul(multiplier);

                    let normalize_shift = product_wide
                        .leading_ones()
                        .max(product_wide.leading_zeros())
                        as isize;

                    let shift_amount = normalize_shift.wrapping_sub(shift_adjust);
                    let normalized_wide = product_wide << shift_amount;
                    (normalized_wide >> 128isize).as_i128().as_()
                }
                _ => GENERAL.prefix.sa(),
            };

            return Self {
                fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let product_fraction;
        let expo_adjust: isize;
        match F::FRACTION_BITS {
            8 => {
                let multiplier: i16 = self.fraction.as_();
                let product_wide = multiplier.wrapping_mul(multiplier);
                if product_wide == 0 {
                    return Self::ZERO;
                }
                expo_adjust = product_wide
                    .leading_ones()
                    .max(product_wide.leading_zeros())
                    .wrapping_sub(2) as isize;
                let shift_amount = expo_adjust.wrapping_add(1);
                let normalized_wide = product_wide << shift_amount;
                product_fraction = (normalized_wide >> 8).as_();
            }
            16 => {
                let multiplier: i32 = self.fraction.as_();
                let product_wide = multiplier.wrapping_mul(multiplier);
                if product_wide == 0 {
                    return Self::ZERO;
                }
                expo_adjust = product_wide
                    .leading_ones()
                    .max(product_wide.leading_zeros())
                    .wrapping_sub(2) as isize;
                let shift_amount = expo_adjust.wrapping_add(1);
                let normalized_wide = product_wide << shift_amount;
                product_fraction = (normalized_wide >> 16).as_();
            }
            32 => {
                let multiplier: i64 = self.fraction.as_();
                let product_wide = multiplier.wrapping_mul(multiplier);
                if product_wide == 0 {
                    return Self::ZERO;
                }
                expo_adjust = product_wide
                    .leading_ones()
                    .max(product_wide.leading_zeros())
                    .wrapping_sub(2) as isize;
                let shift_amount = expo_adjust.wrapping_add(1);
                let normalized_wide = product_wide << shift_amount;
                product_fraction = (normalized_wide >> 32).as_();
            }
            64 => {
                let multiplier: i128 = self.fraction.as_();
                let product_wide = multiplier.wrapping_mul(multiplier);
                if product_wide == 0 {
                    return Self::ZERO;
                }
                expo_adjust = product_wide
                    .leading_ones()
                    .max(product_wide.leading_zeros())
                    .wrapping_sub(2) as isize;
                let shift_amount = expo_adjust.wrapping_add(1);
                let normalized_wide = product_wide << shift_amount;
                product_fraction = (normalized_wide >> 64).as_();
            }
            128 => {
                let multiplier: i128 = self.fraction.as_();
                let multiplier: I256 = multiplier.into();
                let multiplier: I256 = multiplier.into();
                let product_wide: I256 = multiplier.wrapping_mul(multiplier);
                if product_wide == 0.into() {
                    return Self::ZERO;
                }
                expo_adjust = product_wide
                    .leading_ones()
                    .max(product_wide.leading_zeros())
                    .wrapping_sub(2) as isize;
                let shift_amount = expo_adjust.wrapping_add(1);
                let normalized_wide = product_wide << shift_amount;
                product_fraction = (normalized_wide >> 128isize).as_i128().as_();
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        match E::EXPONENT_BITS {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let upcast_exponent: i16 = self_exponent
                    .wrapping_mul(2)
                    .wrapping_sub(expo_adjust as i16);
                let max_e: i16 = E::MAX_EXPONENT.as_();
                let min_e: i16 = E::MIN_EXPONENT.as_();
                if upcast_exponent > max_e {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Scalar {
                        fraction: product_fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let upcast_exponent: i32 = self_exponent
                    .wrapping_mul(2)
                    .wrapping_sub(expo_adjust as i32);
                let max_e: i32 = E::MAX_EXPONENT.as_();
                let min_e: i32 = E::MIN_EXPONENT.as_();
                if upcast_exponent > max_e {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Scalar {
                        fraction: product_fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            32 => {
                let self_exponent: i64 = self.exponent.as_();
                let upcast_exponent: i64 = self_exponent
                    .wrapping_mul(2)
                    .wrapping_sub(expo_adjust as i64);
                let max_e: i64 = E::MAX_EXPONENT.as_();
                let min_e: i64 = E::MIN_EXPONENT.as_();
                if upcast_exponent > max_e {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Scalar {
                        fraction: product_fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            64 => {
                let self_exponent: i128 = self.exponent.as_();
                let upcast_exponent: i128 = self_exponent
                    .wrapping_mul(2)
                    .wrapping_sub(expo_adjust as i128);
                let max_e: i128 = E::MAX_EXPONENT.as_();
                let min_e: i128 = E::MIN_EXPONENT.as_();
                if upcast_exponent > max_e {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < min_e {
                    return Scalar {
                        fraction: product_fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            128 => {
                let self_exponent: I256 = self.exponent.into();
                let e: I256 = (expo_adjust as i128).into();
                let two: I256 = 2.into();
                let upcast_exponent: I256 = self_exponent * two - e;

                if upcast_exponent > E::MAX_EXPONENT.into() {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else if upcast_exponent < E::MIN_EXPONENT.into() {
                    return Scalar {
                        fraction: product_fraction >> 1isize,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                } else {
                    return Scalar {
                        fraction: product_fraction,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Scalar {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }
    }
    // pub fn sqrt(&self) -> Self {
    //     if self.is_undefined() {
    //         return *self;
    //     }
    //     if self.fraction.is_negative() {
    //         let mut prefix: F = SQRT_NEGATIVE.prefix.as_();
    //         prefix = prefix << (F::FRACTION_BITS - 8);
    //         return Self {
    //             fraction: prefix,
    //             exponent: E::ESCAPED_EXPONENT,
    //         };
    //     }
    //     if self.escaped() {
    //         if self.vanished() {
    //             let mut prefix: F = SQRT_SMALL.prefix.as_();
    //             prefix = prefix << (F::FRACTION_BITS - 8);
    //             return Self {
    //                 fraction: prefix,
    //                 exponent: E::ESCAPED_EXPONENT,
    //             };
    //         } else {
    //             let mut prefix: F = SQRT_BIG.prefix.as_();
    //             prefix = prefix << (F::FRACTION_BITS - 8);
    //             return Self {
    //                 fraction: prefix,
    //                 exponent: E::ESCAPED_EXPONENT,
    //             };
    //         }
    //     }

    //     if self.is_zero() {
    //         return Self::ZERO;
    //     }

    //     let exponent = (self.exponent) / 2.as_();
    //     let even = self.exponent & 1.as_();
    //     let mut exponent = exponent + even;
    //     let even: usize = even.as_();
    //     if self.exponent.is_negative() && even != 0 {
    //         exponent = exponent - 1.as_();
    //     }

    //     let fraction = match F::FRACTION_BITS {
    //         8 => {
    //             let f: u16 = self.fraction.as_();
    //             let radicand = f << (9 - even);
    //             let mut bit = 1 << 7;
    //             let mut result = 0u16;
    //             while bit != 0 {
    //                 let guess = result | bit;
    //                 bit >>= 1;
    //                 if guess * guess <= radicand {
    //                     result = guess;
    //                     if guess * guess == radicand {
    //                         break;
    //                     }
    //                 }
    //             }

    //             let leading_zeros = result.leading_zeros() as isize - 1;
    //             let result = result << leading_zeros;
    //             exponent = exponent - (leading_zeros - 7).as_();

    //             (result >> 8).as_()
    //         }
    //         16 => {
    //             let f: u32 = self.fraction.as_();
    //             let radicand = f << (17 - even);
    //             let mut bit = 1 << 15;
    //             let mut result = 0u32;
    //             while bit != 0 {
    //                 let guess = result | bit;
    //                 bit >>= 1;
    //                 if guess * guess <= radicand {
    //                     result = guess;
    //                     if guess * guess == radicand {
    //                         break;
    //                     }
    //                 }
    //             }

    //             let leading_zeros = result.leading_zeros() as isize - 1;
    //             let result = result << leading_zeros;
    //             exponent = exponent - (leading_zeros - 15).as_();

    //             (result >> 16).as_()
    //         }
    //         32 => {
    //             let f: u64 = self.fraction.as_();
    //             let radicand = f << (33 - even);
    //             let mut bit = 1 << 31;
    //             let mut result = 0u64;
    //             while bit != 0 {
    //                 let guess = result | bit;
    //                 bit >>= 1;
    //                 if guess * guess <= radicand {
    //                     result = guess;
    //                     if guess * guess == radicand {
    //                         break;
    //                     }
    //                 }
    //             }

    //             let leading_zeros = result.leading_zeros() as isize - 1;
    //             let result = result << leading_zeros;
    //             exponent = exponent - (leading_zeros - 31).as_();

    //             (result >> 32).as_()
    //         }
    //         64 => {
    //             let f: u128 = self.fraction.as_();
    //             let radicand = f << (65 - even);
    //             let mut bit = 1 << 63;
    //             let mut result = 0u128;
    //             while bit != 0 {
    //                 let guess = result | bit;
    //                 bit >>= 1;
    //                 if guess * guess <= radicand {
    //                     result = guess;
    //                     if guess * guess == radicand {
    //                         break;
    //                     }
    //                 }
    //             }

    //             let leading_zeros = result.leading_zeros() as isize - 1;
    //             let result = result << leading_zeros;
    //             exponent = exponent - (leading_zeros - 63).as_();

    //             (result >> 64).as_()
    //         }
    //         128 => {
    //             let fraction_u128: u128 = self.fraction.as_();
    //             let fraction_u256: i256::U256 = fraction_u128.into();
    //             let radicand = fraction_u256 << (129 - even);
    //             let mut bit: i256::U256 = (1u128 << 127).into();
    //             let mut result: i256::U256 = 0u128.into();
    //             while bit != 0u128.into() {
    //                 let guess = result | bit;
    //                 bit >>= 1;
    //                 if guess * guess <= radicand {
    //                     result = guess;
    //                     if guess * guess == radicand {
    //                         break;
    //                     }
    //                 }
    //             }

    //             let leading_zeros = result.leading_zeros() as isize - 1;
    //             let result = result << leading_zeros;
    //             exponent = exponent - (leading_zeros - 127).as_();

    //             (result >> 128isize).as_i128().as_()
    //         }
    //         _ => {
    //             let prefix: F = GENERAL.prefix.as_();
    //             return Self {
    //                 fraction: prefix << (F::FRACTION_BITS - 8),
    //                 exponent: E::ESCAPED_EXPONENT,
    //             };
    //         }
    //     };

    //     Self { fraction, exponent }
    // }
    pub fn sqrt(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() || self.is_n0() {
                return *self;
            }

            if self.vanished() {
                return Self {
                    fraction: SQRT_VANISHED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            } else {
                return Self {
                    fraction: SQRT_EXPLODED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        }

        if self.fraction.is_negative() {
            return Self {
                fraction: SQRT_NEGATIVE.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        let exponent = (self.exponent) / E::TWO;
        let even = self.exponent & E::ONE;
        let mut exponent = exponent + even;
        let even: usize = even.as_();
        if self.exponent.is_negative() && even != 0 {
            exponent = exponent - E::ONE;
        }

        let fraction = match F::FRACTION_BITS {
            8 => {
                let value: u16 = self.fraction.as_();
                let x = value << 9usize.wrapping_sub(even);
                let mut y = (1 << 8).wrapping_sub(&1);
                while y <= x {
                    let new_y = (y.wrapping_add(x / y)) >> 1;
                    if new_y >= y {
                        break;
                    }
                    y = new_y;
                }
                let shift = y.leading_zeros().wrapping_sub(1);
                let y = y << shift;
                let s: E = (shift as isize).wrapping_sub(7).as_();
                exponent = exponent.wrapping_add(&s);
                (y >> 8).as_()
            }
            16 => {
                let value: u32 = self.fraction.as_();
                let x = value << 17usize.wrapping_sub(even);
                let mut y = (1 << 16).wrapping_sub(&1);
                while y <= x {
                    let new_y = (y.wrapping_add(x / y)) >> 1;
                    if new_y >= y {
                        break;
                    }
                    y = new_y;
                }
                let shift = y.leading_zeros().wrapping_sub(1);
                let y = y << shift;
                let s: E = (shift as isize).wrapping_sub(15).as_();
                exponent = exponent.wrapping_add(&s);
                (y >> 16).as_()
            }
            32 => {
                let value: u64 = self.fraction.as_();
                let x = value << 33usize.wrapping_sub(even);
                let mut y = (1 << 32).wrapping_sub(&1);
                while y <= x {
                    let new_y = (y.wrapping_add(x / y)) >> 1;
                    if new_y >= y {
                        break;
                    }
                    y = new_y;
                }
                let shift = y.leading_zeros().wrapping_sub(1);
                let y = y << shift;
                let s: E = (shift as isize).wrapping_sub(31).as_();
                exponent = exponent.wrapping_add(&s);
                (y >> 32).as_()
            }
            64 => {
                let value: u128 = self.fraction.as_();
                let x = value << 65usize.wrapping_sub(even);
                let mut y = (1 << 64).wrapping_sub(&1);
                while y <= x {
                    let new_y = (y.wrapping_add(x / y)) >> 1;
                    if new_y >= y {
                        break;
                    }
                    y = new_y;
                }
                let shift = y.leading_zeros().wrapping_sub(1);
                let y = y << shift;
                let s: E = (shift as isize).wrapping_sub(63).as_();
                exponent = exponent.wrapping_add(&s);
                (y >> 64).as_()
            }
            128 => {
                let fraction_u128: u128 = self.fraction.as_();
                let value: i256::U256 = fraction_u128.into();
                let x = value << 129usize.wrapping_sub(even);
                let mut y = (i256::U256::from(1u8) << i256::U256::from(128u8))
                    .wrapping_sub(i256::U256::from(1u8));
                while y <= x {
                    let new_y = (y.wrapping_add(x / y)) >> 1;
                    if new_y >= y {
                        break;
                    }
                    y = new_y;
                }
                let shift = y.leading_zeros().wrapping_sub(1);
                let y = y << shift;
                let s: E = (shift as isize).wrapping_sub(127).as_();
                exponent = exponent.wrapping_add(&s);
                (y >> i256::U256::from(128u8)).as_i128().as_()
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        };
        Self { fraction, exponent }
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Self {
                fraction: NEGLIGIBLE_LOG.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        if self.fraction.is_negative() {
            return Self {
                fraction: NEGATIVE_LOG.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Calculate the integer part
        let characteristic = self.exponent.wrapping_sub(&E::ONE);

        // Create a value in [1,2) to calculate the fractional part
        let mut x = *self;
        x.exponent = E::ONE;

        // Calculate fractional part bit by bit
        let mut fraction = F::ZERO;
        let mut rotor: F = F::ONE;
        rotor = rotor << F::FRACTION_BITS.wrapping_sub(2);

        while rotor != F::ZERO {
            x = x.square();
            if x.exponent > E::ONE {
                fraction = fraction | rotor;
                x.exponent = x.exponent.wrapping_sub(&E::ONE);
            }
            rotor = rotor >> 1isize;
        }

        // Combine integer and fractional parts
        let characteristic_scalar = match E::EXPONENT_BITS {
            8 => {
                let exponent: i8 = characteristic.as_();
                Self::from(exponent)
            }
            16 => {
                let exponent: i16 = characteristic.as_();
                Self::from(exponent)
            }
            32 => {
                let exponent: i32 = characteristic.as_();
                Self::from(exponent)
            }
            64 => {
                let exponent: i64 = characteristic.as_();
                Self::from(exponent)
            }
            128 => {
                let exponent: i128 = characteristic.as_();
                Self::from(exponent)
            }
            _ => {
                return Self {
                    fraction: GENERAL.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
        };

        // Add the fractional part to the characteristic
        let mut fractional_part = Self {
            fraction: fraction,
            exponent: E::ZERO,
        };
        fractional_part.normalize();

        let result = characteristic_scalar + fractional_part;
        result
    }

    /// Computes e raised to the power of this Scalar value (e^x)
    ///
    /// # Description
    ///
    /// Calculates the exponential function e^x thru a combination of range reduction and Taylor series expansion.
    /// This implementation uses argument reduction by separating integer and fractional parts to improve convergence speed and numerical stability.
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
    /// // Basic exponential function
    /// let one = ScalarF6E4::ONE;
    /// assert!(one.exp() == ScalarF6E4::E); // e^1 = e
    ///
    /// // Identity property
    /// let zero = ScalarF6E4::ZERO;
    /// assert!(zero.exp() == 1); // e^0 = 1
    ///
    /// // Negative values
    /// let neg_one = ScalarF6E4::NEG_ONE;
    /// assert!(neg_one.exp() * ScalarF6E4::E == ScalarF6E4::ONE); // e^-1 * e = 1
    ///
    /// // Ambiguous values
    /// let large_pos = ScalarF6E4::MAX * 2;
    /// assert!(large_pos.exploded() && large_pos.exp().exploded()); // e^large = infinity
    ///
    /// let large_neg = ScalarF6E4::MIN * 2;
    /// assert!(large_neg.exploded() && large_neg.fraction.is_negative());
    /// assert!(large_neg.exp().is_zero()); // e^-large = 0
    ///
    /// // Tiny values
    /// let tiny_pos = ScalarF6E4::MIN_POS / 10;
    /// assert!(tiny_pos.vanished() && tiny_pos.exp() == 1); // e^tiny ≈ 1
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
                if self.fraction.is_negative() {
                    return Self::ZERO;
                } else {
                    return Self {
                        fraction: POWER_TRANSFINITE.prefix.sa(),
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
            // Vanished values
            if self.fraction.is_negative() {
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

        for _bit in 0..E::EXPONENT_BITS {
            if (remaining_exponent & Self::ONE) == 1 {
                integer_result *= current_power;
            }
            current_power = current_power.square();
            if !current_power.is_normal() {
                break; // Exit if power becomes abnormal
            }
            remaining_exponent = remaining_exponent >> 1;
            if remaining_exponent.vanished() || remaining_exponent.is_zero() {
                break; // Exit when done processing bits
            }
        }

        if integer_part.fraction.is_negative() {
            integer_result = integer_result.reciprocal();
        }

        previous_sum * integer_result
    }

    /// Computes 2 raised to the power of this Scalar value (2^x)
    ///
    /// # Description
    ///
    /// Calculates the binary exponential function 2^x by leveraging the natural exponential function.
    /// Uses the mathematical identity: 2^x = e^(x * ln(2))
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
    /// // Basic binary exponential function
    /// let one = ScalarF6E4::ONE;
    /// assert!(one.powb() == 2); // 2^1 = 2
    ///
    /// // Identity property
    /// let zero = ScalarF6E4::ZERO;
    /// assert!(zero.powb() == 1); // 2^0 = 1
    ///
    /// // Integer powers
    /// let three = ScalarF6E4::from(3);
    /// assert!(three.powb() == 8); // 2^3 = 8
    ///
    /// // Fractional powers
    /// let half = ScalarF6E4::ONE / 2;
    /// assert!(half.powb() == half.sqrt()); // 2^0.5 = √2
    ///
    /// // Negative powers
    /// let neg_two = ScalarF6E4::from(-2);
    /// assert!(neg_two.powb() == ScalarF6E4::ONE / 4); // 2^-2 = 1/4
    /// ```
    pub fn powb(&self) -> Self {
        (self * Self::LN_TWO).exp()
    }
}
