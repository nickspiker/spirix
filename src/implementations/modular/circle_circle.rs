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
            // Rules mirror scalar_modulus_scalar: 1) undefined wins; 2) zero anywhere → zero; 3) transfinite numerator; 4) vanished period; 5) infinite period; 6) exploded period (Circle magnitude is positive, so positive-numerator → numerator; otherwise undefined). The earlier escape block returned *self for both infinite and exploded periods, contradicting the Scalar truth table.
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
            // Exploded denominator with normal/vanished numerator: Circle's magnitude_squared is non-negative, so the "period" is effectively positive. ac+bd may be any sign, so we conservatively return the numerator (matches truth-table same-sign case for typical positive components). Vanished numerator with normal denominator: |↓|<|#| so return self.
            return *self;
        }

        // Factor both shared exponents out. With a = A·2^ea and c = C·2^ec (fraction pairs at the +1.0 binade), the cross products are X·2^(ea+ec) and the period is P·2^(2ec); building them at the stored exponents exploded for |den| ≥ 2^64 and vanished below 2^-64 at E3, long before the result did.
        // Floored modulus is exact and scale-free, so (X·2^(ea+ec)) % (P·2^(2ec)) = ((X·2^α) % (P·2^β))·2^T for any split with α - β = ea - ec. Splitting the difference evenly keeps both reduced operands in range whenever the period itself could be, and every rounding step is scale-invariant, so results the old path computed without escaping are bit-identical.
        let numerator_unit = self.at_unit_binade();
        let denominator_unit = denominator.at_unit_binade();
        let (a, b) = (numerator_unit.r(), numerator_unit.i());
        let (c, d) = (denominator_unit.r(), denominator_unit.i());
        let numerator_offset = self.binade_offset();
        let denominator_offset = denominator.binade_offset();
        let difference = numerator_offset.w_sub(denominator_offset);
        // β = -floor(D/2) and α = ceil(D/2), each within ±(E::MAX) for any pair of normal exponents.
        let beta_wide = difference.w_shr(1).w_neg();
        let alpha_wide = difference.w_add(beta_wide);
        let alpha: E = alpha_wide.deflate();
        let beta: E = beta_wide.deflate();

        // (c² + d²)·2^β
        let period = c
            .square()
            .scalar_add_scalar(&d.square())
            .scalar_shl_integer(&beta);

        // (ac + bd)·2^α % period
        let real_part = a
            .scalar_multiply_scalar(&c)
            .scalar_add_scalar(&b.scalar_multiply_scalar(&d))
            .scalar_shl_integer(&alpha)
            .scalar_modulus_scalar(&period);

        // (bc - ad)·2^α % period
        let imag_part = b
            .scalar_multiply_scalar(&c)
            .scalar_subtract_scalar(&a.scalar_multiply_scalar(&d))
            .scalar_shl_integer(&alpha)
            .scalar_modulus_scalar(&period);

        let reduced = Circle::<F, E>::from_ri(real_part, imag_part);
        if !reduced.is_normal() {
            return reduced;
        }
        // Add T = 2·ec - β back with widened checks; escapes keep the phase like Circle multiply.
        let restore = denominator_offset
            .w_add(denominator_offset)
            .w_sub(beta_wide);
        reduced.circle_shift_settle(reduced.exponent.cycle_widen().w_add(restore))
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
