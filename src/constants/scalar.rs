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
    /// A generic positive exploded Scalar
    const ESCAPED_POS_BIG: Self;
    /// A generic negative exploded Scalar
    const ESCAPED_NEG_BIG: Self;
    /// A generic positive vanished Scalar
    const ESCAPED_POS_SMALL: Self;
    /// A generic negative vanished Scalar
    const ESCAPED_NEG_SMALL: Self;
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
    /// Maximum finite value that can be represented by this type of Scalar.
    pub const MAX: Self = Self {
        fraction: <$f>::MAX_FRACTION,
        exponent: <$e>::MAX_EXPONENT,
    };
    /// Minimum finite value that can be represented by this type of Scalar. Note: Negating this scalar will result in an exploded positive scalar.
    pub const MIN: Self = Self {
        fraction: <$f>::MIN_FRACTION,
        exponent: <$e>::MAX_EXPONENT,
    };
    /// The smallest positive value that can be represented by this type of Scalar. Note: Negating this scalar will result in a vanished negative scalar.
    pub const MIN_POS: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: <$e>::MIN_EXPONENT,
    };
    /// Smallest magnitude negative value that can be represented by this type of Scalar.
    pub const MAX_NEG: Self = Self {
        fraction: !<$f>::POS_ONE_FRACTION,
        exponent: <$e>::MIN_EXPONENT,
    };
    /// Granularity between 1 and 2
    pub const POS_NORMAL_EPSILON: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: (1isize.wrapping_sub(<$f>::FRACTION_BITS as isize)) as $e,
    };
    /// Granularity between -1 and -2
    pub const NEG_NORMAL_EPSILON: Self = Self {
        fraction: <$f>::NEG_ONE_FRACTION,
        exponent: (-(<$f>::FRACTION_BITS as isize)) as $e,
    };
    /// The maximum value that maintains integer contiguity with its neighboring values.
    /// This is one less than MAX_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a+1!=a).
    pub const MAX_CONTIGUOUS: Self = Self {
        fraction: <$f>::MAX_FRACTION,
        exponent: (<$f>::FRACTION_BITS.wrapping_sub(1)) as $e,
    };
    /// The minimum value that maintains integer contiguity with its neighboring values.
    /// This is one more than MIN_FRACTION to ensure the value connects to both its
    /// predecessor and successor in the representable sequence (a-1!=a).
    pub const MIN_CONTIGUOUS: Self = Self {
        fraction: <$f>::MIN_FRACTION + 1,
        exponent: (<$f>::FRACTION_BITS.wrapping_sub(1)) as $e,
    };
    /// Actual Zero, the real deal. Exploded * 0 = 0
    pub const ZERO: Self = Self {
        fraction: 0,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };
    /// Mathematical infinity - the result of division by Zero (1/0).
    /// Unlike IEEE-754's signed infinities, this represents a singular infinity
    /// where the sign is indeterminate. Represented by all fraction bits set (11111111)
    /// with an ambiguous exponent, creating symmetry with ZERO (00000000).
    /// Used for results where magnitude is infinite and direction is ambiguous.
    pub const INFINITY: Self = Self {
        fraction: -1,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };
    /// Exactly one.
    pub const ONE: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: 1,
    };
    /// Exactly Negative one.
    pub const NEG_ONE: Self = Self {
        fraction: <$f>::MIN_FRACTION,
        exponent: 0,
    };
    /// Effectively one (the largest value smaller than one).
    pub const EFFECTIVELY_POS_ONE: Self = Self {
        fraction: <$f>::MAX_FRACTION,
        exponent: 0,
    };
    /// Effectively negative one (the closest value to -1 with magnitude less than 1).
    pub const EFFECTIVELY_NEG_ONE: Self = Self {
        fraction: <$f>::MIN_FRACTION + 1,
        exponent: 0,
    };
    /// Exactly Two.
    pub const TWO: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: 2,
    };
    /// Exacly 1/2.
    pub const HALF: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: 0,
    };
    /// A generic positive exploded Scalar
    pub const ESCAPED_POS_BIG: Self = Self {
        fraction: <$f>::POS_ONE_FRACTION,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };

    /// A generic negative exploded Scalar
    pub const ESCAPED_NEG_BIG: Self = Self {
        fraction: <$f>::NEG_ONE_FRACTION,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };

    /// A generic positive vanished Scalar
    pub const ESCAPED_POS_SMALL: Self = Self {
        fraction: <$f>::POS_SMALL_FRACTION,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };

    /// A generic negative vanished Scalar
    pub const ESCAPED_NEG_SMALL: Self = Self {
        fraction: <$f>::NEG_SMALL_FRACTION,
        exponent: <$e>::AMBIGUOUS_EXPONENT,
    };
    /// Approximately Pi (π ≈ 3.14159265358979323846...)
    pub const PI: Self = Self {
        fraction: (0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 2,
    };
    /// Approximately negative Pi (-π ≈ -3.14159265358979323846...)
    pub const NEG_PI: Self = Self {
        fraction: (-0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 2,
    };
    /// Approximately Tau (2π ≈ 6.28318530717958647693...)
    pub const TAU: Self = Self {
        fraction: (0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 3,
    };
    /// Approximately negative Tau (-2π ≈ -6.28318530717958647693...)
    pub const NEG_TAU: Self = Self {
        fraction: (-0x6487ED5110B4611A62633145C06E0E68i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 3,
    };
    /// Alternative name for TAU (2π)
    pub const TWO_PI: Self = Self::TAU;
    /// Pi divided by two (π/2 ≈ 1.57079632679489661923...)
    pub const HALF_PI: Self = Self {
        fraction: (0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// Negative Pi divided by two (-π/2 ≈ -1.57079632679489661923...)
    pub const NEG_HALF_PI: Self = Self {
        fraction: (-0x6487ED5110B4611A62633145C06E0E68i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// Pi divided by three (π/3 ≈ 1.04719755119659774615...)
    pub const THIRD_PI: Self = Self {
        fraction: (0x430548E0B5CD961196ECCB83D59EB446i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// Pi divided by four (π/4 ≈ 0.78539816339744830962...)
    pub const FOURTH_PI: Self = Self {
        fraction: (0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Negative pi divided by four (-π/4 ≈ 0.78539816339744830962...)
    pub const NEG_FOURTH_PI: Self = Self {
        fraction: (-0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Pi divided by six (π/6 ≈ 0.52359877559829887308...)
    pub const SIXTH_PI: Self = Self {
        fraction: (0x430548E0B5CD961196ECCB83D59EB446i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Pi divided by eight (π/8 ≈ 0.39269908169872415481...)
    pub const EIGHTH_PI: Self = Self {
        fraction: (0x6487ED5110B4611A62633145C06E0E69i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: -1,
    };
    pub const SQRT_PI: Self = Self {
        fraction: (0x716FE246D3BDAA9E70EC1483576E4E0Fi128>>(128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// One divided by pi (1/π ≈ 0.31830988618379067154...)
    pub const ONE_OVER_PI: Self = Self {
        fraction: (0x517CC1B727220A94FE13ABE8FA9A6EE0i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: -1,
    };
    /// Two divided by pi (2/π ≈ 0.63661977236758134308...)
    pub const TWO_OVER_PI: Self = Self {
        fraction: (0x517CC1B727220A94FE13ABE8FA9A6EE0i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// One divided by square root of pi (1/√π ≈ 0.56418958354775628695...)
    pub const ONE_OVER_SQRT_PI: Self = Self {
        fraction: (0x48375D410A6DB446B8EA453FB5FF61A2i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Two divided by square root of pi (2/√π ≈ 1.12837916709551257390...)
    pub const TWO_OVER_SQRT_PI: Self = Self {
        fraction: (0x48375D410A6DB446B8EA453FB5FF61A2i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// One divided by square root of two pi (1/√(2π) ≈ 0.39894228040143267794...)
    pub const ONE_OVER_SQRT_TAU: Self = Self {
        fraction: (0x662114CF50D942343F2CF1402EAE38BFi128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: -1,
    };
    // --- Euler's Number Related Constants ---
    /// Approximately Euler's number (e ≈ 2.71828182845904523536...)
    pub const E: Self = Self {
        fraction: (0x56FC2A2C515DA54D57EE2B10139E9E79i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 2,
    };
    /// Approximately natural logarithm of two (ln(2) ≈ 0.69314718055994530942...)
    pub const LN_TWO: Self = Self {
        fraction: (0x58B90BFBE8E7BCD5E4F1D9CC01F97B57i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Binary logarithm of e (log₂(e) ≈ 1.44269504088896340736...)
    pub const LB_E: Self = Self {
        fraction: (0x5C551D94AE0BF85DDF43FF68348E9F44i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    // --- Square Root Related Constants ---
    /// Approximately square root of two (√2 ≈ 1.41421356237309504880...)
    pub const SQRT_TWO: Self = Self {
        fraction: (0x5A827999FCEF32422CBEC4D9BAA55F4Fi128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// One divided by square root of two (1/√2 ≈ 0.70710678118654752440...)
    pub const ONE_OVER_SQRT_TWO: Self = Self {
        fraction: (0x5A827999FCEF32422CBEC4D9BAA55F4Fi128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Square root of three (√3 ≈ 1.73205080756887729353...)
    pub const SQRT_THREE: Self = Self {
        fraction: (0x6ED9EBA16132A9CEC95D0B5C1E2E0EE2i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// One divided by square root of three (1/√3 ≈ 0.57735026918962576451...)
    pub const ONE_OVER_SQRT_THREE: Self = Self {
        fraction: (0x49E69D1640CC7134863E0792BEC95F41i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    // --- Other Mathematical Constants ---
    /// Approximately Euler-Mascheroni constant (γ ≈ 0.57721566490153286061...)
    pub const EULER_GAMMA: Self = Self {
        fraction: (0x49E233F1BED863D268DF1FC080A965ABi128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
    /// Approximately golden ratio (φ ≈ 1.61803398874989484820...)
    pub const PHI: Self = Self {
        fraction: (0x678DDE6E5FD29F057CE73018173B720Di128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 1,
    };
    /// Catalan's constant (G ≈ 0.915965...)
    pub const CATALAN: Self = Self {
        fraction: (0x753E5C4FA04D742290AC1171BE996863i128 >> (128isize.wrapping_sub(<$f>::FRACTION_BITS))) as $f,
        exponent: 0,
    };
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
            const ESCAPED_POS_BIG: Self = Self::ESCAPED_POS_BIG;
            const ESCAPED_NEG_BIG: Self = Self::ESCAPED_NEG_BIG;
            const ESCAPED_POS_SMALL: Self = Self::ESCAPED_POS_SMALL;
            const ESCAPED_NEG_SMALL: Self = Self::ESCAPED_NEG_SMALL;
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
