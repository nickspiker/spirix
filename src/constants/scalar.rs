use super::ExponentConstants;
use super::FractionConstants;
use crate::Scalar;
pub trait ScalarConstants {
    /// Maximum finite value that can be represented by this type of Scalar.
    const MAX: Self;
    /// Minimum finite value that can be represented by this type of Scalar.
    const MIN: Self;
    /// The smallest positive value that can be represented by this type of Scalar.
    const MIN_POS: Self;
    /// Smallest magnitude negative value that can be represented by this type of Scalar.
    const MAX_NEG: Self;
    /// Granularity between 1 and 2
    const POS_NORMAL_EPSILON: Self;
    /// Granularity between -1 and -2
    const NEG_NORMAL_EPSILON: Self;
    /// The maximum value that maintains integer contiguity with its neighboring values.
    /// This is one less than MAX_FRACTION to ensure the value connects to both its predecessor and successor in the representable sequence (a+1!=a).
    const MAX_CONTIGUOUS: Self;
    /// The minimum value that maintains integer contiguity with its neighboring values.
    /// This is one more than MIN_FRACTION to ensure the value connects to both its predecessor and successor in the representable sequence (a-1!=a).
    const MIN_CONTIGUOUS: Self;
    /// Actual Zero, the real deal. Exploded * 0 = 0
    const ZERO: Self;
    /// Singular infinity — signless, no positive or negative variant. Encoded as all-ones fraction at `AMBIGUOUS_EXPONENT`. A uniform all-ones pattern has no bit transition, so no direction can be derived — the natural encoding for a directionless singularity. Reads as unsigned MAX or signed -1 depending on interpretation; neither is the value, this is a state encoding. The two singularities sit adjacent on the two's complement circle (all-ones + 1 = all-zeros), mirroring their reciprocal relationship: 1/∞ = 0 and 1/0 = ∞.
    const INFINITY: Self;
    /// Exactly one.
    const ONE: Self;
    /// Exactly negative one.
    const NEG_ONE: Self;
    /// Effectively one (the largest value smaller than one).
    const EFFECTIVELY_POS_ONE: Self;
    /// Effectively negative one (the closest value to -1 with magnitude less than 1).
    const EFFECTIVELY_NEG_ONE: Self;
    /// A generic positive exploded Scalar
    const EXPLODED_POS: Self;
    /// A generic negative exploded Scalar
    const EXPLODED_NEG: Self;
    /// A generic positive vanished Scalar
    const VANISHED_POS: Self;
    /// A generic negative vanished Scalar
    const VANISHED_NEG: Self;
    /// Exactly two.
    const TWO: Self;
    /// Exactly 1/2.
    const HALF: Self;
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
    const HALF_PI: Self;
    /// Negative Pi divided by two (-π/2 ≈ -1.57079632679489661923...)
    const NEG_HALF_PI: Self;
    /// Pi divided by three (π/3 ≈ 1.04719755119659774615...)
    const THIRD_PI: Self;
    /// Pi divided by four (π/4 ≈ 0.78539816339744830962...)
    const FOURTH_PI: Self;
    /// Negative pi divided by four (-π/4 ≈ 0.78539816339744830962...)
    const NEG_FOURTH_PI: Self;
    /// Pi divided by six (π/6 ≈ 0.52359877559829887308...)
    const SIXTH_PI: Self;
    /// Pi divided by eight (π/8 ≈ 0.39269908169872415481...)
    const EIGHTH_PI: Self;
    /// One divided by pi (1/π ≈ 0.31830988618379067154...)
    const ONE_OVER_PI: Self;
    /// Two divided by pi (2/π ≈ 0.63661977236758134308...)
    const TWO_OVER_PI: Self;
    /// Square root of pi
    const SQRT_PI: Self;
    /// One divided by square root of pi (1/√π ≈ 0.56418958354775628695...)
    const ONE_OVER_SQRT_PI: Self;
    /// Two divided by square root of pi (2/√π ≈ 1.12837916709551257390...)
    const TWO_OVER_SQRT_PI: Self;
    /// One divided by square root of two pi (1/√(2π) ≈ 0.39894228040143267794...)
    const ONE_OVER_SQRT_TAU: Self;
    /// Approximately Euler's number (e ≈ 2.71828182845904523536...)
    const E: Self;
    /// Approximately natural logarithm of two (ln(2) ≈ 0.69314718055994530942...)
    const LN_TWO: Self;
    /// Binary logarithm of e (log₂(e) ≈ 1.44269504088896340736...)
    const LB_E: Self;
    /// Approximately square root of two (√2 ≈ 1.41421356237309504880...)
    const SQRT_TWO: Self;
    /// One divided by square root of two (1/√2 ≈ 0.70710678118654752440...)
    const ONE_OVER_SQRT_TWO: Self;
    /// Square root of three (√3 ≈ 1.73205080756887729353...)
    const SQRT_THREE: Self;
    /// One divided by square root of three (1/√3 ≈ 0.57735026918962576451...)
    const ONE_OVER_SQRT_THREE: Self;
    /// Approximately Euler-Mascheroni constant (γ ≈ 0.57721566490153286061...)
    const EULER_GAMMA: Self;
    /// Approximately golden ratio (φ ≈ 1.61803398874989484820...)
    const PHI: Self;
    /// Catalan's constant (G ≈ 0.915965...)
    const CATALAN: Self;
}
macro_rules! impl_scalar_constants {
    ($($f:ty, $e:ty);*) => {
        $(
 impl Scalar<$f, $e> {
    pub const MAX: Self = Self { fraction: <$f>::MAX_FRACTION, exponent: <$e>::MAX_EXPONENT };
    pub const MIN: Self = Self { fraction: <$f>::MIN_FRACTION, exponent: <$e>::MAX_EXPONENT };
    pub const MIN_POS: Self = Self { fraction: <$f>::POS_ONE_NORMAL_FRACTION, exponent: <$e>::MIN_EXPONENT };
    pub const MAX_NEG: Self = Self { fraction: !<$f>::POS_ONE_NORMAL_FRACTION, exponent: <$e>::MIN_EXPONENT };
    pub const POS_NORMAL_EPSILON: Self = Self { fraction: <$f>::POS_ONE_NORMAL_FRACTION, exponent: (1isize.wrapping_sub(<$f>::FRACTION_BITS as isize)) as $e };
    pub const NEG_NORMAL_EPSILON: Self = Self { fraction: <$f>::NEG_ONE_NORMAL_FRACTION, exponent: (-(<$f>::FRACTION_BITS as isize)) as $e };
    pub const MAX_CONTIGUOUS: Self = Self { fraction: <$f>::MAX_FRACTION, exponent: <$f>::FRACTION_BITS as $e };
    pub const MIN_CONTIGUOUS: Self = Self { fraction: <$f>::MIN_FRACTION + 1, exponent: <$f>::FRACTION_BITS as $e };
    pub const ZERO: Self = Self { fraction: 0, exponent: <$e>::AMBIGUOUS_EXPONENT };
    pub const INFINITY: Self = Self { fraction: -1, exponent: <$e>::AMBIGUOUS_EXPONENT };
    pub const ONE: Self = Self { fraction: <$f>::POS_ONE_NORMAL_FRACTION, exponent: 1 };
    pub const NEG_ONE: Self = Self { fraction: <$f>::NEG_ONE_NORMAL_FRACTION, exponent: 0 };
    pub const EFFECTIVELY_POS_ONE: Self = Self { fraction: <$f>::MAX_FRACTION, exponent: 0 };
    pub const EFFECTIVELY_NEG_ONE: Self = Self { fraction: <$f>::MIN_FRACTION + 1, exponent: 0 };
    pub const TWO: Self = Self { fraction: <$f>::POS_ONE_NORMAL_FRACTION, exponent: 2 };
    pub const HALF: Self = Self { fraction: <$f>::POS_ONE_NORMAL_FRACTION, exponent: 0 };
    pub const EXPLODED_POS: Self = Self { fraction: <$f>::POS_ONE_EXPLODED_FRACTION, exponent: <$e>::AMBIGUOUS_EXPONENT };
    pub const EXPLODED_NEG: Self = Self { fraction: <$f>::NEG_ONE_EXPLODED_FRACTION, exponent: <$e>::AMBIGUOUS_EXPONENT };
    pub const VANISHED_POS: Self = Self { fraction: <$f>::POS_ONE_VANISHED_FRACTION, exponent: <$e>::AMBIGUOUS_EXPONENT };
    pub const VANISHED_NEG: Self = Self { fraction: <$f>::NEG_ONE_VANISHED_FRACTION, exponent: <$e>::AMBIGUOUS_EXPONENT };

    // All hex constants below are stored fractions derived from basecalc (MPFR), floored to 128 bits.
    // The shift >> (128 - FRACTION_BITS) truncates to the target fraction width. (SA cast)
    // Negative variants use wrapping_neg on the stored fraction.

    // --- Pi family (π/4 fraction shared across power-of-2 multiples) ---
    pub const PI: Self = Self { fraction: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 2 };
    pub const NEG_PI: Self = Self { fraction: (-(0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128) >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 2 };
    pub const TAU: Self = Self { fraction: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 3 };
    pub const NEG_TAU: Self = Self { fraction: (-(0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128) >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 3 };
    pub const TWO_PI: Self = Self::TAU;
    pub const HALF_PI: Self = Self { fraction: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const NEG_HALF_PI: Self = Self { fraction: (-(0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128) >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const FOURTH_PI: Self = Self { fraction: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
    pub const NEG_FOURTH_PI: Self = Self { fraction: (-(0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128) >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
    pub const EIGHTH_PI: Self = Self { fraction: (0xC90FDAA22168C234C4C6628B80DC1CD1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: -1 };

    // --- Pi/3 family ---
    pub const THIRD_PI: Self = Self { fraction: (0x860A91C16B9B2C232DD99707AB3D688Bu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const SIXTH_PI: Self = Self { fraction: (0x860A91C16B9B2C232DD99707AB3D688Bu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };

    // --- 1/π family ---
    pub const ONE_OVER_PI: Self = Self { fraction: (0xA2F9836E4E441529FC2757D1F534DDC0u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: -1 };
    pub const TWO_OVER_PI: Self = Self { fraction: (0xA2F9836E4E441529FC2757D1F534DDC0u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };

    // --- √π family ---
    pub const SQRT_PI: Self = Self { fraction: (0xE2DFC48DA77B553CE1D82906AEDC9C1Fu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const ONE_OVER_SQRT_PI: Self = Self { fraction: (0x906EBA8214DB688D71D48A7F6BFEC344u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
    pub const TWO_OVER_SQRT_PI: Self = Self { fraction: (0x906EBA8214DB688D71D48A7F6BFEC344u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const ONE_OVER_SQRT_TAU: Self = Self { fraction: (0xCC42299EA1B284687E59E2805D5C717Fu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: -1 };

    // --- e family ---
    pub const E: Self = Self { fraction: (0xADF85458A2BB4A9AAFDC5620273D3CF1u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 2 };
    pub const LN_TWO: Self = Self { fraction: (0xB17217F7D1CF79ABC9E3B39803F2F6AFu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
    pub const LB_E: Self = Self { fraction: (0xB8AA3B295C17F0BBBE87FED0691D3E88u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };

    // --- √2 family ---
    pub const SQRT_TWO: Self = Self { fraction: (0xB504F333F9DE6484597D89B3754ABE9Fu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const ONE_OVER_SQRT_TWO: Self = Self { fraction: (0xB504F333F9DE6484597D89B3754ABE9Fu128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };

    // --- √3 family ---
    pub const SQRT_THREE: Self = Self { fraction: (0xDDB3D742C265539D92BA16B83C5C1DC4u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const ONE_OVER_SQRT_THREE: Self = Self { fraction: (0x93CD3A2C8198E2690C7C0F257D92BE83u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };

    // --- Other constants ---
    pub const EULER_GAMMA: Self = Self { fraction: (0x93C467E37DB0C7A4D1BE3F810152CB56u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
    pub const PHI: Self = Self { fraction: (0xCF1BBCDCBFA53E0AF9CE60302E76E41Au128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 1 };
    pub const CATALAN: Self = Self { fraction: (0xEA7CB89F409AE845215822E37D32D0C6u128 as i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f, exponent: 0 };
}

        impl ScalarConstants for Scalar<$f, $e> {
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
            const EXPLODED_POS: Self = Self::EXPLODED_POS;
            const EXPLODED_NEG: Self = Self::EXPLODED_NEG;
            const VANISHED_POS: Self = Self::VANISHED_POS;
            const VANISHED_NEG: Self = Self::VANISHED_NEG;
            const PI: Self = Self::PI;
            const NEG_PI: Self = Self::NEG_PI;
            const TAU: Self = Self::TAU;
            const NEG_TAU: Self = Self::NEG_TAU;
            const TWO_PI: Self = Self::TWO_PI;
            const HALF_PI: Self = Self::HALF_PI;
            const NEG_HALF_PI: Self = Self::NEG_HALF_PI;
            const THIRD_PI: Self = Self::THIRD_PI;
            const FOURTH_PI: Self = Self::FOURTH_PI;
            const NEG_FOURTH_PI: Self = Self::NEG_FOURTH_PI;
            const SIXTH_PI: Self = Self::SIXTH_PI;
            const EIGHTH_PI: Self = Self::EIGHTH_PI;
            const SQRT_PI:Self=Self::SQRT_PI;
            const ONE_OVER_PI: Self = Self::ONE_OVER_PI;
            const TWO_OVER_PI: Self = Self::TWO_OVER_PI;
            const ONE_OVER_SQRT_PI: Self = Self::ONE_OVER_SQRT_PI;
            const TWO_OVER_SQRT_PI: Self = Self::TWO_OVER_SQRT_PI;
            const ONE_OVER_SQRT_TAU: Self = Self::ONE_OVER_SQRT_TAU;
            const E: Self = Self::E;
            const LN_TWO: Self = Self::LN_TWO;
            const LB_E: Self = Self::LB_E;
            const SQRT_TWO: Self = Self::SQRT_TWO;
            const ONE_OVER_SQRT_TWO: Self = Self::ONE_OVER_SQRT_TWO;
            const SQRT_THREE: Self = Self::SQRT_THREE;
            const ONE_OVER_SQRT_THREE: Self = Self::ONE_OVER_SQRT_THREE;
            const EULER_GAMMA: Self = Self::EULER_GAMMA;
            const PHI: Self = Self::PHI;
            const CATALAN: Self = Self::CATALAN;
        }
    )*
}
}
impl_scalar_constants!(
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
