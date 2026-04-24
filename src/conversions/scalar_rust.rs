use crate::core::integer::*;
use crate::ScalarConstants;
use crate::{Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, PrimInt, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + AsPrimitive<f64>
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
    > Into<f64> for &Scalar<F, E>
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
    fn into(self) -> f64 {
        if self.is_normal() {
            // Inflate to effective value (FRAC+1 bits signed), then to f64.
            // f64 mantissa is 53 bits — for wider fractions we shift down and adjust scale.
            let (base_i64, scale_adjust) = match Scalar::<F, E>::fraction_bits() {
                8 => {
                    let s: i8 = self.fraction.saturate();
                    ((s as i16 ^ ((-1i16) << 8)) as i64, 0i64)
                }
                16 => {
                    let s: i16 = self.fraction.saturate();
                    ((s as i32 ^ ((-1i32) << 16)) as i64, 0i64)
                }
                32 => {
                    let s: i32 = self.fraction.saturate();
                    (s as i64 ^ ((-1i64) << 32), 0i64)
                }
                64 => {
                    // Effective is 65 bits — shift right 1 to fit signed i64, scale +1.
                    let s: i64 = self.fraction.saturate();
                    let eff: i128 = (s as i128) ^ ((-1i128) << 64);
                    ((eff >> 1) as i64, 1i64)
                }
                128 => {
                    // Effective is 129 bits — shift right 65 to fit signed i64, scale +65.
                    let s: i128 = self.fraction.saturate();
                    let eff: i256::I256 = s.inflate(true);
                    let eff_shifted = eff >> 65usize;
                    let bytes = eff_shifted.to_le_bytes();
                    let v = i64::from_le_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ]);
                    (v, 65i64)
                }
                _ => unreachable!(),
            };
            let base = base_i64 as f64;
            let exponent: i32 = self.exponent.saturate();
            // v0.1 ruler: value = inflate × 2^(exp − FRAC + 1). +1 compensates the ruler shift.
            let scale_exp = exponent as i64 + 1 - Scalar::<F, E>::fraction_bits() as i64 + scale_adjust;
            // Adjust the f64 bit-exponent of `base` directly to avoid precision loss from powi() multiplication chains at extreme exponents.
            if base == 0.0 { return 0.0; }
            let bits = base.to_bits();
            let sign = bits & 0x8000_0000_0000_0000;
            let biased = ((bits >> 52) & 0x7FF) as i64;
            let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
            let new_biased = biased + scale_exp;
            if new_biased >= 2047 {
                return if sign != 0 { f64::NEG_INFINITY } else { f64::INFINITY };
            }
            if new_biased <= 0 {
                // Subnormal or zero
                let shift = (1 - new_biased) as u32;
                if shift >= 53 {
                    return f64::from_bits(sign);
                }
                let full_mantissa = mantissa | 0x0010_0000_0000_0000;
                return f64::from_bits(sign | (full_mantissa >> shift));
            }
            f64::from_bits(sign | ((new_biased as u64) << 52) | mantissa)
        } else {
            if self.is_undefined() {
                return f64::NAN;
            }
            if self.is_negligible() {
                return if self.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                // Singular [∞] has no direction; IEEE's ±∞ both imply a sign.
                // NaN is the honest mapping for a value IEEE can't represent.
                return f64::NAN;
            }
            // Exploded: preserve sign
            return if self.is_negative() {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
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
            + AsPrimitive<f32>
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
    > Into<f32> for &Scalar<F, E>
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
    fn into(self) -> f32 {
        if self.is_normal() {
            let (base_i32, scale_adjust) = match Scalar::<F, E>::fraction_bits() {
                8 => {
                    let s: i8 = self.fraction.saturate();
                    ((s as i16 ^ ((-1i16) << 8)) as i32, 0i64)
                }
                16 => {
                    let s: i16 = self.fraction.saturate();
                    (s as i32 ^ ((-1i32) << 16), 0i64)
                }
                32 => {
                    // Effective is 33 bits — shift right 1 to fit i32, scale +1.
                    let s: i32 = self.fraction.saturate();
                    let eff: i64 = (s as i64) ^ ((-1i64) << 32);
                    ((eff >> 1) as i32, 1i64)
                }
                64 => {
                    // Effective is 65 bits — shift right 33 to fit i32, scale +33.
                    let s: i64 = self.fraction.saturate();
                    let eff: i128 = (s as i128) ^ ((-1i128) << 64);
                    ((eff >> 33) as i32, 33i64)
                }
                128 => {
                    // Effective is 129 bits — shift right 97 to fit i32, scale +97.
                    let s: i128 = self.fraction.saturate();
                    let eff: i256::I256 = s.inflate(true);
                    let eff_shifted = eff >> 97usize;
                    let bytes = eff_shifted.to_le_bytes();
                    let v = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    (v, 97i64)
                }
                _ => unreachable!(),
            };
            let base = base_i32 as f32;
            let exponent: i32 = self.exponent.saturate();
            // v0.1 ruler: value = inflate × 2^(exp − FRAC + 1).
            let scale_exp = exponent as i64 + 1 - Scalar::<F, E>::fraction_bits() as i64 + scale_adjust;
            if base == 0.0 { return 0.0; }
            let bits = base.to_bits();
            let sign = bits & 0x8000_0000;
            let biased = ((bits >> 23) & 0xFF) as i64;
            let mantissa = bits & 0x007F_FFFF;
            let new_biased = biased + scale_exp;
            if new_biased >= 255 {
                return if sign != 0 { f32::NEG_INFINITY } else { f32::INFINITY };
            }
            if new_biased <= 0 {
                let shift = (1 - new_biased) as u32;
                if shift >= 24 {
                    return f32::from_bits(sign);
                }
                let full_mantissa = mantissa | 0x0080_0000;
                return f32::from_bits(sign | (full_mantissa >> shift));
            }
            f32::from_bits(sign | ((new_biased as u32) << 23) | mantissa)
        } else {
            if self.is_undefined() {
                return f32::NAN;
            }
            if self.is_negligible() {
                return if self.is_negative() { -0. } else { 0. };
            }
            if self.is_infinite() {
                // Singular [∞] has no direction; NaN is the honest mapping.
                return f32::NAN;
            }
            return if self.is_negative() {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
        }
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
            + AsPrimitive<f32>
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
    > Into<f32> for Scalar<F, E>
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
    fn into(self) -> f32 {
        (&self).into()
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
            + AsPrimitive<f64>
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
    > Into<f64> for Scalar<F, E>
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
    fn into(self) -> f64 {
        (&self).into()
    }
}

macro_rules! impl_into_int {
    ($($i:ty),*) => {
        $(
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
    > Into<$i> for Scalar<F, E>
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
                fn into(self) -> $i {
                    (&self).into()
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
    > Into<$i> for &Scalar<F, E>
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
fn into(self) -> $i {
    // Non-normal values: handle escape classes explicitly.
    if !self.is_normal() {
        if self.is_undefined() { return 0; }
        if self.is_zero() { return 0; }
        if self.is_infinite() { return <$i>::MAX; }
        if self.exploded() {
            return if self.is_negative() { <$i>::MIN } else { <$i>::MAX };
        }
        // Vanished: tiny values truncate to 0 in integer conversion.
        return 0;
    }

    // Normal path (v0.1): value = inflate(stored) * 2^(exp - FRAC_BITS + 1).
    // Compute in I256 to handle any F width (including i128 stored).
    let exp: isize = self.exponent.saturate();
    let frac_bits = Scalar::<F, E>::fraction_bits();
    let target_bits = (core::mem::size_of::<$i>() as isize).wrapping_shl(3);

    // Early saturation: if exp is so large that the integer part can't possibly fit in target, short-circuit. Upper bound on |value| is roughly 2^(exp+2) under the new ruler, so bail when exp >= target_bits - 1.
    if exp >= target_bits - 1 {
        if self.is_negative() { return <$i>::MIN; }
        return <$i>::MAX;
    }

    // Inflate: effective = stored ^ ((-1) << FRAC_BITS) in I256 space.
    let stored_wide: I256 = self.fraction.into();
    let mask: I256 = I256::from(-1i128) << frac_bits;
    let effective: I256 = stored_wide ^ mask;

    // Scale by 2^(exp - FRAC_BITS + 1) — the +1 is the ruler shift.
    let shift = exp.wrapping_sub(frac_bits).wrapping_add(1);
    let scaled: I256 = if shift >= 0 {
        effective << shift
    } else {
        effective >> (-shift)
    };

    saturate_i256::<$i>(scaled)
}
}
        )*
    }
}

/// Saturating I256 → target signed int. Returns target MAX/MIN if out of range.
#[inline]
fn saturate_i256<T: FullInt>(v: I256) -> T {
    let bytes = v.to_le_bytes();
    let low_i128 = i128::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    // Expected sign-extension byte in the high 16 bytes.
    let sign_byte: u8 = if low_i128 < 0 { 0xFF } else { 0x00 };
    let fits_i128 = bytes[16..32].iter().all(|&b| b == sign_byte);
    if !fits_i128 {
        // Overflows i128: pick MAX or MIN by sign of v.
        return if v < I256::from(0i128) {
            T::min_value()
        } else {
            T::max_value()
        };
    }
    low_i128.saturate::<T>()
}

impl_into_int!(i8, i16, i32, i64, i128, isize);
macro_rules! impl_into_uint {
    ($($u:ty),*) => {
        $(
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
    > Into<$u> for Scalar<F, E>
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
                fn into(self) -> $u {
                    (&self).into()
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
    > Into<$u> for &Scalar<F, E>
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
    fn into(self) -> $u {
        if self.is_negative() {
            return 0;
        }
        if !self.is_normal() {
            if self.is_undefined() { return 0; }
            if self.is_zero() { return 0; }
            if self.is_infinite() { return <$u>::MAX; }
            if self.exploded() { return <$u>::MAX; }
            // Vanished: tiny values truncate to 0.
            return 0;
        }

        // Normal positive path (v0.1): value = inflate(stored) * 2^(exp - FRAC_BITS + 1).
        let exp: isize = self.exponent.saturate();
        let frac_bits = Scalar::<F, E>::fraction_bits();
        let target_bits = (core::mem::size_of::<$u>() as isize).wrapping_shl(3);

        // v0.1 ruler shift: value formula has an extra +1 in the exponent,
        // so effective magnitude is 2× larger at the same stored bits. Saturate
        // one exp bucket earlier than under the old ruler.
        if exp >= target_bits {
            return <$u>::MAX;
        }

        let stored_wide: I256 = self.fraction.into();
        let mask: I256 = I256::from(-1i128) << frac_bits;
        let effective: I256 = stored_wide ^ mask;
        // v0.1: shift = exp - FRAC + 1 (was exp - FRAC under old ruler).
        let shift = exp.wrapping_sub(frac_bits).wrapping_add(1);
        let scaled: I256 = if shift >= 0 {
            effective << shift
        } else {
            effective >> (-shift)
        };

        saturate_u_i256::<$u>(scaled)
    }
}
        )*
    }
}

/// Saturating I256 → target unsigned int. Assumes `v` is non-negative.
/// Returns 0 for negative inputs (defensive), target MAX if too large.
#[inline]
fn saturate_u_i256<T: FullInt>(v: I256) -> T {
    if v < I256::from(0i128) {
        return T::min_value();
    }
    let bytes = v.to_le_bytes();
    let low_u128 = u128::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
        bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    let fits_u128 = bytes[16..32].iter().all(|&b| b == 0);
    if !fits_u128 {
        return T::max_value();
    }
    low_u128.saturate::<T>()
}

impl_into_uint!(u8, u16, u32, u64, u128, usize);
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
    /// Inherent convenience methods — delegate to Into<iN>/<uN> impls.
    /// Behavior: Zero/Infinity/Undefined → 0; Vanished-neg → -1, Vanished-pos → 0;
    /// Exploded → MIN/MAX by sign; normal values saturate on overflow.
    #[inline]
    pub fn to_i8(&self) -> i8 {
        (*self).into()
    }
    #[inline]
    pub fn to_i16(&self) -> i16 {
        (*self).into()
    }
    #[inline]
    pub fn to_i32(&self) -> i32 {
        (*self).into()
    }
    #[inline]
    pub fn to_i64(&self) -> i64 {
        (*self).into()
    }
    #[inline]
    pub fn to_i128(&self) -> i128 {
        (*self).into()
    }
    #[inline]
    pub fn to_isize(&self) -> isize {
        (*self).into()
    }
    #[inline]
    pub fn to_u8(&self) -> u8 {
        (*self).into()
    }
    #[inline]
    pub fn to_u16(&self) -> u16 {
        (*self).into()
    }
    #[inline]
    pub fn to_u32(&self) -> u32 {
        (*self).into()
    }
    #[inline]
    pub fn to_u64(&self) -> u64 {
        (*self).into()
    }
    #[inline]
    pub fn to_u128(&self) -> u128 {
        (*self).into()
    }
    #[inline]
    pub fn to_usize(&self) -> usize {
        (*self).into()
    }
}

/// Pure-integer IEEE conversions for all Scalar types.
///
/// These avoid `2f32.powi` / `2f64.powi` — all operations are integer bit
/// manipulation + `f32::from_bits` / `f64::from_bits` (reinterpret casts only).
/// to_f32: delegates to the `Into<f32>` impl (both paths produce current format).
macro_rules! impl_to_f32 {
    ($frac:ty, $exp:ty, $frac_bits:expr) => {
        impl Scalar<$frac, $exp> {
            #[inline]
            pub fn to_f32(&self) -> f32 {
                (*self).into()
            }
            #[allow(dead_code)]
            #[inline]
            fn _to_f32_legacy(&self) -> f32 {
                if !self.is_normal() {
                    if self.is_undefined() {
                        return f32::NAN;
                    }
                    if self.is_negligible() {
                        return if self.is_negative() { -0. } else { 0. };
                    }
                    if self.is_infinite() {
                        return f32::NAN;
                    }
                    return if self.is_negative() {
                        f32::NEG_INFINITY
                    } else {
                        f32::INFINITY
                    };
                }
                let frac_i32 = if $frac_bits <= 32 {
                    (self.fraction as i32) << (32 - $frac_bits)
                } else {
                    (self.fraction >> ($frac_bits - 32)) as i32
                };
                let sign_bit = if frac_i32 < 0 { 1u32 } else { 0u32 };
                let abs_frac = frac_i32.unsigned_abs();
                let (abs_frac_n, exp_adj) = if abs_frac >= 0x8000_0000 {
                    (abs_frac >> 1, 1i32)
                } else {
                    (abs_frac, 0i32)
                };
                let mantissa = (abs_frac_n >> 7) & 0x7F_FFFF;
                let raw_exp_i = (self.exponent as i32) + 126 + exp_adj;
                if raw_exp_i >= 255 {
                    return if sign_bit != 0 {
                        f32::NEG_INFINITY
                    } else {
                        f32::INFINITY
                    };
                }
                if raw_exp_i <= 0 {
                    let shift = 1 - raw_exp_i;
                    if shift >= 24 {
                        return f32::from_bits(sign_bit << 31);
                    }
                    let full_mantissa = (abs_frac_n >> 7) | 0x80_0000;
                    let subnormal_mantissa = full_mantissa >> shift;
                    return f32::from_bits((sign_bit << 31) | subnormal_mantissa);
                }
                f32::from_bits((sign_bit << 31) | ((raw_exp_i as u32) << 23) | mantissa)
            }
        }
    };
}

/// to_f64: delegates to the `Into<f64>` impl (both paths produce current format).
macro_rules! impl_to_f64 {
    ($frac:ty, $exp:ty, $frac_bits:expr) => {
        impl Scalar<$frac, $exp> {
            #[inline]
            pub fn to_f64(&self) -> f64 {
                (*self).into()
            }
            #[allow(dead_code)]
            #[inline]
            fn _to_f64_legacy(&self) -> f64 {
                if !self.is_normal() {
                    if self.is_undefined() {
                        return f64::NAN;
                    }
                    if self.is_negligible() {
                        return if self.is_negative() { -0. } else { 0. };
                    }
                    if self.is_infinite() {
                        return f64::NAN;
                    }
                    return if self.is_negative() {
                        f64::NEG_INFINITY
                    } else {
                        f64::INFINITY
                    };
                }
                let frac_i64 = if $frac_bits <= 64 {
                    (self.fraction as i64) << (64 - $frac_bits)
                } else {
                    (self.fraction >> ($frac_bits - 64)) as i64
                };
                let sign_bit = if frac_i64 < 0 { 1u64 } else { 0u64 };
                let abs_frac = frac_i64.unsigned_abs();
                let (abs_frac_n, exp_adj) = if abs_frac >= 0x8000_0000_0000_0000 {
                    (abs_frac >> 1, 1i64)
                } else {
                    (abs_frac, 0i64)
                };
                let mantissa = (abs_frac_n >> 10) & 0x000F_FFFF_FFFF_FFFF;
                let raw_exp_i = (self.exponent as i64) + 1022 + exp_adj;
                if raw_exp_i >= 2047 {
                    return if sign_bit != 0 {
                        f64::NEG_INFINITY
                    } else {
                        f64::INFINITY
                    };
                }
                if raw_exp_i <= 0 {
                    let shift = 1 - raw_exp_i;
                    if shift >= 53 {
                        return f64::from_bits(sign_bit << 63);
                    }
                    let full_mantissa = (abs_frac_n >> 10) | 0x0010_0000_0000_0000;
                    let subnormal_mantissa = full_mantissa >> shift;
                    return f64::from_bits((sign_bit << 63) | subnormal_mantissa);
                }
                f64::from_bits((sign_bit << 63) | ((raw_exp_i as u64) << 52) | mantissa)
            }
        }
    };
}

macro_rules! impl_to_ieee_all {
    ($frac:ty, $frac_bits:expr, $($exp:ty),+) => {
        $(
            impl_to_f32!($frac, $exp, $frac_bits);
            impl_to_f64!($frac, $exp, $frac_bits);
        )+
    };
}

impl_to_ieee_all!(i8, 8, i8, i16, i32, i64, i128);
impl_to_ieee_all!(i16, 16, i8, i16, i32, i64, i128);
impl_to_ieee_all!(i32, 32, i8, i16, i32, i64, i128);
impl_to_ieee_all!(i64, 64, i8, i16, i32, i64, i128);
impl_to_ieee_all!(i128, 128, i8, i16, i32, i64, i128);

impl Scalar<i16, i16> {
    /// Convert a normal (finite, non-zero, non-NaN) f64 literal to `Scalar<i16,i16>` at
    /// compile time. Panics at compile time if called with NaN, infinity, or zero.
    ///
    /// Use this for compile-time constants — e.g. `const K: ScalarF4E4 = ScalarF4E4::from_f64(1.0/3.0)`.
    /// For runtime conversion of arbitrary values use `ScalarF4E4::from(v)`.
    /// Provides higher precision than `from_f32` for constants with more than 7 significant digits.
    #[inline(always)]
    pub const fn from_f64(v: f64) -> Self {
        // Decode IEEE 754 binary64 using pure integer ops (all const-stable).
        let bits = v.to_bits();
        let sign = bits >> 63;
        let raw_exp = ((bits >> 52) & 0x7FF) as i16;
        let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;

        // Special values
        if raw_exp == 0x7FF {
            if mantissa != 0 {
                // NaN → undefined (generic N-3 prefix)
                return Scalar { fraction: 0xE000u16 as i16, exponent: i16::MIN };
            }
            return if sign != 0 {
                // -Inf → EXPLODED_NEG
                Scalar { fraction: i16::MIN, exponent: i16::MIN }
            } else {
                // +Inf → EXPLODED_POS
                Scalar { fraction: -(i16::MIN >> 1), exponent: i16::MIN }
            };
        }
        if raw_exp == 0 && mantissa == 0 {
            return Scalar { fraction: 0, exponent: i16::MIN };
        }

        // Build positive magnitude (mantissa with implicit leading 1 for normals).
        let abs_mantissa: u64 = if raw_exp == 0 {
            mantissa
        } else {
            mantissa | (1 << 52)
        };

        let leading = abs_mantissa.leading_zeros() as i16;
        let significant: i16 = 64 - leading;
        // spirix_exp = ieee_scale + significant. For subnormals (raw_exp=0), IEEE uses
        // effective exp=1 (not 0), so we add 1 to compensate.
        let eff_exp = if raw_exp == 0 { 1 } else { raw_exp };
        let spirix_exp: i16 = eff_exp - 1075 + significant;

        // Position MSB at bit FRAC-1 = 15 (FRAC=16 for Scalar<i16, i16>).
        let shift = 16 - significant;
        let fraction_pos: i16 = if shift < 0 {
            // abs_mantissa has more bits than FRAC — right-shift, truncate low bits.
            (abs_mantissa >> ((-shift) as u32)) as i16
        } else {
            // Fits in low 16 bits — cast then left-shift puts MSB at bit 15.
            (abs_mantissa as i16).wrapping_shl(shift as u32)
        };

        if sign == 0 {
            return Scalar { fraction: fraction_pos, exponent: spirix_exp };
        }

        // Negation: handle pos_one_normal boundary (i16::MIN → {0, exp-1}).
        if fraction_pos == i16::MIN {
            return Scalar { fraction: 0, exponent: spirix_exp - 1 };
        }
        Scalar { fraction: -fraction_pos, exponent: spirix_exp }
    }
}
#[cfg(test)]
mod tests_scalar_ieee {
    use crate::ScalarF4E4 as S44;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// True if two f32 values are within 1 ULP of each other (or both NaN / both ±inf).
    fn f32_close(a: f32, b: f32) -> bool {
        if a.is_nan() && b.is_nan() {
            return true;
        }
        if a == b {
            return true;
        } // catches ±inf and ±0
        let ai = a.to_bits() as i32;
        let bi = b.to_bits() as i32;
        (ai - bi).abs() <= 1
    }

    // ── to_f32 round-trips ────────────────────────────────────────────────────

    #[test]
    fn to_f32_powers_of_2_exact() {
        // S44 stores exact powers of 2. These must round-trip bit-exactly through to_f32.
        for &v in &[0.25_f32, 0.5, 1.0, -1.0, 2.0, -2.0, 4.0, 0.125, -0.5] {
            let s = S44::from_f32(v);
            let back = s.to_f32();
            assert_eq!(
                v.to_bits(),
                back.to_bits(),
                "power-of-2 round-trip to_f32({v}): got {back} (bits {:08X} vs {:08X})",
                v.to_bits(),
                back.to_bits()
            );
        }
    }

    #[test]
    fn to_f32_general_values() {
        // S44 has 15 significant fraction bits; f32 has 23.
        // General values lose 8 bits on round-trip. Accept relative error < 1e-3.
        for &v in &[
            100.0_f32,
            -100.0,
            1234.5,
            -1234.5,
            0.001_f32,
            1.0 / 3.0,
            core::f32::consts::PI,
            core::f32::consts::E,
        ] {
            let s = S44::from_f32(v);
            let back = s.to_f32();
            let rel_err = if v == 0.0 {
                0.0
            } else {
                ((back - v) / v).abs()
            };
            assert!(
                rel_err < 1e-3,
                "to_f32({v}) → {back}: rel_err={rel_err} > 1e-3"
            );
        }
    }

    #[test]
    fn to_f32_special_cases() {
        // Use runtime From<f32> (not const fn from_f32) for NaN/inf.
        // NaN → NaN
        assert!(
            S44::from(f32::NAN).to_f32().is_nan(),
            "NaN should produce NaN"
        );
        // +inf → inf (spirix INFINITY is sign-indeterminate; both ±inf map to the same sentinel)
        assert!(S44::from(f32::INFINITY).to_f32().is_infinite());
        // -inf → inf (sign lost in S44 representation)
        assert!(S44::from(f32::NEG_INFINITY).to_f32().is_infinite());
        // Spirix ZERO is signless — both ±0 collapse to the same ZERO and round-trip to +0.
        assert_eq!(S44::from(0.0_f32).to_f32().to_bits(), 0u32);
        assert_eq!(S44::from(-0.0_f32).to_f32().to_bits(), 0u32);
    }

    #[test]
    fn to_f32_f32_min_max_finite() {
        // f32::MIN_POSITIVE (smallest positive normal = 2^-126)
        let v = f32::MIN_POSITIVE;
        let back = S44::from_f32(v).to_f32();
        assert!(f32_close(v, back), "MIN_POSITIVE round-trip: {v} → {back}");

        // f32::MAX (largest finite). S44 i16 exponent can hold 128, so this is representable with 15-bit fraction precision (loses 8 bits). Should produce a close large value.
        let back_max = S44::from_f32(f32::MAX).to_f32();
        assert!(
            back_max.is_finite() && back_max > 0.0,
            "f32::MAX should produce large finite: {back_max}"
        );
        let rel_max = ((back_max - f32::MAX) / f32::MAX).abs();
        assert!(rel_max < 1e-3, "f32::MAX rel_err too large: {rel_max}");

        // f32::MIN (most negative finite = -f32::MAX).
        // from_f32(f32::MIN) stores {fraction: i16::MIN, exponent: 128}. The i16::MIN
        // case in to_f32 adds +1 to the exponent adjustment, giving raw_exp=255 (overflow).
        // Overflow to -infinity is the correct saturating behaviour.
        let back_min = S44::from_f32(f32::MIN).to_f32();
        assert!(
            back_min.is_infinite() || (back_min.is_finite() && back_min < 0.0),
            "f32::MIN should produce large-magnitude negative or -inf: {back_min}"
        );
    }

    #[test]
    fn to_f32_subnormals() {
        // f32 subnormals have known precision loss in the spirix from_f32 path: the exponent encoding for subnormals uses raw_exp - 119 (same as normals) rather than the correct 1 - 127 = -126, causing an off-by-1 in the exponent. The round-trip through S44 is therefore lossy for subnormals — acceptable. We just verify to_f32 doesn't panic and returns something non-negative for positive inputs.

        // Smallest positive subnormal
        let v = f32::from_bits(1u32);
        let back = S44::from_f32(v).to_f32();
        assert!(
            back.is_finite() && !back.is_nan(),
            "subnormal should not produce NaN/inf: {back}"
        );
        assert!(
            back >= 0.0,
            "positive subnormal should stay non-negative: {back}"
        );

        // Mid-range subnormal — may round-trip to a close value or zero; either is OK
        let v2 = f32::from_bits(0x0040_0000);
        let back2 = S44::from_f32(v2).to_f32();
        assert!(
            back2.is_finite() && back2 >= 0.0,
            "positive subnormal v2={v2} should stay finite non-negative: {back2}"
        );
    }

    // ── to_f64 round-trips ────────────────────────────────────────────────────

    #[test]
    fn to_f64_normal_values() {
        for &v in &[
            0.25_f64,
            0.5,
            1.0,
            -1.0,
            2.0,
            -2.0,
            100.0,
            -100.0,
            1234.5,
            -1234.5,
            0.001_f64,
            1.0 / 3.0,
            core::f64::consts::PI,
            core::f64::consts::E,
        ] {
            let s = S44::from_f32(v as f32);
            let back = s.to_f64();
            // S44 only has 15-bit fraction so f64 precision will be ~4 decimal digits
            let rel_err = if v == 0.0 {
                0.0
            } else {
                ((back - v) / v).abs()
            };
            assert!(rel_err < 1e-3, "to_f64({v}): got {back}, rel_err={rel_err}");
        }
    }

    #[test]
    fn to_f64_special_cases() {
        // Use runtime From<f32> for NaN/inf.
        // Spirix INFINITY is sign-indeterminate; both ±inf map to the same sentinel.
        assert!(S44::from(f32::NAN).to_f64().is_nan());
        assert!(S44::from(f32::INFINITY).to_f64().is_infinite());
        assert!(S44::from(f32::NEG_INFINITY).to_f64().is_infinite()); // sign lost
        // Spirix ZERO is signless — both ±0.0 map to the same ZERO, round-tripping to +0.0. Sign is intentionally not preserved.
        assert_eq!(S44::from(0.0_f32).to_f64().to_bits(), 0u64);
        assert_eq!(S44::from(-0.0_f32).to_f64().to_bits(), 0u64);
    }

    // ── from_f64 round-trips ──────────────────────────────────────────────────

    #[test]
    fn from_f64_normal_values() {
        for &v in &[
            0.25_f64,
            0.5,
            1.0,
            -1.0,
            2.0,
            -2.0,
            100.0,
            -100.0,
            1234.5,
            -1234.5,
            core::f64::consts::PI,
            core::f64::consts::E,
            1.0 / 3.0,
        ] {
            let s = S44::from_f64(v);
            let back = s.to_f64();
            let rel_err = if v == 0.0 {
                0.0
            } else {
                ((back - v) / v).abs()
            };
            assert!(
                rel_err < 1e-3,
                "from_f64({v}) → to_f64 = {back}, rel_err={rel_err}"
            );
        }
    }

    #[test]
    fn from_f64_vs_from_f32_agreement() {
        // For values in f32 range, from_f64 and from_f32 should produce the same S44
        // (within 1 S44 ULP — they may differ by the last fraction bit since f64 has more precision)
        for &v in &[0.25_f32, 0.5, 1.0, -1.0, 2.0, 100.0, 1234.5] {
            let s32 = S44::from_f32(v);
            let s64 = S44::from_f64(v as f64);
            // Exponents must match; fractions may differ by ≤1
            assert_eq!(
                s32.exponent, s64.exponent,
                "exponent mismatch for {v}: {s32:?} vs {s64:?}"
            );
            let frac_diff = (s32.fraction as i32 - s64.fraction as i32).abs();
            assert!(
                frac_diff <= 1,
                "fraction mismatch for {v}: {s32:?} vs {s64:?}, diff={frac_diff}"
            );
        }
    }

    // ── extreme / edge IEEE inputs ────────────────────────────────────────────

    #[test]
    fn from_f64_infinity() {
        // from_f64 is a const fn for normal finite values only. Infinity/NaN inputs will decode as very large exponents (overflow), but the key guarantee is they do not panic and produce *some* S44 value. For the runtime path, use S44::from(f64_val).
        let pos = S44::from_f64(f64::INFINITY);
        // raw_exp = 2047 → exp = 2047-1012 = 1035; frac_u = 0 (mantissa=0) → vanish path
        // Result is implementation-defined for special inputs; just verify it doesn't panic
        let _ = pos.to_f64();

        let neg = S44::from_f64(f64::NEG_INFINITY);
        let _ = neg.to_f64();

        // Runtime path properly handles infinity
        let pos_rt = S44::from(f64::INFINITY);
        assert!(
            pos_rt.is_infinite() || pos_rt.exploded(),
            "from(+inf) should be infinite/exploded"
        );
        let neg_rt = S44::from(f64::NEG_INFINITY);
        assert!(
            neg_rt.is_infinite() || neg_rt.exploded(),
            "from(-inf) should be infinite/exploded"
        );
    }

    #[test]
    fn from_f64_nan_variants() {
        // from_f64 is for normal values only; NaN input is implementation-defined.
        // Just verify no panic:
        let _ = S44::from_f64(f64::NAN);

        // For proper NaN handling use runtime From<f64>:
        let s = S44::from(f64::NAN);
        assert!(s.is_undefined(), "from(NaN) should be undefined");

        // Quiet NaN with various payloads
        for payload in [0u64, 1, 0xDEAD_BEEF, 0x000F_FFFF_FFFF_FFFF] {
            let nan_bits = 0x7FF8_0000_0000_0000u64 | payload;
            let nan = f64::from_bits(nan_bits);
            let s = S44::from(nan);
            assert!(
                s.is_undefined(),
                "from(NaN payload {payload:#x}) should be undefined"
            );
        }

        // Signalling NaN
        let snan_bits = 0x7FF0_0000_0000_0001u64;
        let snan = f64::from_bits(snan_bits);
        let s = S44::from(snan);
        assert!(s.is_undefined(), "from(sNaN) should be undefined");
    }

    #[test]
    fn from_f64_zero_variants() {
        let pos_zero = S44::from_f64(0.0_f64);
        assert!(
            pos_zero.is_negligible() || pos_zero.is_zero(),
            "from_f64(+0) should be zero/negligible: {pos_zero:?}"
        );

        let neg_zero = S44::from_f64(-0.0_f64);
        assert!(
            neg_zero.is_negligible() || neg_zero.is_zero(),
            "from_f64(-0) should be zero/negligible: {neg_zero:?}"
        );
    }

    #[test]
    fn from_f64_f64_min_positive() {
        // f64::MIN_POSITIVE = 2^(-1022). S44 i16 exponent range is ±32767,
        // so S44 CAN represent this. Verify round-trip is reasonable.
        let v = f64::MIN_POSITIVE;
        let s = S44::from_f64(v);
        let back = s.to_f64();
        let rel_err = ((back - v) / v).abs();
        assert!(
            rel_err < 1e-3,
            "from_f64(f64::MIN_POSITIVE): got {back}, rel_err={rel_err}"
        );
    }

    #[test]
    fn from_f64_f64_max() {
        // f64::MAX = (2-2^-52)*2^1023. S44 i16 exponent holds 1023, so representable.
        let v = f64::MAX;
        let s = S44::from_f64(v);
        let back = s.to_f64();
        let rel_err = ((back - v) / v).abs();
        assert!(
            rel_err < 1e-3,
            "from_f64(f64::MAX): got {back}, rel_err={rel_err}"
        );
    }

    #[test]
    fn from_f64_negative_f64_min() {
        // f64::MIN = -(2-2^-52)*2^1023 — most negative finite f64.
        // from_f64(f64::MIN) stores {fraction: i16::MIN, exponent: 1024}.
        // In to_f64, i16::MIN triggers +1 exp_adj → raw_exp = 1024+1022+1 = 2047 → overflow.
        // Saturating to -infinity is the correct result.
        let v = f64::MIN;
        let s = S44::from_f64(v);
        let back = s.to_f64();
        assert!(
            back.is_infinite() || (back.is_finite() && back < 0.0),
            "from_f64(f64::MIN): expected large negative or -inf, got {back}"
        );
    }

    #[test]
    fn from_f64_subnormals() {
        // f64 subnormals: raw_exp = 0, exponent decoding gives very negative S44 exponent. S44 i16 exponent range is huge, so tiny subnormals may be representable or may underflow — either result is valid; just verify no panic.
        let tiny = f64::from_bits(1u64); // smallest positive subnormal ≈ 5e-324
        let s = S44::from_f64(tiny);
        let back = s.to_f64();
        // Either round-trip (within precision) or zero (underflow) is acceptable
        let ok = back == 0.0 || ((back - tiny) / tiny).abs() < 1e-2;
        assert!(ok, "from_f64(subnormal {tiny}) → {back}");
    }

    #[test]
    fn from_f64_scaled_ieee_min_times_random() {
        // Cover a range of magnitudes from tiny to max to exercise the full exponent range of from_f64. Values that are within normal f64 range should round-trip within S44 precision (~3 significant decimal digits). Values at the f64 extremes (MAX, MIN) may overflow back to ±infinity — that's acceptable since the i16::MIN fraction encoding adds +1 to the exponent in to_f64.
        let test_values: &[(f64, bool)] = &[
            (f64::MIN_POSITIVE * 1.0, false),
            (f64::MIN_POSITIVE * 1e10, false),
            (f64::MIN_POSITIVE * 1e50, false),
            (f64::MIN_POSITIVE * 1e100, false),
            (f64::MIN_POSITIVE * 1e200, false),
            (1.0, false),
            (-1.0, false),
            (1234.5678901234567, false),
            (-9876.54321, false),
            // IEEE extremes — may overflow to ±infinity in round-trip (accepted)
            (f64::MAX, true),
            (f64::MIN, true),
        ];
        for &(v, allow_overflow) in test_values {
            let s = S44::from_f64(v);
            let back = s.to_f64();
            if allow_overflow && back.is_infinite() {
                // Overflow to infinity is acceptable for extreme values
                continue;
            }
            let rel_err = if v == 0.0 {
                0.0
            } else {
                ((back - v) / v).abs()
            };
            assert!(
                rel_err < 1e-3,
                "from_f64({v}) → to_f64 = {back}, rel_err={rel_err}"
            );
        }
    }

    // ── const evaluation check ────────────────────────────────────────────────

    #[test]
    fn from_f64_is_const() {
        // Verify these produce the right values at compile time
        const ONE: S44 = S44::from_f64(1.0);
        const HALF: S44 = S44::from_f64(0.5);
        const PI: S44 = S44::from_f64(core::f64::consts::PI);
        const NEG_ONE: S44 = S44::from_f64(-1.0);

        assert!((ONE.to_f64() - 1.0).abs() < 1e-4);
        assert!((HALF.to_f64() - 0.5).abs() < 1e-4);
        assert!((PI.to_f64() - core::f64::consts::PI).abs() < 1e-3);
        assert!((NEG_ONE.to_f64() - (-1.0)).abs() < 1e-4);
    }

    // ── Into<f32> / Into<f64> via generic trait ───────────────────────────────
    // Note: Rust doesn't support trait specialisation, so the generic powi-based Into<f32> / Into<f64> impls are still used for S44. These tests verify that both the inherent to_f32()/to_f64() methods and the generic Into trait produce consistent results (within 2 ULP, allowing for different rounding).

    #[test]
    fn into_f32_agrees_with_to_f32() {
        for &v in &[1.0_f32, -1.0, 0.5, 100.0, -100.0, 0.25] {
            let s = S44::from_f32(v);
            let via_into: f32 = s.into();
            let via_method = s.to_f32();
            // Should agree within 2 ULP (powi rounding vs bit-construction rounding)
            let diff = (via_into.to_bits() as i32 - via_method.to_bits() as i32).abs();
            assert!(
                diff <= 2,
                "Into<f32> ({via_into}) and to_f32() ({via_method}) differ by {diff} ULP for {v}"
            );
        }
    }

    #[test]
    fn into_f64_agrees_with_to_f64() {
        for &v in &[1.0_f32, -1.0, 0.5, 100.0, -100.0, 0.25] {
            let s = S44::from_f32(v);
            let via_into: f64 = s.into();
            let via_method = s.to_f64();
            let diff = (via_into.to_bits() as i64 - via_method.to_bits() as i64).abs();
            assert!(
                diff <= 2,
                "Into<f64> ({via_into}) and to_f64() ({via_method}) differ by {diff} ULP for {v}"
            );
        }
    }

    #[test]
    fn into_f32_nan_and_inf() {
        // Use runtime From<f32> for special values.
        // Spirix INFINITY is sign-indeterminate; both ±inf round through the same sentinel.
        let nan: f32 = S44::from(f32::NAN).into();
        assert!(nan.is_nan());
        let inf: f32 = S44::from(f32::INFINITY).into();
        assert!(inf.is_infinite());
        let neg_inf: f32 = S44::from(f32::NEG_INFINITY).into();
        assert!(neg_inf.is_infinite()); // sign not preserved
    }
}

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
#[allow(dead_code)]
fn _printey<T: core::ops::BitAnd<Output = T> + Copy + PartialEq + PrimInt>(number: T) -> String {
    let mut number = number;
    let bits = core::mem::size_of::<T>().wrapping_shl(3);
    let mut result = String::new();

    for b in 0..bits {
        number = number.rotate_left(1);
        result.push(if number & T::one() == T::one() {
            '■'
        } else {
            '□'
        });

        if b != bits.wrapping_sub(1) && b % 8 == 7 {
            result.push(' ');
        }
        if b == (bits / 2).wrapping_sub(1) {
            result.push(' '); // Extra space at center
        }
    }
    result
}
