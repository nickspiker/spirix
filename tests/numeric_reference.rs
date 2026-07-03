//! Numeric-reference truth tables: run every op over a fixed set of representative values and check the result against an IEEE `f64` oracle.
//!
//! The rule, per op and input(s):
//! - oracle is a *moderate finite* number (|x| in [1e-250, 1e250]) → Spirix must be Normal and equal to the oracle within a relative tolerance.
//! - oracle is `NaN` (a domain error like `sqrt(-4)`, `ln(-1)`, `(-2)^0.5`) → Spirix must be Undefined.
//! - oracle is `0`, `±inf`, or beyond the f64 range → SKIPPED here: Spirix's range and its Zero/Vanished/Exploded/Infinity distinctions exceed what f64 can witness, so those are covered by the class truth tables (with escaped/infinite representatives) instead.
//!
//! So this file pins the *values* on the meat of the domain and the *domain errors*; the class edges live alongside it.
//! Type is `ScalarF6E5` (i64 fraction ≈ 18 digits, i32 exponent) so `2^32`, `2^-32`, `1/42` are all exact and in range.

use spirix::*;

type S = ScalarF6E5;

/// Relative tolerance for value comparison. Loose enough for the transcendental series at this width, tight enough to catch a wrong formula or a leaked escape value.
const TOL: f64 = 1e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Class {
    Zero,
    Vanished,
    Normal,
    Exploded,
    Infinity,
    Undefined,
}
use Class::*;

fn classify(s: &S) -> Class {
    if s.is_undefined() {
        Undefined
    } else if s.is_zero() {
        Zero
    } else if s.is_infinite() {
        Infinity
    } else if s.exploded() {
        Exploded
    } else if s.vanished() {
        Vanished
    } else {
        Normal
    }
}

/// The shared value set: label, Spirix value, and its exact f64 oracle.
/// Finite entries span small/large magnitudes and both signs; specials carry the class the oracle can't represent (their f64 is a sentinel used only when they appear as an *operand*, never asserted on).
fn finite_set() -> Vec<(&'static str, S, f64)> {
    let p2_32 = 4_294_967_296_i64; // 2^32
    vec![
        ("0", S::ZERO, 0.0),
        ("1", S::from(1), 1.0),
        ("-1", S::from(-1), -1.0),
        ("1/2", S::from(1) / S::from(2), 0.5),
        ("-1/2", S::from(-1) / S::from(2), -0.5),
        ("2", S::from(2), 2.0),
        ("-2", S::from(-2), -2.0),
        ("42", S::from(42), 42.0),
        ("-42", S::from(-42), -42.0),
        ("1/42", S::from(1) / S::from(42), 1.0 / 42.0),
        ("-1/42", S::from(-1) / S::from(42), -1.0 / 42.0),
        ("2^32", S::from(p2_32), p2_32 as f64),
        ("-2^32", S::from(-p2_32), -(p2_32 as f64)),
        ("2^-32", S::from(1) / S::from(p2_32), 1.0 / p2_32 as f64),
        ("-2^-32", S::from(-1) / S::from(p2_32), -1.0 / p2_32 as f64),
    ]
}

/// Assert a single result against its oracle, per the rules in the module doc.
/// Returns `true` if a strict check was applied (for coverage counting), `false` if skipped.
fn check(label: &str, r: S, oracle: f64) -> bool {
    let rc = classify(&r);
    if oracle.is_nan() {
        assert!(
            rc == Undefined,
            "{label}: oracle is NaN (domain error) → expected Undefined, got {rc:?} = {r}"
        );
        return true;
    }
    if oracle.is_infinite() || oracle == 0.0 {
        return false; // range/zero distinctions are the class tables' job
    }
    if oracle.abs() > 1e250 || oracle.abs() < 1e-250 {
        return false; // near f64's own limits — Spirix's wider range makes this ambiguous
    }
    // Oracle is a moderate finite number: Spirix must land Normal on the same value.
    assert!(
        rc == Normal,
        "{label}: oracle ≈ {oracle} (finite) → expected Normal, got {rc:?} = {r}"
    );
    let got = r.to_f64();
    let tol = TOL * oracle.abs();
    assert!(
        (got - oracle).abs() <= tol,
        "{label}: value {got} != oracle {oracle} (|Δ| {:.3e} > tol {:.3e})",
        (got - oracle).abs(),
        tol
    );
    true
}

fn run_unary(name: &str, f: impl Fn(S) -> S, oracle: impl Fn(f64) -> f64) {
    let mut checked = 0usize;
    for (la, a, fa) in finite_set() {
        let label = format!("{name}({la})");
        if check(&label, f(a), oracle(fa)) {
            checked += 1;
        }
    }
    assert!(checked > 0, "{name}: no strict checks ran — value set too narrow?");
}

fn run_binary(name: &str, f: impl Fn(S, S) -> S, oracle: impl Fn(f64, f64) -> f64) {
    run_binary_where(name, |_, _| false, f, oracle)
}

/// Like [`run_binary`] but skips input pairs for which `skip(fa, fb)` is true — used to keep an op's genuine class edges (e.g. mod-by-zero) out of the numeric-value layer.
fn run_binary_where(
    name: &str,
    skip: impl Fn(f64, f64) -> bool,
    f: impl Fn(S, S) -> S,
    oracle: impl Fn(f64, f64) -> f64,
) {
    let mut checked = 0usize;
    let set = finite_set();
    for (la, a, fa) in &set {
        for (lb, b, fb) in &set {
            if skip(*fa, *fb) {
                continue;
            }
            let label = format!("{la} {name} {lb}");
            if check(&label, f(*a, *b), oracle(*fa, *fb)) {
                checked += 1;
            }
        }
    }
    assert!(checked > 0, "{name}: no strict checks ran?");
}

// ===================================================================== Unary =====

#[test]
fn u_neg() {
    run_unary("neg", |a| -a, |x| -x);
}
#[test]
fn u_abs() {
    run_unary("abs", |a| a.magnitude(), |x| x.abs());
}
#[test]
fn u_square() {
    run_unary("square", |a| a.square(), |x| x * x);
}
#[test]
fn u_recip() {
    run_unary("recip", |a| a.reciprocal(), |x| 1.0 / x);
}
#[test]
fn u_sqrt() {
    run_unary("sqrt", |a| a.sqrt(), |x| x.sqrt());
}
#[test]
fn u_ln() {
    run_unary("ln", |a| a.ln(), |x| x.ln());
}
#[test]
fn u_lb() {
    run_unary("lb", |a| a.lb(), |x| x.log2());
}
#[test]
fn u_exp() {
    run_unary("exp", |a| a.exp(), |x| x.exp());
}
#[test]
fn u_powb() {
    run_unary("powb", |a| a.powb(), |x| x.exp2());
}
#[test]
fn u_floor() {
    run_unary("floor", |a| a.floor(), |x| x.floor());
}
#[test]
fn u_ceil() {
    run_unary("ceil", |a| a.ceil(), |x| x.ceil());
}
#[test]
fn u_round() {
    // Spirix rounds half-to-even (banker's). `1/2` and `-1/2` in the set ARE ties → oracle must match, so use round_ties_even (round(0.5) = 0, round(-0.5) = 0).
    run_unary("round", |a| a.round(), |x| x.round_ties_even());
}
#[test]
fn u_frac() {
    // Spirix frac is floor-based: x - floor(x), always in [0, 1). (Rust's fract() is trunc-based.)
    run_unary("frac", |a| a.frac(), |x| x - x.floor());
}
#[test]
fn u_sin() {
    run_unary("sin", |a| a.sin(), |x| x.sin());
}
#[test]
fn u_cos() {
    run_unary("cos", |a| a.cos(), |x| x.cos());
}
#[test]
fn u_tan() {
    run_unary("tan", |a| a.tan(), |x| x.tan());
}
#[test]
fn u_asin() {
    run_unary("asin", |a| a.asin(), |x| x.asin());
}
#[test]
fn u_acos() {
    run_unary("acos", |a| a.acos(), |x| x.acos());
}
#[test]
fn u_atan() {
    run_unary("atan", |a| a.atan(), |x| x.atan());
}
#[test]
fn u_sinh() {
    run_unary("sinh", |a| a.sinh(), |x| x.sinh());
}
#[test]
fn u_cosh() {
    run_unary("cosh", |a| a.cosh(), |x| x.cosh());
}
#[test]
fn u_tanh() {
    run_unary("tanh", |a| a.tanh(), |x| x.tanh());
}

// ==================================================================== Binary =====

#[test]
fn b_add() {
    run_binary("+", |a, b| a + b, |x, y| x + y);
}
#[test]
fn b_sub() {
    run_binary("-", |a, b| a - b, |x, y| x - y);
}
#[test]
fn b_mul() {
    run_binary("*", |a, b| a * b, |x, y| x * y);
}
#[test]
fn b_div() {
    run_binary("/", |a, b| a / b, |x, y| x / y);
}
#[test]
fn b_pow() {
    run_binary("^", |a, b| a.pow(b), |x, y| x.powf(y));
}
#[test]
fn b_log() {
    // a $ b = log base b of a.
    run_binary("$", |a, b| a.log(b), |x, y| x.log(y));
}
#[test]
fn b_min() {
    run_binary("min", |a, b| a.min(b), |x, y| x.min(y));
}
#[test]
fn b_max() {
    run_binary("max", |a, b| a.max(b), |x, y| x.max(y));
}
#[test]
fn b_clamp() {
    // Clamp each value into [-1, 1] — a fixed window that exercises below / inside / above.
    let lo = S::from(-1);
    let hi = S::from(1);
    run_unary("clamp[-1,1]", |a| a.clamp(lo, hi), |x| x.clamp(-1.0, 1.0));
}
#[test]
fn b_atan2() {
    // y.atan2(x): oracle is f64 atan2.
    run_binary("atan2", |y, x| y.atan2(x), |y, x| y.atan2(x));
}
#[test]
fn b_shl() {
    // Shift is ×2ⁿ by an integer amount; test each set value against small shift counts so the result stays in a witnessable range. Oracle: a * 2^n.
    let amounts: [(i32, f64); 6] = [
        (0, 1.0),
        (1, 2.0),
        (2, 4.0),
        (10, 1024.0),
        (-1, 0.5),
        (-10, 1.0 / 1024.0),
    ];
    let mut checked = 0usize;
    for (la, a, fa) in finite_set() {
        for (n, scale) in amounts {
            let label = format!("{la} << {n}");
            if check(&label, a << n, fa * scale) {
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
#[test]
fn b_shr() {
    let amounts: [(i32, f64); 6] = [
        (0, 1.0),
        (1, 0.5),
        (2, 0.25),
        (10, 1.0 / 1024.0),
        (-1, 2.0),
        (-10, 1024.0),
    ];
    let mut checked = 0usize;
    for (la, a, fa) in finite_set() {
        for (n, scale) in amounts {
            let label = format!("{la} >> {n}");
            if check(&label, a >> n, fa * scale) {
                checked += 1;
            }
        }
    }
    assert!(checked > 0);
}
#[test]
fn b_mod() {
    // Spirix "proper modulus": result sign follows the divisor (floored modulus), unlike f64's truncated `%` (sign of dividend). Oracle uses the floored form.
    // Mod-by-zero is a class edge (Spirix defines [0]%[0]=[0], n%0 escapes), handled by the class tables, so skip it.
    run_binary_where(
        "%",
        |_, y| y == 0.0,
        |a, b| a % b,
        |x, y| x - y * (x / y).floor(),
    );
}

// ==================================================== Bitwise values & randoms =====

#[test]
fn b_bitwise_values() {
    // Spirix bitwise is two's-complement aligned at the binary point — no f64 oracle exists, so pin known values directly.
    let v = |x: S| x.to_f64();
    let six = S::from(6);
    let five = S::from(5);
    let three = S::from(3);
    // Integer patterns.
    assert_eq!(v(six & three), 2.0, "6 & 3");
    assert_eq!(v(six | three), 7.0, "6 | 3");
    assert_eq!(v(six ^ three), 5.0, "6 ⊻ 3");
    assert_eq!(v(five & three), 1.0, "5 & 3");
    assert_eq!(v(five | three), 7.0, "5 | 3");
    assert_eq!(v(five ^ three), 6.0, "5 ⊻ 3");
    // Fractional alignment: 2.5 = 10.1₂, 1.5 = 01.1₂ → and 00.1₂ = 0.5, or 11.1₂ = 3.5, xor 11.0₂ = 3.
    let half = S::from(1) / S::from(2);
    let two5 = S::from(5) * half;
    let one5 = S::from(3) * half;
    assert_eq!(v(two5 & one5), 0.5, "2.5 & 1.5");
    assert_eq!(v(two5 | one5), 3.5, "2.5 | 1.5");
    assert_eq!(v(two5 ^ one5), 3.0, "2.5 ⊻ 1.5");
    // Identities.
    assert_eq!(six & six, six, "x & x = x");
    assert_eq!(six | six, six, "x | x = x");
    assert!((six ^ six).is_zero(), "x ⊻ x = 0");
    assert_eq!(!!six, six, "!!x = x");
    // -1 is all-ones in two's complement: AND-identity, OR-absorber.
    let n1 = S::from(-1);
    assert_eq!(v(n1 & six), 6.0, "-1 & x = x");
    assert_eq!(v(n1 | six), -1.0, "-1 | x = -1");
}

/// Class + range + statistical sanity for the random generators. Bounds sit >8 sigma from the expected statistics at n=10000, so a healthy generator can't flake this test; a broken one (stuck bit, wrong normalization, class leak) lands far outside.
#[test]
fn rand_uniform_statistics() {
    let n = 10_000usize;
    let mut sum = 0.0f64;
    let mut sumsq = 0.0f64;
    let mut distinct = std::collections::BTreeSet::new();
    for _ in 0..n {
        let r = S::random();
        assert!(!r.is_undefined() && !r.is_infinite() && !r.exploded(), "random() must never produce undefined/∞/exploded, got {r:?}");
        let v = r.to_f64();
        assert!((-1.0..=1.0).contains(&v), "random() out of [-1,1]: {v}");
        sum += v;
        sumsq += v * v;
        distinct.insert(v.to_bits());
    }
    let mean = sum / n as f64;
    let var = sumsq / n as f64 - mean * mean;
    assert!(mean.abs() < 0.05, "uniform mean drifted: {mean}");
    assert!((0.28..=0.39).contains(&var), "uniform variance off (expect ≈1/3): {var}");
    assert!(distinct.len() > n / 2, "generator not varying: {} distinct of {n}", distinct.len());
}

#[test]
fn rand_gauss_statistics() {
    let n = 10_000usize;
    let mut sum = 0.0f64;
    let mut sumsq = 0.0f64;
    let mut absmax = 0.0f64;
    for _ in 0..n {
        let r = S::random_gauss();
        assert!(!r.is_undefined() && !r.is_infinite() && !r.exploded(), "random_gauss() must never produce undefined/∞/exploded, got {r:?}");
        let v = r.to_f64();
        sum += v;
        sumsq += v * v;
        absmax = absmax.max(v.abs());
    }
    let mean = sum / n as f64;
    let var = sumsq / n as f64 - mean * mean;
    assert!(mean.abs() < 0.08, "gauss mean drifted: {mean}");
    assert!((0.85..=1.15).contains(&var), "gauss variance off (expect ≈1): {var}");
    assert!(absmax < 8.0, "gauss tail implausible (P(|z|>8) ≈ 1e-15): {absmax}");
    assert!(absmax > 2.0, "gauss tails missing (10k samples should exceed 2σ): {absmax}");
}

#[test]
fn rand_works_at_narrow_width() {
    // The 8-bit-fraction type exercises the normalization edge cases hardest.
    for _ in 0..2_000 {
        let r = ScalarF3E3::random();
        assert!(!r.is_undefined() && !r.is_infinite() && !r.exploded(), "F3E3 random() bad class: {r:?}");
        let v = r.to_f64();
        assert!((-1.0..=1.0).contains(&v), "F3E3 random() out of [-1,1]: {v}");
    }
}

#[test]
fn constants_match_reference() {
    // Every constant the formula layer exposes, against its f64 reference.
    let cases: [(&str, S, f64); 7] = [
        ("PI", S::PI, std::f64::consts::PI),
        ("TAU", S::TAU, std::f64::consts::TAU),
        ("E", S::E, std::f64::consts::E),
        ("LN_TWO", S::LN_TWO, std::f64::consts::LN_2),
        ("PHI", S::PHI, 1.618_033_988_749_894_8),
        ("EULER_GAMMA", S::EULER_GAMMA, 0.577_215_664_901_532_9),
        ("CATALAN", S::CATALAN, 0.915_965_594_177_219_0),
    ];
    for (name, s, want) in cases {
        let got = s.to_f64();
        assert!(((got - want) / want).abs() < 1e-12, "{name}: {got} vs {want}");
    }
}

#[test]
fn rand_binade_occupancy_and_endpoints() {
    // Locks the sampler's construction at F3E3 (narrow width makes endpoints reachable):
    // - interval is [-1, +1): exact -1 IS drawable (fraction 0 in the top binade is the NEG_ONE pattern, P = 1/512), exact +1 is NOT (positive significands cap at 1.9921875 × 2⁻¹).
    // - binade occupancy halves per level (P(|x| ∈ [2⁻ᵏ⁻¹, 2⁻ᵏ)) = 2⁻ᵏ⁻¹) — the "proper fraction extension" property: fresh full-width significand per draw, geometric binade selection, no zero-fill.
    // - exact zero is never drawn (measure zero on a real uniform).
    type T = ScalarF3E3;
    let n = 100_000usize;
    let (mut exact_neg1, mut exact_pos1, mut zeros) = (0usize, 0usize, 0usize);
    let mut binade = [0usize; 3];
    let mut max_seen = -2.0f64;
    for _ in 0..n {
        let r = T::random();
        if r == T::NEG_ONE {
            exact_neg1 += 1;
        }
        if r == T::ONE {
            exact_pos1 += 1;
        }
        if r.is_zero() {
            zeros += 1;
        }
        let v = r.to_f64();
        max_seen = max_seen.max(v);
        let a = v.abs();
        for k in 0..3 {
            if a < 2f64.powi(-(k as i32)) && a >= 2f64.powi(-(k as i32) - 1) {
                binade[k] += 1;
            }
        }
    }
    assert_eq!(exact_pos1, 0, "+1 must be unreachable ([-1, +1))");
    assert_eq!(zeros, 0, "exact zero must never be drawn");
    // Theory n/512 ≈ 195, sigma ≈ 14; bounds sit ~±8 sigma out.
    assert!((80..=320).contains(&exact_neg1), "exact -1 rate off (theory ≈195): {exact_neg1}");
    assert!(max_seen < 1.0, "max must stay below +1: {max_seen}");
    // First three binades: theory n/2, n/4, n/8 — allow ±10% relative.
    for (k, &c) in binade.iter().enumerate() {
        let want = n >> (k + 1);
        let lo = want - want / 10;
        let hi = want + want / 10;
        assert!((lo..=hi).contains(&c), "binade {k} occupancy {c} outside [{lo}, {hi}] (theory {want})");
    }
}
