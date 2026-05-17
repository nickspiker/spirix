use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
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
    /// Adds this Scalar to a Circle
    ///
    /// # Description
    ///
    /// Performs addition between a Scalar and a Circle, adding the Scalar value to the real component of the Circle while leaving the imaginary component unchanged (except for normalization). Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Addition process:
    /// 0. Checks for any escaped (vanished, exploded, infinity, or undefined) values and handles these first
    /// 1. Checks for Zeros and returns the appropriate result
    /// 2. Aligns fractions by shifting the smaller value right based on exponent difference
    /// 3. Adds the Scalar to the real component of the Circle
    /// 4. Normalizes result and adjusts exponent, considering both components
    /// 5. Escapes for underflow if necessary, overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[∞]` + `[∞]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[∞]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[∞]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↑]` + `[↑]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[↑]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↓]` + `[↓]` ➔ `[℘ ↓+↓]` Undefined vanished plus vanished state
    /// - `[↓]` + `[#]` ➔ `[#]` The Circle unchanged
    /// - `[#]` + `[↓]` ➔ `[#]` The Scalar as real part, unchanged imaginary
    /// - `[0]` + `[#]` ➔ `[#]` The Circle unchanged
    /// - `[#]` + `[0]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[#]` + `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, Circle, ScalarF5E3, CircleF5E3};
    ///
    /// // Adding a Scalar to a Circle
    /// let a = Scalar::<i32, i8>::from(3.875);
    /// let b = CircleF5E3::from((4, 2));
    /// let sum = a + b;
    /// assert!(sum.r() == 7.875);
    /// assert!(sum.i() == 2);
    ///
    /// // Adding with Zero
    /// assert!(a + CircleF5E3::ZERO == CircleF5E3::from((3.875, 0)));
    /// assert!(ScalarF5E3::ZERO + b == b);
    ///
    /// // Adding with infinity (mathematically undefined)
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!((infinity + b).is_undefined()); // Returns [℘ ↑+] (infinity plus finite)
    /// assert!((a + CircleF5E3::INFINITY).is_undefined()); // Returns [℘ +↑] (finite plus infinity)
    ///
    /// // Adding with different exponents
    /// let small_scalar = ScalarF5E3::from(0.125);
    /// let large_circle = CircleF5E3::from((64, 32));
    /// let result = small_scalar + large_circle;
    /// assert!(result.r() == 64.125);
    /// assert!(result.i() == 32);
    ///
    /// // Addition with vanished values
    /// let tiny = ScalarF5E3::MIN_POS / 5;
    /// assert!(tiny.vanished());
    /// assert!((tiny + b) == b);
    /// // Zeros are the only values that can be added to vanished values
    /// assert!(tiny + CircleF5E3::ZERO == tiny);
    ///
    /// // Addition with exploded values
    /// let huge = ScalarF5E3::MAX * 5;
    /// assert!(huge.exploded());
    /// assert!((huge + b).is_undefined());
    ///
    /// // Adding two exploded values
    /// let huge_circle = CircleF5E3::MAX * 3;
    /// assert!((huge + huge_circle).is_undefined());
    /// ```
    pub(crate) fn scalar_add_circle(&self, circle: &Circle<F, E>) -> Circle<F, E> {
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
                return Circle {
                    real: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && circle.vanished() {
                return Circle {
                    real: VANISHED_PLUS_VANISHED.prefix.sa(),
                    imaginary: VANISHED_PLUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_transfinite() {
                return Circle {
                    real: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if circle.is_transfinite() {
                return Circle {
                    real: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return *circle;
            }
            if circle.vanished() {
                return Circle {
                    real: self.fraction,
                    imaginary: F::zero(),
                    exponent: self.exponent,
                };
            }
            if self.is_zero() {
                return *circle;
            }
            return Circle {
                real: self.fraction,
                imaginary: 0.as_(),
                exponent: self.exponent,
            };
        }

        // AMBIG=0 native: dominance via unsigned-cyclic compare.
        if self.exponent.into_unsigned() > circle.exponent.into_unsigned() {
            // Scalar is bigger
            let exp_diff = self.exponent.wrapping_sub(&circle.exponent);
            let frac_bits_e: E = Self::fraction_bits().as_();
            if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
                return Circle {
                    real: self.fraction,
                    imaginary: F::zero(),
                    exponent: self.exponent,
                };
            }

            match Self::fraction_bits() {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i16 = circle.real.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i16 = circle.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i32 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i32 = circle.real.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i32 = circle.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i64 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i64 = circle.real.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i64 = circle.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i128 = self.fraction.as_();
                    big_r <<= shift;
                    let small_r: i128 = circle.real.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: i128 = circle.imaginary.as_();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = circle
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: I256 = self.fraction.into();
                    big_r <<= shift;
                    let small_r: I256 = circle.real.into();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0.into() && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let result_i: I256 = circle.imaginary.into();

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros());
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros());
                    let leading = leading_r.min(leading_i);

                    let offset = circle.exponent.wrapping_add(
                        &(Self::fraction_bits().wrapping_sub(leading as isize)).as_(),
                    );

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_i128()
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                _ => {}
            }
        } else {
            // Circle is bigger
            let exp_diff = circle.exponent.wrapping_sub(&self.exponent);
            let frac_bits_e: E = Self::fraction_bits().as_();
            if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
                return *circle;
            }
            match Self::fraction_bits() {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i16 = self.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i16 = circle.imaginary.as_();
                    result_i <<= shift;
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i32 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i32 = self.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i32 = circle.imaginary.as_();
                    result_i <<= shift;
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i64 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i64 = self.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i64 = circle.imaginary.as_();
                    result_i <<= shift;
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i128 = circle.real.as_();
                    big_r <<= shift;
                    let small_r: i128 = self.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0 && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: i128 = circle.imaginary.as_();
                    result_i <<= shift;
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: I256 = circle.real.into();
                    big_r <<= shift;
                    let small_r: I256 = self.fraction.into();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0.into() && circle.imaginary.is_zero() {
                        return Circle::<F, E>::ZERO;
                    }

                    let mut result_i: I256 = circle.imaginary.into();
                    result_i <<= shift;
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    let final_exp = offset.wrapping_add(&E::one());
                    if final_exp == Self::ambiguous_exponent() {
                        return Circle {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Circle {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(1)))
                            >> Self::fraction_bits())
                        .as_i128()
                        .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                _ => {}
            }
        }
        Circle::<F, E>::ZERO
    }
}
