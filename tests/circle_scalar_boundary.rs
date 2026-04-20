//! Circle ↔ Scalar boundary conversion tests — full truth table coverage.
//!
//! Circle keeps explicit-sign N-1 format; Scalar uses implicit-sign via ~MSB.
//! These tests verify every value class transitions correctly across the
//! boundary, plus spot checks on asymmetric-magnitude normalization.

use spirix::*;

type C = CircleF5E3;
type S = ScalarF5E3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Class {
    Zero,
    Vanished,
    Normal,
    Exploded,
    Infinity,
    Undefined,
}

fn classify_scalar(s: &S) -> Class {
    if s.is_undefined() { Class::Undefined }
    else if s.is_zero() { Class::Zero }
    else if s.is_infinite() { Class::Infinity }
    else if s.exploded() { Class::Exploded }
    else if s.vanished() { Class::Vanished }
    else { Class::Normal }
}

fn class_name(c: Class) -> &'static str {
    match c {
        Class::Zero => "[0]",
        Class::Vanished => "[↓]",
        Class::Normal => "[#]",
        Class::Exploded => "[↑]",
        Class::Infinity => "[∞]",
        Class::Undefined => "[℘]",
    }
}

fn check_extraction(label: &str, c: C, expected_r: &[Class], expected_i: &[Class]) {
    let rc = classify_scalar(&c.r());
    let ic = classify_scalar(&c.i());
    if !expected_r.contains(&rc) {
        panic!("{}: r() class = {}, expected one of {:?}",
            label, class_name(rc),
            expected_r.iter().map(|c| class_name(*c)).collect::<Vec<_>>());
    }
    if !expected_i.contains(&ic) {
        panic!("{}: i() class = {}, expected one of {:?}",
            label, class_name(ic),
            expected_i.iter().map(|c| class_name(*c)).collect::<Vec<_>>());
    }
}

use Class::*;

// ─────────────────────────────────────────────────────────────────────────────
// Circle → Scalar class truth table (via .r() and .i())
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn circle_zero_extracts_to_zeros() {
    check_extraction("ZERO", C::ZERO, &[Zero], &[Zero]);
}

#[test]
fn circle_infinity_extracts_to_infinities() {
    check_extraction("INFINITY", C::INFINITY, &[Infinity], &[Infinity]);
}

#[test]
fn circle_normal_nonzero_components() {
    // Both components normal
    check_extraction("1+1i",   C::from((1.0, 1.0)), &[Normal], &[Normal]);
    check_extraction("3+4i",   C::from((3.0, 4.0)), &[Normal], &[Normal]);
    check_extraction("-2+5i",  C::from((-2.0, 5.0)), &[Normal], &[Normal]);
    check_extraction("42+1.5i",C::from((42.0, 1.5)), &[Normal], &[Normal]);
}

#[test]
fn circle_normal_zero_component() {
    // One component zero, other non-zero — expect that component to extract as Zero
    check_extraction("1+0i", C::from((1.0, 0.0)), &[Normal], &[Zero]);
    check_extraction("0+1i", C::from((0.0, 1.0)), &[Zero], &[Normal]);
    check_extraction("-42+0i", C::from((-42.0, 0.0)), &[Normal], &[Zero]);
}

#[test]
fn circle_exploded_extracts_to_exploded() {
    let e_pos = C::from((f64::MAX, 0.0)); // should explode the real
    let e_neg = C::from((-f64::MAX, 0.0));
    if e_pos.exploded() {
        check_extraction("EXPLODED+", e_pos, &[Exploded, Infinity], &[Exploded, Zero, Infinity]);
    }
    if e_neg.exploded() {
        check_extraction("EXPLODED-", e_neg, &[Exploded, Infinity], &[Exploded, Zero, Infinity]);
    }
}

#[test]
fn circle_vanished_extracts_to_vanished() {
    let v_pos = C::from((f64::MIN_POSITIVE, 0.0));
    if v_pos.vanished() {
        check_extraction("VANISHED+", v_pos, &[Vanished, Zero], &[Vanished, Zero]);
    }
}

#[test]
fn circle_undefined_extracts_to_undefined() {
    let u = C::ZERO / C::ZERO;
    assert!(u.is_undefined());
    check_extraction("UNDEFINED", u, &[Undefined], &[Undefined]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Scalar → Circle class truth table (via from_ri / from((r, i)))
// ─────────────────────────────────────────────────────────────────────────────

// Representatives for each Scalar class
fn reps() -> [(&'static str, Class, S); 9] {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;
    [
        ("[0]",  Zero,      z),
        ("[+↓]", Vanished,  vp),
        ("[-↓]", Vanished,  vn),
        ("[+#]", Normal,    np),
        ("[-#]", Normal,    nn),
        ("[+↑]", Exploded,  ep),
        ("[-↑]", Exploded,  en),
        ("[∞]",  Infinity,  inf),
        ("[℘]",  Undefined, und),
    ]
}

/// Combine two Scalars into a Circle via from_ri, check Circle's class.
fn circle_class(c: C) -> Class {
    if c.is_undefined() { Class::Undefined }
    else if c.is_zero() { Class::Zero }
    else if c.is_infinite() { Class::Infinity }
    else if c.exploded() { Class::Exploded }
    else if c.vanished() { Class::Vanished }
    else { Class::Normal }
}

fn expect_circle(real: S, imag: S, real_name: &str, imag_name: &str, allowed: &[Class]) {
    let c = C::from_ri(real, imag);
    let cc = circle_class(c);
    if !allowed.contains(&cc) {
        panic!(
            "from_ri({} real, {} imag) → Circle class {} expected one of {:?}\n  circle: real={:#x} imag={:#x} exp={}",
            real_name, imag_name, class_name(cc),
            allowed.iter().map(|c| class_name(*c)).collect::<Vec<_>>(),
            c.real as u32, c.imaginary as u32, c.exponent
        );
    }
}

#[test]
fn from_ri_zero_plus_zero_is_zero() {
    expect_circle(S::ZERO, S::ZERO, "[0]", "[0]", &[Zero]);
}

#[test]
fn from_ri_normal_components_is_normal() {
    let np = S::from(7);
    let nn = S::from(-7);
    expect_circle(np, np, "[+#]", "[+#]", &[Normal]);
    expect_circle(np, nn, "[+#]", "[-#]", &[Normal]);
    expect_circle(nn, np, "[-#]", "[+#]", &[Normal]);
    expect_circle(nn, nn, "[-#]", "[-#]", &[Normal]);
}

#[test]
fn from_ri_zero_with_normal_is_normal() {
    let np = S::from(7);
    let nn = S::from(-7);
    expect_circle(np, S::ZERO, "[+#]", "[0]", &[Normal]);
    expect_circle(S::ZERO, np, "[0]", "[+#]", &[Normal]);
    expect_circle(nn, S::ZERO, "[-#]", "[0]", &[Normal]);
    expect_circle(S::ZERO, nn, "[0]", "[-#]", &[Normal]);
}

#[test]
fn from_ri_any_undefined_is_undefined() {
    let und = S::ZERO / S::ZERO;
    let np = S::from(7);
    expect_circle(und, np, "[℘]", "[+#]", &[Undefined]);
    expect_circle(np, und, "[+#]", "[℘]", &[Undefined]);
    expect_circle(und, und, "[℘]", "[℘]", &[Undefined]);
}

#[test]
fn from_ri_any_infinite_is_infinity() {
    let inf = S::INFINITY;
    let np = S::from(7);
    expect_circle(inf, np, "[∞]", "[+#]", &[Infinity]);
    expect_circle(np, inf, "[+#]", "[∞]", &[Infinity]);
    expect_circle(inf, inf, "[∞]", "[∞]", &[Infinity]);
}

#[test]
fn from_ri_both_vanished_is_undefined_or_vanished() {
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    // Both vanished → magnitude is uncertain → INDETERMINATE (an undefined sub-state)
    expect_circle(vp, vp, "[+↓]", "[+↓]", &[Undefined, Vanished]);
    expect_circle(vp, vn, "[+↓]", "[-↓]", &[Undefined, Vanished]);
}

#[test]
fn from_ri_exploded_or_vanished_collapses_to_undefined() {
    // Component mix at the edges is often indeterminate
    let ep = S::EXPLODED_POS;
    let vp = S::VANISHED_POS;
    expect_circle(ep, vp, "[+↑]", "[+↓]", &[Undefined, Exploded, Vanished]);
    expect_circle(vp, ep, "[+↓]", "[+↑]", &[Undefined, Exploded, Vanished]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Offset magnitudes — normalization stress (the motivating cases)
// ─────────────────────────────────────────────────────────────────────────────

fn close(a: f64, b: f64, rel: f64) -> bool {
    if a == b { return true; }
    if b == 0.0 { return a.abs() < rel; }
    ((a - b).abs() / b.abs()) < rel
}

fn check_values(label: &str, c: C, expected_r: f64, expected_i: f64, rel: f64) {
    let r: f64 = c.r().into();
    let i: f64 = c.i().into();
    assert!(close(r, expected_r, rel),
        "{}: r()={} expected {} (rel tol {})", label, r, expected_r, rel);
    assert!(close(i, expected_i, rel),
        "{}: i()={} expected {} (rel tol {})", label, i, expected_i, rel);
}

#[test]
fn offset_user_case_42_plus_1p5i() {
    check_values("42+1.5i", C::from((42.0, 1.5)), 42.0, 1.5, 1e-5);
}

#[test]
fn offset_various_asymmetric() {
    let cases: &[(f64, f64)] = &[
        (42.0, 1.5),
        (100.0, 0.5),
        (1024.0, 1.0),
        (10.0, 0.1),
        (1.5, 42.0),
        (0.5, 100.0),
        (1.0, 1024.0),
        (-42.0, 1.5),
        (42.0, -1.5),
        (-42.0, -1.5),
        (1.5, -42.0),
    ];
    for &(r, i) in cases {
        check_values(&format!("{}+{}i", r, i), C::from((r, i)), r, i, 1e-5);
    }
}

#[test]
fn offset_all_four_sign_quadrants() {
    for &mr in &[3.0_f64, 42.0, 0.5] {
        for &mi in &[4.0_f64, 1.5, 0.25] {
            for &sr in &[1.0, -1.0] {
                for &si in &[1.0, -1.0] {
                    let r = mr * sr;
                    let i = mi * si;
                    check_values(&format!("{}+{}i", r, i), C::from((r, i)), r, i, 1e-5);
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Round-trip stability: Circle → Scalars → Circle → Scalars
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn round_trip_stability() {
    let cases: &[(f64, f64)] = &[
        (1.0, 1.0), (3.0, 4.0), (42.0, 1.5), (-2.0, 3.0), (0.5, 0.25),
        (100.0, 0.01), (0.01, 100.0), (-42.0, -1.5),
    ];
    for &(r, i) in cases {
        let c1 = C::from((r, i));
        let s_r1 = c1.r();
        let s_i1 = c1.i();
        let c2 = C::from_ri(s_r1, s_i1);
        let s_r2 = c2.r();
        let s_i2 = c2.i();
        let r1: f64 = s_r1.into();
        let i1: f64 = s_i1.into();
        let r2: f64 = s_r2.into();
        let i2: f64 = s_i2.into();
        assert!(close(r1, r2, 1e-10), "{}: r stable: {} vs {}", r, r1, r2);
        assert!(close(i1, i2, 1e-10), "{}: i stable: {} vs {}", i, i1, i2);
    }
}
