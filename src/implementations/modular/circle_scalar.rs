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
    /// Mathematical modulus operation for Circle with Scalar
    ///
    /// Performs complex modulus using magnitude:
    /// (a + bi) % s = |a + bi| % s
    /// where |a + bi| = √(a² + b²)
    ///
    /// Returns UNDEFINED if:
    /// - Either number is escaped
    /// - Both numbers are effectively zero
    /// - Scalar denominator is effectively zero
    pub(crate) fn circle_modulus_scalar(&self, denominator: &Scalar<F, E>) -> Scalar<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return Scalar::<F, E> {
                    fraction: self.real,
                    exponent: self.exponent,
                };
            }
            if denominator.is_undefined() {
                return *denominator;
            }
            if self.is_zero() || denominator.is_zero() {
                return Scalar::<F, E>::ZERO;
            }
            if self.is_transfinite() {
                return Scalar::<F, E> {
                    fraction: TRANSFINITE_MODULUS.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if denominator.is_infinite() {
                return Scalar::<F, E>::INFINITY;
            }
            if denominator.vanished() {
                return Scalar::<F, E> {
                    fraction: MODULUS_VANISHED.prefix.sa(),
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Scalar::<F, E> {
                fraction: GENERAL.prefix.sa(),
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }
        // Calculate proper magnitude (sqrt of sum of squares) and take scalar %
        self.magnitude().scalar_modulus_scalar(denominator)
    }
    /// Component-wise modulo operation for Circle with Scalar
    ///
    /// Performs modulo of each component against the scalar value:
    /// (a + bi) ‰ s = (a % s) + (b % s)i
    ///
    /// Returns UNDEFINED if either number is escaped or both are effectively zero.
    /// Useful for grid-aligned or periodic calculations against a fixed value.
    pub(crate) fn circle_modulo_scalar(&self, denominator: &Scalar<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if denominator.is_undefined() {
                return Circle::<F, E> {
                    real: denominator.fraction,
                    imaginary: denominator.fraction,
                    exponent: denominator.exponent,
                };
            }
            if self.is_zero() || denominator.is_zero() {
                return Circle::<F, E>::ZERO;
            }
            if self.is_transfinite() {
                let prefix: F = TRANSFINITE_MODULUS.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if denominator.is_infinite() {
                return *self;
            }
            if denominator.vanished() {
                let prefix: F = MODULUS_VANISHED.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            let prefix: F = GENERAL.prefix.sa();
            return Circle::<F, E> {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        Circle::from_ri(
            self.r().scalar_modulus_scalar(denominator),
            self.i().scalar_modulus_scalar(denominator),
        )
    }
}
