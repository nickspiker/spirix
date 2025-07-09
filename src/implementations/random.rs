use crate::core::integer::FullInt;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::AsPrimitive;
use std::ops::*;

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
            + FractionConstants
            + FullInt
            + RandomFraction
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
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
    #[inline]
    pub fn random() -> Self {
        let mut result = Self {
            fraction: F::random(),
            exponent: 0.as_(),
        };

        loop {
            let leading: E = result
                .fraction
                .leading_ones()
                .max(result.fraction.leading_zeros())
                .as_();

            if leading > E::one() {
                let new_exponent = result.exponent - leading;
                let shift: isize = leading.as_();

                if new_exponent.is_negative() {
                    result.exponent = new_exponent + 1.as_();
                    result.fraction = result.fraction << (shift - 1);

                    let mask: F = (F::one() << (shift - 1)) - F::one();

                    let random_fill: F = F::random() & mask;

                    result.fraction = result.fraction | random_fill;
                } else {
                    result.exponent = E::AMBIGUOUS_EXPONENT;
                    result.fraction = F::random();

                    let mut leading: E = result
                        .fraction
                        .leading_ones()
                        .max(result.fraction.leading_zeros())
                        .as_();
                    while leading != 2.as_() {
                        result.fraction = F::random();
                        leading = result
                            .fraction
                            .leading_ones()
                            .max(result.fraction.leading_zeros())
                            .as_();
                    }
                    break;
                }
            } else {
                break;
            }
        }
        result
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

            if !s.exponent.is_positive() {
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
            + FractionConstants
            + FullInt
            + RandomFraction
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>,
        E: Integer
            + ExponentConstants
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
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
                let new_exponent = result.exponent - leading;
                let shift: isize = leading.as_();

                if new_exponent.is_negative() {
                    // Normal N1 value normalization
                    result.exponent = new_exponent + 1.as_();

                    // Shift both components while preserving their relationship
                    result.real = result.real << (shift - 1);
                    result.imaginary = result.imaginary << (shift - 1);

                    // Fill lower bits with random values
                    let mask: F = (F::one() << (shift - 1)) - F::one();
                    let random_fill_r: F = F::random() & mask;
                    let random_fill_i: F = F::random() & mask;
                    result.real = result.real | random_fill_r;
                    result.imaginary = result.imaginary | random_fill_i;
                } else {
                    // Generate a vanished N2 value
                    result.exponent = E::AMBIGUOUS_EXPONENT;

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

                        if leading_r.min(leading_i) == 2.as_() {
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
