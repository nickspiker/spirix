use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub, Zero};
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
    /// Adds a Scalar to this Circle
    ///
    /// # Description
    ///
    /// Performs addition between a Circle and a Scalar, adding the Scalar value to the real component of the Circle while leaving the imaginary component unchanged (except for normalization).
    /// Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
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
    /// - `[∞]` + `[#, #i]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#, #i]` + `[∞]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[∞]` + `[∞, #i]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[↑]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[↑]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↓]` + `[↓]` ➔ `[℘ ↓+↓]` Undefined vanished plus vanished state
    /// - `[↓]` + `[#]` ➔ `[#]` The finite Scalar as real part, zero imaginary
    /// - `[#]` + `[↓]` ➔ `[#]` The Circle unchanged
    /// - `[0]` + `[#]` ➔ `[#, 0i]` The Scalar as real part, zero imaginary
    /// - `[#]` + `[0]` ➔ `[#]` The Circle unchanged
    /// - `[#]` + `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3, Scalar, ScalarF5E3};
    ///
    /// // Adding a Scalar to a Circle
    /// let a = Circle::<i32, i8>::from((3, 4));
    /// let b = ScalarF5E3::from(5.25);
    /// let sum = a + b;
    /// assert!(sum.r() == 8.25);
    /// assert!(sum.i() == 4);
    ///
    /// // Adding with Zero
    /// assert!(a + ScalarF5E3::ZERO == a);
    /// assert!(CircleF5E3::ZERO + b == CircleF5E3::from((5.25, 0)));
    ///
    /// // Adding with infinity (mathematically undefined)
    /// let infinity = ScalarF5E3::ONE / 0;
    /// assert!((a + infinity).is_undefined()); // Returns [℘ +↑] (finite plus exploded)
    /// assert!((infinity + a).is_undefined()); // Returns [℘ ↑+] (exploded plus finite)
    ///
    /// // Adding with different exponents
    /// let small_circle = CircleF5E3::from((0.125, 0.5));
    /// let large_scalar = ScalarF5E3::from(144);
    /// let result = small_circle + large_scalar;
    /// assert!(result.r() == 144.125);
    /// assert!(result.i() == 0.5);
    ///
    /// // Adding components that cancel
    /// let c = CircleF5E3::from((-2.5, 1.75));
    /// let d = ScalarF5E3::from(2.5);
    /// let result = c + d;
    /// assert!(result.r() == 0);
    /// assert!(result.i() == 1.75);
    ///
    /// // Addition with vanished values
    /// let tiny = ScalarF5E3::MIN_POS / 5;
    /// assert!(tiny.vanished());
    /// assert!((a + tiny) == a);
    ///
    /// // Addition with exploded values
    /// let huge = ScalarF5E3::MAX * 5;
    /// assert!(huge.exploded());
    /// assert!((a + huge).is_undefined());
    ///
    /// // Adding two exploded values
    /// let huge_circle = CircleF5E3::MAX * 3;
    /// assert!((huge_circle + huge).is_undefined());
    /// ```
    pub(crate) fn circle_add_scalar(&self, scalar: &Scalar<F, E>) -> Self {
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
            if self.is_transfinite() && scalar.is_transfinite() {
                return Self {
                    real: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && scalar.vanished() {
                return Self {
                    real: VANISHED_PLUS_VANISHED.prefix.sa(),
                    imaginary: VANISHED_PLUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_transfinite() {
                return Self {
                    real: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if scalar.is_transfinite() {
                return Self {
                    real: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return Circle {
                    real: scalar.fraction,
                    imaginary: F::zero(),
                    exponent: scalar.exponent,
                };
            }
            if scalar.vanished() {
                return *self;
            }
            if self.is_zero() {
                return Circle {
                    real: scalar.fraction,
                    imaginary: F::zero(),
                    exponent: scalar.exponent,
                };
            }
            return *self;
        }

        if self.exponent > scalar.exponent {
            let exp_diff = self.exponent.wrapping_sub(&scalar.exponent);
            if exp_diff.is_negative() {
                return *self;
            }

            if Self::exponent_bits() >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= Self::fraction_bits().as_() {
                    return *self;
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= Self::fraction_bits() {
                    return *self;
                }
            }

            match Self::fraction_bits() {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i16 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let mut big_i: i16 = self.imaginary.as_();
                    big_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = big_i.leading_ones().max(big_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = scalar
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((big_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Self {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((big_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                16 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i32 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i32 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let mut big_i: i32 = self.imaginary.as_();
                    big_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = big_i.leading_ones().max(big_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = scalar
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((big_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Self {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((big_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                32 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i64 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i64 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let mut big_i: i64 = self.imaginary.as_();
                    big_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = big_i.leading_ones().max(big_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = scalar
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((big_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Self {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((big_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                64 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i128 = self.real.as_();
                    big_r <<= shift;
                    let small_r: i128 = scalar.fraction.as_();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let mut big_i: i128 = self.imaginary.as_();
                    big_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = big_i.leading_ones().max(big_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = scalar
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((big_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Self {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((big_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                128 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: I256 = self.real.into();
                    big_r <<= shift;
                    let small_r: I256 = scalar.fraction.into();
                    let result_r = big_r.wrapping_add(small_r);

                    if result_r == 0.into() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let mut big_i: I256 = self.imaginary.into();
                    big_i <<= shift;

                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let leading_i = big_i.leading_ones().max(big_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);

                    let offset = scalar
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                    if self.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            imaginary: ((big_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }

                    return Self {
                        real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                        imaginary: ((big_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                            .as_i128()
                            .as_(),
                        exponent: offset.wrapping_add(&E::one()),
                    };
                }
                _ => {}
            }
        } else {
            // Scalar is bigger
            let exp_diff = scalar.exponent.wrapping_sub(&self.exponent);
            if exp_diff.is_negative() {
                return Circle {
                    real: scalar.fraction,
                    imaginary: self.imaginary,
                    exponent: scalar.exponent,
                };
            }

            if Self::exponent_bits() >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
                if exp_diff >= Self::fraction_bits().as_() {
                    return Circle {
                        real: scalar.fraction,
                        imaginary: self.imaginary,
                        exponent: scalar.exponent,
                    };
                }
            } else {
                let exp_diff_isize: isize = exp_diff.as_();
                if exp_diff_isize >= Self::fraction_bits() {
                    return Circle {
                        real: scalar.fraction,
                        imaginary: self.imaginary,
                        exponent: scalar.exponent,
                    };
                }
            }

            match Self::fraction_bits() {
                8 => {
                    let shift: isize = exp_diff.as_();
                    let mut big_r: i16 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i16 = self.real.as_();
                    let result_r = big_r.wrapping_add(small_r);
                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let result_i: i16 = self.imaginary.as_();
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    return Self {
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
                    let mut big_r: i32 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i32 = self.real.as_();
                    let result_r = big_r.wrapping_add(small_r);
                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let result_i: i32 = self.imaginary.as_();
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    return Self {
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
                    let mut big_r: i64 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i64 = self.real.as_();
                    let result_r = big_r.wrapping_add(small_r);
                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let result_i: i64 = self.imaginary.as_();
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    return Self {
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
                    let mut big_r: i128 = scalar.fraction.as_();
                    big_r <<= shift;
                    let small_r: i128 = self.real.as_();
                    let result_r = big_r.wrapping_add(small_r);
                    if result_r.is_zero() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let result_i: i128 = self.imaginary.as_();
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
                            real: ((result_r << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            imaginary: ((result_i << (leading.wrapping_sub(2)))
                                >> Self::fraction_bits())
                            .as_(),
                            exponent: Self::ambiguous_exponent(),
                        };
                    }
                    return Self {
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
                    let mut big_r: I256 = scalar.fraction.into();
                    big_r <<= shift;
                    let small_r: I256 = self.real.into();
                    let result_r = big_r.wrapping_add(small_r);
                    if result_r == 0.into() && self.imaginary.is_zero() {
                        return Self::ZERO;
                    }
                    let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                    let result_i: I256 = self.imaginary.into();
                    let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                    let leading = leading_r.min(leading_i);
                    let offset = self
                        .exponent
                        .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());
                    if scalar.exponent.is_negative() && !offset.is_negative() {
                        return Self {
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
                    return Self {
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

        Self::ZERO
    }
}
