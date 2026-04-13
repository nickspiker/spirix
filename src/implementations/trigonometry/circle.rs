use crate::core::integer::{Inflate, FullInt, IntConvert};
use crate::core::undefined::*;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    pub fn sin(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = SINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self; // covers undefined, vanished and Zero
        }

        Circle::from((
            self.r().sin() * self.i().cosh(),
            self.r().cos() * self.i().sinh(),
        ))
    }
    pub fn cos(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                let prefix: F = COSINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            if self.is_zero() {
                return Self::ONE;
            }
            return Self::EFFECTIVELY_POS_ONE;
        }

        Circle::from((
            self.r().cos() * self.i().cosh(),
            -self.r().sin() * self.i().sinh(),
        ))
    }

    /// Computes the tangent of a complex number.
    ///
    /// Uses the identity: tan(z) = sin(z) / cos(z)
    ///
    /// # Special Cases:
    /// - Undefined or escaped values return undefined states
    /// - Values where cos(z) = 0 return the TANGENT undefined state
    ///
    /// # Returns:
    /// - The complex tangent of the input
    pub fn tan(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = TANGENT.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self; // covers undefined, vanished and Zero
        }

        // Compute sin(z) and cos(z)
        let sin_z = self.sin();
        let cos_z = self.cos();

        // Check if cos(z) approaches zero
        if cos_z.magnitude().is_negligible() {
            let prefix: F = TANGENT.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Calculate tan(z) = sin(z) / cos(z)
        sin_z / cos_z
    }

    pub fn sinh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = SINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        Circle::from((
            self.r().sinh() * self.i().cos(),
            self.r().cosh() * self.i().sin(),
        ))
    }

    pub fn cosh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                let prefix: F = COSINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Self::ONE;
        }

        Circle::from((
            self.r().cosh() * self.i().cos(),
            self.r().sinh() * self.i().sin(),
        ))
    }

    pub fn tanh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = TANGENT.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        let sinh_z = self.sinh();
        let cosh_z = self.cosh();

        // Check if cosh(z) approaches zero
        if cosh_z.is_negligible() {
            let prefix: F = TANGENT.prefix.sa();
            return Self {
                real: prefix,
                imaginary: prefix,
                exponent: E::AMBIGUOUS_EXPONENT,
            };
        }

        // Calculate tanh(z) = sinh(z) / cosh(z)
        sinh_z / cosh_z
    }

    pub fn asin(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = SINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        // Use identity: asin(z) = -i * ln(iz + sqrt(1 - z²))
        let iz = Self::POS_I * *self;
        let one_minus_z_squared = Self::ONE - self.square();
        let sqrt_term = one_minus_z_squared.sqrt();
        let sum = iz + sqrt_term;

        Self::NEG_I * sum.ln()
    }

    // limits greater than magnitude 1 need checked
    pub fn acos(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                let prefix: F = ARCCOSINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Self::PI_OVER_TWO;
        }

        // Use identity: acos(z) = -i * ln(z + i * sqrt(1 - z²))
        let one_minus_z_squared = Self::ONE - self.square();
        let i_sqrt = Self::POS_I * one_minus_z_squared.sqrt();
        let sum = *self + i_sqrt;

        Self::NEG_I * sum.ln()
    }

    // exploded limits need fixed
    pub fn atan(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = TANGENT.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        // Use identity: atan(z) = (i/2) * ln((i+z)/(i-z))
        let i_plus_z = Self::POS_I + *self;
        let i_minus_z = Self::POS_I - *self;

        let quotient = i_plus_z / i_minus_z;
        let ln_result = quotient.ln();

        (Self::POS_I * Scalar::<F, E>::HALF) * ln_result
    }

    pub fn asinh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = ARCSINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        // Use identity: asinh(z) = ln(z + sqrt(z² + 1))
        let z_squared_plus_one = self.square() + 1i8;
        let sqrt_term = z_squared_plus_one.sqrt();
        let sum = *self + sqrt_term;

        sum.ln()
    }

    pub fn acosh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if self.exploded() {
                let prefix: F = ARCCOSINE.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return Self::POS_I * Self::PI_OVER_TWO;
        }

        // Use identity: acosh(z) = ln(z + sqrt(z² - 1))
        let z_squared_minus_one = self.square() - Self::ONE;
        let sqrt_term = z_squared_minus_one.sqrt();
        let sum = *self + sqrt_term;

        sum.ln()
    }

    pub fn atanh(&self) -> Circle<F, E> {
        if !self.is_normal() {
            if self.exploded() {
                let prefix: F = TANGENT.prefix.sa();
                return Self {
                    real: prefix,
                    imaginary: prefix,
                    exponent: E::AMBIGUOUS_EXPONENT,
                };
            }
            return *self;
        }

        // Use identity: atanh(z) = (1/2) * ln((1+z)/(1-z))
        let one_plus_z = Self::ONE + *self;
        let one_minus_z = Self::ONE - *self;

        let quotient = one_plus_z / one_minus_z;
        let ln_result = quotient.ln();

        Scalar::<F, E>::HALF * ln_result
    }
}
