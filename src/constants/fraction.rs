use core::mem::size_of;
pub trait FractionConstants {
    const FRACTION_BITS: isize;

    // Normal: sign is implicit (~stored[MSB])
    const POS_ONE_NORMAL_FRACTION: Self;
    const NEG_ONE_NORMAL_FRACTION: Self;
    const MAX_FRACTION: Self;
    const MIN_FRACTION: Self;

    // Escaped: sign is stored directly in the bit pattern
    const POS_ONE_EXPLODED_FRACTION: Self;
    const NEG_ONE_EXPLODED_FRACTION: Self;
    const POS_ONE_VANISHED_FRACTION: Self;
    const NEG_ONE_VANISHED_FRACTION: Self;

    // Raw integer constants
    const ZERO: Self;
    const ONE: Self;
    const NEG_ONE: Self;
    const TWO: Self;
}

macro_rules! impl_fraction_constants {
    ($($f:ty),*) => {
        $(
            impl FractionConstants for $f {
                const FRACTION_BITS: isize = (size_of::<$f>() as isize).wrapping_mul(8);

                // Normal: sign is implicit (~stored[MSB])
                const POS_ONE_NORMAL_FRACTION: Self = <$f>::MIN;
                const NEG_ONE_NORMAL_FRACTION: Self = 0;
                const MAX_FRACTION: Self = -1;
                const MIN_FRACTION: Self = 0;

                // Escaped: sign is stored directly (01.. pos, 10.. neg, 001.. pos, 110.. neg)
                const POS_ONE_EXPLODED_FRACTION: Self = -(<$f>::MIN >> 1);
                const NEG_ONE_EXPLODED_FRACTION: Self = <$f>::MIN;
                const POS_ONE_VANISHED_FRACTION: Self = Self::POS_ONE_EXPLODED_FRACTION >> 1;
                const NEG_ONE_VANISHED_FRACTION: Self = Self::NEG_ONE_EXPLODED_FRACTION >> 1;

                // Raw integer constants (no sign convention, used for bit arithmetic)
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const NEG_ONE: Self = -1;
                const TWO: Self = 2;
            }
        )*
    }
}

impl_fraction_constants!(i8, i16, i32, i64, i128);
