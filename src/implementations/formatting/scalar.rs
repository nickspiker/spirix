use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
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
                .to_usize()
        } else {
            Self::TWO.pow(F::FRACTION_BITS).log(base).floor().to_usize()
        };

        if let Some(width) = f.width() {
            digits = width;
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
    fn format_scalar(&self, base: u8, digits: usize) -> String {
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
            let (power, scale) = if self.is_negative() {
                string.push('-');
                if self == Self::MIN {
                    // Special case, can't negate, just use positive MAX
                    let power = self.log(Self::MAX).floor();
                    let scale = -base_scalar.pow(-power);
                    (power, scale)
                } else {
                    let abs_self = -self; // Make it positive for log calculation
                    let power = abs_self.log(base_scalar).floor();
                    let scale = -base_scalar.pow(-power);
                    (power, scale)
                }
            } else {
                string.push('+');
                let power = self.log(base_scalar).floor();
                let scale = base_scalar.pow(-power);
                (power, scale)
            };

            // Three-way split: Big (scientific), Normal (decimal), Small (scientific)
            let mut exponent_int = power.to_isize();
            let use_scientific = exponent_int > digits as isize - 1 || exponent_int < -5;

            if use_scientific {
                let mut scale = scale;
                let mut normalized = self * scale; // normalized to [1, base)

                // Correct the exponent if logarithm was imprecise
                while normalized >= base_scalar {
                    exponent_int += 1;
                    scale = scale / base_scalar;
                    normalized = normalized / base_scalar;
                }
                while normalized < 1 && !normalized.is_zero() {
                    exponent_int -= 1;
                    scale = scale * base_scalar;
                    normalized = normalized * base_scalar;
                }

                // Apply halvzies rounding: if very close to base, round to 1 and increment exponent
                if normalized >= base_scalar - 0.5 {
                    normalized = Self::ONE;
                    exponent_int += 1;
                }

                let mut first_digit = true;
                for _ in 0..digits {
                    let digit = normalized.to_u8();
                    normalized = normalized - digit;
                    normalized = normalized * base;
                    let digit_char = if digit < 10 {
                        (digit + b'0') as char
                    } else {
                        (digit - 10 + b'A') as char
                    };
                    string.push(digit_char);

                    if first_digit {
                        string.push('.');
                        first_digit = false;
                    }

                    if normalized.is_zero() {
                        break;
                    }
                }
                string.push('×');
                let base_char = if base < 10 {
                    (base + b'0') as char
                } else {
                    (base - 10 + b'A') as char
                };
                string.push(base_char);
                string.push('^');
                string.push_str(&exponent_int.to_string());
            } else {
                // Normal notation: use floor to split integer and fractional parts
                let mut integer_part = self.floor();
                let mut fractional_part = *self - integer_part;

                // Extract integer digits using /base
                let mut int_digits = Vec::new();
                integer_part = integer_part.magnitude();
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
