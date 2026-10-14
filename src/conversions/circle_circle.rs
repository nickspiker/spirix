use crate::core::integer::*;
use crate::{Circle, CircleConstants, Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};

/// # Circle Type Conversion
///
/// This implementation provides conversion between Circle types of different sizes. It maintains bitwise consistency when converting between different fraction and exponent sizes and preserving special states.
///
/// ## Implementation Details
///
/// The conversion handles several special cases:
/// - Ambiguous values (exploded, vanished, undefined) have their states preserved
/// - Precision is maintained to the maximum extent possible when converting
/// - Phase and orientation information is preserved
/// - If the exponent cannot fit into the destination type, appropriate escaped states are used
///
/// The left handed cast preserves the most significant bits during conversion, ensuring that phase information (like undefined prefixes) is maintained even when converting between different sizes.
impl<
        FS: Integer
            + FullInt
            + Shl<isize, Output = FS>
            + Shr<isize, Output = FS>
            + Shl<FS, Output = FS>
            + Shr<FS, Output = FS>
            + Shl<ES, Output = FS>
            + Shr<ES, Output = FS>
            + AsPrimitive<FD>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        ES: Integer
            + FullInt
            + Shl<isize, Output = ES>
            + Shr<isize, Output = ES>
            + Shl<ES, Output = ES>
            + Shr<ES, Output = ES>
            + Shl<FS, Output = ES>
            + Shr<FS, Output = ES>
            + AsPrimitive<ED>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        FD: Integer
            + FullInt
            + Shl<isize, Output = FD>
            + Shr<isize, Output = FD>
            + Shl<FD, Output = FD>
            + Shr<FD, Output = FD>
            + Shl<ED, Output = FD>
            + Shr<ED, Output = FD>
            + AsPrimitive<FS>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        ED: Integer
            + FullInt
            + Shl<isize, Output = ED>
            + Shr<isize, Output = ED>
            + Shl<ED, Output = ED>
            + Shr<ED, Output = ED>
            + Shl<FD, Output = ED>
            + Shr<FD, Output = ED>
            + AsPrimitive<ES>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
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
    /// 1. Normal values with representable exponents: Maintains normalization but may lose precision if the destination fraction type is smaller than the source
    ///
    /// 2. Values with exponents that exceed destination range: Converts to appropriate escaped states (exploded for large values, vanished for small values)
    ///
    /// The left handed cast preserves the most significant bits during conversion, ensuring that phase information (like undefined prefixes) is maintained even when converting between different sizes.
    fn from(source: &Circle<FS, ES>) -> Self {
        let real: FD = source.real.sa();
        let imaginary: FD = source.imaginary.sa();

        if !source.is_normal() {
            // INFINITY needs all-ones components in the destination width; sa would shift in zeros.
            if source.is_infinite() {
                return Self::INFINITY;
            }
            // ZERO, exploded, vanished, undefined: sa keeps the prefix bits (and as much phase as fits), and the class lives at the AMBIG exponent (0).
            return Self {
                real,
                imaginary,
                exponent: ED::zero(),
            };
        }

        // AMBIG=0 cross-width exponent: the true signed exponent is stored ^ E::MIN in each width, so translate true to true (XOR the source MIN, sign-extend or truncate, XOR the destination MIN). A plain `.as_()` of the stored exponent sign-extends the wrong bit.
        let logical_es: ES = source.exponent ^ ES::min_value();
        let src_bits = (core::mem::size_of::<ES>() * 8) as isize;
        let dst_bits = (core::mem::size_of::<ED>() * 8) as isize;
        if src_bits > dst_bits {
            // Narrowing: the destination holds true exponents -ED::MAX..=ED::MAX (true ED::MIN would XOR onto AMBIG). Above that explodes (N1 components as-is), below vanishes (components >> 1), both keeping orientation.
            let logical_ed_max_as_es: ES = ED::max_value().as_();
            let logical_ed_min_as_es: ES = ES::zero().wrapping_sub(&logical_ed_max_as_es);
            if logical_es > logical_ed_max_as_es {
                return Self {
                    real,
                    imaginary,
                    exponent: ED::zero(),
                };
            }
            if logical_es < logical_ed_min_as_es {
                return Self {
                    real: real >> 1isize,
                    imaginary: imaginary >> 1isize,
                    exponent: ED::zero(),
                };
            }
        }
        let logical_ed: ED = logical_es.as_();
        Self {
            real,
            imaginary,
            exponent: logical_ed ^ ED::min_value(),
        }
    }
}
