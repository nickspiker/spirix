use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

use crate::{
    core::integer::{FullInt, IntConvert},
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar,
    ScalarConstants,
};

use core::cmp::Ordering;
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
            + WrappingSub
            + WrappingMul,
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
            + WrappingSub
            + WrappingMul,
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
    /// Compares two Circle values for exact equality.
    ///
    /// # Description
    ///
    /// This method determines if two Circle values represent exactly the same complex number.
    /// Unlike Scalar comparison which provides ordering, Circle comparison only tests for equality since complex numbers do not have a natural total ordering relationship.
    ///
    /// For Circle values to be considered equal, they must have identical representations in both their real and imaginary components, as well as matching exponents when normal.
    ///
    /// # Equality Conditions
    ///
    /// Two Circle values are equal if and only if:
    /// - Both are normal `[#]` with identical exponents, real fractions, and imaginary fractions
    /// - Both are Zero `[0]` (regardless of the specific Zero representation)
    ///
    /// # Non-Equal Cases
    ///
    /// Returns `false` for:
    /// - Different normal values (different exponents or fraction components)
    /// - Any escaped states `[↑]`, `[↓]`, `[∞]`, `[℘?]` compared with anything except identical Zero
    /// - Mixed normal and non-normal states
    /// - Different undefined states `[℘?]`
    /// - Exploded or vanished values (even with same escape angle)
    /// - Infinity compared with anything except identical Infinity
    ///
    /// # Mathematical Rationale
    ///
    /// Since escaped Circle values lose magnitude information while retaining only orientation/angle data, they cannot be meaningfully compared for equality even when they share the same escape direction. Only normal values with complete magnitude and phase information support exact equality testing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Circle, CircleF5E3};
    ///
    /// // Normal Circle equality
    /// let z1 = CircleF5E3::from((3, 4)); // 3 + 4i
    /// let z2 = CircleF5E3::from((3, 4)); // 3 + 4i
    /// let z3 = CircleF5E3::from((4, 3)); // 4 + 3i
    /// assert!(z1 == z2); // Same values
    /// assert!(z1 != z3); // Different values
    ///
    /// // Zero equality
    /// let zero = CircleF5E3::from((0, 0));
    /// assert!(zero == 0);// Both represent zero
    ///
    /// // Vanished isn't Zero
    /// let vanished = CircleF5E3::MIN_POS.square();
    /// assert!(zero != vanished);// Does not truncate to Zero!
    ///
    /// // Vanished are not comparable
    /// let same_vanished = CircleF5E3::MIN_POS.square();
    /// assert!(same_vanished != vanished);
    ///
    /// // Mixed normal/non-normal
    /// let normal = CircleF5E3::from((1, 1));
    /// let exploded = CircleF5E3::from((CircleF5E3::MAX, 0)) * 2;
    /// assert!(normal != exploded);
    ///
    /// // Escaped values don't equal each other
    /// let exploded1 = CircleF5E3::MAX * 2;
    /// let exploded2 = CircleF5E3::MAX * 2;// Same escape value
    /// assert!(exploded1 != exploded2); // Escaped values non-equal
    ///
    /// // Undefined values don't equal anything
    /// let undefined = CircleF5E3::ZERO / 0;
    /// assert!(!undefined.equals(&normal));
    /// assert!(!undefined.equals(&undefined)); // Not even themselves
    /// ```
    pub(crate) fn equals(&self, other: &Circle<F, E>) -> bool {
        if self.is_normal() && other.is_normal() {
            self.exponent == other.exponent
                && self.real == other.real
                && self.imaginary == other.imaginary
        } else {
            if self.is_zero() && other.is_zero() {
                return true;
            }
            return false;
        }
    }
}
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
            + WrappingSub
            + WrappingMul,
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
            + WrappingSub
            + WrappingMul,
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
    /// Compares two Scalar values and returns their relative ordering, if determinable.
    ///
    /// # Description
    ///
    /// This method implements a total ordering for Scalar values that can be compared, while returning `None` for values that have no defined ordering relationship.
    /// The comparison follows Spirix's mathematical ordering principles where values are arranged in a continuous spectrum from negative exploded to positive exploded.
    ///
    /// # Ordering Hierarchy
    ///
    /// When both values can be compared, they follow this ordering:
    /// ```text
    /// [-↑] < [-#] < [-↓] < [0] < [+↓] < [+#] < [+↑]
    /// ```
    ///
    /// # Returns
    ///
    /// - `Some(Ordering::Less)` - This Scalar is less than the other
    /// - `Some(Ordering::Equal)` - Both Scalars represent the same value
    /// - `Some(Ordering::Greater)` - This Scalar is greater than the other
    /// - `None` - The values cannot be meaningfully compared
    ///
    /// # Unordered Values
    ///
    /// Returns `None` for comparisons involving:
    /// - Any undefined state `[℘?]`
    /// - Mathematical infinity `[∞]`
    /// - Same-sign escaped values of the same type:
    ///   - Two positive exploded values `[+↑]` with `[+↑]`
    ///   - Two negative exploded values `[-↑]` with `[-↑]`
    ///   - Two positive vanished values `[+↓]` with `[+↓]`
    ///   - Two negative vanished values `[-↓]` with `[-↓]`
    ///
    /// # Implementation Details
    ///
    /// For normal values, comparison is performed by:
    /// 0. Sign comparison (negative < positive)
    /// 1. Exponent comparison (larger absolute exponents for same sign)
    /// 2. Fraction comparison when exponents are equal
    ///
    /// For escaped values, only cross-sign comparisons are deterministic, as same-sign escaped values have unknown relative magnitudes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF5E3};
    /// use core::cmp::Ordering;
    ///
    /// let a = ScalarF5E3::from(42);
    /// let b = ScalarF5E3::from(-17);
    /// let zero = ScalarF5E3::ZERO;
    ///
    /// // Normal value comparisons
    /// assert_eq!(a.compare(&b), Some(Ordering::Greater));
    /// assert_eq!(b.compare(&zero), Some(Ordering::Less));
    /// assert_eq!(a.compare(&a), Some(Ordering::Equal));
    ///
    /// // Escaped value comparisons
    /// let pos_exploded = ScalarF5E3::MAX * 2;
    /// let neg_exploded = ScalarF5E3::MIN * 2;
    /// assert_eq!(pos_exploded.compare(&neg_exploded), Some(Ordering::Greater));
    ///
    /// // Unordered comparisons
    /// let infinity = ScalarF5E3::ONE / 0;
    /// let undefined = ScalarF5E3::ZERO / 0;
    /// assert_eq!(a.compare(&infinity), None);
    /// assert_eq!(a.compare(&undefined), None);
    ///
    /// // Same-sign escaped values are unordered
    /// let another_pos_exploded = ScalarF5E3::MAX * 3;
    /// assert_eq!(pos_exploded.compare(&another_pos_exploded), None);
    /// ```
    pub(crate) fn compare(&self, other: &Scalar<F, E>) -> Option<Ordering> {
        if self.is_normal() && other.is_normal() {
            if self.exponent != other.exponent {
                let cmp = self.exponent.cmp(&other.exponent);
                return Some(if self.is_negative() {
                    cmp.reverse()
                } else {
                    cmp
                });
            }
            return Some(self.fraction.cmp_unsigned(&other.fraction));
        }
        if self.is_undefined() || self.is_infinite() || other.is_undefined() || other.is_infinite()
        {
            return None;
        }
        if self.is_zero() && other.is_zero() {
            return Some(Ordering::Equal);
        }
        if self.is_zero() {
            return Some(if other.is_positive() {
                Ordering::Less
            } else {
                Ordering::Greater
            });
        }
        if other.is_zero() {
            return Some(if self.is_positive() {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if (self.exploded() && other.exploded()) || (self.vanished() && other.vanished()) {
            if self.is_negative() == other.is_negative() {
                return None;
            }
        }
        if self.is_positive() != other.is_positive() {
            return Some(if self.is_positive() {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        if self.exploded() || other.vanished() {
            return Some(if self.is_positive() {
                Ordering::Greater
            } else {
                Ordering::Less
            });
        }
        Some(if self.is_positive() {
            Ordering::Less
        } else {
            Ordering::Greater
        })
    }
}
