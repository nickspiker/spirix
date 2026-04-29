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
    /// Multiplies this Circle by another Circle
    ///
    /// # Description
    ///
    /// Performs complex number multiplication between two Circles, handling special cases according to mathematical principles. For normal Circles, this follows standard complex multiplication formula (a+b·i)(c+d·i) = (a·c-b·d) + (a·d+b·c)·i. Vanished and exploded Circles are multiplied without any exponent handling to maintain orientation and to follow the multiplication rule.
    ///
    /// Multiplication process:
    /// 0. Checks for and handles undefined Circles
    /// 1. Uses wider integer types for complex number multiplication components:
    ///    ```text
    ///    (a+b·i)·(c+d·i) = (a·c-b·d) + (a·d+b·c)·i
    ///    ```
    ///    Each component calculation uses a 2x-bit space:
    ///    ```text
    ///    a   →         ■■■■■■■ = self.real
    ///    b   →         ■■■■■■■ = self.imaginary
    ///
    ///    c   →         ■■■■■■■ = other.real
    ///    d   →         ■■■■■■■ = other.imaginary
    ///
    ///              ⤪⤪⤪⤪⤪⤪       Multiply!
    ///    a·c → ■■■■■■■ ■■■■■■■ = Intermediate product
    ///    b·d → ■■■■■■■ ■■■■■■■ = Intermediate product
    ///    Dif → ■■■■■■■ □□□□□□□ = Difference of a·c - b·d
    ///              ↘↘↘↘↘↘↘
    ///    Real →        ■■■■■■■ = High half of (a·c-b·d)
    ///
    ///              ⤪⤪⤪⤪⤪⤪       Multiply!
    ///    a·d → ■■■■■■■ ■■■■■■■ = Intermediate product
    ///    b·c → ■■■■■■■ ■■■■■■■ = Intermediate product
    ///    Sum → ■■■■■■■ □□□□□□□ = Sum of a·d + b·c
    ///              ↘↘↘↘↘↘↘
    ///    Imaginary →   ■■■■■■■ = High half of (a·d+b·c)
    ///    ```
    /// 2. Calculates leading Zeros/Ones for both real and imaginary parts to find minimum, determining normalization shift
    /// 3. If normal, add exponents and adjust by normalization shift (exponent_result = self.exponent + other.exponent - shift), otherwise follow escaped/vanished rules
    /// 4. Escape cases where exponent exceeds MAX_EXPONENT (explode) or falls below MIN_EXPONENT (vanish)
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with orientation following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with orientation following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with orientation following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with orientation following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Circle
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Complex multiplication!
    /// let z1 = Circle::<i64, i16>::from((1.5, 2));
    /// let z2 = CircleF5E3::from((1, 2));
    /// let product = z1 * z2;
    /// assert!(product.r() == -2.5);
    /// assert!(product.i() == 5);
    ///
    /// // Multiplying by i rotates 90 degrees counterclockwise
    /// let z = CircleF5E3::from((5, 0));
    /// let rotated = z * CircleF5E3::POS_I;
    /// assert!(rotated.r() == 0);
    /// assert!(rotated.i() == 5);
    ///
    /// // Multiplying by Zero produces Zero
    /// let zero = CircleF5E3::ZERO;
    /// assert!((z * zero).is_zero());
    /// // Even when exploded!
    /// let sploded = CircleF5E3::MAX.square();
    /// assert!(sploded * 0 == 0);
    /// // And of course when vanished
    /// let tiny = CircleF5E3::MIN_POS.square();
    ///
    /// // Multiplication preserves orientation
    /// let minus_one = CircleF5E3::I.square();
    /// assert!(minus_one == -1);
    ///
    /// // Vanished values maintain orientation thru multiplication
    /// let huge = CircleF5E3::MAX_POS * 2;
    /// assert!(huge.exploded());
    /// assert!(huge.sign() == 1);
    /// let rotated_huge = huge * i;
    /// assert!(rotated_huge.sign() == CircleF5E3::I);
    ///
    /// // Vanished × Exploded yields an undefined state (orientation indeterminate)
    /// let undefined_product = tiny * huge;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn circle_multiply_circle(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            } else if other.is_undefined() {
                return *other;
            } else if self.is_infinite() && other.is_zero() {
                return Self {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_zero() && other.is_infinite() {
                return Self {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            } else if self.exploded() && other.vanished() {
                return Self {
                    real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else if self.vanished() && other.exploded() {
                return Self {
                    real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            } else {
                let n_level: isize = if self.exploded() || other.exploded() {
                    -1
                } else {
                    -2
                };

                let (product_real, product_imaginary) = match Self::fraction_bits() {
                    8 => {
                        let a: i16 = self.real.as_();
                        let b: i16 = self.imaginary.as_();
                        let c: i16 = other.real.as_();
                        let d: i16 = other.imaginary.as_();

                        let real_product =
                            (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                        let imag_product =
                            (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> Self::fraction_bits()).as_(),
                            (normalized_imag >> Self::fraction_bits()).as_(),
                        )
                    }
                    16 => {
                        let a: i32 = self.real.as_();
                        let b: i32 = self.imaginary.as_();
                        let c: i32 = other.real.as_();
                        let d: i32 = other.imaginary.as_();

                        let real_product =
                            (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                        let imag_product =
                            (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> Self::fraction_bits()).as_(),
                            (normalized_imag >> Self::fraction_bits()).as_(),
                        )
                    }
                    32 => {
                        let a: i64 = self.real.as_();
                        let b: i64 = self.imaginary.as_();
                        let c: i64 = other.real.as_();
                        let d: i64 = other.imaginary.as_();

                        let real_product =
                            (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                        let imag_product =
                            (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> Self::fraction_bits()).as_(),
                            (normalized_imag >> Self::fraction_bits()).as_(),
                        )
                    }
                    64 => {
                        let a: i128 = self.real.as_();
                        let b: i128 = self.imaginary.as_();
                        let c: i128 = other.real.as_();
                        let d: i128 = other.imaginary.as_();

                        let real_product =
                            (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                        let imag_product =
                            (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> Self::fraction_bits()).as_(),
                            (normalized_imag >> Self::fraction_bits()).as_(),
                        )
                    }
                    128 => {
                        let a: I256 = self.real.into();
                        let b: I256 = self.imaginary.into();
                        let c: I256 = other.real.into();
                        let d: I256 = other.imaginary.into();

                        let real_product: I256 =
                            (a.wrapping_mul(c) >> 1usize).wrapping_sub(b.wrapping_mul(d) >> 1);
                        let imag_product: I256 =
                            (a.wrapping_mul(d) >> 1usize).wrapping_add(b.wrapping_mul(c) >> 1);

                        let shift_r = real_product
                            .leading_ones()
                            .max(real_product.leading_zeros());
                        let shift_i = imag_product
                            .leading_ones()
                            .max(imag_product.leading_zeros());
                        let shift = shift_r.min(shift_i) as isize;

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_real = real_product << shift_amount;
                        let normalized_imag = imag_product << shift_amount;
                        (
                            (normalized_real >> Self::fraction_bits()).as_i128().as_(),
                            (normalized_imag >> Self::fraction_bits()).as_i128().as_(),
                        )
                    }
                    _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
                };

                return Self {
                    real: product_real,
                    imaginary: product_imaginary,
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        let real;
        let imaginary;
        let expo_adjust: isize;

        match Self::fraction_bits() {
            8 => {
                let a: i16 = self.real.as_();
                let b: i16 = self.imaginary.as_();
                let c: i16 = other.real.as_();
                let d: i16 = other.imaginary.as_();

                let real_product = (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                let imag_product = (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                if real_product == 0 && imag_product == 0 {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> Self::fraction_bits()).as_();
                imaginary = (normalized_imag >> Self::fraction_bits()).as_();
            }
            16 => {
                let a: i32 = self.real.as_();
                let b: i32 = self.imaginary.as_();
                let c: i32 = other.real.as_();
                let d: i32 = other.imaginary.as_();

                let real_product = (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                let imag_product = (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                if real_product == 0 && imag_product == 0 {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> Self::fraction_bits()).as_();
                imaginary = (normalized_imag >> Self::fraction_bits()).as_();
            }
            32 => {
                let a: i64 = self.real.as_();
                let b: i64 = self.imaginary.as_();
                let c: i64 = other.real.as_();
                let d: i64 = other.imaginary.as_();

                let real_product = (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                let imag_product = (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                if real_product == 0 && imag_product == 0 {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> Self::fraction_bits()).as_();
                imaginary = (normalized_imag >> Self::fraction_bits()).as_();
            }
            64 => {
                let a: i128 = self.real.as_();
                let b: i128 = self.imaginary.as_();
                let c: i128 = other.real.as_();
                let d: i128 = other.imaginary.as_();

                let real_product = (a.wrapping_mul(c) >> 1).wrapping_sub(b.wrapping_mul(d) >> 1);
                let imag_product = (a.wrapping_mul(d) >> 1).wrapping_add(b.wrapping_mul(c) >> 1);

                if real_product == 0 && imag_product == 0 {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> Self::fraction_bits()).as_();
                imaginary = (normalized_imag >> Self::fraction_bits()).as_();
            }
            128 => {
                let a: I256 = self.real.into();
                let b: I256 = self.imaginary.into();
                let c: I256 = other.real.into();
                let d: I256 = other.imaginary.into();

                let real_product: I256 =
                    (a.wrapping_mul(c) >> 1usize).wrapping_sub(b.wrapping_mul(d) >> 1);
                let imag_product: I256 =
                    (a.wrapping_mul(d) >> 1usize).wrapping_add(b.wrapping_mul(c) >> 1);

                if real_product == 0.into() && imag_product == 0.into() {
                    return Self::ZERO;
                }

                let leading_r = real_product
                    .leading_ones()
                    .max(real_product.leading_zeros());
                let leading_i = imag_product
                    .leading_ones()
                    .max(imag_product.leading_zeros());

                expo_adjust = (leading_r.min(leading_i) as isize).wrapping_sub(3);
                let shift = expo_adjust.wrapping_add(2);

                let normalized_real = real_product << shift;
                let normalized_imag = imag_product << shift;

                real = (normalized_real >> Self::fraction_bits()).as_i128().as_();
                imaginary = (normalized_imag >> Self::fraction_bits()).as_i128().as_();
            }
            _ => {
                return Self {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        match Self::exponent_bits() {
            8 => {
                let self_exponent: i16 = self.exponent.as_();
                let other_exponent: i16 = other.exponent.as_();
                let upcast_exponent: i16 = self_exponent
                    .wrapping_add(other_exponent)
                    .wrapping_sub(expo_adjust as i16);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            16 => {
                let self_exponent: i32 = self.exponent.as_();
                let other_exponent: i32 = other.exponent.as_();
                let upcast_exponent: i32 = self_exponent
                    .wrapping_add(other_exponent)
                    .wrapping_sub(expo_adjust as i32);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            32 => {
                let self_exponent: i64 = self.exponent.as_();
                let other_exponent: i64 = other.exponent.as_();
                let upcast_exponent: i64 = self_exponent
                    .wrapping_add(other_exponent)
                    .wrapping_sub(expo_adjust as i64);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            64 => {
                let self_exponent: i128 = self.exponent.as_();
                let other_exponent: i128 = other.exponent.as_();
                let upcast_exponent: i128 = self_exponent
                    .wrapping_add(other_exponent)
                    .wrapping_sub(expo_adjust as i128);

                if upcast_exponent > Self::max_exponent().as_() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().as_() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_(),
                    };
                }
            }
            128 => {
                let self_exponent: I256 = self.exponent.into();
                let other_exponent: I256 = other.exponent.into();
                let e: I256 = (expo_adjust as i128).into();
                let upcast_exponent: I256 =
                    self_exponent.wrapping_add(other_exponent).wrapping_sub(e);

                if upcast_exponent > Self::max_exponent().into() {
                    return Self {
                        real,
                        imaginary,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else if upcast_exponent < Self::min_exponent().into() {
                    return Self {
                        real: real >> 1isize,
                        imaginary: imaginary >> 1isize,
                        exponent: Self::ambiguous_exponent(),
                    };
                } else {
                    return Self {
                        real,
                        imaginary,
                        exponent: upcast_exponent.as_i128().as_(),
                    };
                }
            }
            _ => {
                return Self {
                    real: GENERAL.prefix.sa(),
                    imaginary: GENERAL.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
    }
}
