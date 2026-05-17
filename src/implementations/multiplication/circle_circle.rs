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
        if self.is_normal() && other.is_normal() {
            // AMBIG=0 native unified pipeline. N1 inflation = sign_extend. Complex multiplication: (a + bi)(c + di) = (ac - bd) + (ad + bc)i. The >> 1 on each w_mul keeps the result in wide range (max |a|, |b|, |c|, |d| ~ 2^(FRAC-1), so a*c ~ 2^(2FRAC-2); after >>1 ~ 2^(2FRAC-3); sum/diff stays in wide).
            let a = self.real.sign_extend();
            let b = self.imaginary.sign_extend();
            let c = other.real.sign_extend();
            let d = other.imaginary.sign_extend();
            let real_product = a.w_mul(c).w_shr(1).w_sub(b.w_mul(d).w_shr(1));
            let imag_product = a.w_mul(d).w_shr(1).w_add(b.w_mul(c).w_shr(1));

            if real_product.w_is_zero() && imag_product.w_is_zero() {
                return Self::ZERO;
            }

            let leading_r = real_product.leading_same();
            let leading_i = imag_product.leading_same();
            let leading = leading_r.min(leading_i);

            // Normalize to canonical N1 (narrow leading = 1).
            let shift = leading.wrapping_sub(1);
            let fb = Self::fraction_bits();
            let real = real_product.w_shl(shift).w_shr(fb).deflate();
            let imaginary = imag_product.w_shl(shift).w_shr(fb).deflate();

            // AMBIG=0 native: stored_pos = pa + pb - expo_adjust - binade_origin + 1. expo_adjust = leading - 3 (= -2 for canonical N1 result), sign_extend so wide subtraction handles the negative correctly. cycle_widen bounds catch overflow/underflow that the bounded add can't.
            let pa = self.exponent.cycle_widen();
            let pb = other.exponent.cycle_widen();
            let expo_adjust_e: E = leading.wrapping_sub(3).as_();
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

        // Escape-class handling.
        if self.is_undefined() {
            return *self;
        }
        if other.is_undefined() {
            return *other;
        }
        if self.is_infinite() && other.is_zero() {
            return Self {
                real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.is_zero() && other.is_infinite() {
            return Self {
                real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.is_zero() || other.is_zero() {
            return Self::ZERO;
        }
        if self.exploded() && other.vanished() {
            return Self {
                real: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                imaginary: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self.vanished() && other.exploded() {
            return Self {
                real: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                imaginary: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        // Escape * (normal or escape with same class): compute the product directly, output at AMBIG with N1 (one operand exploded) or N2 (both vanished) shape.
        let n_level: isize = if self.exploded() || other.exploded() { -1 } else { -2 };
        let a = self.real.sign_extend();
        let b = self.imaginary.sign_extend();
        let c = other.real.sign_extend();
        let d = other.imaginary.sign_extend();
        let real_product = a.w_mul(c).w_shr(1).w_sub(b.w_mul(d).w_shr(1));
        let imag_product = a.w_mul(d).w_shr(1).w_add(b.w_mul(c).w_shr(1));

        let leading_r = real_product.leading_same();
        let leading_i = imag_product.leading_same();
        let leading = leading_r.min(leading_i);
        let shift = leading.wrapping_add(n_level);
        let fb = Self::fraction_bits();
        Self {
            real: real_product.w_shl(shift).w_shr(fb).deflate(),
            imaginary: imag_product.w_shl(shift).w_shr(fb).deflate(),
            exponent: Self::ambiguous_exponent(),
        }
    }
}
