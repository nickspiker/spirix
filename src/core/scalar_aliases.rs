use crate::Scalar;

/// ScalarF3E3: Minimal precision and range floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type ScalarF3E3 = Scalar<i8, i8>;

/// ScalarF4E3: Limited precision with small range floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type ScalarF4E3 = Scalar<i16, i8>;

/// ScalarF5E3: Standard precision with small range floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type ScalarF5E3 = Scalar<i32, i8>;

/// ScalarF6E3: High precision with small range floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type ScalarF6E3 = Scalar<i64, i8>;

/// ScalarF7E3: Ultra-high precision with small range floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 8-bit (2³) supports values up to ~10^±38.5
pub type ScalarF7E3 = Scalar<i128, i8>;

/// ScalarF3E4: Minimal precision with medium range floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type ScalarF3E4 = Scalar<i8, i16>;

/// ScalarF4E4: Limited precision and medium range floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type ScalarF4E4 = Scalar<i16, i16>;

/// ScalarF5E4: Standard precision with medium range floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type ScalarF5E4 = Scalar<i32, i16>;

/// ScalarF6E4: High precision with medium range floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type ScalarF6E4 = Scalar<i64, i16>;

/// ScalarF7E4: Ultra-high precision with medium range floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 16-bit (2⁴) supports values up to ~10^±9860
pub type ScalarF7E4 = Scalar<i128, i16>;

/// ScalarF3E5: Minimal precision with large range floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type ScalarF3E5 = Scalar<i8, i32>;

/// ScalarF4E5: Limited precision with large range floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type ScalarF4E5 = Scalar<i16, i32>;

/// ScalarF5E5: Standard precision with large range floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type ScalarF5E5 = Scalar<i32, i32>;

/// ScalarF6E5: High precision with large range floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type ScalarF6E5 = Scalar<i64, i32>;

/// ScalarF7E5: Ultra-high precision with large range floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 32-bit (2⁵) supports values up to ~10^(10^8.81)
pub type ScalarF7E5 = Scalar<i128, i32>;

/// ScalarF3E6: Minimal precision with huge range floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type ScalarF3E6 = Scalar<i8, i64>;

/// ScalarF4E6: Limited precision with huge range floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type ScalarF4E6 = Scalar<i16, i64>;

/// ScalarF5E6: Standard precision with huge range floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type ScalarF5E6 = Scalar<i32, i64>;

/// ScalarF6E6: High precision with huge range floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type ScalarF6E6 = Scalar<i64, i64>;

/// ScalarF7E6: Ultra-high precision with huge range floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 64-bit (2⁶) supports values up to ~10^(10^18.4)
pub type ScalarF7E6 = Scalar<i128, i64>;

/// ScalarF3E7: Minimal precision with ridiculous range floating-point type
/// - Fraction: 8-bit (2³) provides ~2.1 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type ScalarF3E7 = Scalar<i8, i128>;

/// ScalarF4E7: Limited precision with ridiculous range floating-point type
/// - Fraction: 16-bit (2⁴) provides ~4.5 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type ScalarF4E7 = Scalar<i16, i128>;

/// ScalarF5E7: Standard precision with ridiculous range floating-point type
/// - Fraction: 32-bit (2⁵) provides ~9.3 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type ScalarF5E7 = Scalar<i32, i128>;

/// ScalarF6E7: High precision with ridiculous range floating-point type
/// - Fraction: 64-bit (2⁶) provides ~18.9 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type ScalarF6E7 = Scalar<i64, i128>;

/// ScalarF7E7: Ultra-high precision and ridiculous range floating-point type
/// - Fraction: 128-bit (2⁷) provides ~38.2 decimal digits precision
/// - Exponent: 128-bit (2⁷) supports values up to ~10^(10^37.7)
pub type ScalarF7E7 = Scalar<i128, i128>;
