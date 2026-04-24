use crate::{Circle, Integer};
pub trait CircleConstants {
    /// Maximum finite value that can be represented by this type of Circle.
    const MAX: Self;
    /// Minimum finite value that can be represented by this type of Circle.
    const MIN: Self;
    /// The smallest positive value that can be represented by this type of Circle.
    const MIN_POS: Self;
    /// Smallest magnitude negative value that can be represented by this type of Circle.
    const MAX_NEG: Self;
    /// Granularity between 1 and 2
    const POS_NORMAL_EPSILON: Self;
    /// Granularity between -1 and -2
    const NEG_NORMAL_EPSILON: Self;
    /// The maximum value that maintains integer contiguity with its neighboring values.
    /// This is one less than MAX_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a+1!=a).
    const MAX_CONTIGUOUS: Self;
    /// The minimum value that maintains integer contiguity with its neighboring values.
    /// This is one more than MIN_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a-1!=a).
    const MIN_CONTIGUOUS: Self;
    /// Actual Zero, the real deal. Exploded * 0 = 0
    const ZERO: Self;
    /// Mathematical infinity - the result of division by Zero (1/0).
    /// Unlike IEEE-754's signed infinities, this represents a singular infinity
    /// where the sign is indeterminate. Represented by all fraction bits set (11111111)
    /// with an ambiguous exponent, creating symmetry with ZERO (00000000).
    /// Used for results where magnitude is infinite and direction is ambiguous.
    const INFINITY: Self;
    /// Exactly one.
    const ONE: Self;
    /// Exactly negative one.
    const NEG_ONE: Self;
    /// Effectively one (the largest value smaller than one).
    const EFFECTIVELY_POS_ONE: Self;
    /// Effectively negative one (the closest value to -1 with magnitude less than 1).
    const EFFECTIVELY_NEG_ONE: Self;
    /// Exactly two.
    const TWO: Self;
    /// Exactly 1/2.
    const HALF: Self;
    /// The imaginary unit (i).
    const POS_I: Self;
    /// Negative imaginary unit (-i).
    const NEG_I: Self;
    /// Approximately Pi (π ≈ 3.14159265358979323846...)
    const PI: Self;
    /// Approximately negative Pi (-π ≈ -3.14159265358979323846...)
    const NEG_PI: Self;
    /// Approximately Tau (2π ≈ 6.28318530717958647693...)
    const TAU: Self;
    /// Approximately negative Tau (-2π ≈ -6.28318530717958647693...)
    const NEG_TAU: Self;
    /// Alternative name for TAU (2π)
    const TWO_PI: Self;
    /// Pi divided by two (π/2 ≈ 1.57079632679489661923...)
    const PI_OVER_TWO: Self;
    /// Negative Pi divided by two (-π/2 ≈ -1.57079632679489661923...)
    const NEG_PI_OVER_TWO: Self;
    /// Pi divided by three (π/3 ≈ 1.04719755119659774615...)
    const PI_OVER_THREE: Self;
    /// Pi divided by four (π/4 ≈ 0.78539816339744830962...)
    const PI_OVER_FOUR: Self;
    /// Pi divided by six (π/6 ≈ 0.52359877559829887308...)
    const PI_OVER_SIX: Self;
    /// Pi divided by eight (π/8 ≈ 0.39269908169872415481...)
    const PI_OVER_EIGHT: Self;
    /// One divided by pi (1/π ≈ 0.31830988618379067154...)
    const ONE_OVER_PI: Self;
    /// Two divided by pi (2/π ≈ 0.63661977236758134308...)
    const TWO_OVER_PI: Self;
    /// Approximately Euler's number (e ≈ 2.71828182845904523536...)
    const E: Self;
    /// Approximately natural logarithm of two (ln(2) ≈ 0.69314718055994530942...)
    const LN_TWO: Self;
    /// Binary logarithm of e (log(e,2) ≈ 1.44269504088896340736...)
    const LB_E: Self;
    /// Approximately square root of two (√2 ≈ 1.41421356237309504880...)
    const SQRT_TWO: Self;
}
macro_rules! impl_circle_constants {
    ($($f:ty, $e:ty);*) => {
        $(
 impl Circle<$f, $e> {
    /// Maximum finite value that can be represented by this type of Circle.
    pub const MAX: Self = Self {
        real: <$f>::MAX,
        imaginary: 0,
        exponent: <$e>::MAX,
    };
    /// Minimum finite value that can be represented by this type of Circle.
    pub const MIN: Self = Self {
        real: <$f>::MIN,
        imaginary: 0,
        exponent: <$e>::MAX,
    };
    /// The smallest positive value that can be represented by this type of Circle.
    pub const MIN_POS: Self = Self {
        real: (-(<$f>::MIN >> 1)),
        imaginary: 0,
        exponent: (<$e>::MIN + 1),
    };
    /// Smallest magnitude negative value that can be represented by this type of Circle.
    pub const MAX_NEG: Self = Self {
        real: <$f>::MIN,
        imaginary: 0,
        exponent: (<$e>::MIN + 1),
    };
    /// Granularity between 1/2 and 1
    pub const POS_NORMAL_EPSILON: Self = Self {
        real: (-(<$f>::MIN >> 1)),
        imaginary: 0,
        exponent: (2isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize) as isize)) as $e,
    };
    /// Granularity between -1 and -1/2
    pub const NEG_NORMAL_EPSILON: Self = Self {
        real: <$f>::MIN,
        imaginary: 0,
        exponent: (1isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize) as isize)) as $e,
    };
    /// The maximum value that maintains integer contiguity with its neighboring values.
    /// This is one less than MAX_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a+1!=a).
    pub const MAX_CONTIGUOUS: Self = Self {
        real: <$f>::MAX.wrapping_sub(1),
        imaginary: 0,
        exponent: ((((core::mem::size_of::<$e>() * 8) as isize)).wrapping_add(((core::mem::size_of::<$e>() * 8) as isize)).wrapping_sub(1)) as $e,
    };
    /// The minimum value that maintains integer contiguity with its neighboring values.
    /// This is one more than MIN_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a-1!=a).
    pub const MIN_CONTIGUOUS: Self = Self {
        real: <$f>::MIN.wrapping_add(1),
        imaginary: 0,
        exponent: ((((core::mem::size_of::<$e>() * 8) as isize)).wrapping_add(((core::mem::size_of::<$e>() * 8) as isize)).wrapping_sub(1)) as $e,
    };
    /// Actual Zero, the real deal. Exploded * 0 = 0
    pub const ZERO: Self = Self {
        real: 0,
        imaginary: 0,
        exponent: <$e>::MIN,
    };
    /// Mathematical infinity - the result of division by Zero (1/0).
    /// Unlike IEEE-754's signed infinities, this represents a singular infinity
    /// where the sign is indeterminate. Represented by all fraction bits set (11111111)
    /// with an ambiguous exponent, creating symmetry with ZERO (00000000).
    /// Used for results where magnitude is infinite and direction is ambiguous.
    pub const INFINITY: Self = Self {
        real: -1,
        imaginary: -1,
        exponent: <$e>::MIN,
    };
    /// Exactly one.
    pub const ONE: Self = Self {
        real: (-(<$f>::MIN >> 1)),
        imaginary: 0,
        exponent: 1,
    };
    /// Exactly negative one.
    pub const NEG_ONE: Self = Self {
        real: <$f>::MIN,
        imaginary: 0,
        exponent: 0,
    };
    /// Effectively one (the largest value smaller than one).
    pub const EFFECTIVELY_POS_ONE: Self = Self {
        real: <$f>::MAX,
        imaginary: 0,
        exponent: 0,
    };
    /// Effectively negative one (the closest value to -1 with magnitude less than 1).
    pub const EFFECTIVELY_NEG_ONE: Self = Self {
        real: <$f>::MIN.wrapping_add(1),
        imaginary: 0,
        exponent: 0,
    };
    /// Exactly two.
    pub const TWO: Self = Self {
        real: (-(<$f>::MIN >> 1)),
        imaginary: 0,
        exponent: 2,
    };
    /// Exactly 1/2.
    pub const HALF: Self = Self {
        real: (-(<$f>::MIN >> 1)),
        imaginary: 0,
        exponent: 0,
    };
    /// Imaginary unit (i).
    pub const POS_I: Self = Self {
        real: 0,
        imaginary: (-(<$f>::MIN >> 1)),
        exponent: 1,
    };
    /// Negative imaginary unit (-i).
    pub const NEG_I: Self = Self {
        real: 0,
        imaginary: <$f>::MIN,
        exponent: 0,
    };
    pub const PI: Self = Self { real: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 2 };
    pub const NEG_PI: Self = Self { real: -((0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f), imaginary: 0, exponent: 2 };
    pub const TAU: Self = Self { real: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 3 };
    pub const NEG_TAU: Self = Self { real: -((0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f), imaginary: 0, exponent: 3 };
    pub const TWO_PI: Self = Self::TAU;
    pub const PI_OVER_TWO: Self = Self { real: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 1 };
    pub const NEG_PI_OVER_TWO: Self = Self { real: -((0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f), imaginary: 0, exponent: 1 };
    pub const PI_OVER_THREE: Self = Self { real: (0x860A91C16B9B2C232DD99707AB3D688Bu128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 1 };
    pub const PI_OVER_FOUR: Self = Self { real: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 0 };
    pub const PI_OVER_SIX: Self = Self { real: (0x860A91C16B9B2C232DD99707AB3D688Bu128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 0 };
    pub const PI_OVER_EIGHT: Self = Self { real: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: -1 };
    pub const ONE_OVER_PI: Self = Self { real: (0xA2F9836E4E441529FC2757D1F534DDC0u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: -1 };
    pub const TWO_OVER_PI: Self = Self { real: (0xA2F9836E4E441529FC2757D1F534DDC0u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 0 };
    pub const E: Self = Self { real: (0xADF85458A2BB4A9AAFDC5620273D3CF1u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 2 };
    pub const LN_TWO: Self = Self { real: (0xB17217F7D1CF79ABC9E3B39803F2F6AFu128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 0 };
    pub const LB_E: Self = Self { real: (0xB8AA3B295C17F0BBBE87FED0691D3E88u128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 1 };
    pub const SQRT_TWO: Self = Self { real: (0xB504F333F9DE6484597D89B3754ABE9Fu128 >> (129isize.wrapping_sub(((core::mem::size_of::<$f>() * 8) as isize)))) as $f, imaginary: 0, exponent: 1 };
}

        impl CircleConstants for Circle<$f, $e> {
            const MAX: Self = Self::MAX;
            const MIN: Self = Self::MIN;
            const MIN_POS: Self = Self::MIN_POS;
            const MAX_NEG: Self = Self::MAX_NEG;
            const POS_NORMAL_EPSILON: Self = Self::POS_NORMAL_EPSILON;
            const NEG_NORMAL_EPSILON: Self = Self::NEG_NORMAL_EPSILON;
            const MAX_CONTIGUOUS: Self = Self::MAX_CONTIGUOUS;
            const MIN_CONTIGUOUS: Self = Self::MIN_CONTIGUOUS;
            const ZERO: Self = Self::ZERO;
            const INFINITY: Self = Self::INFINITY;
            const ONE: Self = Self::ONE;
            const NEG_ONE: Self = Self::NEG_ONE;
            const EFFECTIVELY_POS_ONE: Self = Self::EFFECTIVELY_POS_ONE;
            const EFFECTIVELY_NEG_ONE: Self = Self::EFFECTIVELY_NEG_ONE;
            const TWO: Self = Self::TWO;
            const HALF: Self = Self::HALF;
            const POS_I: Self = Self::POS_I;
            const NEG_I: Self = Self::NEG_I;
            const PI: Self = Self::PI;
            const NEG_PI: Self = Self::NEG_PI;
            const TAU: Self = Self::TAU;
            const NEG_TAU: Self = Self::NEG_TAU;
            const TWO_PI: Self = Self::TWO_PI;
            const PI_OVER_TWO: Self = Self::PI_OVER_TWO;
            const NEG_PI_OVER_TWO: Self = Self::NEG_PI_OVER_TWO;
            const PI_OVER_THREE: Self = Self::PI_OVER_THREE;
            const PI_OVER_FOUR: Self = Self::PI_OVER_FOUR;
            const PI_OVER_SIX: Self = Self::PI_OVER_SIX;
            const PI_OVER_EIGHT: Self = Self::PI_OVER_EIGHT;
            const ONE_OVER_PI: Self = Self::ONE_OVER_PI;
            const TWO_OVER_PI: Self = Self::TWO_OVER_PI;
            const E: Self = Self::E;
            const LN_TWO: Self = Self::LN_TWO;
            const LB_E: Self = Self::LB_E;
            const SQRT_TWO: Self = Self::SQRT_TWO;
        }
    )*
}
}
impl_circle_constants!(
   i8, i8;
   i16, i8;
   i32, i8;
   i64, i8;
   i128, i8;
   i8, i16;
   i16, i16;
   i32, i16;
   i64, i16;
   i128, i16;
   i8, i32;
   i16, i32;
   i32, i32;
   i64, i32;
   i128, i32;
   i8, i64;
   i16, i64;
   i32, i64;
   i64, i64;
   i128, i64;
   i8, i128;
   i16, i128;
   i32, i128;
   i64, i128;
   i128, i128
);

/// Circle format constants — the encoding's structural anchor points.
/// Explicit sign in MSB, N-1 normalization. The compiler constant-folds these
/// at every call site, so they read as `fn` but cost nothing at runtime.
impl<F: Integer, E: Integer> Circle<F, E> {
    // --- Fraction format (explicit sign) ---
    #[inline]
    pub(crate) fn pos_one_normal() -> F {
        -(F::min_value() >> 1usize)
    }
    #[inline]
    pub(crate) fn neg_one_normal() -> F {
        F::min_value()
    }
    #[inline]
    pub(crate) fn max_fraction() -> F {
        F::max_value()
    }
    #[inline]
    pub(crate) fn min_fraction() -> F {
        F::min_value()
    }
    #[inline]
    pub(crate) fn pos_one_exploded() -> F {
        -(F::min_value() >> 1usize) >> 1usize
    }
    #[inline]
    pub(crate) fn neg_one_exploded() -> F {
        F::min_value() >> 1usize
    }
    #[inline]
    pub(crate) fn pos_one_vanished() -> F {
        -(F::min_value() >> 1usize) >> 2usize
    }
    #[inline]
    pub(crate) fn neg_one_vanished() -> F {
        F::min_value() >> 2usize
    }
    #[inline]
    pub(crate) fn fraction_bits() -> isize {
        (core::mem::size_of::<F>() * 8) as isize
    }

    // --- Exponent (same as Scalar) ---
    #[inline]
    pub(crate) fn ambiguous_exponent() -> E {
        E::min_value()
    }
    #[inline]
    pub(crate) fn max_exponent() -> E {
        E::max_value()
    }
    #[inline]
    pub(crate) fn min_exponent() -> E {
        E::min_value() + E::one()
    }
    #[inline]
    pub(crate) fn exponent_bits() -> isize {
        (core::mem::size_of::<E>() * 8) as isize
    }
}
