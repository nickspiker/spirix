use core::fmt::{Debug, Display};
use num_traits::{AsPrimitive, NumCast, PrimInt, Signed};

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
pub trait Integer: Copy + Ord + PrimInt + Debug + Display + NumCast + Signed + Inflate {}

// Implementation for standard Rust signed integers only
impl Integer for i8 {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}
impl Integer for i128 {}

// Internal traits and implementations
// These traits are not part of the public API and are used internally for type conversions and bit manipulations.

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
/// Unlike standard Rust conversions that may panic or wrap on overflow, this trait ensures specific handling of out-of-range values:
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

    /// Compare as unsigned (zero-cost: same bits, unsigned flags)
    fn cmp_unsigned(&self, other: &Self) -> core::cmp::Ordering;

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
    ($($t:ty => $u:ty),*) => {
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
                fn cmp_unsigned(&self, other: &Self) -> core::cmp::Ordering {
                    (*self as $u).cmp(&(*other as $u))
                }

                #[inline]
                fn sa<I: FullInt>(self) -> I
                where
                    $t: AsPrimitive<I>,
                {
                    let src_bits = core::mem::size_of::<$t>().wrapping_shl(3);
                    let dst_bits = core::mem::size_of::<I>().wrapping_shl(3);
                    if src_bits < dst_bits {
                        (self.as_() << dst_bits.wrapping_sub(src_bits))
                    } else if src_bits > dst_bits {
                        (self >> src_bits.wrapping_sub(dst_bits)).as_()
                    } else {
                        self.as_()
                    }
     // (self as $u).reverse_bits().as_().reverse_bits() // apparently the compiler isn't smart enough to figure out this is a simple left handed cast so it turns into 38 lines of asm when it could be one op and now I have to write a massive macro, oh wait x-86 just completely forgot
                }
            }
        )*
    }
}

impl_int_convert!(
    i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128, isize => usize,
    u8 => u8, u16 => u16, u32 => u32, u64 => u64, u128 => u128, usize => usize
);

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

/// Wide arithmetic operations for inflate/operate/deflate pipeline. Implemented per stored/wide pair: i8/i16, i16/i32, i32/i64, i64/i128, i128/I256.
pub trait WideOps: Sized + Copy {
    fn w_is_zero(&self) -> bool;
    fn w_is_negative(&self) -> bool;
    fn leading_same(&self) -> isize;
    fn w_leading_zeros(&self) -> isize;
    fn w_leading_ones(&self) -> isize;
    fn w_shl(self, n: isize) -> Self;
    fn w_shr(self, n: isize) -> Self;
    fn w_shr_logical(self, n: isize) -> Self;
    fn w_shl_assign(&mut self, n: isize);
    fn w_add(self, other: Self) -> Self;
    fn w_sub(self, other: Self) -> Self;
    fn w_mul(self, other: Self) -> Self;
    fn w_div(self, other: Self) -> Self;
    fn w_div_unsigned(self, other: Self) -> Self;
    fn w_rem_unsigned(self, other: Self) -> Self;
    fn w_neg(self) -> Self;
    fn w_and(self, other: Self) -> Self;
    fn w_or(self, other: Self) -> Self;
    fn w_xor(self, other: Self) -> Self;
    fn w_not(self) -> Self;
}

/// Restore the implicit sign bits into a wider type for arithmetic.
pub trait Inflate: Sized + Copy {
    type Wide: WideOps + Deflate<Self>;
    fn sign_extend(self) -> Self::Wide;
    fn left_hand_load(self) -> Self::Wide;
    /// Branchless inflate-or-sign-extend. Normal class: inflate (XOR mask). Escaped class: sign_extend (no XOR).
    fn inflate(self, is_normal: bool) -> Self::Wide;
}

/// Extract stored fraction from wide result (take low FRAC bits).
pub trait Deflate<Stored>: Sized {
    fn deflate(self) -> Stored;
}

macro_rules! impl_wide_ops {
    ($wide:ty, $uwide:ty, $stored:ty, $frac:expr) => {
        impl Inflate for $stored {
            type Wide = $wide;

            #[inline]
            fn sign_extend(self) -> $wide {
                self as $wide
            }

            #[inline]
            fn left_hand_load(self) -> $wide {
                (self as $wide) << $frac
            }

            #[inline]
            fn inflate(self, is_normal: bool) -> $wide {
                let wide = self as $wide;
                let mask = (-(is_normal as $wide)) & ((-1 as $wide) << $frac);
                wide ^ mask
            }
        }

        impl Deflate<$stored> for $wide {
            #[inline]
            fn deflate(self) -> $stored {
                self as $stored
            }
        }

        impl WideOps for $wide {
            #[inline]
            fn w_is_zero(&self) -> bool {
                *self == 0
            }
            #[inline]
            fn w_is_negative(&self) -> bool {
                *self < 0
            }
            #[inline]
            fn leading_same(&self) -> isize {
                self.leading_ones().max(self.leading_zeros()) as isize
            }
            #[inline]
            fn w_leading_zeros(&self) -> isize {
                (*self as $uwide).leading_zeros() as isize
            }
            #[inline]
            fn w_leading_ones(&self) -> isize {
                (*self as $uwide).leading_ones() as isize
            }
            #[inline]
            fn w_shl(self, n: isize) -> Self {
                self << n
            }
            #[inline]
            fn w_shr(self, n: isize) -> Self {
                self >> n
            }
            #[inline]
            fn w_shr_logical(self, n: isize) -> Self {
                ((self as $uwide) >> n) as $wide
            }
            #[inline]
            fn w_shl_assign(&mut self, n: isize) {
                *self <<= n;
            }
            #[inline]
            fn w_add(self, other: Self) -> Self {
                <$wide>::wrapping_add(self, other)
            }
            #[inline]
            fn w_sub(self, other: Self) -> Self {
                <$wide>::wrapping_sub(self, other)
            }
            #[inline]
            fn w_mul(self, other: Self) -> Self {
                <$wide>::wrapping_mul(self, other)
            }
            #[inline]
            fn w_div(self, other: Self) -> Self {
                self.wrapping_div(other)
            }
            #[inline]
            fn w_div_unsigned(self, other: Self) -> Self {
                ((self as $uwide) / (other as $uwide)) as $wide
            }
            #[inline]
            fn w_rem_unsigned(self, other: Self) -> Self {
                ((self as $uwide) % (other as $uwide)) as $wide
            }
            #[inline]
            fn w_neg(self) -> Self {
                <$wide>::wrapping_neg(self)
            }
            #[inline]
            fn w_and(self, other: Self) -> Self {
                self & other
            }
            #[inline]
            fn w_or(self, other: Self) -> Self {
                self | other
            }
            #[inline]
            fn w_xor(self, other: Self) -> Self {
                self ^ other
            }
            #[inline]
            fn w_not(self) -> Self {
                !self
            }
        }
    };
}

impl_wide_ops!(i16, u16, i8, 8);
impl_wide_ops!(i32, u32, i16, 16);
impl_wide_ops!(i64, u64, i32, 32);
impl_wide_ops!(i128, u128, i64, 64);

// I256 special case: same interface, different method names
impl Inflate for i128 {
    type Wide = i256::I256;

    #[inline]
    fn sign_extend(self) -> i256::I256 {
        self.into()
    }

    #[inline]
    fn left_hand_load(self) -> i256::I256 {
        let wide: i256::I256 = self.into();
        wide << 128
    }

    #[inline]
    fn inflate(self, is_normal: bool) -> i256::I256 {
        let wide: i256::I256 = self.into();
        if is_normal {
            // Flip bits above FRAC=128. Works regardless of From's extension behavior.
            let mask: i256::I256 = i256::I256::from(-1i128) << 128;
            wide ^ mask
        } else {
            wide
        }
    }
}

impl Deflate<i128> for i256::I256 {
    #[inline]
    fn deflate(self) -> i128 {
        let bytes = self.to_le_bytes();
        i128::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ])
    }
}

impl WideOps for i256::I256 {
    #[inline]
    fn w_is_zero(&self) -> bool {
        *self == i256::I256::from(0i128)
    }
    #[inline]
    fn w_is_negative(&self) -> bool {
        *self < i256::I256::from(0i128)
    }
    #[inline]
    fn leading_same(&self) -> isize {
        self.leading_ones().max(self.leading_zeros()) as isize
    }
    #[inline]
    fn w_leading_zeros(&self) -> isize {
        self.leading_zeros() as isize
    }
    #[inline]
    fn w_leading_ones(&self) -> isize {
        self.leading_ones() as isize
    }
    #[inline]
    fn w_shl(self, n: isize) -> Self {
        self << n
    }
    #[inline]
    fn w_shr(self, n: isize) -> Self {
        self >> n
    }
    #[inline]
    fn w_shr_logical(self, n: isize) -> Self {
        let bytes = self.to_le_bytes();
        let unsigned = i256::U256::from_le_bytes(bytes);
        let shifted = unsigned >> n;
        i256::I256::from_le_bytes(shifted.to_le_bytes())
    }
    #[inline]
    fn w_shl_assign(&mut self, n: isize) {
        *self <<= n;
    }
    #[inline]
    fn w_add(self, other: Self) -> Self {
        i256::I256::wrapping_add(self, other)
    }
    #[inline]
    fn w_sub(self, other: Self) -> Self {
        i256::I256::wrapping_sub(self, other)
    }
    #[inline]
    fn w_mul(self, other: Self) -> Self {
        i256::I256::wrapping_mul(self, other)
    }
    #[inline]
    fn w_div(self, other: Self) -> Self {
        i256::I256::wrapping_div(self, other)
    }
    #[inline]
    fn w_div_unsigned(self, other: Self) -> Self {
        let a = i256::U256::from_le_bytes(self.to_le_bytes());
        let b = i256::U256::from_le_bytes(other.to_le_bytes());
        i256::I256::from_le_bytes((a / b).to_le_bytes())
    }
    #[inline]
    fn w_rem_unsigned(self, other: Self) -> Self {
        let a = i256::U256::from_le_bytes(self.to_le_bytes());
        let b = i256::U256::from_le_bytes(other.to_le_bytes());
        i256::I256::from_le_bytes((a % b).to_le_bytes())
    }
    #[inline]
    fn w_neg(self) -> Self {
        i256::I256::wrapping_neg(self)
    }
    #[inline]
    fn w_and(self, other: Self) -> Self {
        self & other
    }
    #[inline]
    fn w_or(self, other: Self) -> Self {
        self | other
    }
    #[inline]
    fn w_xor(self, other: Self) -> Self {
        self ^ other
    }
    #[inline]
    fn w_not(self) -> Self {
        !self
    }
}
