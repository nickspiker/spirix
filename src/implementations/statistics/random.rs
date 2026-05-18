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

        // Compute max_val and the overflow check in E directly. The earlier isize round-trip silently saturated E::MAX for i128 exponents (isize is 64-bit) so stored landed at 2^63 instead of 2^127, putting every draw in a vanished-tail binade where Marsaglia never accepts.
        let max_val_e: E = E::max_value();
        let leading_e: E = leading.as_();
        if leading_e.into_unsigned() >= max_val_e.into_unsigned() {
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

        let stored: E = max_val_e.wrapping_sub(&leading_e);
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
    /// Uniform random Circle in the unit disk (|z| < 1). Geometric distribution over binades (matching Scalar::random's f32-style semantics), with rejection sampling for the unit-disk constraint that only activates at the top binade.
    ///
    /// Algorithm: chained leading-zeros over F-sized random words gives the geometric exp distribution; both real and imaginary get full random bits; `normalize()` canonicalizes the dominant component to N1 (which may pull the exp down further); reject if magnitude² ≥ 1.
    #[inline]
    pub fn random() -> Self {
        let frac_bits: usize = Self::fraction_bits() as usize;
        loop {
            // Geometric exponent via chained leading-zeros, same trick as Scalar::random. stored_exp = MAX_VAL - L lands near the top of the unit disk most often, geometric tail toward zero.
            let mut leading: usize = 0;
            loop {
                let w: F = F::random();
                let lz = w.leading_zeros() as usize;
                leading = leading.wrapping_add(lz);
                if lz < frac_bits {
                    break;
                }
            }

            // Same pattern as Scalar::random — keep max_val and the overflow check in E to avoid the isize-saturate truncation for i128 exponents.
            let max_val_e: E = E::max_value();
            let leading_e: E = leading.as_();
            if leading_e.into_unsigned() >= max_val_e.into_unsigned() {
                // Vanished tail — canonical N2 pattern with random low bits on both components.
                let fr: F = F::random();
                let fi: F = F::random();
                let low_mask: F = (F::one() << frac_bits.wrapping_sub(3)).wrapping_sub(&F::one());
                let base_r = if fr.leading_zeros() == 0 {
                    Self::neg_one_vanished()
                } else {
                    Self::pos_one_vanished()
                };
                let base_i = if fi.leading_zeros() == 0 {
                    Self::neg_one_vanished()
                } else {
                    Self::pos_one_vanished()
                };
                return Self {
                    real: base_r ^ (fr & low_mask),
                    imaginary: base_i ^ (fi & low_mask),
                    exponent: Self::ambiguous_exponent(),
                };
            }

            let stored: E = max_val_e.wrapping_sub(&leading_e);
            let mut result = Self {
                real: F::random(),
                imaginary: F::random(),
                exponent: stored,
            };
            // Canonicalize: dominant component must be N1. normalize() may shift the exp further down for non-canonical input fractions.
            result.normalize();

            // Unit disk check. Under AMBIG=0 the Scalar returned by magnitude_squared has is_positive iff |m²| < 1 (stored exp positive ↔ logical k < 0). Zero and vanished are trivially inside. Anything else (negative stored exp, exploded, etc.) is outside → retry.
            let m2 = result.magnitude_squared();
            if m2.exponent.is_positive() || m2.is_zero() || m2.vanished() {
                return result;
            }
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
            // Marsaglia: accept when s ∈ (0, 1). AMBIG=0 Scalar: stored exp positive ↔ |s|<1; combined with !is_zero covers the open interval.
            if s.exponent.is_positive() && !s.is_zero() {
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
