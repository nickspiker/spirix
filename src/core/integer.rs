use num_traits::{AsPrimitive, NumCast, PrimInt, Signed};
use std::fmt::{Debug, Display};

/// # Integer Trait
///
/// This public trait defines the requirements for numeric types used for the `Circle` and `Scalar` components.
///
/// It restricts types to signed fixed-width integers to ensure:
/// - Precise control over bit layouts and numerical operations
/// - Consistent handling for all variants
/// - Correct exponent representation and scaling
/// - Mathematical comparisons and Rust implementation compatibility
///
/// Only Rust's built-in signed integer types satisfy this requirement, as IEEE floating-point types would clearly break Spirix.
pub trait Integer: Copy + Ord + PrimInt + Debug + Display + NumCast + Signed {}

// Implementation for standard Rust signed integers only
impl Integer for i8 {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}
impl Integer for i128 {}

// Internal traits and implementations
// These traits are not part of the public API and are used internally
// for type conversions and bit manipulations.

/// # FullInt Trait
///
/// Internal trait for numeric type conversion and bit manipulation operations.
///
/// This trait provides unified conversion capabilities across all integer types:
/// - Sign extension when converting to larger types
/// - Truncation when converting to smaller types
/// - Proper handling of signed/unsigned conversions
///
/// The trait handles all low-level bit operations needed for:
/// - Fraction normalization
/// - Exponent adjustments
/// - Prefix manipulations in special states (undefined, exploded, vanished)
/// - Consistent cross-type operations as many Rust operations return unsigned or machine sized integers
pub(crate) trait FullInt:
    Copy
    + PrimInt
    + IntConvert
    + AsPrimitive<i8>
    + AsPrimitive<i16>
    + AsPrimitive<i32>
    + AsPrimitive<i64>
    + AsPrimitive<i128>
    + AsPrimitive<isize>
    + AsPrimitive<u8>
    + AsPrimitive<u16>
    + AsPrimitive<u32>
    + AsPrimitive<u64>
    + AsPrimitive<u128>
    + AsPrimitive<usize>
    + 'static
{
}

/// # IntConvert Trait
///
/// Internal trait providing saturating conversion between integer types.
///
/// Unlike standard Rust conversions that may panic or wrap on overflow,
/// this trait ensures specific handling of out-of-range values:
/// - Returns the target type's maximum value if source is too large
/// - Returns the target type's minimum value if source is too small
/// - Preserves the value exactly when it fits within the target range
///
pub(crate) trait IntConvert {
    /// Converts self to type I, saturating at bounds instead of wrapping.
    ///
    /// When a value cannot be represented in the target type:
    /// - Positive values become the maximum value of type I
    /// - Negative values become the minimum value of type I
    /// - Values within range are converted exactly
    ///
    /// # Examples
    /// ```
    /// // Internal examples (not runnable in public docs)
    /// let large = 1234i16;
    /// let as_i8 = large.saturate::<i8>();    // Becomes 127 (i8::MAX)
    ///
    /// let negative = -42i8;
    /// let as_u8 = negative.saturate::<u8>();  // Becomes 0
    ///
    /// let small = 3i32;
    /// let as_i8 = small.saturate::<i8>();    // Becomes 3 (exact conversion)
    /// ```
    fn saturate<I: FullInt>(self) -> I;

    /// Left aligned cast (as backwards)
    fn sa<I: FullInt>(self) -> I
    where
        Self: AsPrimitive<I>,
        i8: AsPrimitive<I>,
        i16: AsPrimitive<I>,
        i32: AsPrimitive<I>,
        i64: AsPrimitive<I>,
        i128: AsPrimitive<I>,
        isize: AsPrimitive<I>,
        u8: AsPrimitive<I>,
        u16: AsPrimitive<I>,
        u32: AsPrimitive<I>,
        u64: AsPrimitive<I>,
        u128: AsPrimitive<I>,
        usize: AsPrimitive<I>;
}

macro_rules! impl_int_convert {
    ($($t:ty),*) => {
        $(
            impl IntConvert for $t {
                #[inline]
                fn saturate<I: FullInt>(self) -> I {
                    if let Some(val) = I::from(self) {
                        val
                    } else if self > 0 {
                        I::max_value()
                    } else {
                        I::min_value()
                    }
                }

                #[inline]
                fn sa<I: FullInt>(self) -> I
                where
                    $t: AsPrimitive<I>,
                {
                    let src_bits = std::mem::size_of::<$t>().wrapping_mul(8);
                    let dst_bits = std::mem::size_of::<I>().wrapping_mul(8);
                    if src_bits < dst_bits {
                        (self.as_() << dst_bits.wrapping_sub(src_bits))
                    } else if src_bits > dst_bits {
                        (self >> src_bits.wrapping_sub(dst_bits)).as_()
                    } else {
                        self.as_()
                    }
                    // (self as $u).reverse_bits().as_().reverse_bits() // apparently the compiler isn't smart enough to figure out this is a simple left handed as so it turns into 38 lines of asm when it could be one op and now I have to write a massive macro
                }
            }
        )*
    }
}

impl_int_convert!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

impl FullInt for i8 {}
impl FullInt for i16 {}
impl FullInt for i32 {}
impl FullInt for i64 {}
impl FullInt for i128 {}
impl FullInt for isize {}
impl FullInt for u8 {}
impl FullInt for u16 {}
impl FullInt for u32 {}
impl FullInt for u64 {}
impl FullInt for u128 {}
impl FullInt for usize {}
