use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
use core::{borrow::Borrow, ops::*};
use i256::I256;
use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    pub fn sin(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return Self {
                    fraction: SINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }
        if self.exponent > Self::fraction_bits().wrapping_sub(1).as_() {
            return Self {
                fraction: SINE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        let mut reduced = *self;

        if reduced >= Self::TAU {
            let quotient = reduced / Self::TAU;
            reduced = quotient.frac() * Self::TAU;
        } else if reduced <= Self::NEG_TAU {
            let quotient = reduced / Self::NEG_TAU;
            reduced = quotient.frac() * Self::NEG_TAU;
        }

        if reduced > Self::PI {
            reduced = Self::PI - reduced;
        } else if reduced < Self::NEG_PI {
            reduced = Self::NEG_PI - reduced;
        }

        if reduced >= Self::HALF_PI {
            reduced = Self::PI - reduced;
        } else if reduced <= Self::NEG_HALF_PI {
            reduced = Self::NEG_PI - reduced;
        }

        let x_squared = reduced.square();
        let mut sum = reduced;
        let mut prev_sum;
        let mut numerator = reduced;
        let mut denominator = Self::ONE;
        let mut sign = true;

        for i in (2..Self::fraction_bits()).step_by(2) {
            prev_sum = sum;
            numerator = numerator * x_squared;
            denominator = denominator * i.wrapping_mul(i.wrapping_add(1));
            let mut term = numerator / denominator;
            if sign {
                term.scalar_negate();
            }
            sum = sum + term;
            if sum == prev_sum {
                break;
            }
            sign = !sign;
        }

        sum
    }

    pub fn cos(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                return Self {
                    fraction: COSINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() {
                return Self::ONE;
            }
            return Self::EFFECTIVELY_POS_ONE;
        }

        if self.exponent > Self::fraction_bits().wrapping_sub(1).as_() {
            return Self {
                fraction: COSINE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }

        let mut reduced = *self;
        if reduced >= Self::TAU {
            let quotient = reduced / Self::TAU;
            reduced = quotient.frac() * Self::TAU;
        } else if reduced <= Self::NEG_TAU {
            let quotient = reduced / Self::NEG_TAU;
            reduced = quotient.frac() * Self::NEG_TAU;
        }

        if reduced > Self::PI {
            reduced = reduced - Self::TAU;
        } else if reduced < Self::NEG_PI {
            reduced = reduced + Self::TAU;
        }

        let mut sign_adjustment = false;

        if reduced >= Self::HALF_PI {
            reduced = Self::PI - reduced;
            sign_adjustment = true;
        } else if reduced <= Self::NEG_HALF_PI {
            reduced = Self::NEG_PI - reduced;
            sign_adjustment = true;
        }

        let x_squared = reduced.square();
        let mut sum = Self::ONE;
        let mut prev_sum;
        let mut numerator = Self::ONE;
        let mut denominator = Self::ONE;
        let mut term_sign = true;

        for i in (1..Self::fraction_bits()).step_by(2) {
            prev_sum = sum;
            numerator = numerator * x_squared;
            denominator = denominator * i.wrapping_mul(i.wrapping_add(1));
            let mut term = numerator / denominator;
            if term_sign {
                term.scalar_negate();
            }
            sum = sum + term;
            if sum == prev_sum {
                break;
            }
            term_sign = !term_sign;
        }

        if sign_adjustment {
            let mut result = sum;
            result.scalar_negate();
            return result;
        }

        sum
    }

    pub fn tan(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return Self {
                    fraction: TANGENT.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }
        if Self::exponent_bits() >= (core::mem::size_of::<isize>() as isize).wrapping_mul(8) {
            if self.exponent > Self::fraction_bits().as_() {
                return Self {
                    fraction: TANGENT.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        } else {
            let exponent_isize: isize = self.exponent.as_();
            if exponent_isize > Self::fraction_bits() {
                return Self {
                    fraction: TANGENT.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }
        self.sin() / self.cos()
    }

    pub fn asin(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return Self {
                    fraction: ARCSINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }

        if self.exponent.is_positive() {
            if self == 1 {
                return Self::HALF_PI;
            }
            return Self {
                fraction: ARCSINE.prefix.sa(),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if self == -1 {
            return Self::NEG_HALF_PI;
        }

        let three_quarters = Self::HALF + Self::HALF * Self::HALF;
        if self.magnitude() < three_quarters {
            let x_squared = self.square();
            let mut sum = self.clone();
            let mut term = self.clone();
            let mut n = 0;

            loop {
                let prev_sum = sum.clone();
                n = n.wrapping_add(&1);

                let numerator = Self::from(2isize.wrapping_mul(n).wrapping_sub(1));
                let denominator = Self::from(2isize.wrapping_mul(n));
                let coefficient = numerator / denominator;

                term = term * x_squared * coefficient;
                sum = sum + term / Self::from(2isize.wrapping_mul(n).wrapping_add(1));

                if sum == prev_sum || n > (Self::fraction_bits() >> 1) {
                    break;
                }
            }

            return sum;
        }

        let one_minus_abs_x = Self::ONE - self.magnitude();

        if one_minus_abs_x.is_negligible() {
            return Self::HALF_PI * self.sign();
        }

        let one_minus_abs_x_half = one_minus_abs_x >> 1i8;
        let sqrt_term = one_minus_abs_x_half.sqrt();

        let mut series_sum = sqrt_term.clone();
        let mut term = sqrt_term.clone();
        let term_squared = sqrt_term.square();
        let mut n = 0;

        loop {
            let prev_sum = series_sum.clone();
            n = n.wrapping_add(&1);

            let numerator = Self::from(2isize.wrapping_mul(n).wrapping_sub(1));
            let denominator = Self::from(2isize.wrapping_mul(n));
            let coefficient = numerator / denominator;

            term = term * term_squared * coefficient;
            series_sum = series_sum + term / Self::from(2isize.wrapping_mul(n).wrapping_add(1));

            if series_sum == prev_sum || n > (Self::fraction_bits() >> 1) {
                break;
            }
        }

        if self.fraction.is_negative() {
            return (series_sum << 1) - Self::HALF_PI;
        } else {
            return Self::HALF_PI - (series_sum << 1);
        }
    }
    pub fn acos(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                return Self {
                    fraction: ARCCOSINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self::HALF_PI;
        }

        if self.exponent.is_positive() {
            if self == 1 {
                return Self::ZERO;
            } else {
                return Self {
                    fraction: ARCCOSINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
        }

        if self == -1 {
            return Self::PI;
        }

        if self.exponent.is_negative() {
            return Self::HALF_PI - self.asin();
        }

        let one_minus_abs_x = Self::ONE - self.magnitude();

        if one_minus_abs_x.is_negligible() {
            if self.fraction.is_negative() {
                return Self::PI;
            } else {
                return Self::ZERO;
            }
        }

        let one_minus_abs_x_half = one_minus_abs_x >> 1i8;
        let sqrt_term = one_minus_abs_x_half.sqrt();

        let mut series_sum = sqrt_term.clone();
        let mut term = sqrt_term.clone();
        let term_squared = sqrt_term.square();
        let mut n = 0;

        loop {
            let prev_sum = series_sum.clone();
            n = n.wrapping_add(&1);

            let numerator = Self::from(2isize.wrapping_mul(n).wrapping_sub(1));
            let denominator = Self::from(2isize.wrapping_mul(n));
            let coefficient = numerator / denominator;

            term = term * term_squared * coefficient;
            series_sum = series_sum + term / Self::from(2isize.wrapping_mul(n).wrapping_add(1));

            if series_sum == prev_sum || n > (Self::fraction_bits() >> 1) {
                break;
            }
        }

        if self.fraction.is_negative() {
            return Self::PI - (series_sum << 1);
        } else {
            return series_sum << 1;
        }
    }
    pub fn atan(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return if self.fraction.is_negative() {
                    Self::NEG_HALF_PI
                } else {
                    Self::HALF_PI
                };
            }
            return *self;
        }

        let x = *self;
        let magnitude = x.magnitude();
        let is_negative = x.is_negative();
        let sign = self.sign();

        if magnitude < Self::HALF {
            return x.atan_small();
        } else if magnitude > 2 {
            return sign * Self::HALF_PI - x.reciprocal().atan_small();
        } else {
            let pi_4 = Self::FOURTH_PI;

            if is_negative {
                let term = (x + Self::ONE) / (Self::ONE - x);
                return -pi_4 + term.atan_small();
            } else {
                let term = (x - Self::ONE) / (x + Self::ONE);
                return pi_4 + term.atan_small();
            }
        }
    }

    fn atan_small(&self) -> Self {
        let x_squared = self.square();

        let mut previous = Self::ZERO;
        let mut result;

        for iterations in 1..Self::fraction_bits() {
            result = Self::from(2isize.wrapping_mul(iterations).wrapping_sub(1));

            for k in (1..iterations).rev() {
                let denom = 2isize.wrapping_mul(k).wrapping_sub(1);
                result = denom + x_squared / result;
            }

            if previous == result {
                break;
            }

            previous = result;
        }

        return self / previous;
    }
    pub fn atan2<S>(&self, x: S) -> Self
    where
        S: Borrow<Self>,
    {
        let y = *self;
        let x = *x.borrow();

        if x.is_negligible() {
            if y.is_negligible() {
                return Self::ZERO;
            } else if y.is_positive() {
                return Self::HALF_PI;
            } else {
                return Self::NEG_HALF_PI;
            }
        }

        if y.is_negligible() {
            if x.is_positive() {
                return Self::ZERO;
            } else {
                return Self::PI;
            }
        }

        // Calculate based on quadrant
        if x.is_positive() {
            // Quadrants I and IV
            return (y / x).atan();
        } else if y.is_positive() {
            // Quadrant II
            return (y / x).atan() + Self::PI;
        } else {
            // Quadrant III
            return (y / x).atan() - Self::PI;
        }
    }
    /// Computes the hyperbolic sine of a scalar.
    ///
    /// Uses the identity: sinh(x) = (e^x - e^(-x))/2
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return undefined states
    ///
    /// # Returns:
    /// - The hyperbolic sine of the input
    pub fn sinh(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return Self {
                    fraction: SINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }

        // Calculate using the exponential definition: sinh(x) = (e^x - e^(-x))/2
        (self.exp() - (-self).exp()) >> 1
    }

    /// Computes the hyperbolic cosine of a scalar.
    ///
    /// Uses the identity: cosh(x) = (e^x + e^(-x))/2
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return undefined states
    ///
    /// # Returns:
    /// - The hyperbolic cosine of the input
    pub fn cosh(&self) -> Self {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }

            if self.exploded() {
                return Self {
                    fraction: COSINE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self::ONE;
        }

        (self.exp() + (-self).exp()) >> 1
    }
    /// Computes the hyperbolic tangent of a scalar.
    ///
    /// Uses the identity: tanh(x) = sinh(x) / cosh(x)
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return undefined states
    ///
    /// # Returns:
    /// - The hyperbolic tangent of the input
    pub fn tanh(&self) -> Self {
        if !self.is_normal() {
            if self.exploded() {
                return Self {
                    fraction: TANGENT.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return *self;
        }

        self.sinh() / self.cosh()
    }
    // pub fn erf(&self) -> Self {
    //     // Handle special cases first
    //     if !self.is_normal() {
    //         if self.exploded() {
    //             return Self {
    //                 fraction: GENERAL.prefix.sa(),
    //                 exponent: Self::ambiguous_exponent(),
    //             };
    //         }
    //         // Zero returns zero
    //         if self.is_zero() {
    //             return Self::ZERO;
    //         }
    //         return *self;
    //     }

    //     return self.erf_continued_fraction();
    // }

    // // Taylor series implementation for small |x|
    // fn erf_taylor_series(&self) -> Self {
    //     let x = *self;
    //     let x_squared = x.square();
    //     let two_over_sqrt_pi = Self::TWO / Self::PI.sqrt();

    //     let mut sum = x;
    //     let mut prev_sum;
    //     let mut term = x;
    //     let mut n = Self::ZERO;

    //     loop {
    //         prev_sum = sum;
    //         n = n + Self::ONE;

    //         // Term: (-1)^n * x^(2n+1) / (n! * (2n+1))
    //         term = term * x_squared * (-Self::ONE);
    //         term = term / (n * (n + n + Self::ONE));

    //         sum = sum + term;

    //         if sum == prev_sum || n > Self::fraction_bits() {
    //             break;
    //         }
    //     }

    //     two_over_sqrt_pi * sum
    // }

    // // Continued fraction implementation for medium |x|
    // fn erf_continued_fraction(&self) -> Self {
    //     let x = *self;
    //     let two_x_over_sqrt_pi = (Self::TWO * x) / Self::PI.sqrt();
    //     let x_squared = x.square();

    //     // Compute using Lentz's algorithm
    //     let tiny = Self::from(1e-20);
    //     let mut f = Self::ONE;
    //     let mut c = f;
    //     let mut d = Self::ZERO;

    //     let mut j = Self::ONE;
    //     let mut delta;

    //     // Initialize values for continued fraction expansion
    //     // 1 + 2x²/(3 + 4x²/(5 + 6x²/(7 + ...)))
    //     loop {
    //         let a = j + j - Self::ONE;
    //         let b = (j + j) * x_squared;

    //         d = a + b * d;
    //         if d.is_negligible() {
    //             d = tiny;
    //         }

    //         c = a + b / c;
    //         if c.is_negligible() {
    //             c = tiny;
    //         }

    //         d = Self::ONE / d;
    //         delta = c * d;
    //         f = f * delta;

    //         // Check for convergence
    //         if (delta - Self::ONE).magnitude() < Self::POS_NORMAL_EPSILON
    //             || j > (Self::fraction_bits() >> 1)
    //         {
    //             break;
    //         }

    //         j = j + Self::ONE;
    //     }

    //     two_x_over_sqrt_pi / f
    // }

    // // Asymptotic approximation for large |x|
    // fn erf_asymptotic(&self) -> Self {
    //     let x = *self;
    //     let abs_x = x.magnitude();
    //     let sign = x.sign();

    //     // For large x: erf(x) ≈ 1 - e^(-x²)/(x√π) * (1 - 1/(2x²) + ...)
    //     let x_squared = x.square();
    //     let exp_neg_x_squared = (-x_squared).exp();
    //     let inv_x_sqrt_pi = Self::ONE / (abs_x * Self::PI.sqrt());

    //     let correction = exp_neg_x_squared
    //         * inv_x_sqrt_pi
    //         * (Self::ONE - Self::ONE / (Self::TWO * x_squared) + 3 / (4 * x_squared.square()));

    //     if sign.is_positive() {
    //         Self::ONE - correction
    //     } else {
    //         -Self::ONE + correction
    //     }
    // }
}

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
#[allow(dead_code)]
fn _printey<T: core::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = core::mem::size_of::<T>() * 8;
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits - 1 && b % 8 == 7 {
            result.push(' ');
        }
        if b == bits / 2 - 1 {
            result.push(' '); // Extra space at center
        }
    }
    result
}
