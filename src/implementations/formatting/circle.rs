use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar,
    ScalarConstants, ScalarF7E7,
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut base: u8 = 10;
        let mut digits = ScalarF7E7::TWO.pow(F::FRACTION_BITS).log(base).to_usize();

        if let Some(width) = f.width() {
            digits = width;
        }

        if let Some(prec) = f.precision() {
            base = prec as u8;
        }

        if base < 2 || base > 36 {
            return write!(f, "Error: Only bases 2-36 are supported!");
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
            + Shr<E, Output = F>,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>,
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
    Scalar<i128, i128>: From<Scalar<F, E>>,
{
    fn format_circle(&self, base: u8, digits: usize) -> String {
        let mut string = "⦇".to_owned();
        if !self.is_normal() {
            if self.is_undefined() {
                let prefix = self.real.sa();
                string.push_str(Undefined::from_prefix(prefix).symbol);
            } else if self.is_infinite() {
                string.push_str("∞");
            } else if self.exploded() {
                let direction = self.sign();
                let mut mag_r = ScalarF7E7::from(direction.r()).magnitude();
                let mut mag_i = ScalarF7E7::from(direction.i()).magnitude();
                string.push_str("↑");
                let decimal = 1;
                if mag_r.is_normal() {
                    if self.r().fraction.is_negative() {
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
                            string.push((digit + 48) as char)
                        } else {
                            string.push((digit + 55) as char);
                        }
                        mag_r = mag_r.frac() * base;
                    }
                } else {
                    string.push('0');
                }
                string.push(',');
                string.push('↑');
                if mag_i.is_normal() {
                    if self.i().fraction.is_negative() {
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
                            string.push((digit + 48) as char)
                        } else {
                            string.push((digit + 55) as char);
                        }
                        mag_i = mag_i.frac() * base;
                    }
                } else {
                    string.push('0');
                }
            } else if self.vanished() {
                let direction = self.sign();
                let mut mag_r = ScalarF7E7::from(direction.r()).magnitude();
                let mut mag_i = ScalarF7E7::from(direction.i()).magnitude();
                string.push_str("↓");
                let decimal = 1;
                if mag_r.is_normal() {
                    if self.r().fraction.is_negative() {
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
                            string.push((digit + 48) as char)
                        } else {
                            string.push((digit + 55) as char);
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
                    if self.i().fraction.is_negative() {
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
                            string.push((digit + 48) as char)
                        } else {
                            string.push((digit + 55) as char);
                        }
                        mag_i = mag_i.frac() * base;
                    }
                } else {
                    string.push('0');
                }
                string.push('%');
            } else {
                string.push('0');
            }
            string.push('⦊');
        } else {
            let mag_r = ScalarF7E7::from(self.r()).magnitude();
            let mag_i = ScalarF7E7::from(self.i()).magnitude();

            let max = mag_r.max(mag_i);
            let scale = max.log(base).floor();
            let mut scaled_r = mag_r / (ScalarF7E7::ZERO + base).pow(scale);
            let mut scaled_i = mag_i / (ScalarF7E7::ZERO + base).pow(scale);
            let decimal = 1;
            if mag_r.is_normal() {
                if self.r().fraction.is_negative() {
                    string.push('-');
                } else {
                    string.push('+');
                }
                for d in 0..digits {
                    if d == decimal {
                        string.push('.');
                    }
                    let digit = scaled_r.to_u8();
                    if digit < 10 {
                        string.push((digit + 48) as char)
                    } else {
                        string.push((digit + 55) as char);
                    }
                    scaled_r = scaled_r.frac() * base;
                }
            } else {
                string.push('0');
            }
            string.push(',');
            if mag_i.is_normal() {
                if self.i().fraction.is_negative() {
                    string.push('-');
                } else {
                    string.push('+');
                }
                for d in 0..digits {
                    if d == decimal {
                        string.push('.');
                    }
                    let digit = scaled_i.to_u8();
                    if digit < 10 {
                        string.push((digit + 48) as char)
                    } else {
                        string.push((digit + 55) as char);
                    }
                    scaled_i = scaled_i.frac() * base;
                }
            } else {
                string.push('0');
            }
            string.push('⦈');
            if scale.is_normal() {
                string.push('×');
                if base < 10 {
                    string.push((base + 48) as char)
                } else {
                    string.push((base + 55) as char);
                }
                string.push('^');
                let mut power;
                if scale.is_negative() {
                    power = (-scale).to_usize();
                    string.push('-');
                } else {
                    power = scale.to_usize();
                    string.push('+');
                }
                let mut pow_digits = Vec::new();
                while power != 0 {
                    let remainder = (power % base as usize) as u8;
                    if remainder < 10 {
                        pow_digits.push((remainder + 48) as u8);
                    } else {
                        pow_digits.push((remainder + 55) as u8);
                    }
                    power /= base as usize;
                }
                for &digit in pow_digits.iter().rev() {
                    string.push(digit as char);
                }
            }
        }

        string
    }

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

    fn format_debug_fancy(&self) -> String {
        let mut binary = String::new();
        let middle = F::FRACTION_BITS / 2;
        let (unset, set) = self.get_binary_chars();

        // Format real component with appropriate color
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

        // Format imaginary component with appropriate color
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

        // Format exponent bits with their color
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
