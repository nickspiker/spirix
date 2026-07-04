//! Circle numeric-reference: complex ops against the `num_complex::Complex<f64>` oracle, and the Scalar↔Circle consistency axis (a real-axis Circle must behave like its Scalar).
//! Complements the class truth tables the same way tests/numeric_reference.rs does for Scalar: this layer pins VALUES on the meat of the domain; class edges live in tests/truth_tables.rs.

use num_complex::Complex;
use spirix::*;

type C = CircleF6E5;
type S = ScalarF6E5;

/// Norm-relative complex tolerance. Circle transcendentals chain thru atan2/sin/cos series, so the bound is looser than raw fraction precision but tight enough to catch a wrong branch or formula.
const TOL: f64 = 1e-9;

fn to_c64(c: &C) -> Complex<f64> {
    c.into()
}

/// Compare a Circle result to the oracle by norm-relative distance.
fn check(what: &str, got: C, want: Complex<f64>) {
    // Oracle exactly zero (e.g. z - z): Spirix's additive identity gives the true Zero class.
    if want.norm() == 0.0 {
        assert!(got.is_zero(), "{what}: expected Zero, got {got:?}");
        return;
    }
    assert!(got.is_normal(), "{what}: expected normal ≈ {want}, got {got:?}");
    let g = to_c64(&got);
    let d = (g - want).norm();
    let scale = want.norm().max(1e-300);
    assert!(d / scale <= TOL, "{what}: {g} != oracle {want} (rel {:.3e})", d / scale);
}

/// The complex value set: axis points, quadrant representatives, small/large magnitudes.
fn value_set() -> Vec<(Complex<f64>, C)> {
    let pts = [
        (3.0, 4.0),
        (1.0, 1.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.5, -0.25),
        (-2.0, 3.0),
        (-0.75, -0.5),
        (42.0, -1.0),
        (1.0 / 42.0, 2.0),
        // Two's-complement boundary shapes: components at exactly -2^k stress the |MIN| asymmetry in the wide pipelines.
        (-1.0, -1.0),
        (0.0, -1.0),
        (-2.0, 0.0),
    ];
    pts.iter().map(|&(r, i)| (Complex::new(r, i), C::from((r, i)))).collect()
}

#[test]
fn circle_binary_ops_vs_oracle() {
    let set = value_set();
    for (wa, ca) in &set {
        for (wb, cb) in &set {
            check(&format!("({wa})+({wb})"), *ca + *cb, wa + wb);
            check(&format!("({wa})-({wb})"), *ca - *cb, wa - wb);
            check(&format!("({wa})*({wb})"), *ca * *cb, wa * wb);
            check(&format!("({wa})/({wb})"), *ca / *cb, wa / wb);
        }
    }
}

#[test]
fn circle_unary_ops_vs_oracle() {
    for (w, c) in value_set() {
        check(&format!("-({w})"), -c, -w);
        check(&format!("conj({w})"), c.conjugate(), w.conj());
        check(&format!("recip({w})"), c.reciprocal(), w.inv());
        check(&format!("square({w})"), c.square(), w * w);
        check(&format!("sqrt({w})"), c.sqrt(), w.sqrt());
        check(&format!("exp({w})"), c.exp(), w.exp());
        check(&format!("ln({w})"), c.ln(), w.ln());
        // magnitude is a Scalar — compare directly.
        let m = c.magnitude().to_f64();
        let want = w.norm();
        assert!(((m - want) / want).abs() < TOL, "|{w}|: {m} != {want}");
    }
}

#[test]
fn circle_pow_vs_oracle() {
    let z = C::from((3.0, 4.0));
    let wz = Complex::new(3.0, 4.0);
    // Scalar exponents: integer (exact multiply chain) and fractional (ln/exp path).
    for &p in &[2.0f64, 3.0, -1.0, -2.0, 0.5, 2.5, -1.5] {
        check(&format!("(3+4i)^{p}"), z.pow(S::from(p)), wz.powf(p));
    }
    // Complex exponents thru powc.
    for &(pr, pi) in &[(0.5f64, 0.5f64), (1.0, 1.0), (0.0, 1.0), (-1.0, 0.5)] {
        let w = Complex::new(pr, pi);
        check(&format!("(3+4i)^({w})"), z.pow(C::from((pr, pi))), wz.powc(w));
    }
    // i^i is real: e^(-π/2).
    let ii = C::POS_I.pow(C::POS_I);
    let want = (-std::f64::consts::FRAC_PI_2).exp();
    let got = ii.r().to_f64();
    assert!(((got - want) / want).abs() < TOL, "i^i: {got} != {want}");
}

#[test]
fn real_axis_circle_matches_scalar() {
    // A Circle on the real axis must agree with the Scalar for every shared op — the two types are one number system, not two.
    let xs = [3.0f64, 0.5, 42.0, 1.0 / 42.0, 2.0, 1.5];
    for &x in &xs {
        let c = C::from((x, 0.0));
        let s = S::from(x);
        let pairs: [(&str, C, S); 6] = [
            ("sqrt", c.sqrt(), s.sqrt()),
            ("exp", c.exp(), s.exp()),
            ("ln", c.ln(), s.ln()),
            ("square", c.square(), s.square()),
            ("recip", c.reciprocal(), s.reciprocal()),
            ("neg", -c, -s),
        ];
        for (name, cr, sr) in pairs {
            let cv = cr.r().to_f64();
            let sv = sr.to_f64();
            let rel = ((cv - sv) / sv.abs().max(1e-300)).abs();
            assert!(rel < 1e-9, "{name}({x}): circle {cv} vs scalar {sv} (rel {rel:.3e})");
            assert!(cr.i().is_negligible() || cr.i().to_f64().abs() < 1e-9 * cv.abs().max(1.0), "{name}({x}): imaginary leaked {:?}", cr.i());
        }
        // Binary ops between real-axis circles.
        let c2 = C::from((2.5, 0.0));
        let s2 = S::from(2.5);
        let bins: [(&str, C, S); 4] = [
            ("+", c + c2, s + s2),
            ("-", c - c2, s - s2),
            ("*", c * c2, s * s2),
            ("/", c / c2, s / s2),
        ];
        for (name, cr, sr) in bins {
            let rel = ((cr.r().to_f64() - sr.to_f64()) / sr.to_f64().abs().max(1e-300)).abs();
            assert!(rel < 1e-12, "{x} {name} 2.5: circle vs scalar rel {rel:.3e}");
        }
    }
    // Negative real axis: the Circle resolves what the Scalar cannot — sqrt(-4) = 2i, ln(-1) = iπ.
    let neg = C::from((-4.0, 0.0));
    check("sqrt(-4)", neg.sqrt(), Complex::new(0.0, 2.0));
    check("ln(-1)", C::from((-1.0, 0.0)).ln(), Complex::new(0.0, std::f64::consts::PI));
    // While the Scalar goes undefined — both answers are right, for their domains.
    assert!(S::from(-4).sqrt().is_undefined());
    assert!(S::from(-1).ln().is_undefined());
}
