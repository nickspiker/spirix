use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
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
    /// Subtracts another Circle from this Circle
    ///
    /// # Description
    ///
    /// Performs subtraction between two Circles, handling special cases according to mathematical principles. The operation subtracts both the real and imaginary components separately while maintaining fractional alignment and adjusting exponent accordingly.
    /// Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Subtraction process:
    /// 0. Checks for any escaped (vanished, exploded or undefined) Circles and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Subtracts the aligned values for real and imaginary components
    /// 4. Normalizes result and adjusts exponent
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` - `[↑]` ➔ `[℘ ↑-↑]` Undefined exploded minus exploded state
    /// - `[↑]` - `[#]` ➔ `[℘ ↑-]` Undefined exploded minus finite state
    /// - `[#]` - `[↑]` ➔ `[℘ -↑]` Undefined finite minus exploded state
    /// - `[↓]` - `[↓]` ➔ `[℘ ↓-↓]` Undefined vanished minus vanished state
    /// - `[↓]` - `[#]` ➔ `[-#]` Negative of the finite Circle
    /// - `[#]` - `[↓]` ➔ `[#]` The finite Circle
    /// - `[0]` - `[#]` ➔ `[-#]` Negative of the Circle
    /// - `[#]` - `[0]` ➔ `[#]` The Circle
    /// - `[0]` - `[0]` ➔ `[0]` Zero
    /// - `[#]` - `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Subtracting finite Circles
    /// let a = Circle::<i32, i8>::from((3.5, 4));
    /// let b = CircleF5E3::from((1, 2));     
    /// let diff = a - b;                    
    /// assert!(diff.r() == 2.5);
    /// assert!(diff.i() == 2);
    ///
    /// // Subtracting with Zero
    /// assert!(a - Circle::<i32, i8>::ZERO == a);
    /// let neg_a = -a;
    /// assert!(Circle::<i32, i8>::ZERO - a == neg_a);
    ///
    /// // Subtracting Circles with different exponents
    /// let small = CircleF5E3::from((0.125, 0.25));
    /// let large = CircleF5E3::from((128, 64));
    /// let result = large - small;
    /// assert!(result.r() == 127.875);
    /// assert!(result.i() == 63.75);
    ///
    /// // Subtraction that produces Zero
    /// assert!((a - a).is_zero());
    ///
    /// // Subtraction with vanished Circles
    /// let tiny = CircleF5E3::MIN_POS_REAL / 4;
    /// assert!(tiny.vanished());
    /// assert!((a - tiny).r() == a.r());
    /// assert!((a - tiny).i() == a.i());
    /// assert!((tiny - a).r() == -a.r());
    /// assert!((tiny - a).i() == -a.i());
    ///
    /// // Subtraction with exploded Circles
    /// let huge = CircleF5E3::MAX_REAL_CIRCLE * 4;
    /// assert!(huge.exploded());
    /// assert!((a - huge).is_undefined());
    ///
    /// // Subtracting two exploded Circles
    /// assert!((huge - huge).is_undefined());
    /// ```
    pub(crate) fn circle_subtract_circle(&self, circle: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !circle.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if circle.is_undefined() {
                return *circle;
            }
            if self.exploded() && circle.exploded() {
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
            if self.exploded() {
                let prefix: F = TRANSFINITE_MINUS_FINITE.prefix.sa();
                return Circle {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if circle.exploded() {
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
                return *self;
            }
            if self.is_zero() {
                return -circle;
            }
            return *self;
        }

        if self.exponent > circle.exponent {
            let exp_diff = self.exponent.wrapping_sub(&circle.exponent);
            if exp_diff.is_negative() {
                return *self;
            }

            if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
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
                    let small_r: i16 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    let mut big_i: i16 = self.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i16 = circle.imaginary.as_();
                    let result_i = big_i.wrapping_sub(small_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i32 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    let mut big_i: i32 = self.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i32 = circle.imaginary.as_();
                    let result_i = big_i.wrapping_sub(small_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i64 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    let mut big_i: i64 = self.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i64 = circle.imaginary.as_();
                    let result_i = big_i.wrapping_sub(small_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i128 = circle.real.as_();
                    let result_r = big_r.wrapping_sub(small_r);

                    let mut big_i: i128 = self.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i128 = circle.imaginary.as_();
                    let result_i = big_i.wrapping_sub(small_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: I256 = circle.real.into();
                    let result_r = big_r.wrapping_sub(small_r);

                    let mut big_i: I256 = self.imaginary.into();
                    big_i <<= shift;
                    let small_i: I256 = circle.imaginary.into();
                    let result_i = big_i.wrapping_sub(small_i);

                    if result_r == 0.into() && result_i == 0.into() {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading - 2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            imaginary: ((result_i << (leading - 2)) >> F::FRACTION_BITS)
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
            let exp_diff = circle.exponent.wrapping_sub(&self.exponent);
            if exp_diff.is_negative() {
                return -circle;
            }

            if E::EXPONENT_BITS >= (std::mem::size_of::<isize>() as isize).wrapping_mul(8) {
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
                    let small_r: i16 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    let mut big_i: i16 = circle.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i16 = self.imaginary.as_();
                    let result_i = small_i.wrapping_sub(big_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i32 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    let mut big_i: i32 = circle.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i32 = self.imaginary.as_();
                    let result_i = small_i.wrapping_sub(big_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i64 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    let mut big_i: i64 = circle.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i64 = self.imaginary.as_();
                    let result_i = small_i.wrapping_sub(big_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: i128 = self.real.as_();
                    let result_r = small_r.wrapping_sub(big_r);

                    let mut big_i: i128 = circle.imaginary.as_();
                    big_i <<= shift;
                    let small_i: i128 = self.imaginary.as_();
                    let result_i = small_i.wrapping_sub(big_i);

                    if result_r == 0 && result_i == 0 {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

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
                    let small_r: I256 = self.real.into();
                    let result_r = small_r.wrapping_sub(big_r);

                    let mut big_i: I256 = circle.imaginary.into();
                    big_i <<= shift;
                    let small_i: I256 = self.imaginary.into();
                    let result_i = small_i.wrapping_sub(big_i);

                    if result_r == 0.into() && result_i == 0.into() {
                        return Circle::<F, E>::ZERO;
                    }

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(F::FRACTION_BITS.wrapping_sub(leading).as_()));

                    if circle.exponent.is_negative() && !offset.is_negative() {
                        return Circle {
                            real: ((result_r << (leading - 2)) >> F::FRACTION_BITS)
                                .as_i128()
                                .as_(),
                            imaginary: ((result_i << (leading - 2)) >> F::FRACTION_BITS)
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
