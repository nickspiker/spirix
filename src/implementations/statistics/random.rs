use crate::core::integer::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

trait RandomFraction {
    fn random() -> Self;
}

impl RandomFraction for i8 {
    fn random() -> Self {
        rand::random()
    }
}

impl RandomFraction for i16 {
    fn random() -> Self {
        rand::random()
    }
}

impl RandomFraction for i32 {
    fn random() -> Self {
        rand::random()
    }
}

impl RandomFraction for i64 {
    fn random() -> Self {
        rand::random()
    }
}

impl RandomFraction for i128 {
    fn random() -> Self {
        rand::random()
    }
}

#[allow(private_bounds)]
impl<
        F: Integer
            + FullInt
            + RandomFraction
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
    /// Uniform random Scalar in [-1, +1). Each value-range half-binade gets probability proportional to its width — i.e. P(|x| ∈ [2⁻ᵏ⁻¹, 2⁻ᵏ)) = 2⁻ᵏ⁻¹, matching `f32::random()`-style semantics on a real-valued sampler.
    ///
    /// Algorithm: fraction is uniform random bits (sign self-emerges from the N0 MSB); the exponent comes from a chained leading-zeros count L over fraction-sized random words. stored_exp = MAX_VAL − L, so most draws (small L) land in the upper binades close to ±1 and the geometric tail trails toward zero. Chain step probability is 2⁻ᶠᴿᴬᶜ (1/256 for FRAC=8) so the loop almost always exits on the first lead-word draw. If L ever exceeds MAX_VAL the value is below MIN_NORMAL → returns a canonical N2 vanished pattern with random low bits.
    #[inline]
    pub fn random() -> Self {
        let fraction: F = F::random();
        let frac_bits: usize = Self::fraction_bits() as usize;

        let mut leading: usize = 0;
        loop {
            let w: F = F::random();
            let lz = w.leading_zeros() as usize;
            leading = leading.wrapping_add(lz);
            if lz < frac_bits {
                break;
            }
        }

        let max_val: isize = E::max_value().saturate();
        if (leading as isize) >= max_val {
            // Vanished tail. Canonical N2 fraction: top three bits `a a !a`, low FRAC-3 bits random. Reuse the fraction draw — its MSB picks the sign, the low bits feed the entropy. Spirix N0: MSB=1 (leading_zeros=0) ↔ positive value.
            let low_mask: F = (F::one() << frac_bits.wrapping_sub(3)).wrapping_sub(&F::one());
            let entropy: F = fraction & low_mask;
            let base = if fraction.leading_zeros() == 0 {
                Self::pos_one_vanished()
            } else {
                Self::neg_one_vanished()
            };
            return Self {
                fraction: base ^ entropy,
                exponent: Self::ambiguous_exponent(),
            };
        }

        let stored: E = (max_val.wrapping_sub(leading as isize)).as_();
        Self {
            fraction,
            exponent: stored,
        }
    }
    #[inline]
    pub fn random_gauss() -> Self {
        let mut s: Self;
        let mut u: Self;
        let mut v: Self;

        loop {
            u = Self::random();
            v = Self::random();

            s = u.square() + v.square();

            // Marsaglia: accept when s ∈ (0, 1). AMBIG=0: stored exp positive ↔ |s|<1; combined with non-zero check covers the open interval.
            if s.exponent.is_positive() && !s.is_zero() {
                break;
            }
        }
        if !s.is_normal() {
            let mut normal = Self::random();
            normal.normalize_vanished();
            return normal;
        }
        let multiplier: Self = -2 * s.ln() / s;
        if !multiplier.is_normal() {
            let mut normal = Self::random();
            normal.normalize_exploded();
            return normal;
        }
        multiplier.sqrt() * u
    }
}

#[allow(private_bounds)]
impl<
        F: Integer
            + FullInt
            + RandomFraction
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
    #[inline]
    pub fn random() -> Self {
        loop {
            // Start with random components and zero exponent
            let mut result = Self {
                real: F::random(),
                imaginary: F::random(),
                exponent: 0.as_(),
            };

            let leading_r: E = result
                .real
                .leading_ones()
                .max(result.real.leading_zeros())
                .as_();
            let leading_i: E = result
                .imaginary
                .leading_ones()
                .max(result.imaginary.leading_zeros())
                .as_();
            let leading = leading_r.min(leading_i);

            if leading > E::one() {
                // Needs normalization
                let new_exponent = result.exponent.wrapping_sub(&leading);
                let shift: isize = leading.as_();

                if new_exponent.is_negative() {
                    // Normal N1 value normalization
                    result.exponent = new_exponent.wrapping_add(&E::one());

                    // Shift both components while preserving their relationship
                    result.real = result.real << shift.wrapping_sub(1);
                    result.imaginary = result.imaginary << shift.wrapping_sub(1);

                    // Fill lower bits with random values
                    let mask: F =
                        (Self::pos_one_normal() << shift.wrapping_sub(1)).wrapping_sub(&F::one());
                    let random_fill_r: F = F::random() & mask;
                    let random_fill_i: F = F::random() & mask;
                    result.real = result.real | random_fill_r;
                    result.imaginary = result.imaginary | random_fill_i;
                } else {
                    // Generate a vanished N2 value
                    result.exponent = Self::ambiguous_exponent();

                    // Keep generating random bits until we get valid N2 patterns for both components
                    loop {
                        result.real = F::random();
                        result.imaginary = F::random();

                        let leading_r: E = result
                            .real
                            .leading_ones()
                            .max(result.real.leading_zeros())
                            .as_();
                        let leading_i: E = result
                            .imaginary
                            .leading_ones()
                            .max(result.imaginary.leading_zeros())
                            .as_();

                        if leading_r.min(leading_i) == (E::one() + E::one()) {
                            break;
                        }
                    }
                }
            }

            // Check if we have a valid point within the unit circle
            if !result.magnitude_squared().exponent.is_positive() {
                return result;
            }
            // If outside unit circle, loop and try again
        }
    }
    pub fn random_gauss() -> Self {
        let mut s: Scalar<F, E>;
        let mut u: Scalar<F, E>;
        let mut v: Scalar<F, E>;

        loop {
            u = Scalar::<F, E>::random();
            v = Scalar::<F, E>::random();
            s = u.square() + v.square();
            if !s.exponent.is_positive() {
                break;
            }
        }

        if !s.is_normal() {
            let mut normal = Self::random();
            normal.normalize_vanished();
            return normal;
        }

        let multiplier: Scalar<F, E> = -2 * s.ln() / s;
        if !multiplier.is_normal() {
            let mut normal = Self::random();
            normal.normalize_exploded();
            return normal;
        }

        let factor = multiplier.sqrt();

        Circle::<F, E>::from((factor * u, factor * v))
    }
}
