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
    /// Mathematical modulus operation for Circle complex numbers
    ///
    /// Performs complex modulus using the mathematical formula derived from complex division: (a + bi) % (c + di) = ((ac + bd) + (bc - ad)i) % (c² + d²)
    ///
    /// Note: This is NOT simply applying the magnitude-based scalar modulus. For magnitude-based modulus, use cincle % scalar Instead, it's based on the principles of complex division followed by modulus:
    /// - Complex division gives us: (a + bi)/(c + di) = (ac + bd)/(c² + d²) + (bc - ad)/(c² + d²)i
    /// - For modulus, we extract the remainder after division
    /// - This approach preserves the algebraic properties expected of modular arithmetic in the complex plane
    ///
    /// Returns UNDEFINED if either Circle escaped or both are negligible. Handles special cases for pure real and pure imaginary Circles.
    pub(crate) fn circle_modulus_circle(&self, denominator: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return *self;
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
                    exponent: Self::ambiguous_exponent(),
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
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }

        let magnitude_squared = denominator
            .r()
            .square()
            .scalar_add_scalar(&denominator.i().square());

        // (ac + bd) % (c² + d²)
        let real_part = self
            .r()
            .scalar_multiply_scalar(&denominator.r())
            .scalar_add_scalar(&self.i().scalar_multiply_scalar(&denominator.i()))
            .scalar_modulus_scalar(&magnitude_squared);

        // (bc - ad) % (c² + d²)
        let imag_part = self
            .i()
            .scalar_multiply_scalar(&denominator.r())
            .scalar_subtract_scalar(&self.r().scalar_multiply_scalar(&denominator.i()))
            .scalar_modulus_scalar(&magnitude_squared);

        Circle::<F, E>::from_ri(real_part, imag_part)
    }

    /// Component-wise modulo operation for Circle complex numbers
    ///
    /// Performs modulo separately on real and imaginary components: (a + bi) ‰ (c + di) = (a % c) + (b % d)i circle.modulo(circle);
    ///
    /// Returns UNDEFINED if either Circle escaped or both are negligible. Handles ambiguous cases for pure real and pure imaginary numbers.
    ///
    /// This operation is useful for coordinate-based or grid-aligned calculations.
    pub(crate) fn circle_modulo_circle(&self, denominator: &Circle<F, E>) -> Circle<F, E> {
        if !self.is_normal() || !denominator.is_normal() {
            if self.is_undefined() {
                return *self;
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
                    exponent: Self::ambiguous_exponent(),
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
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }

        Circle::<F, E>::from_ri(
            self.r().scalar_modulus_scalar(&denominator.r()),
            self.i().scalar_modulus_scalar(&denominator.i()),
        )
    }
}
