// src/conversions/scalar_scalar.rs
use crate::constants::ScalarConstants;
use crate::core::integer::FullInt;
use crate::{ExponentConstants, FractionConstants, Integer, Scalar};
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
use core::ops::*;

impl<
        FS: Integer
            + FractionConstants
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
            + ExponentConstants
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
            + FractionConstants
            + FullInt
            + Shl<isize, Output = FD>
            + Shr<isize, Output = FD>
            + Shl<FD, Output = FD>
            + Shr<FD, Output = FD>
            + Shl<ED, Output = FD>
            + Shr<ED, Output = FD>
            + AsPrimitive<FS>,
        ED: Integer
            + ExponentConstants
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
        let fraction: FD = source.fraction.sa();
        if !source.is_normal() {
            return Self {
                fraction,
                exponent: ED::AMBIGUOUS_EXPONENT,
            };
        }
        if ES::EXPONENT_BITS <= ED::EXPONENT_BITS {
            let exponent = source.exponent.as_();
            return Self { fraction, exponent };
        }
        if source.exponent > ED::MAX_EXPONENT.as_() {
            let exponent = ED::AMBIGUOUS_EXPONENT;
            return Self { fraction, exponent };
        }
        if source.exponent < ED::MIN_EXPONENT.as_() {
            let fraction = fraction >> 1isize;
            let exponent = ED::AMBIGUOUS_EXPONENT;
            return Self { fraction, exponent };
        }
        let exponent = source.exponent.as_();
        Self { fraction, exponent }
    }
}
