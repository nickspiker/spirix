//! Scalar Display and Debug formatting implementations.
//!
//! This module implements `Display` and `Debug` traits for `Scalar<F, E>` types, providing flexible number formatting in any base (2-36) with any number of digits.
//!
//! # Key Design Principle
//!
//! **All digit extraction uses Spirix arithmetic directly** - the formatter does NOT convert numbers to u8 or use bitmasks. Instead, it extracts each digit by:
//! 1. Using `floor()` to separate integer and fractional parts
//! 2. Using division and multiplication by the base to extract individual digits
//! 3. Using `to_u8()` only for the final character conversion of already-extracted single digits
//!
//! This approach works for ANY base and ANY precision without needing special-case handling.
//!
//! # Formatting Modes
//!
//! The formatter automatically selects one of three display modes based on magnitude:
//! - **Big Numbers** (`|value| >= base^digits`): Scientific notation for large numbers
//! - **Normal Numbers**: Standard decimal-like notation with integer.fractional format
//! - **Small Numbers** (`|value| < base^-4`): Scientific notation for tiny numbers
//!
//! # Examples
//!
//! ```rust
//! use spirix::ScalarF5E3;
//!
//! let x = ScalarF5E3::from(42.5);
//!
//! // Default formatting (base-10) println!("{}", x);  // +42.5
//!
//! // Hexadecimal (base-16) println!("{:.16}", x);  // +2A.8
//!
//! // Binary (base-2) with 32 digits println!("{:32.2}", x);  // +101010.1
//!
//! // Debug output shows internal bit pattern println!("{:?}", x);
//! ```

use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::{
    Circle, CircleConstants, Integer, Scalar, ScalarConstants, ScalarF4E4, ScalarF5E5, ScalarF6E6,
    ScalarF7E7,
};
use ::core::fmt;
use ::core::ops::*;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
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
    > fmt::Display for Scalar<F, E>
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
    /// Formats a Scalar for display using precision and width specifiers.
    ///
    /// # Format Parameters
    ///
    /// - **Precision** (`.N`): Specifies the base (2-36). Default is 10.
    /// - **Width** (`:N`): Specifies how many digits to display. Default is calculated from the fraction bits as `log_base(2^fraction_bits)`.
    /// - **Alternate** (`#`): Emit digits in the PRIVATE-USE block — digit N as codepoint `0x10 + N` — instead of ASCII, so a consumer whose font maps that range draws its own numerals with no transliteration pass (see [`digit_char`]). Requires base ≤ 16, since the block holds sixteen slots; above that the flag is ignored and digits stay ASCII.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::ScalarF5E3; let x = ScalarF5E3::from(255); assert_eq!(format!("{:.16}", x), "+FF");  // Hex assert_eq!(format!("{:.2}", x), "+11111111");  // Binary assert_eq!(format!("{:4.10}", x), "+255");  // Base-10, max 4 digits
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

        // Calculate default digit count based on fraction bit precision For very large fraction types, use a smaller type to avoid overflow in the calculation
        let mut digits = if Self::fraction_bits() > 100 && Self::exponent_bits() < 12 {
            crate::ScalarF7E4::TWO
                .pow(Self::fraction_bits())
                .log(base)
                .floor()
                .to_isize()
        } else {
            Self::TWO
                .pow(Self::fraction_bits())
                .log(base)
                .floor()
                .to_isize()
        };

        // Override digit count if width is specified
        if let Some(width) = f.width() {
            digits = width as isize;
        }

        // `#` asks for glyph-coded digits. The C0 block from 0x10 holds sixteen slots, so above base sixteen there is nowhere to put digit 16 that is not 0x20 (space) — the flag is dropped rather than silently emitting whitespace as a numeral.
        let glyphs = f.alternate() && base <= 16;

        let string = self.format_scalar(base, digits, glyphs);
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
    > fmt::Debug for Scalar<F, E>
where
    F: Integer,
    E: Integer,
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
{
    /// Formats a Scalar for debug output showing internal bit representation.
    ///
    /// # Debug Modes
    ///
    /// - **Plain (`{:?}`)**: Shows raw binary bits as 0s and 1s
    /// - **Fancy (`{:#?}`)**: Shows coloured binary with special characters:
    ///   - Normal values: □ (unset) and ■ (set)
    ///   - Undefined values: ▵ (unset) and ▴ (set)
    ///   - Zero: 0 (unset) and | (set)
    ///   - Other states: ○ (unset) and ● (set)
    ///
    /// # Format
    ///
    /// The output shows: `fraction_bits *2^ exponent_bits`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::ScalarF5E3; let x = ScalarF5E3::from(5); println!("{:?}", x);   // Plain binary println!("{:#?}", x);  // Coloured with special chars
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

/// ONE digit → character, for every emission site in this module.
///
/// `glyphs` selects the PRIVATE-USE encoding: digit N becomes codepoint `0x10 + N`, the otherwise-unused C0 block (DLE..US). A consumer that ships a font mapping those codepoints draws its own numerals — photon's dozenal digits live exactly there — and gets them straight out of the formatter instead of transliterating ASCII afterwards, which cannot be done safely: the output also carries a base marker, exponent digits and escape-class tags, and a post-hoc mapper has no way to tell a digit `B` from a structural one.
///
/// The block holds sixteen slots, so glyph output is valid only for **base ≤ 16**; `Display` falls back to ASCII above that rather than walking into `0x20` (space).
fn digit_char(digit: u8, glyphs: bool) -> char {
    if glyphs {
        char::from(0x10u8.wrapping_add(digit))
    } else if digit < 10 {
        digit.wrapping_add(b'0') as char
    } else {
        digit.wrapping_sub(10).wrapping_add(b'A') as char
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
    > Scalar<F, E>
where
    F: Integer,
    E: Integer,
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
    /// Core formatting function that converts a Scalar to a string representation.
    ///
    /// This function uses **Spirix arithmetic exclusively** to extract digits - it does NOT convert the entire number to u8 or use bitmasks. The formatter can handle any base (2-36) and any number of digits because it works with the Spirix number directly.
    ///
    /// # Algorithm
    ///
    /// 1. **Special Values**: Check for undefined, infinity, exploded, vanished, or zero
    /// 2. **Mode Selection**: Choose between scientific (big/small) or normal notation
    /// 3. **Digit Extraction**:
    ///    - Use `floor()` to separate integer and fractional parts
    ///    - For integer part: repeatedly divide by base, extract remainder digit
    ///    - For fractional part: repeatedly multiply by base, extract integer digit
    ///    - Convert each extracted single digit to a character using `to_u8()`
    ///
    /// # Parameters
    ///
    /// - `base`: The numeric base (2-36) to use for digit extraction
    /// - `digits`: Maximum number of significant digits to display
    ///
    /// # Returns
    ///
    /// A string with the format `[+/-]digits` for normal values (no brackets — signs always shown, decimal omitted for integers, scientific notation `±d.ddd×base^exp` once the value exceeds the digit window). Non-normal values wrap escape-class tags in `⦉ ⦊`: `⦉∞⦊`, `⦉+↑⦊`, `⦉-↓⦊`, `0` (bare), or `℘<reason>` for undefined.
    ///
    /// # Important Note
    ///
    /// The `to_u8()` call is ONLY used for converting already-extracted single digits (0-35) to their character representation. The actual digit extraction uses Spirix division and multiplication, which works for any base and precision.
    fn format_scalar(&self, base: u8, digits: isize, glyphs: bool) -> String {
        if !self.is_normal() {
            if self.is_undefined() {
                let prefix = self.prefix();
                return Undefined::from_prefix(prefix).symbol.to_owned();
            }
        }

        let mut string = String::new();
        if !self.is_normal() {
            // Zero: bare "0" (no brackets, no sign — zero has no sign in math). Escape classes (infinity, exploded, vanished) wrap in ⦉ ⦊ so the special tag is clearly demarcated from any surrounding text. Undefined was already handled above with no brackets.
            if self.is_infinite() {
                string.push('⦉');
                string.push('∞');
                string.push('⦊');
            } else if self.exploded() {
                string.push('⦉');
                if self.is_negative() {
                    string.push_str("-↑");
                } else {
                    string.push_str("+↑");
                };
                string.push('⦊');
            } else if self.vanished() {
                string.push('⦉');
                if self.is_negative() {
                    string.push_str("-↓");
                } else {
                    string.push_str("+↓");
                };
                string.push('⦊');
            } else {
                // Zero is a DIGIT, so it takes the glyph encoding like any other — an ASCII '0' loose in glyph output would resolve from a different font face and read as a hole in the number.
                string.push(digit_char(0, glyphs));
            }
        } else {
            let base_scalar = Self::from(base);

            // Three-way split: Big (scientific), Normal (decimal), Small (scientific)
            if self <= -base_scalar.pow(digits) || self >= base_scalar.pow(digits) {
                if Self::fraction_bits() < Self::exponent_bits() {
                    match Self::exponent_bits() {
                        16 => string
                            .push_str(&ScalarF4E4::from(self).format_scientific_big(base, digits, glyphs)),
                        32 => string
                            .push_str(&ScalarF5E5::from(self).format_scientific_big(base, digits, glyphs)),
                        64 => string
                            .push_str(&ScalarF6E6::from(self).format_scientific_big(base, digits, glyphs)),
                        128 => string
                            .push_str(&ScalarF7E7::from(self).format_scientific_big(base, digits, glyphs)),
                        _ => string.push_str(&self.format_scientific_big(base, digits, glyphs)),
                    }
                } else {
                    string.push_str(&self.format_scientific_big(base, digits, glyphs))
                }
            } else if self < base_scalar.pow(-4) && self > -base_scalar.pow(-4) {
                if Self::fraction_bits() < Self::exponent_bits() {
                    match Self::exponent_bits() {
                        16 => string.push_str(
                            &ScalarF4E4::from(self).format_scientific_small(base, digits, glyphs),
                        ),
                        32 => string.push_str(
                            &ScalarF5E5::from(self).format_scientific_small(base, digits, glyphs),
                        ),
                        64 => string.push_str(
                            &ScalarF6E6::from(self).format_scientific_small(base, digits, glyphs),
                        ),
                        128 => string.push_str(
                            &ScalarF7E7::from(self).format_scientific_small(base, digits, glyphs),
                        ),
                        _ => string.push_str(&self.format_scientific_small(base, digits, glyphs)),
                    }
                } else {
                    string.push_str(&self.format_scientific_small(base, digits, glyphs))
                }
            } else {
                if self.is_negative() {
                    string.push('-');
                } else {
                    string.push('+');
                }
                // Normal notation: use floor to split integer and fractional parts
                let magnitude = self.magnitude();
                let mut integer_part = magnitude.floor();
                let mut fractional_part = magnitude - integer_part;

                // Extract integer digits using /base
                let mut int_digits = Vec::new();
                let mut digit_count = 0;

                if integer_part.is_zero() {
                    int_digits.push(0u8);
                } else {
                    let mut leading = true;
                    while !integer_part.is_zero() && digit_count < digits {
                        let scaled = integer_part / base_scalar;
                        integer_part = scaled.floor();
                        let digit = ((scaled - scaled.floor()) * base_scalar + Self::HALF).to_u8();
                        int_digits.push(digit);

                        // Only count non-leading digits
                        if leading && digit == 0 {
                            // Still leading zeros, don't increment counter
                        } else {
                            leading = false;
                            digit_count += 1;
                        }
                    }
                }

                // Convert integer part
                for &digit in int_digits.iter().rev() {
                    let digit_char = digit_char(digit, glyphs);
                    string.push(digit_char);
                }

                // Handle fractional part if it exists
                if !fractional_part.is_zero() {
                    string.push('.');
                    while digit_count < digits && !fractional_part.is_zero() {
                        fractional_part = fractional_part * base_scalar;
                        let digit = fractional_part.to_u8();
                        fractional_part = fractional_part - digit;

                        // Don't count leading fractional zeros
                        if !(digit == 0 && digit_count == 0) {
                            digit_count += 1;
                        }

                        let digit_char = digit_char(digit, glyphs);
                        string.push(digit_char);
                    }
                }
            }
        }

        string
    }

    /// Formats large numbers in scientific notation: `±d.ddd...×base^exponent`
    ///
    /// Used when the number's magnitude is greater than or equal to `base^digits`.
    ///
    /// # Algorithm
    ///
    /// 1. **Find Scale**: Calculate the exponent by taking `log_base(magnitude).floor()`
    /// 2. **Normalize**: Divide the number by `base^exponent` to get a value in [1, base)
    /// 3. **Extract Digits**: Use Spirix arithmetic to extract each digit:
    ///    - Get integer part with `to_u8()` (which is 0-9 or 0-35)
    ///    - Subtract that digit from the scaled value
    ///    - Multiply by base to shift next digit into integer position
    ///    - Repeat for the specified number of digits
    /// 4. **Format Exponent**: Extract exponent digits using the same division technique
    ///
    /// # Key Point
    ///
    /// This function demonstrates that **digit extraction works for any base** because it uses Spirix division and multiplication, not bitmasks or u8 conversions. The `to_u8()` is only called on individual digits (0-35), not on the full number.
    fn format_scientific_big(&self, base: u8, digits: isize, glyphs: bool) -> String {
        let base_scalar = Self::from(base);

        let magnitude = if self == Self::MIN {
            Self::MAX
        } else {
            self.magnitude()
        };
        let mut power = magnitude.log(base_scalar).floor();
        let mut scaled = self / base_scalar.pow(power);
        while scaled.magnitude() >= base_scalar {
            power += 1;
            scaled = self / base_scalar.pow(power);
        }
        while scaled.magnitude() < 1 {
            power -= 1;
            scaled = self / base_scalar.pow(power);
        }

        let mut result = String::new();

        if scaled.is_negative() {
            result.push('-');
            scaled = -scaled;
        } else {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled.to_u8();
            scaled = (scaled - digit) * base_scalar;

            let digit_char = digit_char(digit, glyphs);
            result.push(digit_char);

            if d == 0 {
                result.push('.');
            }

            if scaled.is_zero() {
                break;
            }
        }

        result.push('×');
        let base_char = digit_char(base, false);
        result.push(base_char);
        result.push('^');
        result.push('+');

        // Format exponent
        let mut exp_value = power;
        let mut exp_digits = Vec::new();

        if exp_value.is_zero() {
            exp_digits.push(0u8);
        } else {
            while exp_value.is_normal() {
                let scaled = exp_value / base_scalar;
                exp_value = scaled.floor();
                let digit = ((scaled - exp_value) * base_scalar + Self::HALF).to_u8();
                exp_digits.push(digit);
            }
        }

        for &digit in exp_digits.iter().rev() {
            let digit_char = digit_char(digit, glyphs);
            result.push(digit_char);
        }

        result
    }

    /// Formats tiny numbers in scientific notation: `±d.ddd...×base^-exponent`
    ///
    /// Used when the number's magnitude is less than `base^-4`.
    ///
    /// # Algorithm
    ///
    /// 1. **Find Scale**: Calculate the negative exponent by taking `-log_base(magnitude).floor()`
    /// 2. **Normalize**: Multiply the number by `base^exponent` to get a value in [1, base)
    /// 3. **Handle Overflow**: If `base^exponent` would explode, use incremental multiplication
    /// 4. **Extract Digits**: Same process as `format_scientific_big`:
    ///    - Extract each digit using `to_u8()` on the integer part
    ///    - Subtract and multiply by base to get next digit
    ///    - All arithmetic is done with Spirix operations
    /// 5. **Format Exponent**: Extract negative exponent digits using division
    ///
    /// # Why This Works
    ///
    /// The formatter handles arbitrary precision because it never converts the whole number to a primitive type. It only extracts one digit at a time using Spirix arithmetic (division/multiplication), then converts that single digit to a char.
    fn format_scientific_small(&self, base: u8, digits: isize, glyphs: bool) -> String {
        let base_scalar = Self::from(base);

        let magnitude = self.magnitude();
        let mut power = -magnitude.log(base_scalar).floor();
        let mut scaled = magnitude * base_scalar.pow(power);

        if scaled.exploded() {
            while scaled.exploded() {
                power -= 1;
                scaled = magnitude * base_scalar.pow(power);
            }
            while scaled.magnitude() < 1 {
                power += 1;
                scaled *= base_scalar;
            }
        } else {
            // First adjustment loop - while scaled >= base
            while scaled.magnitude() > base_scalar {
                power -= 1;
                scaled = magnitude * base_scalar.pow(power);
            }
            while scaled.magnitude() < 1 {
                power += 1;
                scaled = magnitude * base_scalar.pow(power);
            }
        }

        let mut result = String::new();

        if self.is_negative() {
            result.push('-');
        } else {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled.to_u8();
            scaled = (scaled - digit) * base_scalar;

            let digit_char = digit_char(digit, glyphs);
            result.push(digit_char);

            if d == 0 {
                result.push('.');
            }

            if scaled.is_zero() {
                break;
            }
        }

        result.push('×');
        let base_char = digit_char(base, false);
        result.push(base_char);
        result.push('^');
        result.push('-');

        // Format exponent
        let mut exp_value = power;
        let mut exp_digits = Vec::new();

        if exp_value.is_zero() {
            exp_digits.push(0u8);
        } else {
            while exp_value.is_normal() {
                let scaled = exp_value / base_scalar;
                exp_value = scaled.floor();
                let digit = ((scaled - exp_value) * base_scalar + Self::HALF).to_u8();
                exp_digits.push(digit);
            }
        }

        for &digit in exp_digits.iter().rev() {
            let digit_char = digit_char(digit, glyphs);
            result.push(digit_char);
        }

        result
    }

    /// Formats the Scalar as plain binary for debug output (`{:?}`).
    ///
    /// Shows the raw bit representation of the internal fraction and exponent components. Unlike display formatting which extracts digits using arithmetic, debug formatting directly inspects the bits using `rotate_left()` to examine each bit position.
    ///
    /// # Output Format
    ///
    /// `fraction_bits *2^ exponent_bits`
    ///
    /// Where:
    /// - `fraction_bits`: Binary representation of the fraction field (0s and 1s)
    /// - `exponent_bits`: Binary representation of the exponent field (0s and 1s)
    /// - Bits are shown from MSB to LSB (most significant first)
    /// - Spaces are added every 8 bits for readability
    /// - Double space in the middle of the fraction (except for 8-bit fractions)
    fn format_debug_plain(&self) -> String {
        let mut binary = String::new();
        let mut rotating = self.fraction;
        let middle = Self::fraction_bits() / 2;

        // Format fraction bits
        for b in 0..Self::fraction_bits() {
            if Self::fraction_bits() != 8 && b == middle {
                binary.push_str("  "); // Double space in middle
            }
            binary.push(if rotating.is_negative() { '1' } else { '0' });
            rotating = rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary.push_str("*2^ ");

        // Format exponent bits
        let mut exp_rotating = self.exponent;
        for b in 0..Self::exponent_bits() {
            binary.push(if exp_rotating.is_negative() { '1' } else { '0' });
            exp_rotating = exp_rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary
    }

    /// Formats the Scalar with colours and special characters for debug output (`{:#?}`).
    ///
    /// Similar to `format_debug_plain()`, but with ANSI colour codes and special Unicode characters that indicate the number's state visually.
    ///
    /// # Visual Elements
    ///
    /// - **Colours**: Different RGB colours for different states (see `COLOURS` constant)
    ///   - Normal positive/negative: Light red/blue
    ///   - Exploded: Bright red/blue
    ///   - Vanished: Bright red/blue
    ///   - Zero: Light green
    ///   - Undefined: Light magenta
    ///   - Integer/fractional exponents: Light yellow/cyan
    /// - **Characters**:
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
        let mut rotating = self.fraction;
        let (unset, set) = self.get_binary_chars();
        let middle = Self::fraction_bits() / 2;
        let scheme = self.get_colour_scheme();
        let rgb = &scheme.colour;

        // Start fraction colour
        binary.push_str(&format!("\x1B[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2]));

        // Format fraction bits
        for b in 0..Self::fraction_bits() {
            if Self::fraction_bits() != 8 && b == middle {
                binary.push(' ');
            }

            binary.push(if rotating.is_negative() { set } else { unset });
            rotating = rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        // Reset colour and add separator
        binary.push_str("\x1B[0m*2^ ");

        // Format exponent bits with their colour
        let exp_scheme = if self.exponent.is_negative() {
            &COLOURS.fractional_exponent
        } else if self.exponent.is_positive() {
            &COLOURS.integer_exponent
        } else {
            &COLOURS.zero
        };

        // Start exponent colour
        binary.push_str(&format!(
            "\x1B[38;2;{};{};{}m",
            exp_scheme.colour[0], exp_scheme.colour[1], exp_scheme.colour[2]
        ));

        let mut exp_rotating = self.exponent;
        for b in 0..Self::exponent_bits() {
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

        // Reset colour at end
        binary.push_str("\x1B[0m");

        binary
    }

    /// Selects the appropriate colour scheme based on the Scalar's state.
    ///
    /// Used by `format_debug_fancy()` to choose the right colour for the fraction bits. The exponent bits use a separate colour selection based on their sign.
    ///
    /// # Returns
    ///
    /// A reference to the appropriate `ColourScheme` from the global `COLOURS` palette.
    fn get_colour_scheme(&self) -> &'static ColourScheme {
        if self.is_normal() {
            if self.is_negative() {
                return &COLOURS.normal_negative;
            } else {
                return &COLOURS.normal_positive;
            }
        } else {
            if self.is_undefined() {
                return &COLOURS.undefined;
            }
            if self.vanished() {
                if self.is_negative() {
                    return &COLOURS.vanished_negative;
                } else {
                    return &COLOURS.vanished_positive;
                }
            }
            if self.exploded() {
                if self.is_negative() {
                    return &COLOURS.exploded_negative;
                } else {
                    return &COLOURS.exploded_positive;
                }
            }
            &COLOURS.zero
        }
    }

    /// Selects the appropriate Unicode characters for representing bits.
    ///
    /// Used by `format_debug_fancy()` to choose special characters based on the number's state.
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

#[cfg(test)]
mod glyph_tests {
    use crate::ScalarF6E4;
    extern crate alloc;
    use alloc::format;
    use alloc::string::{String, ToString};

    /// Render the private-use digits back to something a test failure can print.
    fn readable(s: &str) -> String {
        s.chars().map(|c| match c as u32 {
            d @ 0x10..=0x1F => char::from_digit(d - 0x10, 16).unwrap().to_ascii_uppercase(),
            _ => c,
        }).collect()
    }

    /// `#` emits digit N as codepoint 0x10+N and touches nothing else — the sign, the radix point and the digit ORDER all survive, which is what lets a consumer print the result straight out instead of transliterating ASCII afterwards.
    #[test]
    fn alternate_emits_private_use_digits() {
        let x = ScalarF6E4::from(255);
        assert_eq!(format!("{:.16}", x), "+FF", "ASCII hex unchanged");
        let g = format!("{:#.16}", x);
        assert_eq!(g.chars().next(), Some('+'), "the sign is structure, not a digit");
        assert_eq!(&g[1..].chars().map(|c| c as u32).collect::<alloc::vec::Vec<_>>(), &[0x1F, 0x1F], "F is digit fifteen → 0x1F");
        assert_eq!(readable(&g), "+FF", "and it reads back identically");
    }

    /// THE photon case (docs/languages.md, the Base page): an hour is log2(3600) doublings, and in dozenal that is Ɛ.992… — a fractional magnitude that ASCII would spell `B.992` and the glyph form hands over ready to draw.
    #[test]
    fn dozenal_fraction_of_an_hour() {
        let hour = ScalarF6E4::from(3600.0f64).lb();
        assert_eq!(format!("{:3.12}", hour), "+B.99", "ASCII dozenal");
        let g = format!("{:#3.12}", hour);
        assert_eq!(readable(&g), "+B.99", "same number, private-use digits");
        // B is digit eleven (Stelor), 9 is digit nine; the point stays an ASCII '.'.
        let cp: alloc::vec::Vec<u32> = g.chars().map(|c| c as u32).collect();
        assert_eq!(cp, alloc::vec![b'+' as u32, 0x1B, '.' as u32, 0x19, 0x19]);
    }

    /// Exact values stay exact and negatives keep their sign: half a second is precisely one halving.
    #[test]
    fn exact_and_negative_survive() {
        let half = ScalarF6E4::from(0.5f64).lb();
        assert_eq!(format!("{:#3.12}", half), "-1".chars().map(|c| if c == '1' { char::from(0x11) } else { c }).collect::<String>());
        let sec = ScalarF6E4::from(1.0f64).lb();
        assert_eq!(format!("{:#3.12}", sec), char::from(0x10).to_string(), "zero is the Zil glyph, no sign");
    }

    /// The block has sixteen slots, so base seventeen has nowhere to put digit sixteen except 0x20 — a space masquerading as a numeral. The flag drops instead.
    #[test]
    fn above_base_sixteen_falls_back_to_ascii() {
        let x = ScalarF6E4::from(255);
        assert_eq!(format!("{:#.17}", x), format!("{:.17}", x), "base 17 ignores the flag");
        assert!(!format!("{:#.17}", x).chars().any(|c| (c as u32) < 0x20), "no C0 bytes escaped");
    }
}
