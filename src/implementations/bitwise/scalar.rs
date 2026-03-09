use crate::core::integer::FullInt;
use crate::core::integer::IntConvert;
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::AsPrimitive;
use num_traits::WrappingAdd;
use num_traits::WrappingMul;
use num_traits::WrappingNeg;
use num_traits::WrappingSub;
use std::ops::*;
#[allow(private_bounds)]
impl<
        F: Integer
            + FullInt
            + FractionConstants
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub
            + 'static,
        E: Integer
            + FullInt
            + ExponentConstants
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub
            + 'static,
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
    /// Performs bitwise AND operation on two Scalars after aligning their fractions
    ///
    /// # Two's Complement Behavior
    /// In two's complement, negative numbers have their most significant bit set to 1 and all bits above the highest 1 bit are also 1. This implementation respects these properties:
    ///
    /// - If a smaller number is non-negative (highest bit 0), its higher bits are 0, so AND with any larger number will produce 0 in those bit positions
    /// - If a smaller number is negative, all high bits are ones, so AND with a larger number preserves the larger number's bits in those positions
    ///
    /// # Specifics
    /// 0. Special case handling for ambiguous values (undefined, exploded, vanished and Zero)
    /// 1. Determination of which Scalar has larger exponent to align the values
    /// 2. Calculation of exponent difference and early return cases if the difference is too large (when bits don't overlap after alignment) and relevant sign extensions etc.
    /// 3. Alignment of the fraction with larger exponent by left-shifting by the exponent difference
    /// 4. Performing the bitwise AND on aligned fractions
    /// 5. Normalization of the result by counting leading zeros/ones and adjusting the exponent
    ///
    /// # Arguments
    /// * `other` - The Scalar to AND with
    ///
    /// # Returns
    /// * A Scalar containing the result of the aligned AND of both Scalars
    /// * Returns undefined AND if both operands escaped the same way
    /// * Returns Zero if either operand is Zero, or if non-negative operand is AND'ed with a
    ///   larger value (no bit overlap)
    /// * Returns the non-vanished operand if the other is vanished and negative
    ///
    /// # Representation Patterns
    /// ```txt
    /// Exploded Normal Vanished
    /// ↓↓↓↓↓↓↓↓ ↓↓↓↓↓↓ ↓↓↓↓↓↓↓↓
    /// □■?????? ?????? ???????? - Exploded positive
    /// ■□?????? ?????? ???????? - Exploded negative
    /// □□□□□□□□ □■???? ???????? - Normal positive
    /// ■■■■■■■■ ■□???? ???????? - Normal negative
    /// □□□□□□□□ □□□□□□ □■?????? - Vanished positive
    /// ■■■■■■■■ ■■■■■■ ■□?????? - Vanished negative
    /// ```
    pub(crate) fn aligned_and(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
                || self.is_infinite()
                || other.is_infinite()
            {
                return Self {
                    fraction: AND.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.vanished() {
                if self.is_negative() {
                    return *other;
                }
                return Self::ZERO;
            }
            if other.vanished() {
                if other.is_negative() {
                    return *self;
                }
                return Self::ZERO;
            }
            if self.is_normal() {
                if self.is_negative() {
                    return *other;
                }
                return Self::ZERO;
            }
            if other.is_normal() {
                if other.is_negative() {
                    return *self;
                }
                return Self::ZERO;
            }
            return Self {
                fraction: AND.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            if small.fraction.is_negative() {
                return *big;
            } else {
                return Self::ZERO;
            }
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                if small.fraction.is_negative() {
                    return *big;
                } else {
                    return Self::ZERO;
                }
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                if small.fraction.is_negative() {
                    return *big;
                } else {
                    return Self::ZERO;
                }
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_f: i16 = big.fraction.as_();
                big_f <<= shift;
                let small_f: i16 = small.fraction.as_();
                let result = big_f & small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f & small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f & small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f & small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f & small_f;
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
            _ => Self {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    /// Performs bitwise OR operation on two Scalars after aligning their fractions
    ///
    /// # Two's Complement Behavior
    /// In two's complement, negative numbers have their most significant bit set to 1 and all bits above the highest 1 bit are also 1. This implementation respects these properties:
    ///
    /// - If a smaller number is non-negative (highest bit 0), its higher bits are 0, so OR with a larger number preserves the larger number's bits in those positions
    /// - If a smaller number is negative, all high bits are ones, so OR with any larger number results in 1s in those positions (effectively returning the negative value)
    ///
    /// # Specifics
    /// 0. Special case handling for ambiguous values (undefined, exploded, vanished and Zero)
    /// 1. Determination of which Scalar has larger exponent to align the values
    /// 2. Calculation of exponent difference and early return cases if the difference is too large (when bits don't overlap after alignment) and relevant sign extensions etc.
    /// 3. Alignment of the fraction with larger exponent by left-shifting by the exponent difference
    /// 4. Performing the bitwise OR on aligned fractions
    /// 5. Normalization of the result by counting leading zeros/ones and adjusting the exponent
    ///
    /// # Arguments
    /// * `other` - The Scalar to OR with
    ///
    /// # Returns
    /// * A Scalar containing the result of the aligned OR of both Scalars
    /// * Returns undefined OR if both operands escaped the same way
    /// * Returns the non-zero operand if either operand is Zero
    /// * Returns the negative operand if either is negative with substantially larger exponent
    ///   (due to sign extension in two's complement)
    ///
    /// # Representation Patterns
    /// ```txt
    /// Exploded Normal Vanished
    /// ↓↓↓↓↓↓↓↓ ↓↓↓↓↓↓ ↓↓↓↓↓↓↓↓
    /// □■?????? ?????? ???????? - Exploded positive
    /// ■□?????? ?????? ???????? - Exploded negative
    /// □□□□□□□□ □■???? ???????? - Normal positive
    /// ■■■■■■■■ ■□???? ???????? - Normal negative
    /// □□□□□□□□ □□□□□□ □■?????? - Vanished positive
    /// ■■■■■■■■ ■■■■■■ ■□?????? - Vanished negative
    /// ```
    pub(crate) fn aligned_or(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }

            if self.is_infinite() || other.is_infinite() || self.exploded() && other.exploded() {
                return Self {
                    fraction: OR.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }

            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.exploded() {
                if other.is_negative() {
                    return *other;
                }
                return *self;
            }
            if other.exploded() {
                if self.is_negative() {
                    return *self;
                }
                return *other;
            }
            if self.is_normal() {
                if other.is_negative() {
                    return *other;
                }
                return *self;
            }
            if other.is_normal() {
                if self.is_negative() {
                    return *self;
                }
                return *other;
            }
            return Self {
                fraction: OR.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            if small.fraction.is_negative() {
                return *small;
            } else {
                return *big;
            }
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                if small.fraction.is_negative() {
                    return *small;
                } else {
                    return *big;
                }
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                if small.fraction.is_negative() {
                    return *small;
                } else {
                    return *big;
                }
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_f: i16 = big.fraction.as_();
                big_f <<= shift;
                let small_f: i16 = small.fraction.as_();
                let result = big_f | small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f | small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f | small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f | small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f | small_f;
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
            _ => Self {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    /// Performs bitwise XOR operation on two Scalars after aligning their fractions
    ///
    /// # Two's Complement Behavior
    /// In two's complement, negative numbers have their most significant bit set to 1 and all bits above the highest 1 bit are also 1. This implementation respects these properties:
    ///
    /// - If a smaller number is non-negative (highest bit 0), its higher bits are 0, so XOR with a larger number preserves the larger number's bits in those positions
    /// - If a smaller number is negative, all high bits are ones, so XOR with a larger number flips the larger number's bits in those positions
    ///
    /// # Specifics
    /// 0. Special case handling for ambiguous values (undefined, exploded, vanished and Zero)
    /// 1. Determination of which Scalar has larger exponent to align the values
    /// 2. Calculation of exponent difference and early return cases if the difference is too large (when bits don't overlap after alignment) and relevant sign extensions etc.
    /// 3. Alignment of the fraction with larger exponent by left-shifting by the exponent difference
    /// 4. Performing the bitwise XOR on aligned fractions
    /// 5. Normalization of the result by counting leading zeros/ones and adjusting the exponent
    ///
    /// # Arguments
    /// * `other` - The Scalar to XOR with
    ///
    /// # Returns
    /// * A Scalar containing the result of the aligned XOR of both Scalars
    /// * Returns undefined XOR if both operands escaped the same way
    /// * Returns the non-zero operand if either operand is Zero
    /// * Returns the complement of the larger operand if the smaller operand is negative with
    ///   substantially larger exponent difference (due to sign extension in two's complement)
    ///
    /// # Representation Patterns
    /// ```txt
    /// Exploded Normal Vanished
    /// ↓↓↓↓↓↓↓↓ ↓↓↓↓↓↓ ↓↓↓↓↓↓↓↓
    /// □■?????? ?????? ???????? - Exploded positive
    /// ■□?????? ?????? ???????? - Exploded negative
    /// □□□□□□□□ □■???? ???????? - Normal positive
    /// ■■■■■■■■ ■□???? ???????? - Normal negative
    /// □□□□□□□□ □□□□□□ □■?????? - Vanished positive
    /// ■■■■■■■■ ■■■■■■ ■□?????? - Vanished negative
    /// ```
    pub(crate) fn aligned_xor(&self, other: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite()
                || other.is_infinite()
                || (self.exploded() && other.exploded())
                || (self.vanished() && other.vanished())
            {
                return Self {
                    fraction: XOR.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() {
                return *other;
            }
            if other.is_zero() {
                return *self;
            }
            if self.vanished() {
                if self.is_negative() {
                    return !other;
                }
                return *other;
            }
            if other.vanished() {
                if other.is_negative() {
                    return !self;
                }
                return *self;
            }
            if self.is_normal() {
                if self.is_negative() {
                    return !other;
                }
                return *other;
            }
            if other.is_normal() {
                if other.is_negative() {
                    return !self;
                }
                return *self;
            }
            return Self {
                fraction: XOR.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        let (big, small) = if self.exponent > other.exponent {
            (self, other)
        } else {
            (other, self)
        };
        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        if exp_diff.is_negative() {
            if small.fraction.is_negative() {
                return !big;
            } else {
                return *big;
            }
        }

        if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if exp_diff >= F::FRACTION_BITS.as_() {
                if small.fraction.is_negative() {
                    return !big;
                } else {
                    return *big;
                }
            }
        } else {
            let exp_diff_isize: isize = exp_diff.as_();
            if exp_diff_isize >= F::FRACTION_BITS {
                if small.fraction.is_negative() {
                    return !big;
                } else {
                    return *big;
                }
            }
        }
        match F::FRACTION_BITS {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_f: i16 = big.fraction.as_();
                big_f <<= shift;
                let small_f: i16 = small.fraction.as_();
                let result = big_f ^ small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f ^ small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f ^ small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f ^ small_f;
                if result == 0 {
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
                        fraction: ((result << (leading.wrapping_sub(2))) >> F::FRACTION_BITS).as_(),
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
                let result = big_f ^ small_f;
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
            _ => Self {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            },
        }
    }
    pub(crate) fn not_scalar(&self) -> Scalar<F, E> {
        if self.is_undefined() {
            return *self;
        }
        return Self {
            fraction: !self.fraction,
            exponent: self.exponent,
        };
    }
    pub fn scalar_shl_integer(&self, shift: &E) -> Scalar<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_add(shift);
        if !shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                fraction: self.fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if shift.is_negative()
            && self.exponent.is_negative()
            && !(new_exp.wrapping_sub(&E::ONE)).is_negative()
        {
            return Self {
                fraction: self.fraction >> 1isize,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        return Self {
            fraction: self.fraction,
            exponent: new_exp,
        };
    }
    pub fn scalar_shr_integer(&self, shift: &E) -> Scalar<F, E> {
        if !self.is_normal() {
            return *self;
        }
        let new_exp = self.exponent.wrapping_sub(shift);
        if shift.is_negative() && !self.exponent.is_negative() && new_exp.is_negative() {
            return Self {
                fraction: self.fraction,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        if !shift.is_negative()
            && self.exponent.is_negative()
            && !(new_exp.wrapping_sub(&E::ONE)).is_negative()
        {
            return Self {
                fraction: self.fraction >> 1isize,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        return Self {
            fraction: self.fraction,
            exponent: new_exp,
        };
    }
}
