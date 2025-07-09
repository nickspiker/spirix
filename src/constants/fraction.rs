use std::mem::size_of;
pub trait FractionConstants {
    const FRACTION_BITS: isize;
    const POS_ONE_FRACTION: Self;
    const NEG_ONE_FRACTION: Self;
    const POS_SMALL_FRACTION: Self;
    const NEG_SMALL_FRACTION: Self;
    const MAX_FRACTION: Self;
    const MIN_FRACTION: Self;
}

macro_rules! impl_fraction_constants {
    ($($f:ty),*) => {
        $(
            impl FractionConstants for $f {
                const FRACTION_BITS: isize = size_of::<$f>() as isize * 8;
                const POS_ONE_FRACTION: Self = -(<$f>::MIN >> 1);
                const NEG_ONE_FRACTION: Self = <$f>::MIN;
                const POS_SMALL_FRACTION: Self = Self::POS_ONE_FRACTION >> 1;
                const NEG_SMALL_FRACTION: Self = Self::NEG_ONE_FRACTION >> 1;
                const MAX_FRACTION: Self = <$f>::MAX;
                const MIN_FRACTION: Self = <$f>::MIN;
            }
        )*
    }
}

impl_fraction_constants!(i8, i16, i32, i64, i128);
