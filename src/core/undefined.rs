//! Spirix Undefined Fraction States and Denormalization Levels
//!
//! ```txt
//! L0: Normal numbers XXXXXXXX Any known magnitude non-zero value
//!
//! L1: Exploded ↑ □■XXXXXX & ■□XXXXXX
//!
//! L2: Vanished ↓ □□■XXXXX & ■■□XXXXX
//!
//! L3: Undefined Basic Arithmetic & Logic ℘? □□□■XXXX & ■■■□XXXX
//!
//! L4: Undefined Exponentials ℘? □□□□■XXX & ■■■■□XXX
//!
//! L5: Undefined Trigonometry ℘? □□□□□■XX & ■■■■■□XX
//!
//! L6: Undefined Roots ℘? □□□□□□■X & ■■■■■■□X
//!
//! L7: Reserved for future extensions and ℘ □□□□□□□■ & ■■■■■■■□
//!
//! Uniform: □□□□□□□□ 0 Exactly Zero ■■■■■■■■ ∞ Singular Infinity
//! ```
//! Note: For fraction sizes greater than 8 bits, extensions can be utilized for more undefined granularity or for future implementation improvements. For any undefined operations that include both exploded and infinite or negligible and zero the wider arrows are used. `⬆` = Transfinite, includes ↑ and ∞ `⬇` = Negligible, includes ↓ and 0
//!
//! - Undefined prefix allocations
//! ```txt
//! L3 ℘ Basic Arithmetic & Logic (32 slots) □□□■■■■■ +0x1F General ℘ (IEEE-754 NaN maps to this) ■■■□□□□□ -0x20 ℘& □□□■■■■□ +0x1E ℘| ■■■□□□□■ -0x1F ℘⊻ □□□■■■□■ +0x1D ℘↓+↓ ■■■□□□■□ -0x1E ℘↓-↓ □□□■■■□□ +0x1C ℘⬆+ ■■■□□□■■ -0x1D ℘⬆- □□□■■□■■ +0x1B ℘+⬆ ■■■□□■□□ -0x1C ℘-⬆ □□□■■□■□ +0x1A ℘⬆+⬆ ■■■□□■□■ -0x1B ℘⬆-⬆ □□□■■□□■ +0x19 ℘⬇×⬆ ■■■□□■■□ -0x1A ℘⬆×⬇ □□□■■□□□ +0x18 ℘⬇/⬇ ■■■□□■■■ -0x19 ℘⬆/⬆ □□□■□■■■ +0x17 ℘%↓ ■■■□■□□□ -0x18 ℘‰↓ □□□■□■■□ +0x16 ℘↓%↓ ■■■□■□□■ -0x17 ℘↓‰↓ □□□■□■□■ +0x15 ℘⬆% ■■■□■□■□ -0x16 ℘⬆‰ □□□■□■□□ +0x14 ℘%⬆ ■■■□■□■■ -0x15 ℘‰⬆ □□□■□□■■ +0x13 ℘⬆%⬆ ■■■□■■□□ -0x14 ℘⬆‰⬆ □□□■□□■□ +0x12 ℘⌈ ■■■□■■□■ -0x13 ℘⌊ □□□■□□□■ +0x11 ℘∩ ■■■□■■■□ -0x12 ℘⊥⊙ □□□■□□□□ +0x10 ℘±∅ ■■■□■■■■ -0x11 ℘⨅∞
//!
//! L4 ℘ Exponentials (16 slots) □□□□■■■■ +0x0F ℘↓^ ■■■■□□□□ -0x10 ℘^↓ □□□□■■■□ +0x0E ℘⬆^ ■■■■□□□■ -0x0F ℘^⬆ □□□□■■□■ +0x0D ℘-^ ■■■■□□■□ -0x0E □□□□■■□□ +0x0C ■■■■□□■■ -0x0D □□□□■□■■ +0x0B ■■■■□■□□ -0x0C ℘@1 □□□□■□■□ +0x0A ℘⬇@ ■■■■□■□■ -0x0B ℘@⬇ □□□□■□□■ +0x09 ℘⬆@ ■■■■□■■□ -0x0A ℘@⬆ □□□□■□□□ +0x08 ℘-@ ■■■■□■■■ -0x09 ℘@-
//!
//! L5 ℘ Trigonometry (8 slots) □□□□□■■■ +0x07 ℘s ■■■■■□□□ -0x08 ℘c □□□□□■■□ +0x06 ℘S ■■■■■□□■ -0x07 ℘C □□□□□■□■ +0x05 ℘t ■■■■■□■□ -0x06 □□□□□■□□ +0x04 ■■■■■□■■ -0x05
//!
//! L6 ℘ Roots (4 slots) □□□□□□■■ +0x03 ℘√↓ ■■■■■■□□ -0x04 ℘√↑ □□□□□□■□ +0x02 ℘√- ■■■■■■□■ -0x03
//!
//! L7 ℘ Reserved (2 slots) □□□□□□□■ +0x01 RESERVED FOR EXTENSIONS ■■■■■■■□ -0x02 RESERVED FOR EXTENSIONS
//!
//! Zero □□□□□□□□ 0
//!
//! Infinity ■■■■■■■■ ∞
//! ```
pub struct Undefined {
    pub prefix: i8,
    pub symbol: &'static str,
    pub description: &'static str,
}
impl Undefined {
    /// Find the Undefined instance matching a given prefix. Falls back to GENERAL.
    pub fn from_prefix(prefix: i8) -> &'static Self {
        match prefix {
            // L3 Basic Arithmetic & Logic
            x if x == GENERAL.prefix => &GENERAL,
            x if x == AND.prefix => &AND,
            x if x == OR.prefix => &OR,
            x if x == XOR.prefix => &XOR,
            x if x == VANISHED_PLUS_VANISHED.prefix => &VANISHED_PLUS_VANISHED,
            x if x == VANISHED_MINUS_VANISHED.prefix => &VANISHED_MINUS_VANISHED,
            x if x == TRANSFINITE_PLUS_FINITE.prefix => &TRANSFINITE_PLUS_FINITE,
            x if x == TRANSFINITE_MINUS_FINITE.prefix => &TRANSFINITE_MINUS_FINITE,
            x if x == FINITE_PLUS_TRANSFINITE.prefix => &FINITE_PLUS_TRANSFINITE,
            x if x == FINITE_MINUS_TRANSFINITE.prefix => &FINITE_MINUS_TRANSFINITE,
            x if x == TRANSFINITE_PLUS_TRANSFINITE.prefix => &TRANSFINITE_PLUS_TRANSFINITE,
            x if x == TRANSFINITE_MINUS_TRANSFINITE.prefix => &TRANSFINITE_MINUS_TRANSFINITE,
            x if x == NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix => &NEGLIGIBLE_MULTIPLY_TRANSFINITE,
            x if x == TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix => &TRANSFINITE_MULTIPLY_NEGLIGIBLE,
            x if x == NEGLIGIBLE_DIVIDE_NEGLIGIBLE.prefix => &NEGLIGIBLE_DIVIDE_NEGLIGIBLE,
            x if x == TRANSFINITE_DIVIDE_TRANSFINITE.prefix => &TRANSFINITE_DIVIDE_TRANSFINITE,
            x if x == MODULUS_VANISHED.prefix => &MODULUS_VANISHED,
            x if x == MODULO_VANISHED.prefix => &MODULO_VANISHED,
            x if x == VANISHED_MODULUS_VANISHED.prefix => &VANISHED_MODULUS_VANISHED,
            x if x == VANISHED_MODULO_VANISHED.prefix => &VANISHED_MODULO_VANISHED,
            x if x == TRANSFINITE_MODULUS.prefix => &TRANSFINITE_MODULUS,
            x if x == TRANSFINITE_MODULO.prefix => &TRANSFINITE_MODULO,
            x if x == MODULUS_TRANSFINITE.prefix => &MODULUS_TRANSFINITE,
            x if x == MODULO_TRANSFINITE.prefix => &MODULO_TRANSFINITE,
            x if x == TRANSFINITE_MODULUS_TRANSFINITE.prefix => &TRANSFINITE_MODULUS_TRANSFINITE,
            x if x == TRANSFINITE_MODULO_TRANSFINITE.prefix => &TRANSFINITE_MODULO_TRANSFINITE,
            x if x == MAX_UNORDERED.prefix => &MAX_UNORDERED,
            x if x == MIN_UNORDERED.prefix => &MIN_UNORDERED,
            x if x == CLAMP_UNORDERED.prefix => &CLAMP_UNORDERED,
            x if x == INDETERMINATE.prefix => &INDETERMINATE,
            x if x == SIGN_INDETERMINATE.prefix => &SIGN_INDETERMINATE,
            // L4 Exponentials
            x if x == VANISHED_POWER.prefix => &VANISHED_POWER,
            x if x == POWER_VANISHED.prefix => &POWER_VANISHED,
            x if x == TRANSFINITE_POWER.prefix => &TRANSFINITE_POWER,
            x if x == POWER_TRANSFINITE.prefix => &POWER_TRANSFINITE,
            x if x == NEGATIVE_POWER.prefix => &NEGATIVE_POWER,
            x if x == LOG_ONE.prefix => &LOG_ONE,
            x if x == NEGLIGIBLE_LOG.prefix => &NEGLIGIBLE_LOG,
            x if x == LOG_NEGLIGIBLE.prefix => &LOG_NEGLIGIBLE,
            x if x == TRANSFINITE_LOG.prefix => &TRANSFINITE_LOG,
            x if x == LOG_TRANSFINITE.prefix => &LOG_TRANSFINITE,
            x if x == NEGATIVE_LOG.prefix => &NEGATIVE_LOG,
            x if x == LOG_NEGATIVE.prefix => &LOG_NEGATIVE,
            // L5 Trigonometry
            x if x == SINE.prefix => &SINE,
            x if x == COSINE.prefix => &COSINE,
            x if x == ARCSINE.prefix => &ARCSINE,
            x if x == ARCCOSINE.prefix => &ARCCOSINE,
            x if x == TANGENT.prefix => &TANGENT,
            // L6 Roots
            x if x == SQRT_VANISHED.prefix => &SQRT_VANISHED,
            x if x == SQRT_EXPLODED.prefix => &SQRT_EXPLODED,
            x if x == SQRT_NEGATIVE.prefix => &SQRT_NEGATIVE,
            // L7 Extensions
            x if x == FRACTIONAL_INFINITY.prefix => &FRACTIONAL_INFINITY,
            _ => &GENERAL,
        }
    }
}

// L3 Basic Arithmetic & Logic
pub const GENERAL: Undefined = Undefined {
    prefix: 0x1F,
    symbol: "℘",
    description: "General undefined or unimplemented",
};
pub const AND: Undefined = Undefined {
    prefix: -0x20,
    symbol: "℘&",
    description: "AND with escaped value",
};
pub const OR: Undefined = Undefined {
    prefix: 0x1E,
    symbol: "℘|",
    description: "OR with escaped value",
};
pub const XOR: Undefined = Undefined {
    prefix: -0x1F,
    symbol: "℘⊻",
    description: "XOR with escaped value",
};
pub const VANISHED_PLUS_VANISHED: Undefined = Undefined {
    prefix: 0x1D,
    symbol: "℘↓+↓",
    description: "Vanished value addition with vanished value",
};
pub const VANISHED_MINUS_VANISHED: Undefined = Undefined {
    prefix: -0x1E,
    symbol: "℘↓-↓",
    description: "Vanished value subtraction with vanished value",
};
pub const TRANSFINITE_PLUS_FINITE: Undefined = Undefined {
    prefix: 0x1C,
    symbol: "℘⬆+",
    description: "Transfinite value addition with finite value",
};
pub const TRANSFINITE_MINUS_FINITE: Undefined = Undefined {
    prefix: -0x1D,
    symbol: "℘⬆-",
    description: "Transfinite value subtraction with finite value",
};
pub const FINITE_PLUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0x1B,
    symbol: "℘+⬆",
    description: "Finite value addition with transfinite value",
};
pub const FINITE_MINUS_TRANSFINITE: Undefined = Undefined {
    prefix: -0x1C,
    symbol: "℘-⬆",
    description: "Finite value subtraction with transfinite value",
};
pub const TRANSFINITE_PLUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0x1A,
    symbol: "℘⬆+⬆",
    description: "Transfinite value addition with transfinite value",
};
pub const TRANSFINITE_MINUS_TRANSFINITE: Undefined = Undefined {
    prefix: -0x1B,
    symbol: "℘⬆-⬆",
    description: "Transfinite value subtraction with transfinite value",
};
pub const NEGLIGIBLE_MULTIPLY_TRANSFINITE: Undefined = Undefined {
    prefix: 0x19,
    symbol: "℘⬇×⬆",
    description: "Negligible value multiplication with transfinite value",
};
pub const TRANSFINITE_MULTIPLY_NEGLIGIBLE: Undefined = Undefined {
    prefix: -0x1A,
    symbol: "℘⬆×⬇",
    description: "Transfinite value multiplication with negligible value",
};
pub const NEGLIGIBLE_DIVIDE_NEGLIGIBLE: Undefined = Undefined {
    prefix: 0x18,
    symbol: "℘⬇/⬇",
    description: "Negligible value division by negligible value",
};
pub const TRANSFINITE_DIVIDE_TRANSFINITE: Undefined = Undefined {
    prefix: -0x19,
    symbol: "℘⬆/⬆",
    description: "Transfinite value division by transfinite value",
};
pub const MODULUS_VANISHED: Undefined = Undefined {
    prefix: 0x17,
    symbol: "℘%↓",
    description: "Modulus with vanished period",
};
pub const MODULO_VANISHED: Undefined = Undefined {
    prefix: -0x18,
    symbol: "℘‰↓",
    description: "Modulo with vanished period",
};
pub const VANISHED_MODULUS_VANISHED: Undefined = Undefined {
    prefix: 0x16,
    symbol: "℘↓%↓",
    description: "Vanished numerator modulus with vanished period",
};
pub const VANISHED_MODULO_VANISHED: Undefined = Undefined {
    prefix: -0x17,
    symbol: "℘↓‰↓",
    description: "Vanished numerator modulo with vanished period",
};
pub const TRANSFINITE_MODULUS: Undefined = Undefined {
    prefix: 0x15,
    symbol: "℘⬆%",
    description: "Transfinite numerator modulus",
};
pub const TRANSFINITE_MODULO: Undefined = Undefined {
    prefix: -0x16,
    symbol: "℘⬆‰",
    description: "Transfinite numerator modulo",
};
pub const MODULUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0x14,
    symbol: "℘%⬆",
    description: "Modulus with transfinite period (signless or mismatched signs)",
};
pub const MODULO_TRANSFINITE: Undefined = Undefined {
    prefix: -0x15,
    symbol: "℘‰⬆",
    description: "Modulo with transfinite period (signless or mismatched signs)",
};
pub const TRANSFINITE_MODULUS_TRANSFINITE: Undefined = Undefined {
    prefix: 0x13,
    symbol: "℘⬆%⬆",
    description: "Transfinite numerator modulus with transfinite period",
};
pub const TRANSFINITE_MODULO_TRANSFINITE: Undefined = Undefined {
    prefix: -0x14,
    symbol: "℘⬆‰⬆",
    description: "Transfinite numerator modulo with transfinite period",
};
pub const MAX_UNORDERED: Undefined = Undefined {
    prefix: 0x12,
    symbol: "℘⌈",
    description: "Maximum of non-ordered ambiguous values",
};
pub const MIN_UNORDERED: Undefined = Undefined {
    prefix: -0x13,
    symbol: "℘⌊",
    description: "Minimum of non-ordered ambiguous values",
};
pub const CLAMP_UNORDERED: Undefined = Undefined {
    prefix: 0x11,
    symbol: "℘∩",
    description: "Clamp with non-ordered ambiguous values",
};
pub const INDETERMINATE: Undefined = Undefined {
    prefix: -0x12,
    symbol: "℘⊥⊙",
    description: "Indeterminate Scalar → Circle conversion",
};
pub const SIGN_INDETERMINATE: Undefined = Undefined {
    prefix: 0x10,
    symbol: "℘±∅",
    description: "Sign/direction of Zero or Infinity is indeterminate",
};
pub const FRACTIONAL_INFINITY: Undefined = Undefined {
    prefix: -0x11,
    symbol: "℘⨅∞",
    description: "Fractional part of Infinity",
};

// L4 Exponentials
pub const VANISHED_POWER: Undefined = Undefined {
    prefix: 0x0F,
    symbol: "℘↓^",
    description: "Vanished value raised to power",
};
pub const POWER_VANISHED: Undefined = Undefined {
    prefix: -0x10,
    symbol: "℘^↓",
    description: "Value raised to vanished power",
};
pub const TRANSFINITE_POWER: Undefined = Undefined {
    prefix: 0x0E,
    symbol: "℘⬆^",
    description: "Transfinite value raised to power",
};
pub const POWER_TRANSFINITE: Undefined = Undefined {
    prefix: -0x0F,
    symbol: "℘^⬆",
    description: "Value raised to transfinite power",
};
pub const NEGATIVE_POWER: Undefined = Undefined {
    prefix: 0x0D,
    symbol: "℘-^",
    description: "Negative value raised to irrational power",
};
pub const LOG_ONE: Undefined = Undefined {
    prefix: -0x0C,
    symbol: "℘@1",
    description: "Logarithm base One",
};
pub const NEGLIGIBLE_LOG: Undefined = Undefined {
    prefix: 0x0A,
    symbol: "℘⬇@",
    description: "Logarithm of negligible value",
};
pub const LOG_NEGLIGIBLE: Undefined = Undefined {
    prefix: -0x0B,
    symbol: "℘@⬇",
    description: "Logarithm with negligible base",
};
pub const TRANSFINITE_LOG: Undefined = Undefined {
    prefix: 0x09,
    symbol: "℘⬆@",
    description: "Logarithm of transfinite value",
};
pub const LOG_TRANSFINITE: Undefined = Undefined {
    prefix: -0x0A,
    symbol: "℘@⬆",
    description: "Logarithm with transfinite base",
};
pub const NEGATIVE_LOG: Undefined = Undefined {
    prefix: 0x08,
    symbol: "℘-@",
    description: "Logarithm of negative value",
};
pub const LOG_NEGATIVE: Undefined = Undefined {
    prefix: -0x09,
    symbol: "℘@-",
    description: "Logarithm with negative base",
};

// L5 Trigonometry
pub const SINE: Undefined = Undefined {
    prefix: 0x07,
    symbol: "℘s",
    description: "Sine of value with imprecise period position",
};
pub const COSINE: Undefined = Undefined {
    prefix: -0x08,
    symbol: "℘c",
    description: "Cosine of value with imprecise period position",
};
pub const ARCSINE: Undefined = Undefined {
    prefix: 0x06,
    symbol: "℘S",
    description: "Arcsine of value outside domain [-1,1]",
};
pub const ARCCOSINE: Undefined = Undefined {
    prefix: -0x07,
    symbol: "℘C",
    description: "Arccosine of value outside domain [-1,1]",
};
pub const TANGENT: Undefined = Undefined {
    prefix: 0x05,
    symbol: "℘t",
    description: "Tangent of value with imprecise period position",
};

// L6 Roots
pub const SQRT_VANISHED: Undefined = Undefined {
    prefix: 0x03,
    symbol: "℘√↓",
    description: "Square root of vanished value",
};
pub const SQRT_EXPLODED: Undefined = Undefined {
    prefix: -0x04,
    symbol: "℘√↑",
    description: "Square root of transfinite value",
};
pub const SQRT_NEGATIVE: Undefined = Undefined {
    prefix: 0x02,
    symbol: "℘√-",
    description: "Square root of negative value",
};
