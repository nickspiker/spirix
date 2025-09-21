use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar,
    ScalarConstants, ScalarF4E4, ScalarF5E5, ScalarF6E6, ScalarF7E7,
};
use i256::I256;
use num_traits::AsPrimitive;
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut base: u8 = 10;
        if let Some(prec) = f.precision() {
            base = prec as u8;
            if base < 2 || base > 36 {
                return write!(f, "Error: Only bases 2-36 are supported!");
            }
        }

        let mut digits = if F::FRACTION_BITS > 100 && E::EXPONENT_BITS < 12 {
            crate::ScalarF7E4::TWO
                .pow(F::FRACTION_BITS)
                .log(base)
                .floor()
                .to_isize()
        } else {
            Self::TWO.pow(F::FRACTION_BITS).log(base).floor().to_isize()
        };

        if let Some(width) = f.width() {
            digits = width as isize;
        }

        let string = self.format_scalar(base, digits);
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
            + Shr<E, Output = F>,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > fmt::Debug for Scalar<F, E>
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
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            // {:#?}
            write!(f, "{}", self.format_debug_fancy())
        } else {
            // {:?}
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
            + Shr<E, Output = F>,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
    > Scalar<F, E>
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
    fn format_scalar(&self, base: u8, digits: isize) -> String {
        let mut string = "⦉".to_owned();
        if !self.is_normal() {
            if self.is_undefined() {
                let prefix = self.prefix();
                string.push_str(Undefined::from_prefix(prefix).symbol);
            } else if self.is_infinite() {
                string.push_str("∞");
            } else if self.exploded() {
                if self.fraction.is_negative() {
                    string.push_str("-↑");
                } else {
                    string.push_str("+↑")
                };
            } else if self.vanished() {
                if self.fraction.is_negative() {
                    string.push_str("-↓");
                } else {
                    string.push_str("+↓");
                };
            } else {
                string.push('0');
            }
        } else {
            let base_scalar = Self::from(base);

            // Three-way split: Big (scientific), Normal (decimal), Small (scientific)
            if self <= -base_scalar.pow(digits) || self >= base_scalar.pow(digits) {
                if F::FRACTION_BITS < E::EXPONENT_BITS {
                    match E::EXPONENT_BITS {
                        16 => string
                            .push_str(&ScalarF4E4::from(self).format_scientific_big(base, digits)),
                        32 => string
                            .push_str(&ScalarF5E5::from(self).format_scientific_big(base, digits)),
                        64 => string
                            .push_str(&ScalarF6E6::from(self).format_scientific_big(base, digits)),
                        128 => string
                            .push_str(&ScalarF7E7::from(self).format_scientific_big(base, digits)),
                        _ => string.push_str(&self.format_scientific_big(base, digits)),
                    }
                } else {
                    string.push_str(&self.format_scientific_big(base, digits))
                }
            } else if self < base_scalar.pow(-4) && self > -base_scalar.pow(-4) {
                if F::FRACTION_BITS < E::EXPONENT_BITS {
                    match E::EXPONENT_BITS {
                        16 => string.push_str(
                            &ScalarF4E4::from(self).format_scientific_small(base, digits),
                        ),
                        32 => string.push_str(
                            &ScalarF5E5::from(self).format_scientific_small(base, digits),
                        ),
                        64 => string.push_str(
                            &ScalarF6E6::from(self).format_scientific_small(base, digits),
                        ),
                        128 => string.push_str(
                            &ScalarF7E7::from(self).format_scientific_small(base, digits),
                        ),
                        _ => string.push_str(&self.format_scientific_small(base, digits)),
                    }
                } else {
                    string.push_str(&self.format_scientific_small(base, digits))
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
                        let digit = ((scaled - scaled.floor()) * base_scalar + 0.5f32).to_u8();
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
                    let digit_char = if digit < 10 {
                        (digit + b'0') as char
                    } else {
                        (digit - 10 + b'A') as char
                    };
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

                        let digit_char = if digit < 10 {
                            (digit + b'0') as char
                        } else {
                            (digit - 10 + b'A') as char
                        };
                        string.push(digit_char);
                    }
                }
            }
        }

        string.push('⦊');
        string
    }

    fn format_scientific_big(&self, base: u8, digits: isize) -> String {
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

        for _ in 0..digits {
            let digit = scaled.to_u8();
            scaled = (scaled - digit) * base_scalar;

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
            };
            result.push(digit_char);

            if result.len() == 2 {
                result.push('.');
            }

            if scaled.is_zero() {
                break;
            }
        }

        result.push('×');
        let base_char = if base < 10 {
            (base + b'0') as char
        } else {
            (base - 10 + b'A') as char
        };
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
            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
            };
            result.push(digit_char);
        }

        result
    }

    fn format_scientific_small(&self, base: u8, digits: isize) -> String {
        let base_scalar = Self::from(base);

        let magnitude = self.magnitude();
        let mut power = -magnitude.log(base_scalar).floor();
        let mut scaled = magnitude * base_scalar.pow(power);

        // First adjustment loop - while scaled >= base
        while scaled.magnitude() > base_scalar {
            power -= 1;
            scaled = magnitude * base_scalar.pow(power);
        }

        // Second adjustment loop - while scaled < 1, with oscillation detection
        let mut small_coeff_power = None;
        let mut small_coeff_scaled = None;

        while scaled.magnitude() < 1 {
            // Store the current state before incrementing (this might be our fallback)
            if scaled.is_normal() && !scaled.is_zero() {
                small_coeff_power = Some(power);
                small_coeff_scaled = Some(scaled);
            }

            power += 1;
            scaled = magnitude * base_scalar.pow(power);

            // If we jumped to exploded/non-normal, we hit the oscillation
            if !scaled.is_normal() || scaled.exploded() {
                // Use the smaller coefficient approach
                if let (Some(fallback_power), Some(fallback_scaled)) =
                    (small_coeff_power, small_coeff_scaled)
                {
                    power = fallback_power;
                    scaled = fallback_scaled;

                    // Keep scaling up until coefficient >= 1 (proper scientific notation)
                    while scaled < 1 && scaled.is_normal() {
                        scaled = scaled * base_scalar;
                        power = power - 1; // Make exponent more negative
                    }
                }
                break;
            }
        }

        let mut result = String::new();

        if self.is_negative() {
            result.push('-');
        } else {
            result.push('+');
        }

        for _ in 0..digits {
            let digit = scaled.to_u8();
            scaled = (scaled - digit) * base_scalar;

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
            };
            result.push(digit_char);

            if result.len() == 2 {
                result.push('.');
            }

            if scaled.is_zero() {
                break;
            }
        }

        result.push('×');
        let base_char = if base < 10 {
            (base + b'0') as char
        } else {
            (base - 10 + b'A') as char
        };
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
            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
            };
            result.push(digit_char);
        }

        result
    }

    fn format_debug_plain(&self) -> String {
        let mut binary = String::new();
        let mut rotating = self.fraction;
        let middle = F::FRACTION_BITS / 2;

        // Format fraction bits
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
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
        for b in 0..E::EXPONENT_BITS {
            binary.push(if exp_rotating.is_negative() { '1' } else { '0' });
            exp_rotating = exp_rotating.rotate_left(1);
            if b % 8 == 7 || b == 0 {
                binary.push(' ');
            }
        }

        binary
    }

    fn format_debug_fancy(&self) -> String {
        let mut binary = String::new();
        let mut rotating = self.fraction;
        let (unset, set) = self.get_binary_chars();
        let middle = F::FRACTION_BITS / 2;
        let scheme = self.get_colour_scheme();
        let rgb = &scheme.colour;

        // Start fraction colour
        binary.push_str(&format!("\x1B[38;2;{};{};{}m", rgb[0], rgb[1], rgb[2]));

        // Format fraction bits
        for b in 0..F::FRACTION_BITS {
            if F::FRACTION_BITS != 8 && b == middle {
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

        // Reset colour at end
        binary.push_str("\x1B[0m");

        binary
    }

    fn get_colour_scheme(&self) -> &'static ColourScheme {
        if self.is_normal() {
            if self.fraction.is_negative() {
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
