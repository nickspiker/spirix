use crate::Circle;

/// CircleF3E3: Minimal precision and range complex floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type CircleF3E3 = Circle<i8, i8>;

/// CircleF4E3: Limited precision with small range complex floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type CircleF4E3 = Circle<i16, i8>;

/// CircleF5E3: Standard precision with small range complex floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type CircleF5E3 = Circle<i32, i8>;

/// CircleF6E3: High precision with small range complex floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type CircleF6E3 = Circle<i64, i8>;

/// CircleF7E3: Ultra-high precision with small range complex floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type CircleF7E3 = Circle<i128, i8>;

/// CircleF3E4: Minimal precision with medium range complex floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type CircleF3E4 = Circle<i8, i16>;

/// CircleF4E4: Limited precision and medium range complex floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type CircleF4E4 = Circle<i16, i16>;

/// CircleF5E4: Standard precision with medium range complex floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type CircleF5E4 = Circle<i32, i16>;

/// CircleF6E4: High precision with medium range complex floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type CircleF6E4 = Circle<i64, i16>;

/// CircleF7E4: Ultra-high precision with medium range complex floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type CircleF7E4 = Circle<i128, i16>;

/// CircleF3E5: Minimal precision with large range complex floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type CircleF3E5 = Circle<i8, i32>;

/// CircleF4E5: Limited precision with large range complex floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type CircleF4E5 = Circle<i16, i32>;

/// CircleF5E5: Standard precision with large range complex floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type CircleF5E5 = Circle<i32, i32>;

/// CircleF6E5: High precision with large range complex floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type CircleF6E5 = Circle<i64, i32>;

/// CircleF7E5: Ultra-high precision with large range complex floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type CircleF7E5 = Circle<i128, i32>;

/// CircleF3E6: Minimal precision with huge range complex floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type CircleF3E6 = Circle<i8, i64>;

/// CircleF4E6: Limited precision with huge range complex floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type CircleF4E6 = Circle<i16, i64>;

/// CircleF5E6: Standard precision with huge range complex floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type CircleF5E6 = Circle<i32, i64>;

/// CircleF6E6: High precision with huge range complex floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type CircleF6E6 = Circle<i64, i64>;

/// CircleF7E6: Ultra-high precision with huge range complex floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type CircleF7E6 = Circle<i128, i64>;

/// CircleF3E7: Minimal precision with ridiculous range complex floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type CircleF3E7 = Circle<i8, i128>;

/// CircleF4E7: Limited precision with ridiculous range complex floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type CircleF4E7 = Circle<i16, i128>;

/// CircleF5E7: Standard precision with ridiculous range complex floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type CircleF5E7 = Circle<i32, i128>;

/// CircleF6E7: High precision with ridiculous range complex floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type CircleF6E7 = Circle<i64, i128>;

/// CircleF7E7: Ultra-high precision and ridiculous range complex floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type CircleF7E7 = Circle<i128, i128>;