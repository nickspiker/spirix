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
    /// Mathematical complex modulus operation for Scalar with Circle
    ///
    /// Performs complex modulus using the mathematical formula: For scalar s and complex number (a + bi), calculates: real = (s * a) % (a² + b²) imag = (s * b) % (a² + b²)
    ///
    /// Returns UNDEFINED if:
    /// - Either number is escaped
    /// - Both numbers are effectively zero
    /// - Denominator magnitude (a² + b²) is effectively zero
    pub(crate) fn scalar_modulus_circle(&self, denominator: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            // Mirrors scalar_modulus_scalar's rule order with Circle's magnitude treated as the period: 1) undefined, 2) zero anywhere, 3) transfinite numerator (distinguishes whether period is also transfinite), 4) vanished period (distinguishes whether numerator vanished), 5) infinite period, 6) exploded period (Circle magnitude is non-negative, so positive numerator → numerator; vanished+diff returns modulus), 7) vanished numerator with normal period. The earlier block returned INFINITY for infinite denominators (rule 5 violation) and copied self.fraction into real+imaginary as N1 (corrupted class — N0 magnitude bit lands at N1 vanished position).
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
                let prefix: F = if denominator.is_transfinite() {
                    TRANSFINITE_MODULUS_TRANSFINITE.prefix.sa()
                } else {
                    TRANSFINITE_MODULUS.prefix.sa()
                };
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if denominator.vanished() {
                let prefix: F = if self.vanished() {
                    VANISHED_MODULUS_VANISHED.prefix.sa()
                } else {
                    MODULUS_VANISHED.prefix.sa()
                };
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if denominator.is_infinite() {
                let prefix: F = MODULUS_TRANSFINITE.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Exploded Circle period: magnitude is non-negative so positive scalar (any class) keeps numerator; negative scalar diverges.
            if denominator.exploded() {
                if !self.is_negative() {
                    return Circle::<F, E>::from_ri(*self, Scalar::<F, E>::ZERO);
                }
                if self.vanished() {
                    return *denominator;
                }
                let prefix: F = MODULUS_TRANSFINITE.prefix.sa();
                return Circle::<F, E> {
                    real: prefix,
                    imaginary: prefix,
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Vanished numerator with normal Circle period (other classes already handled). |↓|<|#| so |↓| % positive_circle = ↓ for positive scalar; negative diverges to modulus.
            if self.vanished() {
                if !self.is_negative() {
                    return Circle::<F, E>::from_ri(*self, Scalar::<F, E>::ZERO);
                }
                return *denominator;
            }
            // Fallthrough — shouldn't reach.
            return Circle::<F, E>::from_ri(*self, Scalar::<F, E>::ZERO);
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
    /// Performs modulo separately on real and imaginary components: s ‰ (a + bi) = (s % a) + (s % b)i scalar.modulo(circle);
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
                    exponent: Self::ambiguous_exponent(),
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
                    exponent: Self::ambiguous_exponent(),
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
