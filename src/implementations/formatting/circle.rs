//! Circle Display and Debug formatting implementations.
//!
//! This module implements `Display` and `Debug` traits for `Circle<F, E>` types,
//! providing flexible complex number formatting in any base (2-36) with any number of digits.
//!
//! # Key Design Principle
//!
//! **All digit extraction uses Spirix arithmetic directly** - just like the Scalar formatter,
//! the Circle formatter does NOT convert numbers to u8 or use bitmasks. Instead, it:
//! 1. Formats each component (real and imaginary) using the same Spirix arithmetic as Scalar
//! 2. Uses `floor()` to separate integer and fractional parts
//! 3. Uses division and multiplication by the base to extract individual digits
//! 4. Uses `to_u8()` only for converting already-extracted single digits to characters
//!
//! # Circle Format
//!
//! Circle values are displayed as `⦇real,imaginary⦈` where both real and imaginary
//! components are formatted using the same algorithm as Scalar formatting.
//!
//! # Examples
//!
//! ```rust
//! use spirix::CircleF5E3;
//!
//! let z = CircleF5E3::new(3, 4);  // 3 + 4i
//!
//! // Default formatting (base-10)
//! println!("{}", z);  // ⦇+3,+4⦈
//!
//! // Hexadecimal (base-16)
//! println!("{:.16}", z);  // ⦇+3,+4⦈
//!
//! // Binary (base-2) with 16 digits
//! println!("{:16.2}", z);  // ⦇+11,+100⦈
//!
//! // Debug output shows internal bit pattern for both components
//! println!("{:?}", z);
//! ```

use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use std::fmt::{self};
use std::ops::*;

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
    > fmt::Display for Circle<F, E>
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
    Scalar<i128, i128>: From<Scalar<F, E>>,
{
    /// Formats a Circle for display using precision and width specifiers.
    ///
    /// # Format Parameters
    ///
    /// - **Precision** (`.N`): Specifies the base (2-36). Default is 10.
    /// - **Width** (`:N`): Specifies how many digits to display per component. Default is
    ///   calculated from the fraction bits as `log_base(2^fraction_bits)`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::CircleF5E3;
    /// let z = CircleF5E3::new(15, 31);
    /// assert_eq!(format!("{:.16}", z), "⦇+F,+1F⦈");  // Hex
    /// assert_eq!(format!("{:.2}", z), "⦇+1111,+11111⦈");  // Binary
    /// assert_eq!(format!("{:3.10}", z), "⦇+15,+31⦈");  // Base-10, max 3 digits
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Extract base from precision specifier (default: base-10)
        let mut base: u8 = 10;
        if let Some(prec) = f.precision() {
            base = prec as u8;
            if base < 2 || base > 36 {
                return write!(f, "Error: Only bases 2-36 are supported!");
            }
        }

        // Calculate default digit count based on fraction bit precision
        // For very large fraction types, use a smaller type to avoid overflow
        let mut digits = if F::FRACTION_BITS > 100 && E::EXPONENT_BITS < 12 {
            crate::ScalarF7E4::TWO
                .pow(F::FRACTION_BITS)
                .log(base)
                .floor()
                .to_isize()
        } else {
            Scalar::<F, E>::TWO
                .pow(F::FRACTION_BITS)
                .log(base)
                .floor()
                .to_isize()
        };

        // Override digit count if width is specified
        if let Some(width) = f.width() {
            digits = width as isize;
        }

        let string = self.format_circle(base, digits);
        write!(f, "{}", string)
    }
}

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
    > fmt::Debug for Circle<F, E>
where
    F: FractionConstants,
    E: ExponentConstants,
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
    I256: From<F>,
    I256: From<E>,
    Scalar<i128, i128>: From<Scalar<F, E>>,
{
    /// Formats a Circle for debug output showing internal bit representation.
    ///
    /// # Debug Modes
    ///
    /// - **Plain (`{:?}`)**: Shows raw binary bits as 0s and 1s for both components
    /// - **Fancy (`{:#?}`)**: Shows coloured binary with special characters
    ///
    /// # Format
    ///
    /// The output shows: `real_fraction_bits | imaginary_fraction_bits *2^ exponent_bits`
    ///
    /// Both real and imaginary components share the same exponent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::CircleF5E3;
    /// let z = CircleF5E3::new(1, -1);
    /// println!("{:?}", z);   // Plain binary
    /// println!("{:#?}", z);  // Coloured with special chars
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // {:#?} - Fancy coloured output
            write!(f, "{}", self.format_debug_fancy())
        } else {
            // {:?} - Plain binary output
            write!(f, "{}", self.format_debug_plain())
        }
    }
}

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
    F: FractionConstants,
    E: ExponentConstants,
    Scalar<F, E>: ScalarConstants,
    Circle<F, E>: CircleConstants,
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
    I256: From<F>,
    I256: From<E>,
{
    /// Core formatting function that converts a Circle to a string representation.
    ///
    /// This function formats both the real and imaginary components using the same
    /// **Spirix arithmetic** approach as `Scalar::format_scalar()`. It does NOT
    /// convert numbers to u8 or use bitmasks - instead, it extracts digits one at a time
    /// using division, multiplication, floor, and subtraction operations.
    ///
    /// # Algorithm
    ///
    /// 1. **Special Values**: Check for undefined, infinity, exploded, vanished, or zero
    /// 2. **Mode Selection**: Choose between scientific (big/small) or normal notation
    /// 3. **Format Real Component**: Extract digits using Spirix arithmetic
    /// 4. **Format Imaginary Component**: Extract digits using Spirix arithmetic
    /// 5. **Combine**: Return as `⦇real,imaginary⦈`
    ///
    /// # Parameters
    ///
    /// - `base`: The numeric base (2-36) to use for digit extraction
    /// - `digits`: Maximum number of significant digits to display per component
    ///
    /// # Returns
    ///
    /// A string with the format `⦇real,imaginary⦈` for normal values, or special
    /// symbols for non-normal values.
    ///
    /// # Key Point
    ///
    /// Just like Scalar formatting, this uses `to_u8()` ONLY for converting
    /// already-extracted single digits (0-35) to characters. All digit extraction
    /// is done using Spirix division and multiplication.
    fn format_circle(&self, base: u8, digits: isize) -> String {
        if !self.is_normal() {
            if self.is_undefined() {
                let prefix = self.real.sa();
                return Undefined::from_prefix(prefix).symbol.to_owned();
            }
        }

        let mut string = "⦇".to_owned();
        if !self.is_normal() {
            if self.is_infinite() {
                string.push_str("∞");
            } else if self.exploded() {
                let direction = self.sign();
                let mut mag_r = direction.r().magnitude();
                let mut mag_i = direction.i().magnitude();
                string.push_str("↑");
                let decimal = 1;
                if mag_r.is_normal() {
                    if self.r().is_negative() {
                        string.push('-');
                    } else {
                        string.push('+');
                    }
                    for d in 0..digits {
                        if d == decimal {
                            string.push('.');
                        }
                        let digit = mag_r.to_u8();
                        if digit < 10 {
                            string.push(digit.wrapping_add(48) as char)
                        } else {
                            string.push(digit.wrapping_add(55) as char);
                        }
                        mag_r = mag_r.frac() * base;
                    }
                } else {
                    string.push('0');
                }
                string.push(',');
                string.push('↑');
                if mag_i.is_normal() {
                    if self.i().is_negative() {
                        string.push('-');
                    } else {
                        string.push('+');
                    }
                    for d in 0..digits {
                        if d == decimal {
                            string.push('.');
                        }
                        let digit = mag_i.to_u8();
                        if digit < 10 {
                            string.push(digit.wrapping_add(48) as char)
                        } else {
                            string.push(digit.wrapping_add(55) as char);
                        }
                        mag_i = mag_i.frac() * base;
                    }
                } else {
                    string.push('0');
                }
            } else if self.vanished() {
                let direction = self.sign();
                let mut mag_r = direction.r().magnitude();
                let mut mag_i = direction.i().magnitude();
                string.push_str("↓");
                let decimal = 1;
                if mag_r.is_normal() {
                    if self.r().is_negative() {
                        string.push('-');
                    } else {
                        string.push('+');
                    }
                    for d in 0..digits {
                        if d == decimal {
                            string.push('.');
                        }
                        let digit = mag_r.to_u8();
                        if digit < 10 {
                            string.push(digit.wrapping_add(48) as char)
                        } else {
                            string.push(digit.wrapping_add(55) as char);
                        }
                        mag_r = mag_r.frac() * base;
                    }
                } else {
                    string.push('0');
                }
                string.push('%');
                string.push(',');
                string.push('↓');
                if mag_i.is_normal() {
                    if self.i().is_negative() {
                        string.push('-');
                    } else {
                        string.push('+');
                    }
                    for d in 0..digits {
                        if d == decimal {
                            string.push('.');
                        }
                        let digit = mag_i.to_u8();
                        if digit < 10 {
                            string.push(digit.wrapping_add(48) as char)
                        } else {
                            string.push(digit.wrapping_add(55) as char);
                        }
                        mag_i = mag_i.frac() * base;
                    }
                } else {
                    string.push('0');
                }
                string.push('%');
            } else {
                string.push('0');
                string.push('⦈');
            }
        } else {
            let base_scalar = Scalar::<F, E>::from(base);

            // Three-way split: Big (scientific), Normal (decimal), Small (scientific)
            if self.r() <= -base_scalar.pow(digits)
                || self.r() >= base_scalar.pow(digits)
                || self.i() <= -base_scalar.pow(digits)
                || self.i() >= base_scalar.pow(digits)
            {
                if F::FRACTION_BITS < E::EXPONENT_BITS {
                    match E::EXPONENT_BITS {
                        16 => string
                            .push_str(&CircleF4E4::from(self).format_scientific_big(base, digits)),
                        32 => string
                            .push_str(&CircleF5E5::from(self).format_scientific_big(base, digits)),
                        64 => string
                            .push_str(&CircleF6E6::from(self).format_scientific_big(base, digits)),
                        128 => string
                            .push_str(&CircleF7E7::from(self).format_scientific_big(base, digits)),
                        _ => string.push_str(&self.format_scientific_big(base, digits)),
                    }
                } else {
                    string.push_str(&self.format_scientific_big(base, digits))
                }
            } else if self.r().magnitude().max(self.i().magnitude()) < base_scalar.pow(-4) {
                if F::FRACTION_BITS < E::EXPONENT_BITS {
                    match E::EXPONENT_BITS {
                        16 => string.push_str(
                            &CircleF4E4::from(self).format_scientific_small(base, digits),
                        ),
                        32 => string.push_str(
                            &CircleF5E5::from(self).format_scientific_small(base, digits),
                        ),
                        64 => string.push_str(
                            &CircleF6E6::from(self).format_scientific_small(base, digits),
                        ),
                        128 => string.push_str(
                            &CircleF7E7::from(self).format_scientific_small(base, digits),
                        ),
                        _ => string.push_str(&self.format_scientific_small(base, digits)),
                    }
                } else {
                    string.push_str(&self.format_scientific_small(base, digits))
                }
            } else {
                // Normal notation: use floor to split integer and fractional parts
                let real = self.r();
                let real_magnitude = real.magnitude();
                let mut real_integer = real_magnitude.floor();
                let mut real_fraction = real_magnitude - real_integer;

                // Extract integer digits using /base
                let mut int_digits = Vec::new();
                let mut digit_count = 0;

                if real_integer.is_zero() {
                    int_digits.push(0u8);
                } else {
                    let mut leading = true;
                    while !real_integer.is_zero() && digit_count < digits {
                        let scaled = real_integer / base_scalar;
                        real_integer = scaled.floor();
                        let digit = ((scaled - scaled.floor()) * base_scalar + 0.5f32).to_u8();
                        int_digits.push(digit);

                        // Only count non-leading digits
                        if leading && digit == 0 {
                            // Still leading zeros, don't increment counter
                        } else {
                            leading = false;
                            digit_count = digit_count.wrapping_add(1);
                        }
                    }
                }

                if real.is_negative() {
                    string.push('-');
                } else if real.is_positive() {
                    string.push('+');
                }

                // Convert integer part
                for &digit in int_digits.iter().rev() {
                    let digit_char = if digit < 10 {
                        digit.wrapping_add(b'0') as char
                    } else {
                        digit.wrapping_sub(10).wrapping_add(b'A') as char
                    };
                    string.push(digit_char);
                }

                // Handle fractional part if it exists
                if !real_fraction.is_zero() {
                    string.push('.');
                    while digit_count < digits && !real_fraction.is_zero() {
                        real_fraction = real_fraction * base_scalar;
                        let digit = real_fraction.to_u8();
                        real_fraction = real_fraction - digit;

                        // Don't count leading fractional zeros
                        if !(digit == 0 && digit_count == 0) {
                            digit_count = digit_count.wrapping_add(1);
                        }

                        let digit_char = if digit < 10 {
                            digit.wrapping_add(b'0') as char
                        } else {
                            digit.wrapping_sub(10).wrapping_add(b'A') as char
                        };
                        string.push(digit_char);
                    }
                }
                string.push(',');
                let imaginary = self.i();
                let imaginary_magnitude = imaginary.magnitude();
                let mut imaginary_integer = imaginary_magnitude.floor();
                let mut imaginary_fraction = imaginary_magnitude - imaginary_integer;

                // Extract integer digits using /base
                int_digits = Vec::new();
                digit_count = 0;

                if imaginary_integer.is_zero() {
                    int_digits.push(0u8);
                } else {
                    let mut leading = true;
                    while !imaginary_integer.is_zero() && digit_count < digits {
                        let scaled = imaginary_integer / base_scalar;
                        imaginary_integer = scaled.floor();
                        let digit = ((scaled - scaled.floor()) * base_scalar + 0.5f32).to_u8();
                        int_digits.push(digit);

                        // Only count non-leading digits
                        if leading && digit == 0 {
                            // Still leading zeros, don't increment counter
                        } else {
                            leading = false;
                            digit_count = digit_count.wrapping_add(1);
                        }
                    }
                }

                if imaginary.is_negative() {
                    string.push('-');
                } else if imaginary.is_positive() {
                    string.push('+');
                }

                // Convert integer part
                for &digit in int_digits.iter().rev() {
                    let digit_char = if digit < 10 {
                        digit.wrapping_add(b'0') as char
                    } else {
                        digit.wrapping_sub(10).wrapping_add(b'A') as char
                    };
                    string.push(digit_char);
                }

                // Handle fractional part if it exists
                if !imaginary_fraction.is_zero() {
                    string.push('.');
                    while digit_count < digits && !imaginary_fraction.is_zero() {
                        imaginary_fraction = imaginary_fraction * base_scalar;
                        let digit = imaginary_fraction.to_u8();
                        imaginary_fraction = imaginary_fraction - digit;

                        // Don't count leading fractional zeros
                        if !(digit == 0 && digit_count == 0) {
                            digit_count = digit_count.wrapping_add(1);
                        }

                        let digit_char = if digit < 10 {
                            digit.wrapping_add(b'0') as char
                        } else {
                            digit.wrapping_sub(10).wrapping_add(b'A') as char
                        };
                        string.push(digit_char);
                    }
                }
                string.push('⦈');
            }
        }

        string
    }

    /// Formats large complex numbers in scientific notation.
    ///
    /// Used when either component's magnitude is greater than or equal to `base^digits`.
    /// Both components are scaled by the same exponent to maintain their relative magnitudes.
    ///
    /// # Algorithm
    ///
    /// 1. **Find Unified Scale**: Use the larger of the two component magnitudes
    /// 2. **Calculate Exponent**: Take `log_base(max_magnitude).floor()`
    /// 3. **Normalize Both**: Divide both components by `base^exponent`
    /// 4. **Extract Digits**: Use Spirix arithmetic on each component separately
    /// 5. **Add Exponent**: Append `×base^exponent` suffix
    ///
    /// # Unified Scaling
    ///
    /// Unlike formatting two separate Scalars, Circle uses a single shared exponent
    /// based on the larger component. This ensures the relative magnitudes of real
    /// and imaginary parts are preserved in the output.
    ///
    /// # Output Format
    ///
    /// `⦇±d.ddd...,±d.ddd...⦈×base^±exp`
    fn format_scientific_big(&self, base: u8, digits: isize) -> String {
        let base_scalar = Scalar::<F, E>::from(base);

        // Find the largest component magnitude for unified scaling
        let mag_r = if self.r() == Scalar::<F, E>::MIN {
            Scalar::<F, E>::MAX
        } else {
            self.r().magnitude()
        };
        let mag_i = if self.i() == Scalar::<F, E>::MIN {
            Scalar::<F, E>::MAX
        } else {
            self.i().magnitude()
        };
        let max_magnitude = mag_r.max(mag_i);
        let mut scale = max_magnitude.log(base_scalar).floor();
        let mut scaled_r = self.r() / base_scalar.pow(scale);
        let mut scaled_i = self.i() / base_scalar.pow(scale);

        // Adjust scale to ensure largest component is in range [1, base)
        while scaled_r.magnitude().max(scaled_i.magnitude()) >= base_scalar {
            scale = scale + 1;
            scaled_r = self.r() / base_scalar.pow(scale);
            scaled_i = self.i() / base_scalar.pow(scale);
        }
        while scaled_r.magnitude().max(scaled_i.magnitude()) < 1 {
            scale = scale - 1;
            scaled_r = self.r() / base_scalar.pow(scale);
            scaled_i = self.i() / base_scalar.pow(scale);
        }

        let mut result = String::new();

        if scaled_r.is_negative() {
            result.push('-');
            scaled_r = -scaled_r;
        } else if scaled_r.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_r.to_u8();
            scaled_r = (scaled_r - digit) * base_scalar;

            let digit_char = if digit < 10 {
                digit.wrapping_add(b'0') as char
            } else {
                digit.wrapping_sub(10).wrapping_add(b'A') as char
            };
            result.push(digit_char);
            if scaled_r.is_zero() {
                break;
            }
            if d == 0 {
                result.push('.');
            }
        }
        result.push(',');

        if scaled_i.is_negative() {
            result.push('-');
            scaled_i = -scaled_i;
        } else if scaled_i.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_i.to_u8();
            scaled_i = (scaled_i - digit) * base_scalar;

            let digit_char = if digit < 10 {
                digit.wrapping_add(b'0') as char
            } else {
                digit.wrapping_sub(10).wrapping_add(b'A') as char
            };
            result.push(digit_char);
            if scaled_i.is_zero() {
                break;
            }
            if d == 0 {
                result.push('.');
            }
        }
        result.push('⦈');

        // Add scientific notation suffix
        if !scale.is_zero() {
            result.push('×');
            if base < 10 {
                result.push(base.wrapping_add(48) as char)
            } else {
                result.push(base.wrapping_add(55) as char);
            }
            result.push('^');
            if scale.is_negative() {
                result.push('-');
                // Extract digits from exponent using Spirix arithmetic (avoids saturation)
                let mut exp_value = -scale;
                let mut exp_digits = Vec::new();
                if exp_value.is_zero() {
                    exp_digits.push(0u8);
                } else {
                    while exp_value.is_normal() && !exp_value.is_zero() {
                        let scaled = exp_value / base_scalar;
                        let floored = scaled.floor();
                        let digit =
                            ((scaled - floored) * base_scalar + Scalar::<F, E>::HALF).to_u8();
                        exp_digits.push(digit);
                        exp_value = floored;
                    }
                }
                for &digit in exp_digits.iter().rev() {
                    let digit_char = if digit < 10 {
                        digit.wrapping_add(b'0') as char
                    } else {
                        digit.wrapping_sub(10).wrapping_add(b'A') as char
                    };
                    result.push(digit_char);
                }
            } else {
                result.push('+');
                // Extract digits from exponent using Spirix arithmetic (avoids saturation)
                let mut exp_value = scale;
                let mut exp_digits = Vec::new();
                if exp_value.is_zero() {
                    exp_digits.push(0u8);
                } else {
                    while exp_value.is_normal() && !exp_value.is_zero() {
                        let scaled = exp_value / base_scalar;
                        let floored = scaled.floor();
                        let digit =
                            ((scaled - floored) * base_scalar + Scalar::<F, E>::HALF).to_u8();
                        exp_digits.push(digit);
                        exp_value = floored;
                    }
                }
                for &digit in exp_digits.iter().rev() {
                    let digit_char = if digit < 10 {
                        digit.wrapping_add(b'0') as char
                    } else {
                        digit.wrapping_sub(10).wrapping_add(b'A') as char
                    };
                    result.push(digit_char);
                }
            }
        }

        result
    }

    /// Formats tiny complex numbers in scientific notation.
    ///
    /// Used when both components have magnitude less than `base^-4`.
    /// Like `format_scientific_big()`, uses unified scaling for both components.
    ///
    /// # Algorithm
    ///
    /// 1. **Find Unified Scale**: Use the larger of the two component magnitudes
    /// 2. **Calculate Exponent**: Take `-log_base(max_magnitude).floor()`
    /// 3. **Normalize Both**: Multiply both components by `base^exponent`
    /// 4. **Handle Overflow**: If `base^exponent` would explode, use incremental multiplication
    /// 5. **Extract Digits**: Use Spirix arithmetic on each component separately
    /// 6. **Add Exponent**: Append `×base^-exp` suffix
    ///
    /// # Unified Scaling
    ///
    /// Both components share the same negative exponent to preserve their relative
    /// magnitudes in the output, just like `format_scientific_big()`.
    ///
    /// # Output Format
    ///
    /// `⦇±d.ddd...,±d.ddd...⦈×base^-exp`
    fn format_scientific_small(&self, base: u8, digits: isize) -> String {
        let base_scalar = Scalar::<F, E>::from(base);

        // Find the largest component magnitude for unified scaling (same as big numbers)
        let mag_r = if self.r() == Scalar::<F, E>::MIN {
            Scalar::<F, E>::MAX
        } else {
            self.r().magnitude()
        };
        let mag_i = if self.i() == Scalar::<F, E>::MIN {
            Scalar::<F, E>::MAX
        } else {
            self.i().magnitude()
        };
        let max_magnitude = mag_r.max(mag_i);

        let mut power = -max_magnitude.log(base_scalar).floor();

        let mut scaled_r;
        let mut scaled_i;

        if base_scalar.pow(power).exploded() {
            while base_scalar.pow(power).exploded() {
                power -= 1;
            }
            scaled_r = self.r() * base_scalar.pow(power);
            scaled_i = self.i() * base_scalar.pow(power);
            // Use incremental multiplication to get into [1, base) range
            while scaled_r.magnitude().max(scaled_i.magnitude()) < 1 {
                power += 1;
                scaled_r *= base_scalar;
                scaled_i *= base_scalar;
            }
        } else {
            // Normal adjustment loops
            scaled_r = self.r() * base_scalar.pow(power);
            scaled_i = self.i() * base_scalar.pow(power);
            while scaled_r.magnitude().max(scaled_i.magnitude()) >= base_scalar {
                power -= 1;
                scaled_r = self.r() * base_scalar.pow(power);
                scaled_i = self.i() * base_scalar.pow(power);
            }
            while scaled_r.magnitude().max(scaled_i.magnitude()) < 1 {
                power += 1;
                scaled_r = self.r() * base_scalar.pow(power);
                scaled_i = self.i() * base_scalar.pow(power);
            }
        }

        let mut result = String::new();

        if scaled_r.is_negative() {
            result.push('-');
            scaled_r = -scaled_r;
        } else if scaled_r.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_r.to_u8();
            scaled_r = (scaled_r - digit) * base_scalar;

            let digit_char = if digit < 10 {
                digit.wrapping_add(b'0') as char
            } else {
                digit.wrapping_sub(10).wrapping_add(b'A') as char
            };
            result.push(digit_char);
            if scaled_r.is_zero() {
                break;
            }
            if d == 0 {
                result.push('.');
            }
        }
        result.push(',');

        if scaled_i.is_negative() {
            result.push('-');
            scaled_i = -scaled_i;
        } else if scaled_i.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_i.to_u8();
            scaled_i = (scaled_i - digit) * base_scalar;

            let digit_char = if digit < 10 {
                digit.wrapping_add(b'0') as char
            } else {
                digit.wrapping_sub(10).wrapping_add(b'A') as char
            };
            result.push(digit_char);
            if scaled_i.is_zero() {
                break;
            }
            if d == 0 {
                result.push('.');
            }
        }
        result.push('⦈');

        // Add scientific notation suffix
        if !power.is_zero() {
            result.push('×');
            if base < 10 {
                result.push(base.wrapping_add(48) as char)
            } else {
                result.push(base.wrapping_add(55) as char);
            }
            result.push('^');
            result.push('-');
            // Extract digits from exponent using Spirix arithmetic (avoids saturation)
            let mut exp_value = power;
            let mut exp_digits = Vec::new();
            if exp_value.is_zero() {
                exp_digits.push(0u8);
            } else {
                while exp_value.is_normal() && !exp_value.is_zero() {
                    let scaled = exp_value / base_scalar;
                    let floored = scaled.floor();
                    let digit = ((scaled - floored) * base_scalar + Scalar::<F, E>::HALF).to_u8();
                    exp_digits.push(digit);
                    exp_value = floored;
                }
            }
            for &digit in exp_digits.iter().rev() {
                let digit_char = if digit < 10 {
                    digit.wrapping_add(b'0') as char
                } else {
                    digit.wrapping_sub(10).wrapping_add(b'A') as char
                };
                result.push(digit_char);
            }
        }

        result
    }

    /// Formats the Circle as plain binary for debug output (`{:?}`).
    ///
    /// Shows the raw bit representation of both fraction components and the shared exponent.
    /// Unlike display formatting which uses arithmetic, debug formatting directly inspects
    /// the bits using `rotate_left()`.
    ///
    /// # Output Format
    ///
    /// `real_fraction_bits | imaginary_fraction_bits *2^ exponent_bits`
    ///
    /// Where:
    /// - `real_fraction_bits`: Binary representation of the real component's fraction (0s and 1s)
    /// - `imaginary_fraction_bits`: Binary representation of the imaginary component's fraction
    /// - `exponent_bits`: Binary representation of the shared exponent field
    /// - Bits are shown from MSB to LSB (most significant first)
    /// - Spaces are added every 8 bits for readability
    /// - Double space in the middle of each fraction component (except for 8-bit fractions)
    fn format_debug_plain(&self) -> String {
        let mut binary = String::new();
        let middle = F::FRACTION_BITS / 2;

        // Format real component bits
        let mut rotating_real = self.real;
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
                binary.push_str("  "); // Double space in middle
            }
            binary.push(if rotating_real.is_negative() {
                '1'
            } else {
                '0'
            });
            rotating_real = rotating_real.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str("| ");

        // Format imaginary component bits
        let mut rotating_imag = self.imaginary;
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
                binary.push_str("  "); // Double space in middle
            }
            binary.push(if rotating_imag.is_negative() {
                '1'
            } else {
                '0'
            });
            rotating_imag = rotating_imag.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str(" *2^ ");

        // Format exponent bits
        let mut exp_rotating = self.exponent;
        for b in 0..E::EXPONENT_BITS {
            binary.push(if exp_rotating.is_negative() { '1' } else { '0' });
            exp_rotating = exp_rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary
    }

    /// Formats the Circle with colours and special characters for debug output (`{:#?}`).
    ///
    /// Similar to `format_debug_plain()`, but with ANSI colour codes and Unicode characters
    /// that indicate each component's state visually.
    ///
    /// # Visual Elements
    ///
    /// - **Colours**: Each component gets its own colour based on its state:
    ///   - Real component: Coloured based on real value's state
    ///   - Imaginary component: Coloured based on imaginary value's state
    ///   - Exponent: Coloured based on sign (integer/fractional)
    /// - **Characters**: Same as Scalar debug formatting:
    ///   - Normal: □ (unset bit), ■ (set bit)
    ///   - Undefined: ▵ (unset bit), ▴ (set bit)
    ///   - Zero: 0 (unset bit), | (set bit)
    ///   - Other: ○ (unset bit), ● (set bit)
    ///
    /// # ANSI Colour Format
    ///
    /// Uses `\x1B[38;2;R;G;Bm` for 24-bit RGB colours and `\x1B[0m` for reset.
    fn format_debug_fancy(&self) -> String {
        let mut binary = String::new();
        let middle = F::FRACTION_BITS / 2;
        let (unset, set) = self.get_binary_chars();

        // Format real component with appropriate colour
        let real_scheme = self.get_real_colour_scheme();
        let real_rgb = &real_scheme.colour;
        binary.push_str(&format!(
            "\x1B[38;2;{};{};{}m",
            real_rgb[0], real_rgb[1], real_rgb[2]
        ));

        let mut rotating_real = self.real;
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
                binary.push(' ');
            }

            binary.push(if rotating_real.is_negative() {
                set
            } else {
                unset
            });
            rotating_real = rotating_real.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str("\x1B[0m| ");

        // Format imaginary component with appropriate colour
        let imag_scheme = self.get_imag_colour_scheme();
        let imag_rgb = &imag_scheme.colour;
        binary.push_str(&format!(
            "\x1B[38;2;{};{};{}m",
            imag_rgb[0], imag_rgb[1], imag_rgb[2]
        ));

        let mut rotating_imag = self.imaginary;
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
                binary.push(' ');
            }

            binary.push(if rotating_imag.is_negative() {
                set
            } else {
                unset
            });
            rotating_imag = rotating_imag.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str("\x1B[0m *2^ ");

        // Format exponent bits with their colour
        let exp_scheme = if self.exponent.is_negative() {
            &COLOURS.fractional_exponent
        } else if self.exponent.is_positive() {
            &COLOURS.integer_exponent
        } else {
            &COLOURS.zero
        };

        binary.push_str(&format!(
            "\x1B[38;2;{};{};{}m",
            exp_scheme.colour[0], exp_scheme.colour[1], exp_scheme.colour[2]
        ));

        let mut exp_rotating = self.exponent;
        for b in 0..E::EXPONENT_BITS {
            binary.push(if exp_rotating.is_negative() {
                '■'
            } else {
                '□'
            });
            exp_rotating = exp_rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str("\x1B[0m");
        binary
    }

    /// Selects the appropriate colour scheme for the real component.
    ///
    /// Used by `format_debug_fancy()` to choose the right colour for the real fraction bits.
    ///
    /// # Returns
    ///
    /// A reference to the appropriate `ColourScheme` from the global `COLOURS` palette.
    fn get_real_colour_scheme(&self) -> &'static ColourScheme {
        if self.is_normal() {
            if self.real.is_negative() {
                &COLOURS.normal_negative
            } else {
                &COLOURS.normal_positive
            }
        } else if self.is_undefined() {
            &COLOURS.undefined
        } else if self.vanished() {
            if self.real.is_negative() {
                &COLOURS.vanished_negative
            } else {
                &COLOURS.vanished_positive
            }
        } else if self.exploded() {
            if self.real.is_negative() {
                &COLOURS.exploded_negative
            } else {
                &COLOURS.exploded_positive
            }
        } else {
            &COLOURS.zero
        }
    }

    /// Selects the appropriate colour scheme for the imaginary component.
    ///
    /// Used by `format_debug_fancy()` to choose the right colour for the imaginary fraction bits.
    ///
    /// # Returns
    ///
    /// A reference to the appropriate `ColourScheme` from the global `COLOURS` palette.
    fn get_imag_colour_scheme(&self) -> &'static ColourScheme {
        if self.is_normal() {
            if self.imaginary.is_negative() {
                &COLOURS.normal_negative
            } else {
                &COLOURS.normal_positive
            }
        } else if self.is_undefined() {
            &COLOURS.undefined
        } else if self.vanished() {
            if self.imaginary.is_negative() {
                &COLOURS.vanished_negative
            } else {
                &COLOURS.vanished_positive
            }
        } else if self.exploded() {
            if self.imaginary.is_negative() {
                &COLOURS.exploded_negative
            } else {
                &COLOURS.exploded_positive
            }
        } else {
            &COLOURS.zero
        }
    }

    /// Selects the appropriate Unicode characters for representing bits.
    ///
    /// Used by `format_debug_fancy()` to choose special characters based on the Circle's state.
    ///
    /// # Returns
    ///
    /// A tuple of `(unset_char, set_char)` representing 0 and 1 bits respectively.
    fn get_binary_chars(&self) -> (char, char) {
        if self.is_normal() {
            ('□', '■')
        } else if self.is_undefined() {
            ('▵', '▴')
        } else if self.is_zero() {
            ('0', '|')
        } else {
            ('○', '●')
        }
    }
}
