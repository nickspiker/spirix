use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    > Circle<F, E>
where
    Circle<F, E>: CircleConstants,
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
    /// Subtracts a Scalar from this Circle
    ///
    /// # Description
    ///
    /// Performs subtraction between a Circle and a Scalar, subtracting the Scalar value from the real component
    /// of the Circle while leaving the imaginary component unchanged (except for normalization).
    /// Returns a finite Circle unless the result exceeds representable range, in which case it may
    /// return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) Circles and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the Scalar from the real component of the Circle
    /// 4. Normalizes result and adjusts exponent, considering both components
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` - `[↑]` ➔ `[℘ ↑-↑]` Undefined exploded minus exploded state
    /// - `[↑]` - `[#]` ➔ `[℘ ↑-]` Undefined exploded minus finite state
    /// - `[#]` - `[↑]` ➔ `[℘ -↑]` Undefined finite minus exploded state
    /// - `[↓]` - `[↓]` ➔ `[℘ ↓-↓]` Undefined vanished minus vanished state
    /// - `[↓]` - `[#]` ➔ `[-#, 0i]` Negative of the Scalar as real part, zero imaginary
    /// - `[#]` - `[↓]` ➔ `[#]` The Circle unchanged
    /// - `[0]` - `[#]` ➔ `[-#, 0i]` Negative of the Scalar as real part, zero imaginary
    /// - `[#]` - `[0]` ➔ `[#]` The Circle unchanged
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3, Scalar, ScalarF5E3};
    ///
    /// // Subtracting a Scalar from a Circle
    /// let a = CircleF5E3::from((7.5, 4));
    /// let b = ScalarF5E3::from(3.5);
    /// let diff = a - b;
    /// assert!(diff.r() == 4);
    /// assert!(diff.i() == 4);
    ///
    /// // Subtracting with Zero
    /// assert!(a - ScalarF5E3::ZERO == a);
    /// assert!(CircleF5E3::ZERO - b == CircleF5E3::from((-3.5, 0)));
    ///
    /// // Subtracting with different exponents
    /// let large_circle = CircleF5E3::from((128, 64));
    /// let small_scalar = ScalarF5E3::from(0.25);
    /// let result = large_circle - small_scalar;
    /// assert!(result.r() == 127.75);
    /// assert!(result.i() == 64);
    ///
    /// // Subtraction with vanished values
    /// let tiny = ScalarF5E3::MIN_POS / 4;
    /// assert!(tiny.vanished());
    /// assert!((a - tiny) == a);
    ///
    /// // Subtraction with exploded values
    /// let huge = ScalarF5E3::MAX * 4;
    /// assert!(huge.exploded());
    /// assert!((a - huge).is_undefined());
    /// ```
    pub(crate) fn circle_subtract_scalar(&self, scalar: &Scalar<F, E>) -> Self {
        if !self.is_normal() || !scalar.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if scalar.is_undefined() {
                return Circle {
                    real: scalar.fraction,
                    imaginary: scalar.fraction,
                    exponent: scalar.exponent,
                };
            }
            if self.exploded() && scalar.exploded() {
                let prefix: F = TRANSFINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && scalar.vanished() {
                let prefix: F = VANISHED_MINUS_VANISHED.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.exploded() {
                let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if scalar.exploded() {
                let prefix: F = FINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                let neg_scalar = -scalar;
                return Circle {
                    real: neg_scalar.fraction,
                    imaginary: F::ZERO,
                    exponent: neg_scalar.exponent,
                };
            }
            if scalar.vanished() {
                return *self;
            }
            if self.is_zero() {
                let neg_scalar = -scalar;
                return Circle {
                    real: neg_scalar.fraction,
                    imaginary: F::ZERO,
                    exponent: neg_scalar.exponent,
                };
            }
            return *self;
        }

        if self.exponent > scalar.exponent {
            let exp_diff = self.exponent - scalar.exponent;
            if exp_diff.is_negative() {
                return *self;
            }

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
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
                    let mut big_r: i16 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i16 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i16 = self.imaginary.as_();
                    result_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = scalar.exponent.wrapping_add(&o);

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i32 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i32 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i32 = self.imaginary.as_();
                    result_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = scalar.exponent.wrapping_add(&o);

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i64 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i64 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i64 = self.imaginary.as_();
                    result_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = scalar.exponent.wrapping_add(&o);

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i128 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i128 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i128 = self.imaginary.as_();
                    result_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = scalar.exponent.wrapping_add(&o);

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: I256 = self.real.into();
                    big_r <<= shift;
                    let small_r: I256 = scalar.fraction.into();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0.into() && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: I256 = self.imaginary.into();
                    result_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = scalar.exponent.wrapping_add(&o);

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << leading.wrapping_sub(2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            imaginary: ((result_i << leading.wrapping_sub(2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {
                    let prefix: F = GENERAL.prefix.sa();
                    return Circle {
                        real: prefix,
                        imaginary: prefix,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        } else {
            let exp_diff = scalar.exponent.wrapping_sub(&self.exponent);
            if exp_diff.is_negative() {
                let negated_scalar_fraction = scalar.fraction.wrapping_neg();
                return Circle {
                    real: negated_scalar_fraction,
                    imaginary: F::ZERO,
                    exponent: scalar.exponent,
                };
            }

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    let negated_scalar_fraction = scalar.fraction.wrapping_neg();
                    return Circle {
                        real: negated_scalar_fraction,
                        imaginary: F::ZERO,
                        exponent: scalar.exponent,
                    };
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    let negated_scalar_fraction = scalar.fraction.wrapping_neg();
                    return Circle {
                        real: negated_scalar_fraction,
                        imaginary: F::ZERO,
                        exponent: scalar.exponent,
                    };
                }
            }
            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i16 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i16 = self.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i32 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i32 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i32 = self.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i64 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i64 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i64 = self.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i128 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i128 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i128 = self.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2))) >> F::FRACTION_BITS)
                                .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> F::FRACTION_BITS)
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> F::FRACTION_BITS).as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1))) >> F::FRACTION_BITS)
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: I256 = scalar.fraction.into();
                    big_r <<= shift;
                    let small_r: I256 = self.real.into();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0.into() && self.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: I256 = self.imaginary.into();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << leading.wrapping_sub(2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            imaginary: ((result_i << leading.wrapping_sub(2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            exponent: E::AMBIGUOUS_EXPONENT,
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << (leading - 1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {
                    let prefix: F = GENERAL.prefix.sa();
                    return Circle {
                        real: prefix,
                        imaginary: prefix,
                        exponent: E::AMBIGUOUS_EXPONENT,
                    };
                }
            }
        }
    }
}
