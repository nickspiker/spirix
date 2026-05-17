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
    /// Multiplies this Circle by a Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between a Circle and a Scalar, scaling both real and imaginary components by the Scalar value while preserving the orientation of the Circle. For normal values, this multiplies each component of the Circle by the Scalar, following standard complex-real multiplication formula: (a+b·i)·c = (a·c) + (b·c)·i.
    ///
    /// Multiplication process:
    /// 0. Checks for and handle undefined states
    /// 1. Uses wider integer types for component multiplication:
    ///    ```text
    ///    a   →         ■■■■■■■ = self.real
    ///    c   →         ■■■■■■■ = scalar.fraction
    ///                  ⤪⤪⤪⤪⤪⤪ Multiply!
    ///    a·c → ■■■■■■■ □□□□□□□ = Intermediate product for real part
    ///              ↘↘↘↘↘↘↘
    ///    Real →        ■■■■■■■ = High bits kept for real component
    ///
    ///    b   →         ■■■■■■■ = self.imaginary
    ///    c   →         ■■■■■■■ = scalar.fraction
    ///                  ⤪⤪⤪⤪⤪⤪ Multiply!
    ///    b·c → ■■■■■■■ □□□□□□□ = Intermediate product for imaginary part
    ///              ↘↘↘↘↘↘↘
    ///    Imaginary →   ■■■■■■■ = High bits kept for imaginary component
    ///    ```
    /// 2. Calculates leading Zeros/Ones for both components to determine normalization shift
    /// 3. If normal, adds exponents and adjusts by normalization shift (exponent_result = self.exponent + scalar.exponent - shift)
    /// 4. Checks for overflow/underflow and returns appropriate escaped values if necessary
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
    /// use spirix::{Circle, Scalar, CircleF6E4, ScalarF6E4};
    ///
    /// // Basic multiplication - scales both components
    /// let z = Circle::<i64, i16>::from((3, 4));
    /// let s = ScalarF6E4::from(2);
    /// let result = z * s;
    /// assert!(result.r() == 6);
    /// assert!(result.i() == 8);
    ///
    /// // Multiplying by negative Scalar negates the Circle
    /// let neg = ScalarF6E4::from(-1);
    /// let negated = z * neg;
    /// assert!(negated.r() == -3);
    /// assert!(negated.i() == -4);
    ///
    /// // Multiplying by Zero produces Zero
    /// let Zero = CircleF6E4::ZERO;
    /// let scale = ScalarF6E4::from(10);
    /// assert!((Zero * scale).is_zero());
    /// assert!((z * ScalarF6E4::ZERO).is_zero());
    ///
    /// // Fractional scaling preserves orientation
    /// let unit_circle = CircleF6E4::from((0.6, 0.8)); // magnitude = 1
    /// let half = ScalarF6E4::from(0.5);
    /// let half_circle = unit_circle * half;
    /// assert!(half_circle.magnitude() == 0.5);
    /// // Direction remains the same
    /// assert!(half_circle.r() / half_circle.magnitude() == unit_circle.r());
    /// assert!(half_circle.i() / half_circle.magnitude() == unit_circle.i());
    ///
    /// // Vanished values maintain orientation through multiplication
    /// let tiny = CircleF6E4::MIN_POS_REAL / 10;
    /// let tiny_circle = CircleF6E4::from((tiny.r(), tiny.r()));
    /// assert!(tiny_circle.vanished());
    /// let scaled_tiny = tiny_circle * ScalarF6E4::from(3);
    /// assert!(scaled_tiny.vanished()); // Still vanished, preserves orientation
    ///
    /// // Exploded values interact consistently with Scalars
    /// let huge = CircleF6E4::MAX_REAL_CIRCLE * 10;
    /// assert!(huge.exploded());
    /// let neg_huge = huge * ScalarF6E4::NEG_ONE;
    /// assert!(neg_huge.exploded());
    /// assert!(neg_huge.r() < 0); // Orientation is flipped
    ///
    /// // Vanished × Exploded yields an undefined state
    /// let undefined_product = tiny_circle * ScalarF6E4::from(1) / 0;
    /// assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn circle_multiply_scalar(&self, other: &Scalar<F, E>) -> Self {
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 unified pipeline. Circle * Scalar: each Circle component multiplies by the (N0→N1 converted) Scalar fraction; no cross-term since Scalar has no imaginary.
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let s = ((other.fraction >> 1isize) ^ F::min_value()).sign_extend();
            let fb = Self::fraction_bits();

            let real_product = a.w_mul(s);
            let imag_product = b.w_mul(s);

            if real_product.w_is_zero() && imag_product.w_is_zero() {
                return Self::ZERO;
            }

            let leading_r = real_product.leading_same();
            let leading_i = imag_product.leading_same();
            let leading = leading_r.min(leading_i);
            let shift = leading.wrapping_sub(1);
            let real = real_product.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_product.w_shl(shift).w_shr(fb).deflate();

            // Exp: stored_pos = pa + pb - expo_adjust - bo + 1 (same as Circle*Circle mul).
            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let expo_adjust_e: E = leading.wrapping_sub(2).as_();
            let w_adj = expo_adjust_e.sign_extend();
            let w_bo = Self::binade_origin().cycle_widen();
            let w_one = E::one().cycle_widen();
            let stored_pos = pa.w_add(pb).w_sub(w_adj).w_sub(w_bo).w_add(w_one);
            let max_pos = Self::max_exponent().cycle_widen();
            let min_pos = Self::min_exponent().cycle_widen();

            return if stored_pos > max_pos {
                Self {
                    real,
                    imaginary,
                    exponent: Self::ambiguous_exponent(),
                }
            } else if stored_pos < min_pos {
                Self {
                    real: real >> 1isize,
                    imaginary: imaginary >> 1isize,
                    exponent: Self::ambiguous_exponent(),
                }
            } else {
                Self {
                    real,
                    imaginary,
                    exponent: stored_pos.deflate(),
                }
            };
        }

        // Escape-class handling (at least one operand non-normal).
        {
            if self.is_undefined() {
                return *self;
            } else if other.is_undefined() {
                return Self {
                    real: other.fraction,
                    imaginary: other.fraction,
                    exponent: other.exponent,
                };
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
                let (real, imaginary) = match Self::fraction_bits() {
                    8 => {
                        let multiplier_r: i16 = self.real.as_();
                        let multiplier_i: i16 = self.imaginary.as_();
                        // N0→N1 conversion at the Scalar/Circle interaction: (s >> 1) ^ F::MIN.
                        let multiplicand: i16 =
                            ((other.fraction >> 1isize) ^ F::min_value()).as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    16 => {
                        let multiplier_r: i32 = self.real.as_();
                        let multiplier_i: i32 = self.imaginary.as_();
                        let multiplicand: i32 =
                            ((other.fraction >> 1isize) ^ F::min_value()).as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    32 => {
                        let multiplier_r: i64 = self.real.as_();
                        let multiplier_i: i64 = self.imaginary.as_();
                        let multiplicand: i64 =
                            ((other.fraction >> 1isize) ^ F::min_value()).as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    64 => {
                        let multiplier_r: i128 = self.real.as_();
                        let multiplier_i: i128 = self.imaginary.as_();
                        let multiplicand: i128 =
                            ((other.fraction >> 1isize) ^ F::min_value()).as_();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> Self::fraction_bits()).as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_(),
                        )
                    }
                    128 => {
                        let multiplier_r: I256 = self.real.into();
                        let multiplier_i: I256 = self.imaginary.into();
                        let multiplicand: I256 =
                            ((other.fraction >> 1isize) ^ F::min_value()).into();
                        let product_wide_r = multiplier_r.wrapping_mul(multiplicand);
                        let product_wide_i = multiplier_i.wrapping_mul(multiplicand);

                        let shift_r = product_wide_r
                            .leading_ones()
                            .max(product_wide_r.leading_zeros())
                            as isize;
                        let shift_i = product_wide_i
                            .leading_ones()
                            .max(product_wide_i.leading_zeros())
                            as isize;
                        let shift = shift_r.min(shift_i);

                        let shift_amount = shift.wrapping_add(n_level);
                        let normalized_wide_r = product_wide_r << shift_amount;
                        let normalized_wide_i = product_wide_i << shift_amount;
                        (
                            (normalized_wide_r >> Self::fraction_bits()).as_i128().as_(),
                            (normalized_wide_i >> Self::fraction_bits()).as_i128().as_(),
                        )
                    }
                    _ => (GENERAL.prefix.sa(), GENERAL.prefix.sa()),
                };

                return Self {
                    real,
                    imaginary,
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
    }
}
