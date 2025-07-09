use crate::core::integer::FullInt;
use crate::{
    Circle, CircleConstants, ExponentConstants, FractionConstants, Integer, Scalar, ScalarConstants,
};
use i256::I256;
use num_traits::AsPrimitive;
use std::ops::*;

/// # Circle Type Conversion
///
/// This implementation provides conversion between Circle types of different sizes.
/// It maintains bitwise consistency when converting between different fraction
/// and exponent sizes and preserving special states.
///
/// ## Implementation Details
///
/// The conversion handles several special cases:
/// - Ambiguous values (exploded, vanished, undefined) have their states preserved
/// - Precision is maintained to the maximum extent possible when converting
/// - Phase and orientation information is preserved
/// - If the exponent cannot fit into the destination type, appropriate escaped states are used
///
/// The left handed cast preserves the most significant bits during conversion,
/// ensuring that phase information (like undefined prefixes) is maintained even when
/// converting between different sizes.
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
    > From<&Circle<FS, ES>> for Circle<FD, ED>
where
    Circle<FS, ES>: CircleConstants,
    Circle<FD, ED>: CircleConstants,
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
    /// Converts a Circle from one precision/range type to another
    ///
    /// This implementation handles several conversion cases:
    ///
    /// 0. Escaped values: Preserves escaped states (exploded, vanished, undefined)
    ///
    /// 1. Normal values with representable exponents: Maintains normalization but may
    ///    lose precision if the destination fraction type is smaller than the source
    ///
    /// 2. Values with exponents that exceed destination range: Converts to appropriate
    ///    escaped states (exploded for large values, vanished for small values)
    ///
    /// The left handed cast preserves the most significant bits during conversion,
    /// ensuring that phase information (like undefined prefixes) is maintained even when
    /// converting between different sizes.
    fn from(source: &Circle<FS, ES>) -> Self {
        let real = source.real.sa();
        let imaginary = source.imaginary.sa();

        if !source.is_normal() {
            return Self {
                real,
                imaginary,
                exponent: ED::AMBIGUOUS_EXPONENT,
            };
        }

        if ES::EXPONENT_BITS <= ED::EXPONENT_BITS {
            return Self {
                real,
                imaginary,
                exponent: source.exponent.as_(),
            };
        }
        if source.exponent > ED::MAX_EXPONENT.as_() {
            return Self {
                real,
                imaginary,
                exponent: ED::AMBIGUOUS_EXPONENT,
            };
        }
        if source.exponent < ED::MIN_EXPONENT.as_() {
            let real = real >> 1isize;
            let imaginary = imaginary >> 1isize;
            return Self {
                real,
                imaginary,
                exponent: ED::AMBIGUOUS_EXPONENT,
            };
        }
        let exponent = source.exponent.as_();
        Self {
            real,
            imaginary,
            exponent,
        }
    }
}
