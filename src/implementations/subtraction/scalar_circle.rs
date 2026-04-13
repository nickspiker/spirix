use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    /// Subtracts a Circle from this Scalar
    ///
    /// # Description
    ///
    /// Performs subtraction between a Scalar and a Circle, with the Scalar value becoming the real component
    /// and the negative of the Circle's imaginary component becoming the imaginary component of the result.
    /// Returns a finite Circle unless the result exceeds representable range, in which case it may
    /// return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) values and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the Circle's real component from the Scalar and negates the Circle's imaginary component
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
    /// - `[↓]` - `[#]` ➔ `[-#, -#i]` Negative of the Circle with negated imaginary component
    /// - `[#]` - `[↓]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[0]` - `[#]` ➔ `[-#r, -#i]` Negative of the Circle
    /// - `[#]` - `[0]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF6E4, Scalar, ScalarF6E4};
    ///
    /// // Subtracting a Circle from a Scalar
    /// let a = Scalar::<i64,i16>::from(15);
    /// let b = Circle::<i64,i16>::from((-5, 4.5));
    /// let diff = a - b;
    /// assert!(diff.r() == 20);
    /// assert!(diff.i() == -4.5);
    ///
    /// // Subtracting with Zero
    /// assert!(a - CircleF6E4::ZERO == CircleF6E4::from((a, 0)));
    /// assert!(ScalarF6E4::ZERO - b == -b;
    ///
    /// // Subtracting with different exponents
    /// let large_scalar = ScalarF6E4::from(72);
    /// let small_circle = CircleF6E4::from((1.5, 0.0625));
    /// let result = large_scalar - small_circle;
    /// assert!(result.r() == 70.5);
    /// assert!(result.i() == -0.0625);
    ///
    /// // Subtraction with vanished values
    /// let tiny = CircleF6E4::MIN_POS_REAL / 4;
    /// assert!(tiny.vanished());
    /// assert!((a - tiny).r() == 7);
    /// assert!((a - tiny).i() == 0);
    ///
    /// // Subtraction with exploded values
    /// let huge = CircleF6E4::MAX_REAL_CIRCLE * 4;
    /// assert!(huge.exploded());
    /// assert!((a - huge).is_undefined());
    /// ```
    pub(crate) fn scalar_subtract_circle(&self, circle: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !circle.is_normal() {
            if self.is_undefined() {
                return Circle {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if circle.is_undefined() {
                return *circle;
            }
            if self.is_transfinite() && circle.is_transfinite() {
                let prefix: F = TRANSFINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() && circle.vanished() {
                let prefix: F = VANISHED_MINUS_VANISHED.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_transfinite() {
                let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if circle.is_transfinite() {
                let prefix: F = FINITE_MINUS_TRANSFINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.vanished() {
                return -circle;
            }
            if circle.vanished() {
                return Circle {
                    real: self.fraction,
                    imaginary: F::ZERO,
                    exponent: self.exponent,
                };
            }
            if self.is_zero() {
                return -circle;
            }
            return Circle {
                real: self.fraction,
                imaginary: F::ZERO,
                exponent: self.exponent,
            };
        }

        if self.exponent > circle.exponent {
            let exp_diff = self.exponent.wrapping_sub(&circle.exponent);
            if exp_diff.is_negative() {
                return Circle {
                    real: self.fraction,
                    imaginary: F::ZERO,
                    exponent: self.exponent,
                };
            }

            if E::EXPONENT_BITS >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return Circle {
                        real: self.fraction,
                        imaginary: F::ZERO,
                        exponent: self.exponent,
                    };
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return Circle {
                        real: self.fraction,
                        imaginary: F::ZERO,
                        exponent: self.exponent,
                    };
                }
            }
            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i16 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);
                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let small_i: i16 = circle.imaginary.as_();
                    let result_i = small_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = circle.exponent.wrapping_add(&o);

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
                    let mut big_r: i32 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i32 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let small_i: i32 = circle.imaginary.as_();
                    let result_i = small_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = circle.exponent.wrapping_add(&o);

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
                    let mut big_r: i64 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i64 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let small_i: i64 = circle.imaginary.as_();
                    let result_i = small_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = circle.exponent.wrapping_add(&o);

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
                    let mut big_r: i128 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i128 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let small_i: i128 = circle.imaginary.as_();
                    let result_i = small_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = circle.exponent.wrapping_add(&o);

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
                    let mut big_r: I256 = self.fraction.into();
                    big_r <<= shift;
                    let small_r: I256 = circle.real.into();
                    let result_r = big_r.wrapping_sub(small_r);

                    if result_r == 0.into() && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let small_i: I256 = circle.imaginary.into();
                    let result_i = small_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = circle.exponent.wrapping_add(&o);

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
                        real: ((result_r << leading.wrapping_sub(1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << leading.wrapping_sub(1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {}
            }
        } else {
            let exp_diff = circle.exponent.wrapping_sub(&self.exponent);
            if exp_diff.is_negative() {
                return -circle;
            }

            if E::EXPONENT_BITS >= core::mem::size_of::<isize>() as isize * 8 {
                if exp_diff >= F::FRACTION_BITS.as_() {
                    return -circle;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= F::FRACTION_BITS {
                    return -circle;
                }
            }
            match F::FRACTION_BITS {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i16 = self.fraction.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }
                    let mut big_i: i16 = circle.imaginary.as_();
                    big_i <<= shift;
                    let result_i = big_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if circle.exponent.is_negative() && !offset.is_negative() {
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
                    let mut big_r: i32 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i32 = self.fraction.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut big_i: i32 = circle.imaginary.as_();
                    big_i <<= shift;
                    let result_i = big_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if circle.exponent.is_negative() && !offset.is_negative() {
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
                    let mut big_r: i64 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i64 = self.fraction.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut big_i: i64 = circle.imaginary.as_();
                    big_i <<= shift;
                    let result_i = big_i.wrapping_neg();
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if circle.exponent.is_negative() && !offset.is_negative() {
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
                    let mut big_r: i128 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i128 = self.fraction.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut big_i: i128 = circle.imaginary.as_();
                    big_i <<= shift;
                    let result_i = big_i.wrapping_neg();
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if circle.exponent.is_negative() && !offset.is_negative() {
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
                    let mut big_r: I256 = circle.real.into();
                    big_r <<= shift;
                    let small_r: I256 = self.fraction.into();
                    let result_r = small_r.wrapping_sub(big_r);

                    if result_r == 0.into() && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut big_i: I256 = circle.imaginary.into();
                    big_i <<= shift;
                    let result_i = big_i.wrapping_neg();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let o: E = F::FRACTION_BITS.wrapping_sub(leading).as_();
                    let offset = self.exponent.wrapping_add(&o);

                    if circle.exponent.is_negative() && !offset.is_negative() {
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
                        real: ((result_r << leading.wrapping_sub(1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << leading.wrapping_sub(1)) >> F::FRACTION_BITS)
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::ONE),
                    };
                }
                _ => {}
            }
        }
        Circle::<F, E>::ZERO
    }
}
