use std::mem::size_of;
pub trait ExponentConstants {
    const EXPONENT_BITS: isize;
    const MAX_EXPONENT: Self;
    const MIN_EXPONENT: Self;
    const AMBIGUOUS_EXPONENT: Self;
}

macro_rules! impl_exponent_constants {
    ($($e:ty),*) => {
        $(
            impl ExponentConstants for $e {
                const EXPONENT_BITS: isize = size_of::<$e>() as isize * 8;
                const MAX_EXPONENT: Self = <$e>::MAX;
                const MIN_EXPONENT: Self = <$e>::MIN+1;
                const AMBIGUOUS_EXPONENT: Self = <$e>::MIN;
            }
        )*
    }
}

impl_exponent_constants!(i8, i16, i32, i64, i128);
