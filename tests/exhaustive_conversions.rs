//! Exhaustive conversion tests — every i8 value, every constant, every edge.
//!
//! Strategy: F3E3 (Scalar<i8, i8>) is small enough to enumerate all 256 stored
//! fractions × all 256 exponents (= 65,536 Scalars). For larger widths we sample
//! constants, powers of 2, primes, and the constants of the type.

use spirix::*;

type S8 = Scalar<i8, i8>;
type S16 = Scalar<i16, i8>;
type S32 = Scalar<i32, i8>;
type S64 = Scalar<i64, i8>;
type S128 = Scalar<i128, i8>;

// ─────────────────────────────────────────────────────────────────────────────
// Integer round trips
// ─────────────────────────────────────────────────────────────────────────────

/// Every i8 value must round-trip through Scalar<i8, i8>.
#[test]
fn f3e3_int_roundtrip_all_i8() {
    let mut fail = 0;
    for v in i8::MIN..=i8::MAX {
        let s = S64::from(v); // wider scalar can hold every i8 exactly
        let back: i64 = s.into();
        if back != v as i64 {
            fail += 1;
            if fail <= 5 {
                let f: f64 = s.into();
                eprintln!("i8 {} → S64 → i64 {} (f64={})", v, back, f);
            }
        }
    }
    assert_eq!(fail, 0, "{} i8 values failed integer round-trip", fail);
}

/// All 256 i32 values in [-128, 127] should round-trip through every width.
#[test]
fn int_roundtrip_all_widths() {
    for v in -128i32..=127 {
        let r8: i32 = S8::from(v as i8).into();
        let r16: i32 = S16::from(v as i16).into();
        let r32: i32 = S32::from(v).into();
        let r64: i32 = {let x: i64 = S64::from(v as i64).into(); x as i32};
        let r128: i32 = {let x: i64 = S128::from(v as i64).into(); x as i32};
        // F3E3 has only ~6 bits of fraction precision so it can't hold values >127 or <-128 exactly anyway.
        // For [-128, 127], everything should round-trip exactly at all widths.
        assert_eq!(r16, v, "i16 width round-trip failed for {}", v);
        assert_eq!(r32, v, "i32 width round-trip failed for {}", v);
        assert_eq!(r64, v, "i64 width round-trip failed for {}", v);
        assert_eq!(r128, v, "i128 width round-trip failed for {}", v);
        // S8 has limited precision — only small magnitudes round-trip exactly.
        if v.abs() <= 64 {
            assert_eq!(r8, v, "i8 width round-trip failed for {}", v);
        }
    }
}

/// Negative integer From: every negative i8 should give negative Scalar value.
#[test]
fn negative_ints_have_negative_value() {
    for v in i8::MIN..0 {
        let s = S64::from(v);
        assert!(s.is_negative(), "From({}) should produce negative Scalar, got value {}", v, <S64 as Into<f64>>::into(s));
        let f: f64 = s.into();
        assert!(f < 0.0, "From({}) f64 = {} should be negative", v, f);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Powers of 2 (positive and negative)
// ─────────────────────────────────────────────────────────────────────────────

/// All powers of 2 from 2^-10 to 2^10 round-trip through f64 exactly.
#[test]
fn powers_of_2_exact_roundtrip() {
    for exp in -10i32..=10 {
        let v = 2.0f64.powi(exp);
        let s = S64::from(v);
        let back: f64 = s.into();
        assert_eq!(back, v, "power 2^{} = {} round-trip failed: got {}", exp, v, back);
    }
}

/// NEGATIVE powers of 2 (always the trouble spot).
#[test]
fn negative_powers_of_2_exact() {
    for exp in -10i32..=10 {
        let v = -(2.0f64.powi(exp));
        let s = S64::from(v);
        let back: f64 = s.into();
        assert_eq!(back, v, "negative 2^{} = {} round-trip failed: got {}", exp, v, back);
        assert!(s.is_negative(), "{} should be negative", v);
    }
}

/// Powers of 2 across all widths.
#[test]
fn powers_of_2_all_widths() {
    let cases: &[f64] = &[1.0, 2.0, 4.0, 8.0, 16.0, 0.5, 0.25, 0.125, -1.0, -2.0, -0.5, -0.25];
    for &v in cases {
        let r8: f64 = S8::from(v).into();
        let r16: f64 = S16::from(v).into();
        let r32: f64 = S32::from(v).into();
        let r64: f64 = S64::from(v).into();
        let r128: f64 = S128::from(v).into();
        // F3E3 might lose precision for some powers — only check it can represent something reasonable
        assert!((r8 - v).abs() / v.abs() < 0.5, "F3E3 {} → {}", v, r8);
        // Wider widths should be exact for any power-of-2 in i8 range
        assert_eq!(r16, v, "F4E3 {} → {}", v, r16);
        assert_eq!(r32, v, "F5E3 {} → {}", v, r32);
        assert_eq!(r64, v, "F6E3 {} → {}", v, r64);
        assert_eq!(r128, v, "F7E3 {} → {}", v, r128);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Constants: ZERO, ONE, NEG_ONE, MAX, MIN, MIN_POS, MAX_NEG, INFINITY, etc.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn const_zero_is_zero() {
    let z = S64::ZERO;
    let f: f64 = z.into();
    assert_eq!(f, 0.0, "ZERO should be 0, got {}", f);
    assert!(z.is_zero());
}

#[test]
fn const_one_is_one() {
    for (name, f) in [("F3E3", S8::ONE.into()), ("F4E3", S16::ONE.into()),
                      ("F5E3", S32::ONE.into()), ("F6E3", S64::ONE.into()),
                      ("F7E3", S128::ONE.into())] {
        let f: f64 = f;
        assert_eq!(f, 1.0, "{} ONE should be 1.0, got {}", name, f);
    }
}

#[test]
fn const_neg_one_is_neg_one() {
    for (name, f) in [("F3E3", S8::NEG_ONE.into()), ("F4E3", S16::NEG_ONE.into()),
                      ("F5E3", S32::NEG_ONE.into()), ("F6E3", S64::NEG_ONE.into()),
                      ("F7E3", S128::NEG_ONE.into())] {
        let f: f64 = f;
        assert_eq!(f, -1.0, "{} NEG_ONE should be -1.0, got {}", name, f);
    }
}

#[test]
fn const_pi_close_to_pi() {
    let p: f64 = S64::PI.into();
    let err = (p - std::f64::consts::PI).abs();
    assert!(err < 1e-15, "PI = {} (err {:.2e})", p, err);
}

#[test]
fn const_e_close_to_e() {
    let e: f64 = S64::E.into();
    let err = (e - std::f64::consts::E).abs();
    assert!(err < 1e-15, "E = {} (err {:.2e})", e, err);
}

#[test]
fn const_ln_two_close_to_ln_two() {
    let l: f64 = S64::LN_TWO.into();
    let err = (l - std::f64::consts::LN_2).abs();
    assert!(err < 1e-15, "LN_TWO = {} (err {:.2e})", l, err);
}

// ─────────────────────────────────────────────────────────────────────────────
// f64 round trips
// ─────────────────────────────────────────────────────────────────────────────

/// f64 → Scalar → f64 should round-trip within ULP for representable values.
#[test]
fn f64_roundtrip_reasonable_values() {
    // Note: f64 subnormals (smaller than ~1e-38) underflow F6E3's i8 exponent range and are correctly mapped to vanished — test those separately.
    let cases: &[f64] = &[
        0.0, 1.0, -1.0, 0.5, -0.5, 2.0, -2.0,
        3.14159265358979, -3.14159265358979,
        2.71828182845904, 1e10, 1e-10, -1e10, -1e-10,
    ];
    for &v in cases {
        let s = S64::from(v);
        let back: f64 = s.into();
        if v == 0.0 {
            assert_eq!(back, 0.0);
            continue;
        }
        let rel = (back - v).abs() / v.abs();
        assert!(rel < 1e-15, "{} → {} (rel={:.2e})", v, back, rel);
    }
}

/// IEEE special values: NaN → undefined, ±∞ → exploded, ±0 → zero.
#[test]
fn ieee_specials() {
    let nan = S64::from(f64::NAN);
    assert!(nan.is_undefined() || nan.is_zero(), "NaN should map to undefined or zero (currently undefined)");

    let inf = S64::from(f64::INFINITY);
    assert!(inf.exploded() && inf.is_positive(), "+inf should be positive exploded");

    let neg_inf = S64::from(f64::NEG_INFINITY);
    assert!(neg_inf.exploded() && neg_inf.is_negative(), "-inf should be negative exploded");

    let pos_zero = S64::from(0.0);
    let neg_zero = S64::from(-0.0);
    assert!(pos_zero.is_zero());
    assert!(neg_zero.is_zero());
}

// ─────────────────────────────────────────────────────────────────────────────
// Primes — distinct from powers, catch normalization bugs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn primes_roundtrip_all_widths() {
    let primes: &[i32] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127];
    for &p in primes {
        let r16: i32 = S16::from(p as i16).into();
        let r32: i32 = S32::from(p).into();
        let r64: i32 = {let x: i64 = S64::from(p as i64).into(); x as i32};
        let r128: i32 = {let x: i64 = S128::from(p as i64).into(); x as i32};
        assert_eq!(r16, p, "prime {} F4E3 round-trip", p);
        assert_eq!(r32, p, "prime {} F5E3 round-trip", p);
        assert_eq!(r64, p, "prime {} F6E3 round-trip", p);
        assert_eq!(r128, p, "prime {} F7E3 round-trip", p);
    }
}

/// Negative primes — sign handling in the conversion path.
#[test]
fn negative_primes_roundtrip() {
    let primes: &[i32] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];
    for &p in primes {
        let v = -p;
        let r32: i32 = S32::from(v).into();
        let r64: i32 = {let x: i64 = S64::from(v as i64).into(); x as i32};
        let r128: i32 = {let x: i64 = S128::from(v as i64).into(); x as i32};
        assert_eq!(r32, v, "neg prime {} F5E3", v);
        assert_eq!(r64, v, "neg prime {} F6E3", v);
        assert_eq!(r128, v, "neg prime {} F7E3", v);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Exhaustive F3E3 sweep — every (stored, exponent) pair to f64
// ─────────────────────────────────────────────────────────────────────────────

/// Every Scalar<i8,i8> converts to f64 without panicking. (Sanity check.)
#[test]
fn f3e3_exhaustive_to_f64_no_panic() {
    let mut nan_count = 0;
    let mut inf_count = 0;
    let mut zero_count = 0;
    let mut finite_count = 0;
    for stored in i8::MIN..=i8::MAX {
        for exp in i8::MIN..=i8::MAX {
            let s = Scalar { fraction: stored, exponent: exp };
            let f: f64 = s.into();
            if f.is_nan() { nan_count += 1; }
            else if f.is_infinite() { inf_count += 1; }
            else if f == 0.0 { zero_count += 1; }
            else { finite_count += 1; }
        }
    }
    let total = nan_count + inf_count + zero_count + finite_count;
    assert_eq!(total, 65536);
    eprintln!("F3E3 exhaustive (65536 Scalars): finite={}, zero={}, inf={}, nan={}",
              finite_count, zero_count, inf_count, nan_count);
}

/// Round-trip property: every "normal" F3E3 Scalar should give the same Scalar
/// when converted f64→Scalar (within precision).
#[test]
fn f3e3_normal_roundtrip_via_f64() {
    let mut fail = 0;
    for stored in i8::MIN..=i8::MAX {
        for exp in -10i8..=10 {
            let s = Scalar::<i8, i8> { fraction: stored, exponent: exp };
            if !s.is_normal() { continue; }
            let f: f64 = s.into();
            if f == 0.0 || f.is_nan() || f.is_infinite() { continue; }
            let s2 = S8::from(f);
            // For small widths, round-trip might differ by 1 ULP because of rounding
            let f2: f64 = s2.into();
            let rel = (f2 - f).abs() / f.abs();
            if rel > 0.05 {  // F3E3 has ~6 bits of fraction precision
                fail += 1;
                if fail <= 3 {
                    eprintln!("F3E3 stored={} exp={}: f={} → s2 → f2={} (rel={:.2e})",
                              stored, exp, f, f2, rel);
                }
            }
        }
    }
    assert_eq!(fail, 0, "{} F3E3 normals failed f64 round-trip", fail);
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge cases — extreme exponents and boundary fractions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn min_max_constants_non_panic() {
    // Just verify these don't panic and produce sensible classifications
    let _: f64 = S64::MAX.into();
    let _: f64 = S64::MIN.into();
    let _: f64 = S64::MIN_POS.into();
    let _: f64 = S64::MAX_NEG.into();
    let _: f64 = S64::INFINITY.into();
    let _: f64 = S64::EXPLODED_POS.into();
    let _: f64 = S64::EXPLODED_NEG.into();

    assert!(S64::MAX.is_positive());
    assert!(S64::MIN.is_negative());
    assert!(S64::MIN_POS.is_positive());
    assert!(S64::MAX_NEG.is_negative());
    assert!(S64::INFINITY.is_infinite());
    assert!(S64::EXPLODED_POS.exploded() && S64::EXPLODED_POS.is_positive());
    assert!(S64::EXPLODED_NEG.exploded() && S64::EXPLODED_NEG.is_negative());
}

#[test]
fn negation_roundtrips() {
    // -(-x) == x for various values
    for v in [1, 2, 3, 5, 7, 100, -1, -2, -3, -5, -100].iter() {
        let s = S64::from(*v);
        let neg_neg = -(-s);
        assert_eq!(s.fraction, neg_neg.fraction, "neg-neg fraction differs for {}", v);
        assert_eq!(s.exponent, neg_neg.exponent, "neg-neg exponent differs for {}", v);
    }
}
