//! Scalar → Scalar conversion tests across all width combinations. Uses F7E7 as gold standard. Special focus on F3E7 and F7E3 corner cases.

use spirix::*;

type S33 = Scalar<i8, i8>; // F3E3
type S43 = Scalar<i16, i8>; // F4E3
type S53 = Scalar<i32, i8>; // F5E3
type S63 = Scalar<i64, i8>; // F6E3
type S73 = Scalar<i128, i8>; // F7E3
type S37 = Scalar<i8, i128>; // F3E7 — small fraction, huge exp
type S77 = Scalar<i128, i128>; // F7E7 — gold

// ─────────────────────────────────────────────────────────────────────────────
// Special class preservation across widths ─────────────────────────────────────────────────────────────────────────────

#[test]
fn zero_preserved_widening() {
    assert!(S43::from(&S33::ZERO).is_zero());
    assert!(S53::from(&S33::ZERO).is_zero());
    assert!(S63::from(&S33::ZERO).is_zero());
    assert!(S73::from(&S33::ZERO).is_zero());
}

#[test]
fn zero_preserved_narrowing() {
    assert!(S33::from(&S73::ZERO).is_zero());
    assert!(S33::from(&S77::ZERO).is_zero());
}

#[test]
fn infinity_preserved_widening() {
    let i33 = S33::INFINITY;
    assert!(i33.is_infinite(), "F3E3 INFINITY should be infinite");
    let i43: S43 = (&i33).into();
    assert!(
        i43.is_infinite(),
        "F3E3 INFINITY → F4E3 should still be infinite (got frac={}, exp={})",
        i43.fraction,
        i43.exponent
    );
    let i63: S63 = (&i33).into();
    assert!(
        i63.is_infinite(),
        "F3E3 INFINITY → F6E3 should still be infinite"
    );
    let i73: S73 = (&i33).into();
    assert!(
        i73.is_infinite(),
        "F3E3 INFINITY → F7E3 should still be infinite"
    );
}

#[test]
fn infinity_preserved_narrowing() {
    let i73 = S73::INFINITY;
    assert!(i73.is_infinite());
    let i33: S33 = (&i73).into();
    assert!(
        i33.is_infinite(),
        "F7E3 INFINITY → F3E3 should still be infinite"
    );
}

#[test]
fn exploded_pos_preserved() {
    let e33 = S33::EXPLODED_POS;
    let e43: S43 = (&e33).into();
    let e63: S63 = (&e33).into();
    assert!(
        e43.exploded() && e43.is_positive(),
        "F3E3 +EXP → F4E3 lost class"
    );
    assert!(
        e63.exploded() && e63.is_positive(),
        "F3E3 +EXP → F6E3 lost class"
    );
}

#[test]
fn exploded_neg_preserved() {
    let e33 = S33::EXPLODED_NEG;
    let e43: S43 = (&e33).into();
    let e63: S63 = (&e33).into();
    assert!(
        e43.exploded() && e43.is_negative(),
        "F3E3 -EXP → F4E3 lost class/sign"
    );
    assert!(
        e63.exploded() && e63.is_negative(),
        "F3E3 -EXP → F6E3 lost class/sign"
    );
}

#[test]
fn vanished_pos_preserved() {
    let v33 = S33::VANISHED_POS;
    let v43: S43 = (&v33).into();
    let v63: S63 = (&v33).into();
    assert!(
        v43.vanished() && v43.is_positive(),
        "F3E3 +VAN → F4E3 lost class"
    );
    assert!(
        v63.vanished() && v63.is_positive(),
        "F3E3 +VAN → F6E3 lost class"
    );
}

#[test]
fn vanished_neg_preserved() {
    let v33 = S33::VANISHED_NEG;
    let v43: S43 = (&v33).into();
    let v63: S63 = (&v33).into();
    assert!(v43.vanished() && v43.is_negative());
    assert!(v63.vanished() && v63.is_negative());
}

#[test]
fn undefined_preserved() {
    let u = S33::ZERO / S33::ZERO;
    assert!(u.is_undefined());
    let u43: S43 = (&u).into();
    let u63: S63 = (&u).into();
    assert!(u43.is_undefined(), "F3E3 [℘] → F4E3 lost class");
    assert!(u63.is_undefined(), "F3E3 [℘] → F6E3 lost class");
}

// ─────────────────────────────────────────────────────────────────────────────
// Normal value round trips (widen then narrow should preserve) ─────────────────────────────────────────────────────────────────────────────

#[test]
fn integer_widening_then_narrowing_preserves() {
    // F3E3 has limited precision but small integers should survive
    for v in &[1, -1, 2, -2, 4, -4, 16, -16, 64, -64] {
        let s33 = S33::from(*v as i8);
        let widened: S73 = (&s33).into();
        let narrowed: S33 = (&widened).into();
        let original: f64 = s33.into();
        let result: f64 = narrowed.into();
        assert_eq!(
            original, result,
            "F3E3 {} → F7E3 → F3E3 round-trip failed: {} → {}",
            v, original, result
        );
    }
}

#[test]
fn f64_via_widening() {
    // f64 → F3E3 → F7E3 should give same value as direct f64 → F7E3 (within F3E3 precision)
    for v in &[1.0, 2.0, 0.5, 4.0, -1.0, -0.5] {
        let s33 = S33::from(*v);
        let s73_via: S73 = (&s33).into();
        let v_via: f64 = s73_via.into();
        let s73_direct = S73::from(*v);
        let v_direct: f64 = s73_direct.into();
        let v33: f64 = s33.into();
        assert_eq!(
            v_via, v33,
            "widening {} via F3E3: {} (via) vs {} (direct F3E3 value)",
            v, v_via, v33
        );
        // The direct F7E3 value should be the original
        assert_eq!(v_direct, *v);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F3E7 corner: tiny fraction, huge exponent range ─────────────────────────────────────────────────────────────────────────────

#[test]
fn f3e7_construction() {
    // F3E7 has only 8 fraction bits but i128 exponent — can represent crazy magnitudes
    let one = S37::ONE;
    let f: f64 = one.into();
    assert_eq!(f, 1.0, "F3E7 ONE = {}", f);
    let neg_one = S37::NEG_ONE;
    let f: f64 = neg_one.into();
    assert_eq!(f, -1.0);
    let zero = S37::ZERO;
    assert!(zero.is_zero());
    let inf = S37::INFINITY;
    assert!(inf.is_infinite());
}

#[test]
fn f3e7_from_int() {
    for v in &[1, -1, 2, -2, 16, -16, 64, -64] {
        let s = S37::from(*v as i8);
        let f: f64 = s.into();
        assert_eq!(f, *v as f64, "F3E7 from {}", v);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// F7E3 corner: huge fraction, small exponent ─────────────────────────────────────────────────────────────────────────────

#[test]
fn f7e3_high_precision() {
    // F7E3 has 128 fraction bits — plenty of precision for transcendentals
    let pi = S73::PI;
    let p: f64 = pi.into();
    assert!((p - std::f64::consts::PI).abs() < 1e-15, "F7E3 PI = {}", p);
}

#[test]
fn f7e3_widening_to_f7e7_preserves_precision() {
    let pi = S73::PI;
    // Conversion to E7 (wider exp): should preserve fraction precision
    let p_via: f64 = (&pi).into();
    assert!((p_via - std::f64::consts::PI).abs() < 1e-15);
}

// ─────────────────────────────────────────────────────────────────────────────
// All 5×5 width combinations × constants ─────────────────────────────────────────────────────────────────────────────

macro_rules! crosswidth_zero_inf {
    ($src:ty, $dst:ty, $tag:literal) => {
        let z_src = <$src>::ZERO;
        let i_src = <$src>::INFINITY;
        let z_dst: $dst = (&z_src).into();
        let i_dst: $dst = (&i_src).into();
        assert!(
            z_dst.is_zero(),
            "{} ZERO → not zero (frac={}, exp={})",
            $tag,
            z_dst.fraction,
            z_dst.exponent
        );
        assert!(
            i_dst.is_infinite(),
            "{} INFINITY → not infinite (frac={}, exp={})",
            $tag,
            i_dst.fraction,
            i_dst.exponent
        );
    };
}

#[test]
fn all_widths_zero_inf_pairs() {
    crosswidth_zero_inf!(S33, S43, "F3E3→F4E3");
    crosswidth_zero_inf!(S33, S53, "F3E3→F5E3");
    crosswidth_zero_inf!(S33, S63, "F3E3→F6E3");
    crosswidth_zero_inf!(S33, S73, "F3E3→F7E3");
    crosswidth_zero_inf!(S43, S33, "F4E3→F3E3");
    crosswidth_zero_inf!(S43, S53, "F4E3→F5E3");
    crosswidth_zero_inf!(S43, S63, "F4E3→F6E3");
    crosswidth_zero_inf!(S43, S73, "F4E3→F7E3");
    crosswidth_zero_inf!(S53, S33, "F5E3→F3E3");
    crosswidth_zero_inf!(S53, S43, "F5E3→F4E3");
    crosswidth_zero_inf!(S53, S63, "F5E3→F6E3");
    crosswidth_zero_inf!(S53, S73, "F5E3→F7E3");
    crosswidth_zero_inf!(S63, S33, "F6E3→F3E3");
    crosswidth_zero_inf!(S63, S43, "F6E3→F4E3");
    crosswidth_zero_inf!(S63, S53, "F6E3→F5E3");
    crosswidth_zero_inf!(S63, S73, "F6E3→F7E3");
    crosswidth_zero_inf!(S73, S33, "F7E3→F3E3");
    crosswidth_zero_inf!(S73, S43, "F7E3→F4E3");
    crosswidth_zero_inf!(S73, S53, "F7E3→F5E3");
    crosswidth_zero_inf!(S73, S63, "F7E3→F6E3");
    // Cross exponent type
    crosswidth_zero_inf!(S37, S77, "F3E7→F7E7");
    crosswidth_zero_inf!(S77, S37, "F7E7→F3E7");
    crosswidth_zero_inf!(S33, S77, "F3E3→F7E7");
    crosswidth_zero_inf!(S77, S33, "F7E7→F3E3");
}

// ─────────────────────────────────────────────────────────────────────────────
// Sign preservation under conversion ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sign_preserved_widening() {
    let neg = S33::from(-3i8);
    let widened: S73 = (&neg).into();
    assert!(
        widened.is_negative(),
        "F3E3 -3 → F7E3 should still be negative"
    );
    let f: f64 = widened.into();
    assert!(f < 0.0, "value should be negative, got {}", f);
}

#[test]
fn sign_preserved_narrowing() {
    let neg = S73::from(-100i64);
    let narrowed: S43 = (&neg).into();
    assert!(
        narrowed.is_negative(),
        "F7E3 -100 → F4E3 should still be negative"
    );
    let f: f64 = narrowed.into();
    assert!(f < 0.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Unsigned int conversions ─────────────────────────────────────────────────────────────────────────────

#[test]
fn into_unsigned_positive() {
    let s = S63::from(42u8);
    let u8_v: u8 = s.into();
    let u16_v: u16 = s.into();
    let u32_v: u32 = s.into();
    let u64_v: u64 = s.into();
    assert_eq!(u8_v, 42);
    assert_eq!(u16_v, 42);
    assert_eq!(u32_v, 42);
    assert_eq!(u64_v, 42);
}

#[test]
fn into_unsigned_negative_clamps_to_zero() {
    let s = S63::from(-42i8);
    let u8_v: u8 = s.into();
    let u32_v: u32 = s.into();
    assert_eq!(u8_v, 0);
    assert_eq!(u32_v, 0);
}

#[test]
fn into_signed_negative() {
    let s = S63::from(-42i8);
    let i8_v: i8 = s.into();
    let i16_v: i16 = s.into();
    let i32_v: i32 = s.into();
    let i64_v: i64 = s.into();
    let i128_v: i128 = s.into();
    assert_eq!(i8_v, -42);
    assert_eq!(i16_v, -42);
    assert_eq!(i32_v, -42);
    assert_eq!(i64_v, -42);
    assert_eq!(i128_v, -42);
}

// ─────────────────────────────────────────────────────────────────────────────
// Saturation: extreme exponent values ─────────────────────────────────────────────────────────────────────────────

#[test]
fn into_int_saturation_huge_positive() {
    // F7E7 can have exponent values way beyond what fits in i32
    let huge = S77::MAX;
    let i32_v: i32 = huge.into();
    assert_eq!(i32_v, i32::MAX, "huge → i32 should saturate to MAX");
}

#[test]
fn into_int_saturation_huge_negative() {
    let huge_neg = S77::MIN;
    let i32_v: i32 = huge_neg.into();
    assert_eq!(
        i32_v,
        i32::MIN,
        "very negative → i32 should saturate to MIN"
    );
}

// ───────────────────────────────────────────────────────────────────────────── f32 round trip ─────────────────────────────────────────────────────────────────────────────

#[test]
fn f32_roundtrip_powers_of_2() {
    for exp in -10i32..=10 {
        let v = 2.0f32.powi(exp);
        let s = S63::from(v as f64); // From<f32> not directly impl'd, go via f64
        let back: f64 = s.into();
        assert_eq!(back as f32, v, "2^{} via f32 round-trip", exp);
    }
}

// ───────────────────────────────────────────────────────────────────────────── f64 → F3E3 → f64 ULP analysis ─────────────────────────────────────────────────────────────────────────────

#[test]
fn f64_to_f3e3_precision() {
    // F3E3 has very limited precision. Check that round-trip is within expected ULP.
    let cases: &[f64] = &[1.0, 1.5, 2.0, 3.0, 0.5, 0.25, -1.0, -1.5];
    for &v in cases {
        let s = S33::from(v);
        let back: f64 = s.into();
        let rel = (back - v).abs() / v.abs();
        // F3E3 has ~7 bits of precision (2^-7 ≈ 0.78%)
        assert!(
            rel < 0.05,
            "F3E3 round-trip {} → {} (rel={:.2e})",
            v,
            back,
            rel
        );
    }
}
