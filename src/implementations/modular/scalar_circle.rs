use crate::core::integer::{FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    /// Mathematical complex modulus operation for Scalar with Circle
    ///
    /// Performs complex modulus using the mathematical formula:
    /// For scalar s and complex number (a + bi), calculates:
    /// real = (s * a) % (a² + b²)
    /// imag = (s * b) % (a² + b²)
    ///
    /// Returns UNDEFINED if:
    /// - Either number is escaped
    /// - Both numbers are effectively zero
    /// - Denominator magnitude (a² + b²) is effectively zero
    pub(crate) fn scalar_modulus_circle(&self, denominator: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return Circle::<F, E> {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if denominator.is_undefined() {
                return *denominator;
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
                return Circle::<F, E>::INFINITY;
            }
            if denominator.vanished() {
                let prefix: F = MODULUS_VANISHED.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Circle::<F, E> {
                real: self.fraction,
                imaginary: self.fraction,
                exponent: self.exponent,
            };
        }
        // Calculate magnitude squared (a² + b²)
        let magnitude_squared = denominator
            .r()
            .square()
            .scalar_add_scalar(&denominator.i().square());

        // Calculate real = (s * a) % (a² + b²)
        let real = self
            .scalar_multiply_scalar(&denominator.r())
            .scalar_modulus_scalar(&magnitude_squared);

        // Calculate imag = (s * b) % (a² + b²)
        let imag = self
            .scalar_multiply_scalar(&denominator.i())
            .scalar_modulus_scalar(&magnitude_squared);

        Circle::<F, E>::from_ri(real, imag)
    }
    /// Component-wise modulo operation for Scalar with Circle
    ///
    /// Performs modulo separately on real and imaginary components:
    /// s ‰ (a + bi) = (s % a) + (s % b)i
    /// scalar.modulo(circle);
    ///
    /// Returns UNDEFINED if:
    /// - Either number is escaped
    /// - Both numbers are effectively zero
    pub(crate) fn scalar_modulo_circle(&self, denominator: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return Circle::<F, E> {
                    real: self.fraction,
                    imaginary: self.fraction,
                    exponent: self.exponent,
                };
            }
            if denominator.is_undefined() {
                return *denominator;
            }
            if self.is_zero() || denominator.is_zero() {
                return Circle::<F, E>::ZERO;
            }
            if self.is_transfinite() {
                let prefix: F = TRANSFINITE_MODULO.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if denominator.is_infinite() {
                return Circle::<F, E>::INFINITY;
            }
            if denominator.vanished() {
                let prefix: F = MODULO_VANISHED.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Circle::<F, E> {
                real: self.fraction,
                imaginary: self.fraction,
                exponent: self.exponent,
            };
        }
        Circle::<F, E>::from_ri(
            self.scalar_modulus_scalar(&denominator.r()),
            self.scalar_modulus_scalar(&denominator.i()),
        )
    }
}
