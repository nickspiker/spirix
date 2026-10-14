// src/conversions/scalar_scalar.rs
use crate::constants::ScalarConstants;
use crate::core::integer::*;
use crate::{Integer, Scalar};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

impl<
        FS: Integer
            + FullInt
            + Shl<isize, Output = FS>
            + Shr<isize, Output = FS>
            + Shl<FS, Output = FS>
            + Shr<FS, Output = FS>
            + Shl<ES, Output = FS>
            + Shr<ES, Output = FS>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub
            + AsPrimitive<FD>,
        ES: Integer
            + FullInt
            + Shl<isize, Output = ES>
            + Shr<isize, Output = ES>
            + Shl<ES, Output = ES>
            + Shr<ES, Output = ES>
            + Shl<FS, Output = ES>
            + Shr<FS, Output = ES>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub
            + AsPrimitive<ED>,
        FD: Integer
            + FullInt
            + Shl<isize, Output = FD>
            + Shr<isize, Output = FD>
            + Shl<FD, Output = FD>
            + Shr<FD, Output = FD>
            + Shl<ED, Output = FD>
            + Shr<ED, Output = FD>
            + AsPrimitive<FS>,
        ED: Integer
            + FullInt
            + Shl<isize, Output = ED>
            + Shr<isize, Output = ED>
            + Shl<ED, Output = ED>
            + Shr<ED, Output = ED>
            + Shl<FD, Output = ED>
            + Shr<FD, Output = ED>
            + AsPrimitive<ES>,
    > From<&Scalar<FS, ES>> for Scalar<FD, ED>
where
    Scalar<FS, ES>: ScalarConstants,
    Scalar<FD, ED>: ScalarConstants,
    // Standard conversion set for source types
    u8: AsPrimitive<FS>,
    u16: AsPrimitive<FS>,
    u32: AsPrimitive<FS>,
    u64: AsPrimitive<FS>,
    u128: AsPrimitive<FS>,
    usize: AsPrimitive<FS>,
    i8: AsPrimitive<FS>,
    i16: AsPrimitive<FS>,
    i32: AsPrimitive<FS>,
    i64: AsPrimitive<FS>,
    i128: AsPrimitive<FS>,
    isize: AsPrimitive<FS>,
    I256: From<FS>,
    u8: AsPrimitive<ES>,
    u16: AsPrimitive<ES>,
    u32: AsPrimitive<ES>,
    u64: AsPrimitive<ES>,
    u128: AsPrimitive<ES>,
    usize: AsPrimitive<ES>,
    i8: AsPrimitive<ES>,
    i16: AsPrimitive<ES>,
    i32: AsPrimitive<ES>,
    i64: AsPrimitive<ES>,
    i128: AsPrimitive<ES>,
    isize: AsPrimitive<ES>,
    I256: From<ES>,
    // Standard conversion set for destination types
    u8: AsPrimitive<FD>,
    u16: AsPrimitive<FD>,
    u32: AsPrimitive<FD>,
    u64: AsPrimitive<FD>,
    u128: AsPrimitive<FD>,
    usize: AsPrimitive<FD>,
    i8: AsPrimitive<FD>,
    i16: AsPrimitive<FD>,
    i32: AsPrimitive<FD>,
    i64: AsPrimitive<FD>,
    i128: AsPrimitive<FD>,
    isize: AsPrimitive<FD>,
    I256: From<FD>,
    u8: AsPrimitive<ED>,
    u16: AsPrimitive<ED>,
    u32: AsPrimitive<ED>,
    u64: AsPrimitive<ED>,
    u128: AsPrimitive<ED>,
    usize: AsPrimitive<ED>,
    i8: AsPrimitive<ED>,
    i16: AsPrimitive<ED>,
    i32: AsPrimitive<ED>,
    i64: AsPrimitive<ED>,
    i128: AsPrimitive<ED>,
    isize: AsPrimitive<ED>,
    I256: From<ED>,
{
    fn from(source: &Scalar<FS, ES>) -> Self {
        if !source.is_normal() {
            // INFINITY needs all-1s fraction in destination width; sa would shift in zeros.
            if source.is_infinite() {
                return Self::INFINITY;
            }
            // ZERO, escaped, undefined: sa preserves the prefix bits (top byte determines class). AMBIG sentinel is exp == 0 under AMBIG=0; previously this returned ED::min_value() which is now the +1.0 binade and corrupted ZERO/escape classes.
            return Self {
                fraction: source.fraction.sa(),
                exponent: ED::zero(),
            };
        }
        // AMBIG=0 cross-width exp conversion: logical k = stored ^ E::MIN. Recover the logical from source, re-encode in destination width. Plain `.as_()` only works when src and dst widths share the same E::MIN bit position, which they don't (i8's MIN is -128 but i16's MIN is -32768). binade_origin() is private outside its defining module, so use E::min_value() directly (semantically equivalent).
        let logical_es: ES = source.exponent ^ ES::min_value();
        let fraction: FD = source.fraction.sa();
        let src_bits = (core::mem::size_of::<ES>() * 8) as isize;
        let dst_bits = (core::mem::size_of::<ED>() * 8) as isize;
        if src_bits <= dst_bits {
            // Widening: logical_es fits in ED. Convert via signed widening then re-XOR for dest's binade origin.
            let logical_ed: ED = logical_es.as_();
            return Self {
                fraction,
                exponent: logical_ed ^ ED::min_value(),
            };
        }
        // Narrowing: clamp logical k to dest's representable range [-ED::MAX, ED::MAX] (the reserved -ED::MAX-1 slot would XOR back into the AMBIG sentinel). Anything outside escapes at the AMBIG sentinel, exploded (high) or vanished (low), keeping its significand as phase.
        let logical_ed_max_as_es: ES = ED::max_value().as_();
        let logical_ed_min_as_es: ES = ES::zero().wrapping_sub(&logical_ed_max_as_es);
        let explodes = logical_es > logical_ed_max_as_es;
        if explodes || logical_es < logical_ed_min_as_es {
            // The raw N0 fraction of a normal is NOT an escape pattern (shifting it arithmetically turned +exploded into -vanished and negatives into zero or undefined). Escapes store the positive significand `1mmm` shifted down by class (exploded `01mmm` = >> 1, vanished `001mm` = >> 2), negatives being the two's complement of that with the canonical poles swapped, exactly as scalar_negate and multiply build them.
            let negative = source.is_negative();
            // |x| significand: positives store it directly; negatives are its two's complement, except the zero fraction (-2^(k+1)) whose magnitude is the 1.0 pole.
            let magnitude: FD = if !negative {
                fraction
            } else if source.fraction == FS::zero() {
                Self::pos_one_normal()
            } else {
                source.fraction.wrapping_neg().sa()
            };
            let class_shift: isize = if explodes { 1 } else { 2 };
            let positive: FD = magnitude.cycle_widen().w_shr_logical(class_shift).deflate();
            let fraction = if !negative {
                positive
            } else if positive == Self::pos_one_exploded() {
                Self::neg_one_exploded()
            } else if positive == Self::pos_one_vanished() {
                Self::neg_one_vanished()
            } else {
                FD::zero() - positive
            };
            return Self {
                fraction,
                exponent: Self::ambiguous_exponent(),
            };
        }
        let logical_ed: ED = logical_es.as_();
        Self {
            fraction,
            exponent: logical_ed ^ ED::min_value(),
        }
    }
}
