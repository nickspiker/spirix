use crate::core::integer::FullInt;
use crate::core::undefined::*;
use crate::implementations::formatting::colours::{ColourScheme, COLOURS};
use crate::*;
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
            Scalar::<F, E>::TWO
                .pow(F::FRACTION_BITS)
                .log(base)
                .floor()
                .to_isize()
        };

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
{
    fn format_circle(&self, base: u8, digits: isize) -> String {
        let mut string = "⦇".to_owned();
        if !self.is_normal() {
            if self.is_undefined() {
                let prefix = self.real.sa();
                string.push_str(Undefined::from_prefix(prefix).symbol);
            } else if self.is_infinite() {
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
            } else if self.r() < base_scalar.pow(-4)
                && self.r() > -base_scalar.pow(-4)
                && self.i() < base_scalar.pow(-4)
                && self.i() > -base_scalar.pow(-4)
            {
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
                let mut real_integer = real.floor();
                let mut real_fraction = real - real_integer;

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
                            digit_count += 1;
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
                        (digit + b'0') as char
                    } else {
                        (digit - 10 + b'A') as char
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
                string.push(',');
                let imaginary = self.i();
                let mut imaginary_integer = imaginary.floor();
                let mut imaginary_fraction = imaginary - imaginary_integer;

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
                            digit_count += 1;
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
                        (digit + b'0') as char
                    } else {
                        (digit - 10 + b'A') as char
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
                string.push('⦈');
            }
        }

        string
    }

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
        } else if scaled_r.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_r.to_u8();
            scaled_r = (scaled_r - digit) * base_scalar;

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
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
        } else if scaled_i.is_positive() {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_i.to_u8();
            scaled_i = (scaled_i - digit) * base_scalar;

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
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
                result.push((base + 48) as char)
            } else {
                result.push((base + 55) as char);
            }
            result.push('^');
            if scale.is_negative() {
                result.push('-');
                let power = (-scale).to_usize();
                let mut pow_digits = Vec::new();
                let mut remaining = power;
                if remaining == 0 {
                    pow_digits.push(48); // '0'
                } else {
                    while remaining != 0 {
                        let remainder = (remaining % base as usize) as u8;
                        if remainder < 10 {
                            pow_digits.push((remainder + 48) as u8);
                        } else {
                            pow_digits.push((remainder + 55) as u8);
                        }
                        remaining /= base as usize;
                    }
                }
                for &digit in pow_digits.iter().rev() {
                    result.push(digit as char);
                }
            } else {
                result.push('+');
                let power = scale.to_usize();
                let mut pow_digits = Vec::new();
                let mut remaining = power;
                if remaining == 0 {
                    pow_digits.push(48); // '0'
                } else {
                    while remaining != 0 {
                        let remainder = (remaining % base as usize) as u8;
                        if remainder < 10 {
                            pow_digits.push((remainder + 48) as u8);
                        } else {
                            pow_digits.push((remainder + 55) as u8);
                        }
                        remaining /= base as usize;
                    }
                }
                for &digit in pow_digits.iter().rev() {
                    result.push(digit as char);
                }
            }
        }

        result
    }

    fn format_scientific_small(&self, base: u8, digits: isize) -> String {
        let base_scalar = Scalar::<F, E>::from(base);

        // Find the smallest non-zero component magnitude for unified scaling
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

        // For small numbers, we want the non-zero minimum
        let min_magnitude = if mag_r.is_zero() && mag_i.is_zero() {
            base_scalar.pow(-4) // fallback
        } else if mag_r.is_zero() {
            mag_i
        } else if mag_i.is_zero() {
            mag_r
        } else {
            mag_r.min(mag_i)
        };

        let mut scale = -min_magnitude.log(base_scalar).floor();
        let mut scaled_r = self.r() * base_scalar.pow(scale);
        let mut scaled_i = self.i() * base_scalar.pow(scale);

        // Adjust scale to ensure largest component is in range [1, base)
        while scaled_r.magnitude().max(scaled_i.magnitude()) >= base_scalar {
            scale = scale - 1;
            scaled_r = self.r() * base_scalar.pow(scale);
            scaled_i = self.i() * base_scalar.pow(scale);
        }
        while scaled_r.magnitude().max(scaled_i.magnitude()) < 1 {
            scale = scale + 1;
            scaled_r = self.r() * base_scalar.pow(scale);
            scaled_i = self.i() * base_scalar.pow(scale);
        }

        let mut result = String::new();

        if scaled_r.is_negative() {
            result.push('-');
        } else {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_r.magnitude().to_u8();
            scaled_r = (scaled_r.magnitude() - digit) * base_scalar;
            if self.r().is_negative() {
                scaled_r = -scaled_r;
            }

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
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
        } else {
            result.push('+');
        }

        for d in 0..digits {
            let digit = scaled_i.magnitude().to_u8();
            scaled_i = (scaled_i.magnitude() - digit) * base_scalar;
            if self.i().is_negative() {
                scaled_i = -scaled_i;
            }

            let digit_char = if digit < 10 {
                (digit + b'0') as char
            } else {
                (digit - 10 + b'A') as char
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
                result.push((base + 48) as char)
            } else {
                result.push((base + 55) as char);
            }
            result.push('^');
            result.push('-');
            let power = scale.to_usize();
            let mut pow_digits = Vec::new();
            let mut remaining = power;
            if remaining == 0 {
                pow_digits.push(48); // '0'
            } else {
                while remaining != 0 {
                    let remainder = (remaining % base as usize) as u8;
                    if remainder < 10 {
                        pow_digits.push((remainder + 48) as u8);
                    } else {
                        pow_digits.push((remainder + 55) as u8);
                    }
                    remaining /= base as usize;
                }
            }
            for &digit in pow_digits.iter().rev() {
                result.push(digit as char);
            }
        }

        result
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
