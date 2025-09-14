use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants};
use i256::I256;
use num_traits::AsPrimitive;
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
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
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
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && scalar.vanished() {
                return Self {
                    fraction: VANISHED_MINUS_VANISHED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_transfinite() {
                return Self {
                    fraction: TRANSFINITE_MINUS_FINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if scalar.is_transfinite() {
                return Self {
                    fraction: FINITE_MINUS_TRANSFINITE.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                return -scalar;
            }
            if scalar.vanished() {
                return *self;
            }
            if self.is_zero() {
                return -scalar;
            }
            return *self;
        }

        if self.exponent > scalar.exponent {
            let exp_diff = self.exponent - scalar.exponent;
            if exp_diff.is_negative() {
                return *self;
            }

            if E::EXPONENT_BITS >= std::mem::size_of::<isize>() as isize * 8 {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return *self;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return *self;
                }
            }

            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i16 = self.fraction.as_();
                    big <<= shift;
                    let small: i16 = scalar.fraction.as_();
                    let result = big - small;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = scalar.exponent + (F::FRACTION_BITS - leading).as_();
                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i32 = self.fraction.as_();
                    big <<= shift;
                    let small: i32 = scalar.fraction.as_();
                    let result = big - small;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = scalar.exponent + (F::FRACTION_BITS - leading).as_();
                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i64 = self.fraction.as_();
                    big <<= shift;
                    let small: i64 = scalar.fraction.as_();
                    let result = big - small;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = scalar.exponent + (F::FRACTION_BITS - leading).as_();
                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i128 = self.fraction.as_();
                    big <<= shift;
                    let small: i128 = scalar.fraction.as_();
                    let result = big - small;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = scalar.exponent + (F::FRACTION_BITS - leading).as_();
                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: I256 = self.fraction.into();
                    big <<= shift;
                    let small: I256 = scalar.fraction.into();
                    let result = big - small;
                    if result == 0.into() {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = scalar.exponent + (F::FRACTION_BITS - leading).as_();
                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                _ => {
                    let prefix: F = GENERAL.prefix.sa();
                    return Self {
                        fraction: prefix,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        } else {
            let exp_diff = scalar.exponent - self.exponent;
            if exp_diff.is_negative() {
                return -scalar;
            }

            if E::EXPONENT_BITS >= std::mem::size_of::<isize>() as isize * 8 {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return -scalar;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return -scalar;
                }
            }

            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i16 = scalar.fraction.as_();
                    big <<= shift;
                    let small: i16 = self.fraction.as_();
                    let result = small - big;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = self.exponent + (F::FRACTION_BITS - leading).as_();
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i32 = scalar.fraction.as_();
                    big <<= shift;
                    let small: i32 = self.fraction.as_();
                    let result = small - big;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = self.exponent + (F::FRACTION_BITS - leading).as_();
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i64 = scalar.fraction.as_();
                    big <<= shift;
                    let small: i64 = self.fraction.as_();
                    let result = small - big;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = self.exponent + (F::FRACTION_BITS - leading).as_();
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: i128 = scalar.fraction.as_();
                    big <<= shift;
                    let small: i128 = self.fraction.as_();
                    let result = small - big;
                    if result == 0 {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = self.exponent + (F::FRACTION_BITS - leading).as_();
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS).as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS).as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big: I256 = scalar.fraction.into();
                    big <<= shift;
                    let small: I256 = self.fraction.into();
                    let result = small - big;
                    if result == 0.into() {
                        return Self::ZERO;
                    }
                    let leading = result.leading_ones().max(result.leading_zeros()) as isize;
                    let offset = self.exponent + (F::FRACTION_BITS - leading).as_();
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            fraction: ((result << (leading - 2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }
                    return Self {
                        fraction: ((result << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset + 1.as_(),
                    };
                }
                _ => {
                    let prefix: F = GENERAL.prefix.sa();
                    return Self {
                        fraction: prefix,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        }
    }
}

use num_traits::PrimInt;
fn _printey<T: std::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = std::mem::size_of::<T>() * 8;
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits - 1 && b % 8 == 7 {
            result.push(' ');
        }
        if b == bits / 2 - 1 {
            result.push(' '); // Extra space at center
        }
    }
    result
}
