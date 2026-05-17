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
    /// Adds this Circle to another Circle
    ///
    /// # Description
    ///
    /// Performs addition between two Circles, handling special cases according to mathematical principles. Returns a finite Circle unless the result exceeds representable range, in which case it may return an exploded or vanished Circle.
    ///
    /// Addition process:
    /// 0. Checks for any abnormal Circles (Zeros, vanished, exploded or undefined) and handles these cases
    /// 1. Aligns fractions by shifting the larger value left based on exponent difference
    /// 2. Adds the aligned values for both real and imaginary components simultaneously
    /// 3. Normalizes the result and adjusts exponent accordingly
    /// 4. Escapes for underflow if necessary (vanished), overflow is naturally handled by escaped exponent alignment
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[↑]` + `[↑]` ➔ `[℘ ↑+↑]` Undefined exploded plus exploded state
    /// - `[↑]` + `[#]` ➔ `[℘ ↑+]` Undefined exploded plus finite state
    /// - `[#]` + `[↑]` ➔ `[℘ +↑]` Undefined finite plus exploded state
    /// - `[↓]` + `[↓]` ➔ `[℘ ↓+↓]` Undefined vanished plus vanished state
    /// - `[↓]` + `[#]` or `[#]` + `[↓]` ➔ `[#]` The finite Circle
    /// - `[0]` + `[#]` or `[#]` + `[0]` ➔ `[#]` The non-Zero Circle
    /// - `[0]` + `[0]` ➔ `[0]` Zero
    /// - `[#]` + `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Adding finite Circles
    /// let a = Circle::<i32, i8>::from((3, 4.5));
    /// let b = CircleF5E3::from((1, 2));
    /// let sum = a + b;
    /// assert!(sum.r() == 4);
    /// assert!(sum.i() == 6.5);
    ///
    /// // Adding with Zero
    /// assert!(a + 0 == a);
    /// assert!(0 + a == a);
    ///
    /// // Adding Circles with different exponents
    /// let small = CircleF5E3::from((0.125, 0.25));
    /// let large = CircleF5E3::from((128, 64));
    /// let result = small + large;
    /// assert!(result.r() == 128.125);
    /// assert!(result.i() == 64.25);
    ///
    /// // Adding Circles that produce Zero
    /// let pos = CircleF5E3::from((2.5, 3.75));
    /// let neg = CircleF5E3::from((-2.5, -3.75));
    /// assert!((pos + neg).is_zero());
    /// assert!((neg + pos).is_zero());
    ///
    /// // Addition with vanished Circles
    /// let tiny = CircleF5E3::MIN_POS_REAL / 5;
    /// assert!(tiny.vanished());
    /// assert!((a + tiny).r() == a.r());
    /// assert!((a + tiny).i() == a.i());
    ///
    /// // Addition with exploded Circles
    /// let huge = CircleF5E3::MAX_REAL_CIRCLE * 5;
    /// assert!(huge.exploded());
    /// assert!((a + huge).is_undefined());
    ///
    /// // Adding two exploded Circles
    /// assert!((huge + huge).is_undefined());
    /// ```
    pub(crate) fn circle_add_circle(&self, circle: &Self) -> Self {
        if !self.is_normal() || !circle.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if circle.is_undefined() {
                return *circle;
            }
            // Infinity absorbs everything. [∞] is the signless Riemann-sphere point reached only by n/0; −∞ is a no-op so [∞]−[∞] = [∞] too.
            if self.is_infinite() || circle.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() {
                return *circle;
            }
            if circle.is_zero() {
                return *self;
            }
            if self.exploded() && circle.exploded() {
                return Self {
                    real: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && circle.vanished() {
                return Self {
                    real: VANISHED_PLUS_VANISHED.prefix.sa(),
                    imaginary: VANISHED_PLUS_VANISHED.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.exploded() {
                if circle.vanished() {
                    return *self;
                }
                return Self {
                    real: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    imaginary: TRANSFINITE_PLUS_FINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if circle.exploded() {
                if self.vanished() {
                    return *circle;
                }
                return Self {
                    real: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    imaginary: FINITE_PLUS_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() {
                return *circle;
            }
            if circle.vanished() {
                return *self;
            }
            return *self;
        }

        // AMBIG=0 native: dominance via unsigned-cyclic compare on stored exp (matches Scalar add/sub).
        let (big, small) = if self.exponent.into_unsigned() > circle.exponent.into_unsigned() {
            (self, circle)
        } else {
            (circle, self)
        };

        let exp_diff = big.exponent.wrapping_sub(&small.exponent);
        // Under cmp_unsigned ordering, exp_diff (interpreted unsigned) is non-negative. If the unsigned diff exceeds FRAC_BITS, small is negligible against big.
        let frac_bits_e: E = Self::fraction_bits().as_();
        if exp_diff.into_unsigned() >= frac_bits_e.into_unsigned() {
            return *big;
        }

        match Self::fraction_bits() {
            8 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i16 = big.real.as_();
                big_r <<= shift;
                let small_r: i16 = small.real.as_();
                let result_r = big_r.wrapping_add(small_r);

                let mut big_i: i16 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i16 = small.imaginary.as_();
                let result_i = big_i.wrapping_add(small_i);

                if result_r.is_zero() && result_i.is_zero() {
                    return Self {
                        real: F::zero(),
                        imaginary: F::zero(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }

                let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                let leading = leading_r.min(leading_i);
                let offset = small
                    .exponent
                    .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                // AMBIG=0 wrap detection: catches both `final_exp == AMBIG` (offset = MAX, +1 wraps to 0) and `offset wrapped past AMBIG to small-positive stored` (offset-1 in unsigned < small.exp.unsigned means the addition crossed the cycle's max/0 boundary).
                let final_exp = offset.wrapping_add(&E::one());
                let one_e: E = 1u8.as_();
                if final_exp == Self::ambiguous_exponent()
                    || offset.wrapping_sub(&one_e).into_unsigned()
                        < small.exponent.into_unsigned()
                {
                    return Self {
                        real: ((result_r << (leading.wrapping_sub(2))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(2)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits()).as_(),
                    imaginary: ((result_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                        .as_(),
                    exponent: final_exp,
                };
            }
            16 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i32 = big.real.as_();
                big_r <<= shift;
                let small_r: i32 = small.real.as_();
                let result_r = big_r.wrapping_add(small_r);

                let mut big_i: i32 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i32 = small.imaginary.as_();
                let result_i = big_i.wrapping_add(small_i);

                if result_r.is_zero() && result_i.is_zero() {
                    return Self {
                        real: F::zero(),
                        imaginary: F::zero(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }

                let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                let leading = leading_r.min(leading_i);
                let offset = small
                    .exponent
                    .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                // AMBIG=0 wrap detection: catches both `final_exp == AMBIG` (offset = MAX, +1 wraps to 0) and `offset wrapped past AMBIG to small-positive stored` (offset-1 in unsigned < small.exp.unsigned means the addition crossed the cycle's max/0 boundary).
                let final_exp = offset.wrapping_add(&E::one());
                let one_e: E = 1u8.as_();
                if final_exp == Self::ambiguous_exponent()
                    || offset.wrapping_sub(&one_e).into_unsigned()
                        < small.exponent.into_unsigned()
                {
                    return Self {
                        real: ((result_r << (leading.wrapping_sub(2))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(2)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits()).as_(),
                    imaginary: ((result_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                        .as_(),
                    exponent: final_exp,
                };
            }
            32 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i64 = big.real.as_();
                big_r <<= shift;
                let small_r: i64 = small.real.as_();
                let result_r = big_r.wrapping_add(small_r);

                let mut big_i: i64 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i64 = small.imaginary.as_();
                let result_i = big_i.wrapping_add(small_i);

                if result_r.is_zero() && result_i.is_zero() {
                    return Self {
                        real: F::zero(),
                        imaginary: F::zero(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }

                let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                let leading = leading_r.min(leading_i);
                let offset = small
                    .exponent
                    .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                // AMBIG=0 wrap detection: catches both `final_exp == AMBIG` (offset = MAX, +1 wraps to 0) and `offset wrapped past AMBIG to small-positive stored` (offset-1 in unsigned < small.exp.unsigned means the addition crossed the cycle's max/0 boundary).
                let final_exp = offset.wrapping_add(&E::one());
                let one_e: E = 1u8.as_();
                if final_exp == Self::ambiguous_exponent()
                    || offset.wrapping_sub(&one_e).into_unsigned()
                        < small.exponent.into_unsigned()
                {
                    return Self {
                        real: ((result_r << (leading.wrapping_sub(2))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(2)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits()).as_(),
                    imaginary: ((result_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                        .as_(),
                    exponent: final_exp,
                };
            }
            64 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: i128 = big.real.as_();
                big_r <<= shift;
                let small_r: i128 = small.real.as_();
                let result_r = big_r.wrapping_add(small_r);

                let mut big_i: i128 = big.imaginary.as_();
                big_i <<= shift;
                let small_i: i128 = small.imaginary.as_();
                let result_i = big_i.wrapping_add(small_i);

                if result_r.is_zero() && result_i.is_zero() {
                    return Self {
                        real: F::zero(),
                        imaginary: F::zero(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }

                let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                let leading = leading_r.min(leading_i);
                let offset = small
                    .exponent
                    .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                // AMBIG=0 wrap detection: catches both `final_exp == AMBIG` (offset = MAX, +1 wraps to 0) and `offset wrapped past AMBIG to small-positive stored` (offset-1 in unsigned < small.exp.unsigned means the addition crossed the cycle's max/0 boundary).
                let final_exp = offset.wrapping_add(&E::one());
                let one_e: E = 1u8.as_();
                if final_exp == Self::ambiguous_exponent()
                    || offset.wrapping_sub(&one_e).into_unsigned()
                        < small.exponent.into_unsigned()
                {
                    return Self {
                        real: ((result_r << (leading.wrapping_sub(2))) >> Self::fraction_bits())
                            .as_(),
                        imaginary: ((result_i << (leading.wrapping_sub(2)))
                            >> Self::fraction_bits())
                        .as_(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }
                return Self {
                    real: ((result_r << (leading.wrapping_sub(1))) >> Self::fraction_bits()).as_(),
                    imaginary: ((result_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                        .as_(),
                    exponent: final_exp,
                };
            }
            128 => {
                let shift: isize = exp_diff.as_();
                let mut big_r: I256 = big.real.into();
                big_r <<= shift;
                let small_r: I256 = small.real.into();
                let result_r = big_r.wrapping_add(small_r);

                let mut big_i: I256 = big.imaginary.into();
                big_i <<= shift;
                let small_i: I256 = small.imaginary.into();
                let result_i = big_i.wrapping_add(small_i);

                if result_r == 0.into() && result_i == 0.into() {
                    return Self {
                        real: F::zero(),
                        imaginary: F::zero(),
                        exponent: Self::ambiguous_exponent(),
                    };
                }

                let leading_r = result_r.leading_ones().max(result_r.leading_zeros()) as isize;
                let leading_i = result_i.leading_ones().max(result_i.leading_zeros()) as isize;
                let leading = leading_r.min(leading_i);
                let offset = small
                    .exponent
                    .wrapping_add(&(Self::fraction_bits().wrapping_sub(leading)).as_());

                let final_exp = offset.wrapping_add(&E::one());
                if final_exp == Self::ambiguous_exponent() {
                    return Self {
                        real: ((result_r << (leading.wrapping_sub(2))) >> Self::fraction_bits())
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
                    imaginary: ((result_i << (leading.wrapping_sub(1))) >> Self::fraction_bits())
                        .as_i128()
                        .as_(),
                    exponent: final_exp,
                };
            }
            _ => Self {
                real: GENERAL.prefix.sa(),
                imaginary: GENERAL.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            },
        }
    }
}
