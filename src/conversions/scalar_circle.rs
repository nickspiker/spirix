use crate::constants::{CircleConstants, ScalarConstants};
use crate::core::integer::{Deflate, FullInt, Inflate, IntConvert, WideOps};
use crate::core::undefined::*;
use crate::{Circle, Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
/// # Scalar to Circle Conversions
///
/// This module provides implementations for creating Circle complex number values
/// from Scalar real number values. It handles combining separate real and imaginary
/// components while maintaining proper normalization and handling special cases.
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
    /// # Create a Circle from Components
    ///
    /// Constructs a Circle from separate real and imaginary Scalar components,
    /// handling complex normalization requirements and special state preservation.
    ///
    /// ## How Component Combination Works
    ///
    /// This method:
    /// 0. Aligns both components to share an exponent
    /// 1. Preserves special states (Zero, Infinity, exploded, vanished, undefined) appropriately
    /// 2. Creates a valid normalized Circle
    ///
    /// ## Special Case Handling
    ///
    /// | Components | Result |
    /// |------------|--------|
    /// | Either is undefined | First undefined encountered |
    /// | Infinity & defined | Infinite Circle |
    /// | Defined & Infinity | Infinite Circle |
    /// | Zero & any | Returns the non-zero component's state |
    /// | Any & Zero | Returns the non-zero component's state |
    /// | Exploded & Zero | Preserves the exploded state |
    /// | Zero & exploded | Preserves the exploded state |
    /// | Vanished & normal | Normal with vanished treated as zero |
    /// | Normal & vanished | Normal with vanished treated as zero |
    /// | Both exploded | Undefined result (magnitude indeterminate) |
    /// | Both vanished | Undefined result (magnitude indeterminate) |
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spirix::{Circle, Scalar, CircleF5E3, ScalarF5E3};
    ///
    /// // Normal components
    /// let real = ScalarF5E3::from(1.5);
    /// let imag = ScalarF5E3::from(2);
    /// let z = Circle::<i32, i8>::from_ri(real, imag);
    /// assert_eq!(z.magnitude(), 2.5);
    ///
    /// // Special case: Zero & Exploded
    /// let zero = ScalarF5E3::ZERO;
    /// let huge = ScalarF5E3::MAX * 3;  // Exploded value
    /// assert!(huge.exploded());
    ///
    /// let z = Circle::<i32, i8>::from_ri(zero, huge);
    /// assert!(z.exploded());  // Exploded state is preserved
    /// assert_eq!(z.r().is_zero());
    /// assert!(z.i() > 0);   // Sign is also preserved
    /// ```
    pub(crate) fn from_ri(real: Scalar<F, E>, imaginary: Scalar<F, E>) -> Self {
        if real.is_undefined() {
            return Circle {
                real: real.fraction,
                imaginary: real.fraction,
                exponent: real.exponent,
            };
        }
        if imaginary.is_undefined() {
            return Circle {
                real: imaginary.fraction,
                imaginary: imaginary.fraction,
                exponent: imaginary.exponent,
            };
        }
        if real.is_zero() && imaginary.is_zero() {
            return Circle::<F, E>::ZERO;
        }
        if real.is_zero() {
            let imag_c: F = imaginary
                .fraction
                .inflate(imaginary.is_normal())
                .w_shr(1isize)
                .deflate();
            return Circle {
                real: 0.as_(),
                imaginary: imag_c,
                exponent: imaginary.exponent,
            };
        }
        if imaginary.is_zero() {
            let real_c: F = real
                .fraction
                .inflate(real.is_normal())
                .w_shr(1isize)
                .deflate();
            return Circle {
                real: real_c,
                imaginary: 0.as_(),
                exponent: real.exponent,
            };
        }
        if real.is_infinite() || imaginary.is_infinite() {
            return Circle::<F, E>::INFINITY;
        }
        if real.vanished() && imaginary.vanished() || real.exploded() || imaginary.exploded() {
            let prefix: F = INDETERMINATE.prefix.sa();
            return Circle {
                real: prefix,
                imaginary: prefix,
                exponent: Self::ambiguous_exponent(),
            };
        }
        if real.vanished() {
            return Circle {
                real: 0.as_(),
                imaginary: imaginary.fraction,
                exponent: imaginary.exponent,
            };
        }
        if imaginary.vanished() {
            return Circle {
                real: real.fraction,
                imaginary: 0.as_(),
                exponent: real.exponent,
            };
        }
        // Normal case: translate each Scalar into Circle format (inflate>>1),
        // then align exponents (shift smaller component's fraction right).
        // Scalar format: value = inflate * 2^(exp - FRAC).
        // Circle format: value = stored * 2^(exp - FRAC + 1) = (inflate/2) * 2^(exp - FRAC + 1).
        // So: circle_stored = scalar_inflate >> 1, same exp.
        let real_c: F = real.fraction.inflate(true).w_shr(1isize).deflate();
        let imag_c: F = imaginary.fraction.inflate(true).w_shr(1isize).deflate();
        let exp_diff = real.exponent.wrapping_sub(&imaginary.exponent);
        if exp_diff == 0.as_() {
            return Circle {
                real: real_c,
                imaginary: imag_c,
                exponent: real.exponent,
            };
        } else if exp_diff > 0.as_() {
            let shift: isize = exp_diff.as_();
            if shift >= Self::fraction_bits() {
                return Circle {
                    real: real_c,
                    imaginary: 0.as_(),
                    exponent: real.exponent,
                };
            }
            return Circle {
                real: real_c,
                imaginary: imag_c >> shift,
                exponent: real.exponent,
            };
        } else {
            let zero_e: E = 0.as_();
            let shift: isize = zero_e.wrapping_sub(&exp_diff).as_();
            if shift >= Self::fraction_bits() {
                return Circle {
                    real: 0.as_(),
                    imaginary: imag_c,
                    exponent: imaginary.exponent,
                };
            }
            return Circle {
                real: real_c >> shift,
                imaginary: imag_c,
                exponent: imaginary.exponent,
            };
        }
    }
}
