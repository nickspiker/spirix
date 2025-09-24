//! Spirix Undefined Fraction States and Denormalization Levels
//!
//! ```txt
//! Level 0:
//! □□□□□□□□ 0 Exactly Zero
//! ■■■■■■■■ ∞ Singular Infinity
//!
//! L1: Normal Numbers
//! □■XXXXXX Positive real or exploded
//! ■□XXXXXX Negative real or exploded
//!
//! L2: Effectively Zero
//! □□■XXXXX Positive vanished `[↓]`
//! ■■□XXXXX Negative vanished `[↓]` IEEE-754 Negative Zero maps to this
//!
//! L3: Basic Operations
//! □□□XXXXX +, ×, ÷, %, ‰, &, ⊻
//! ■■■XXXXX -, ×, ÷, %, ‰, |,
//!
//! L4: Powers and Roots
//! □□□□XXXX
//! ■■■■XXXX
//!
//! L5: Logarithms
//! □□□□□XXX
//! ■■■■■XXX
//!
//! L6: Trigonometry
//! □□□□□□XX Sine, Arcsine
//! ■■■■■■XX Cosine, Arccosine
//!
//! L7: General and Tangent
//! □□□□□□□■ Tangent
//! ■■■■■■■□ General ℘ (IEEE-754 NaN maps to this)
//!```
//! Note: For fraction sizes greater than 8 bits, extensions can be utilized for more undefined granularity or for future implementation improvements
//!
//! - Undefined prefix allocations
//! ```txt
//! L3 ℘ Basic Operators
//! □□□■■■■■ ℘⬆+⬆
//! ■■■□□□□□ ℘⬆-⬆
//! □□□■■■■□ ℘↓+↓
//! ■■■□□□□■ ℘↓-↓
//! □□□■■■□■
//! ■■■□□□■□
//! □□□■■■□□ ℘⬆+
//! ■■■□□□■■ ℘⬆-
//! □□□■■□■■ ℘⨅∞
//! ■■■□□■□□ ℘±∅
//! □□□■■□■□ ℘⊥⊙
//! ■■■□□■□■ ℘∩
//! □□□■■□□■ ℘⌈
//! ■■■□□■■□ ℘⌊
//! □□□■■□□□ ℘+⬆
//! ■■■□□■■■ ℘-⬆
//! □□□■□■■□ ℘⬆/⬆
//! ■■■□■□□■ ℘⬇/⬇
//! □□□■□■□■ ℘⬆%
//! ■■■□■□■□ ℘⬆‰
//! □□□■□■□□ ℘%↓
//! ■■■□■□■■ ℘‰↓
//! □□□■□□■■ ℘%↑
//! ■■■□■■□□ ℘‰↑
//! □□□■□□■□ ℘&
//! ■■■□■■□■ ℘|
//! □□□■□□□■ ℘⊻
//! ■■■□■■■□
//! □□□■□□□□ ℘⬇×⬆
//! ■■■□■■■■ ℘⬆×⬇
//!
//! L4 ℘ Powers and Roots
//! □□□□■■■■ ℘⬆^
//! ■■■■□□□□ ℘↓^
//! □□□□■■■□ ℘^⬆
//! ■■■■□□□■ ℘^↓
//! □□□□■■□■ ℘-^
//! ■■■■□□■□
//! □□□□■■□□
//! ■■■■□□■■ ℘@1
//! □□□□■□■■
//! ■■■■□■□□
//! □□□□■□■□
//! ■■■■□■□■
//! □□□□■□□■
//! ■■■■□■■□ ℘√-
//! □□□□■□□□ ℘√↑
//! ■■■■□■■■ ℘√↓
//!
//! L5 ℘ Logarithms
//! □□□□□■■■ ℘⬆@
//! ■■■■■□□□ ℘⬇@
//! □□□□□■■□ ℘@⬆
//! ■■■■■□□■ ℘@⬇
//! □□□□□■□■ ℘-@
//! ■■■■■□■□ ℘@-
//! □□□□□■□□
//! ■■■■■□■■
//!
//! L6 ℘ Sine/Cosine
//! □□□□□□■■ ℘s
//! ■■■■■■□□ ℘c
//! □□□□□□■□ ℘S
//! ■■■■■■□■ ℘C
//!
//! L7 ℘ General and Tangent
//! □□□□□□□■ ℘t
//! ■■■■■■■□ General ℘ (IEEE-754 NaN maps to this)
//!
//! Infinity
//! ■■■■■■■■ ∞
//!
//! Zero
//! □□□□□□□□ Actual Zero
//!```
//! `⬆` = Transfinite
//! `↑` = Exploded
//! `↓` = Vanished
//! `⬇` = Negligible
pub struct Undefined {
    pub prefix: i8,
    pub symbol: &'static str,
    pub description: &'static str,
}
impl Undefined {
    // Find the appropriate Undefined instance by prefix
    pub fn from_prefix(prefix: i8) -> &'static Self {
        match prefix {
            x if x == TRANSFINITE_PLUS_TRANSFINITE.prefix => &TRANSFINITE_PLUS_TRANSFINITE,
            x if x == TRANSFINITE_MINUS_TRANSFINITE.prefix => &TRANSFINITE_MINUS_TRANSFINITE,
            x if x == VANISHED_PLUS_VANISHED.prefix => &VANISHED_PLUS_VANISHED,
            x if x == VANISHED_MINUS_VANISHED.prefix => &VANISHED_MINUS_VANISHED,
            x if x == TRANSFINITE_PLUS_FINITE.prefix => &TRANSFINITE_PLUS_FINITE,
            x if x == TRANSFINITE_MINUS_FINITE.prefix => &TRANSFINITE_MINUS_FINITE,
            x if x == CLAMP_UNORDERED.prefix => &CLAMP_UNORDERED,
            x if x == MAX_UNORDERED.prefix => &MAX_UNORDERED,
            x if x == MIN_UNORDERED.prefix => &MIN_UNORDERED,
            x if x == FINITE_PLUS_TRANSFINITE.prefix => &FINITE_PLUS_TRANSFINITE,
            x if x == FINITE_MINUS_TRANSFINITE.prefix => &FINITE_MINUS_TRANSFINITE,
            x if x == TRANSFINITE_DIVIDE_TRANSFINITE.prefix => &TRANSFINITE_DIVIDE_TRANSFINITE,
            x if x == NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix => &NEGLIGIBLE_DIVIDE_NEGLIGIBLE,
            x if x == TRANSFINITE_MODULUS.prefix => &TRANSFINITE_MODULUS,
            x if x == TRANSFINITE_MODULO.prefix => &TRANSFINITE_MODULO,
            x if x == MODULUS_VANISHED.prefix => &MODULUS_VANISHED,
            x if x == MODULO_VANISHED.prefix => &MODULO_VANISHED,
            x if x == MODULUS_EXPLODED.prefix => &MODULUS_EXPLODED,
            x if x == MODULO_EXPLODED.prefix => &MODULO_EXPLODED,
            x if x == AND.prefix => &AND,
            x if x == OR.prefix => &OR,
            x if x == XOR.prefix => &XOR,
            x if x == NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix => &NEGLIGIBLE_MULTIPLY_TRANSFINITE,
            x if x == TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix => &TRANSFINITE_MULTIPLY_NEGLIGIBLE,
            x if x == TRANSFINITE_POWER.prefix => &TRANSFINITE_POWER,
            x if x == NEGLIGIBLE_POWER.prefix => &NEGLIGIBLE_POWER,
            x if x == POWER_TRANSFINITE.prefix => &POWER_TRANSFINITE,
            x if x == POWER_NEGLIGIBLE.prefix => &POWER_NEGLIGIBLE,
            x if x == NEGATIVE_POWER.prefix => &NEGATIVE_POWER,
            x if x == LOG_ONE.prefix => &LOG_ONE,
            x if x == SQRT_NEGATIVE.prefix => &SQRT_NEGATIVE,
            x if x == SQRT_EXPLODED.prefix => &SQRT_EXPLODED,
            x if x == SQRT_VANISHED.prefix => &SQRT_VANISHED,
            x if x == TRANSFINITE_LOG.prefix => &TRANSFINITE_LOG,
            x if x == NEGLIGIBLE_LOG.prefix => &NEGLIGIBLE_LOG,
            x if x == LOG_TRANSFINITE.prefix => &LOG_TRANSFINITE,
            x if x == LOG_NEGLIGIBLE.prefix => &LOG_NEGLIGIBLE,
            x if x == NEGATIVE_LOG.prefix => &NEGATIVE_LOG,
            x if x == LOG_NEGATIVE.prefix => &LOG_NEGATIVE,
            x if x == SINE.prefix => &SINE,
            x if x == COSINE.prefix => &COSINE,
            x if x == ARCSINE.prefix => &ARCSINE,
            x if x == ARCCOSINE.prefix => &ARCCOSINE,
            x if x == TANGENT.prefix => &TANGENT,
            x if x == GENERAL.prefix => &GENERAL,
            _ => &GENERAL, // Default fallback
        }
    }
}
// L3 Basic operators
/// □□□■■■■■
pub const TRANSFINITE_PLUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00011111u8 as i8,
    symbol: "℘ ⬆+⬆",
    description: "Transfinite value addition with transfinite value",
};
/// ■■■□□□□□
pub const TRANSFINITE_MINUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0b11100000u8 as i8,
    symbol: "℘ ⬆-⬆",
    description: "Transfinite value subtraction with transfinite value",
};
/// □□□■■■■□
pub const VANISHED_PLUS_VANISHED: Undefined = Undefined {
    prefix: 0b00011110u8 as i8,
    symbol: "℘ ↓+↓",
    description: "Vanished value addition with vanished value",
};
/// ■■■□□□□■
pub const VANISHED_MINUS_VANISHED: Undefined = Undefined {
    prefix: 0b11100001u8 as i8,
    symbol: "℘ ↓-↓",
    description: "Vanished value subtraction with vanished value",
};
/// □□□■■■□□
pub const TRANSFINITE_PLUS_FINITE: Undefined = Undefined {
    prefix: 0b00011100u8 as i8,
    symbol: "℘ ⬆+",
    description: "Transfinite value addition with finite value",
};
/// ■■■□□□■■
pub const TRANSFINITE_MINUS_FINITE: Undefined = Undefined {
    prefix: 0b11100011u8 as i8,
    symbol: "℘ ⬆-",
    description: "Transfinite value subtraction with finite value",
};
/// □□□■■□■■
pub const FRACTIONAL_INFINITY: Undefined = Undefined {
    prefix: 0b00011011u8 as i8,
    symbol: "℘ ⨅∞",
    description: "Fractional part of Infinity",
};
/// ■■■□□■□□
pub const SIGN_INDETERMINATE: Undefined = Undefined {
    prefix: 0b11100100u8 as i8,
    symbol: "℘ ±∅",
    description: "Sign/direction of Zero or Infinity is indeterminate",
};
/// □□□■■□■□
pub const INDETERMINATE: Undefined = Undefined {
    prefix: 0b00011010u8 as i8,
    symbol: "℘ ⊥⊙",
    description: "Indeterminate Scalar → Circle conversion",
};
/// ■■■□□■□■
pub const CLAMP_UNORDERED: Undefined = Undefined {
    prefix: 0b11100101u8 as i8,
    symbol: "℘ ∩",
    description: "Clamp with non-ordered ambiguous values",
};
/// □□□■■□□■
pub const MAX_UNORDERED: Undefined = Undefined {
    prefix: 0b00011001u8 as i8,
    symbol: "℘ ⌈",
    description: "Maximum of non-ordered ambiguous values",
};
/// ■■■□□■■□
pub const MIN_UNORDERED: Undefined = Undefined {
    prefix: 0b11100110u8 as i8,
    symbol: "℘ ⌊",
    description: "Minimum of non-ordered ambiguous values",
};
/// □□□■■□□□
pub const FINITE_PLUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00011000u8 as i8,
    symbol: "℘ +⬆",
    description: "Finite value addition with transfinite value",
};
/// ■■■□□■■■
pub const FINITE_MINUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0b11100111u8 as i8,
    symbol: "℘ -⬆",
    description: "Finite value subtraction with transfinite value",
};
/// □□□■□■■□
pub const TRANSFINITE_DIVIDE_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00010110u8 as i8,
    symbol: "℘ ⬆/⬆",
    description: "Transfinite value division by transfinite value",
};
/// ■■■□■□□■
pub const NEGLIGIBLE_DIVIDE_NEGLIGIBLE: Undefined = Undefined {
    prefix: 0b11101001u8 as i8,
    symbol: "℘ ⬇/⬇",
    description: "Negligible value division by negligible value",
};
/// □□□■□■□■
pub const TRANSFINITE_MODULUS: Undefined = Undefined {
    prefix: 0b00010101u8 as i8,
    symbol: "℘ ⬆%",
    description: "Transfinite value modulus operation",
};
/// ■■■□■□■□
pub const TRANSFINITE_MODULO: Undefined = Undefined {
    prefix: 0b11101010u8 as i8,
    symbol: "℘ ⬆‰",
    description: "Transfinite value modulo operation",
};
/// □□□■□■□□
pub const MODULUS_VANISHED: Undefined = Undefined {
    prefix: 0b00010100u8 as i8,
    symbol: "℘ %↓",
    description: "Finite value modulus with vanished value",
};
/// ■■■□■□■■
pub const MODULO_VANISHED: Undefined = Undefined {
    prefix: 0b11101011u8 as i8,
    symbol: "℘ ‰↓",
    description: "Finite value modulo with vanished value",
};
/// □□□■□□■■
pub const MODULUS_EXPLODED: Undefined = Undefined {
    prefix: 0b00010011u8 as i8,
    symbol: "℘ %↑",
    description: "Modulus with exploded denominator and mismatched signs",
};
/// ■■■□■■□□
pub const MODULO_EXPLODED: Undefined = Undefined {
    prefix: 0b11101100u8 as i8,
    symbol: "℘ ‰↑",
    description: "Modulo with exploded denominator and mismatched signs",
};
/// □□□■□□■□
pub const AND: Undefined = Undefined {
    prefix: 0b00010010u8 as i8,
    symbol: "℘ &",
    description: "Logical AND with escaped value",
};
/// ■■■□■■□■
pub const OR: Undefined = Undefined {
    prefix: 0b11101101u8 as i8,
    symbol: "℘ |",
    description: "Logical OR with escaped value",
};
/// □□□■□□□■
pub const XOR: Undefined = Undefined {
    prefix: 0b00010001u8 as i8,
    symbol: "℘ ⊻",
    description: "Logical XOR with escaped value",
};
/// □□□■□□□□
pub const NEGLIGIBLE_MULTIPLY_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00010000u8 as i8,
    symbol: "℘ ⬇×⬆",
    description: "Negligible value multiplication with transfinite value",
};
/// ■■■□■■■■
pub const TRANSFINITE_MULTIPLY_NEGLIGIBLE: Undefined = Undefined {
    prefix: 0b11101111u8 as i8,
    symbol: "℘ ⬆×⬇",
    description: "Transfinite value multiplication with negligible value",
};

// L4 Powers and roots
/// □□□□■■■■
pub const TRANSFINITE_POWER: Undefined = Undefined {
    prefix: 0b00001111u8 as i8,
    symbol: "℘ ⬆^",
    description: "Transfinite value raised to power",
};
/// ■■■■□□□□
pub const NEGLIGIBLE_POWER: Undefined = Undefined {
    prefix: 0b11110000u8 as i8,
    symbol: "℘ ⬇^",
    description: "Vanished value raised to power",
};
/// □□□□■■■□
pub const POWER_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00001110u8 as i8,
    symbol: "℘ ^⬆",
    description: "Value raised to transfinite power",
};
/// ■■■■□□□■
pub const POWER_NEGLIGIBLE: Undefined = Undefined {
    prefix: 0b11110001u8 as i8,
    symbol: "℘ ^⬇",
    description: "Value raised to vanished power",
};
/// □□□□■■□■
pub const NEGATIVE_POWER: Undefined = Undefined {
    prefix: 0b00001101u8 as i8,
    symbol: "℘ -^",
    description: "Negative value raised to irrational power",
};
/// ■■■■□□■■
pub const LOG_ONE: Undefined = Undefined {
    prefix: 0b11110011u8 as i8,
    symbol: "℘ @1",
    description: "Logarithm base One",
};
/// ■■■■□■■□
pub const SQRT_NEGATIVE: Undefined = Undefined {
    prefix: 0b11110110u8 as i8,
    symbol: "℘ √-",
    description: "Square root of negative value",
};
/// □□□□■□□□
pub const SQRT_EXPLODED: Undefined = Undefined {
    prefix: 0b00001000 as i8,
    symbol: "℘ √↑",
    description: "Square root of transfinite value",
};
/// ■■■■□■■■
pub const SQRT_VANISHED: Undefined = Undefined {
    prefix: 0b11110111u8 as i8,
    symbol: "℘ √↓",
    description: "Square root of vanished value",
};

// L5 Logarithms
/// □□□□□■■■
pub const TRANSFINITE_LOG: Undefined = Undefined {
    prefix: 0b00000111u8 as i8,
    symbol: "℘ ⬆@",
    description: "Logarithm of transfinite value",
};
/// ■■■■■□□□
pub const NEGLIGIBLE_LOG: Undefined = Undefined {
    prefix: 0b11111000u8 as i8,
    symbol: "℘ ⬇@",
    description: "Logarithm of negligible value",
};
/// □□□□□■■□
pub const LOG_TRANSFINITE: Undefined = Undefined {
    prefix: 0b00000110u8 as i8,
    symbol: "℘ @⬆",
    description: "Logarithm with transfinite base",
};
/// ■■■■■□□■
pub const LOG_NEGLIGIBLE: Undefined = Undefined {
    prefix: 0b11111001u8 as i8,
    symbol: "℘ @⬇",
    description: "Logarithm with negligible base",
};
/// □□□□□■□■
pub const NEGATIVE_LOG: Undefined = Undefined {
    prefix: 0b00000101u8 as i8,
    symbol: "℘ -@",
    description: "Logarithm of negative value",
};
/// ■■■■■□■□
pub const LOG_NEGATIVE: Undefined = Undefined {
    prefix: 0b11111010u8 as i8,
    symbol: "℘ @-",
    description: "Logarithm with negative base",
};

// L6 Sine/Cosine
/// □□□□□□■■
pub const SINE: Undefined = Undefined {
    prefix: 0b00000011u8 as i8,
    symbol: "℘ s",
    description: "Sine of value with imprecise period position",
};
/// ■■■■■■□□
pub const COSINE: Undefined = Undefined {
    prefix: 0b11111100u8 as i8,
    symbol: "℘ c",
    description: "Cosine of value with imprecise period position",
};
/// □□□□□□■□
pub const ARCSINE: Undefined = Undefined {
    prefix: 0b00000010u8 as i8,
    symbol: "℘ S",
    description: "Arcsine of value outside domain [-1,1]",
};
/// ■■■■■■□■
pub const ARCCOSINE: Undefined = Undefined {
    prefix: 0b11111101u8 as i8,
    symbol: "℘ C",
    description: "Arccosine of value outside domain [-1,1]",
};

// L7 Tangent and general
/// □□□□□□□■
pub const TANGENT: Undefined = Undefined {
    prefix: 0b00000001u8 as i8,
    symbol: "℘ t",
    description: "Tangent of value with imprecise period position",
};
/// ■■■■■■■□
pub const GENERAL: Undefined = Undefined {
    prefix: 0b11111110u8 as i8,
    symbol: "℘",
    description: "General undefined or unimplemented",
};
