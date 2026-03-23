use core::mem::size_of;
pub trait ExponentConstants {
    const EXPONENT_BITS: isize;
    const MAX_EXPONENT: Self;
    const MIN_EXPONENT: Self;
    const AMBIGUOUS_EXPONENT: Self;
    const ZERO: Self;
    const ONE: Self;
    const NEG_ONE: Self;
    const TWO: Self;
}

macro_rules! impl_exponent_constants {
    ($($e:ty),*) => {
        $(
            impl ExponentConstants for $e {
                const EXPONENT_BITS: isize = (size_of::<$e>() as isize).wrapping_mul(8);
                const MAX_EXPONENT: Self = <$e>::MAX;
                const MIN_EXPONENT: Self = <$e>::MIN.wrapping_add(1);
                const AMBIGUOUS_EXPONENT: Self = <$e>::MIN;
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const NEG_ONE: Self = -1;
                const TWO: Self = 2;
            }
        )*
    }
}

impl_exponent_constants!(i8, i16, i32, i64, i128);
