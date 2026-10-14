//! Numeric edge regressions found by audit: Circle magnitude, sqrt and ln across the whole exponent range, Scalar sinh / cosh / tanh at large |x|, the wide-overflow sign flip in Circle division, the bottom-binade Scalar reciprocal, and frac of a tiny negative.
//! Oracles are f64 and num_complex::Complex<f64>. Circle values are decoded straight from the raw fields (component × 2^(k - FRAC + 2)), so the oracle never goes thru the component extraction under test.

use num_complex::Complex;
use spirix::*;

type C = CircleF4E3;
type S = ScalarF4E3;

/// Smallest and largest normal magnitudes at E3: [2^-127, 2^128).
const MIN_NORMAL: f64 = 5.877471754111438e-39;
const MAX_NORMAL: f64 = 3.402823669209385e38;

/// True signed exponent to the stored cyclic exponent.
fn stored(k: i32) -> i8 {
    (k as i8) ^ i8::MIN
}

/// Exact value of a normal F4E3 Circle from its raw fields.
fn raw_c(c: &C) -> Complex<f64> {
    let k = (c.exponent ^ i8::MIN) as i32;
    let scale = 2f64.powi(k - 16 + 2);
    Complex::new(c.real as f64 * scale, c.imaginary as f64 * scale)
}

/// Fraction pairs run at every exponent: canonical shapes, the two's-complement boundary, a minor component far below the major one, and non-canonical (pub-field) pairs.
const PAIRS: [(i16, i16); 9] = [
    (0x3000, 0x4000),
    (0x4000, 0x2000),
    (0x4000, 0x0100),
    (0x4000, 0x0001),
    (-0x8000, -0x8000),
    (0x7fff, -0x7fff),
    (-0x2a3c, 0x1d17),
    (0x0003, 0x0004),
    (-0x0001, 0x0040),
];

/// Every normal exponent of every pair.
fn circles() -> impl Iterator<Item = C> {
    PAIRS.iter().flat_map(|&(r, i)| {
        (-127..=127).map(move |k| C {
            real: r,
            imaginary: i,
            exponent: stored(k),
        })
    })
}

#[test]
fn circle_magnitude_across_exponent_range() {
    let mut failures = Vec::new();
    for c in circles() {
        let want = raw_c(&c).norm();
        let got = c.magnitude();
        let ok = if want >= MAX_NORMAL {
            got.exploded() && got.is_positive()
        } else if want < MIN_NORMAL {
            got.vanished() && got.is_positive()
        } else {
            got.is_normal() && ((got.to_f64() - want) / want).abs() < 2f64.powi(-12)
        };
        if !ok {
            failures.push(format!("|{c:?}| = {got:?}, want {want:e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn circle_magnitude_squared_across_exponent_range() {
    // At the stored exponent the minor square could vanish under a normal sum, and an exploded square plus a normal one was undefined.
    let mut failures = Vec::new();
    for c in circles() {
        let want = raw_c(&c).norm_sqr();
        let got = c.magnitude_squared();
        let ok = if want >= MAX_NORMAL {
            got.exploded() && got.is_positive()
        } else if want < MIN_NORMAL {
            got.vanished() && got.is_positive()
        } else {
            got.is_normal() && ((got.to_f64() - want) / want).abs() < 2f64.powi(-12)
        };
        if !ok {
            failures.push(format!("|{c:?}|² = {got:?}, want {want:e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn circle_magnitude_specific_repros() {
    let z = C::from((3.0f64, 4.0f64));
    // |3+4i| × 2^62 exploded: the naive r² + i² passed 2^127 long before |z| did.
    let m = (z << 62i8).magnitude();
    assert!(m.is_normal() && m.to_f64() == 5.0 * 2f64.powi(62), "{m:?}");
    // |3+4i| × 2^-100 was undefined: both squares vanished.
    let m = (z >> 100i8).magnitude();
    assert!(
        m.is_normal() && m.to_f64() == 5.0 * 2f64.powi(-100),
        "{m:?}"
    );
    // Past the top of the range the magnitude escapes as exploded, keeping its significand: |1.5 - 2i| × 2^127 = 1.25 × 2^128.
    let edge = C {
        real: 0x6000,
        imaginary: -0x8000,
        exponent: stored(127),
    };
    let m = edge.magnitude();
    assert!(m.exploded() && m.is_positive(), "{m:?}");
    assert_eq!(
        m.fraction, 0x5000,
        "exploded |z| keeps the 1.25 significand as 01mmm"
    );
}

#[test]
fn circle_sqrt_across_exponent_range() {
    let mut failures = Vec::new();
    for c in circles() {
        let want = raw_c(&c).sqrt();
        let got = c.sqrt();
        let ok = got.is_normal() && (raw_c(&got) - want).norm() / want.norm() < 2f64.powi(-10);
        if !ok {
            failures.push(format!("sqrt({c:?}) = {got:?}, want {want}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn circle_sqrt_specific_repros() {
    // sqrt of 2^64 was ℘√↑ and sqrt of 2^-70 was ℘√↓ (the magnitude escaped inside the half-angle formulas).
    let big = C::from((1.0f64, 0.0f64)) << 64i8;
    let root = big.sqrt();
    assert!(
        root.is_normal() && raw_c(&root) == Complex::new(2f64.powi(32), 0.0),
        "{root:?}"
    );
    let small = C::from((1.0f64, 0.0f64)) >> 70i8;
    let root = small.sqrt();
    assert!(
        root.is_normal() && raw_c(&root) == Complex::new(2f64.powi(-35), 0.0),
        "{root:?}"
    );
}

#[test]
fn circle_ln_across_exponent_range() {
    let mut failures = Vec::new();
    for c in circles() {
        let want = raw_c(&c).ln();
        let got = c.ln();
        // ln|z| runs to about ±88 at E3, so the bound is absolute near zero and relative beyond one.
        let ok =
            got.is_normal() && (raw_c(&got) - want).norm() < 2f64.powi(-9) * want.norm().max(1.0);
        if !ok {
            failures.push(format!("ln({c:?}) = {got:?}, want {want}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

/// Checks one hyperbolic result against f64: normal within `tol` relative, or escaped exploded with the right sign once the true value passes the top of the range.
fn check_hyperbolic(name: &str, x: f64, got: S, want: f64, tol: f64, failures: &mut Vec<String>) {
    let ok = if want.abs() >= MAX_NORMAL {
        got.exploded() && got.is_negative() == (want < 0.0)
    } else if want == 0.0 {
        got.is_zero()
    } else {
        got.is_normal() && ((got.to_f64() - want) / want).abs() < tol
    };
    if !ok {
        failures.push(format!(
            "{name}({x}) = {got:?} ({}), want {want:e}",
            got.to_f64()
        ));
    }
}

#[test]
fn hyperbolic_large_arguments() {
    let mut failures = Vec::new();
    // Quarter steps over ±127 are exact at F4E3 and cross both exp() escape edges (about -88.0 and +88.7) and the one-binade window where exp(|x|) explodes but exp(|x|)/2 still fits.
    for step in -508..=508 {
        let x = step as f64 / 4.0;
        let s = S::from(x);
        // exp() at F4E3 builds e^|x| from a 16-bit e, so its relative error grows to about |x| × 2^-16; 2^-7 covers |x| = 127 with margin.
        check_hyperbolic("sinh", x, s.sinh(), x.sinh(), 2f64.powi(-7), &mut failures);
        check_hyperbolic("cosh", x, s.cosh(), x.cosh(), 2f64.powi(-7), &mut failures);
        check_hyperbolic("tanh", x, s.tanh(), x.tanh(), 2f64.powi(-7), &mut failures);
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn sinh_specific_repros() {
    // sinh(-112) returned the vanished exp(-112) (positive!) instead of -exp(112)/2, which is past the top of the range.
    let got = S::from(-112).sinh();
    assert!(got.exploded() && got.is_negative(), "{got:?}");
    let got = S::from(112).sinh();
    assert!(got.exploded() && got.is_positive(), "{got:?}");
    // sinh(±89) = ±2.24e38 fits under 2^128 even tho exp(89) does not.
    let got = S::from(-89).sinh();
    assert!(
        got.is_normal()
            && ((got.to_f64() + 2.2448064095871728e38) / 2.2448064095871728e38).abs()
                < 2f64.powi(-8),
        "{got:?}"
    );
    let got = S::from(89).cosh();
    assert!(
        got.is_normal()
            && ((got.to_f64() - 2.2448064095871728e38) / 2.2448064095871728e38).abs()
                < 2f64.powi(-8),
        "{got:?}"
    );
}

#[test]
fn circle_division_imaginary_sign_repro() {
    // Over every exponent of the numerator, the quotient's imaginary sign flipped: the reciprocal-multiply wide result carries q × 2^30, and Im(q) = -2.05 wrapped past i32::MIN.
    let d = C {
        real: 0x4469,
        imaginary: 0xe885u16 as i16,
        exponent: 0xdcu8 as i8,
    };
    let mut failures = Vec::new();
    for e in 1..=255u8 {
        let n = C {
            real: 0x8e63u16 as i16,
            imaginary: 0x89c6u16 as i16,
            exponent: e as i8,
        };
        let q = n / d;
        let want = raw_c(&n) / raw_c(&d);
        let ok = if q.is_normal() {
            (raw_c(&q) - want).norm() / want.norm() < 2f64.powi(-10)
        } else {
            // Escaped: the stored pair is the direction.
            (q.exploded() || q.vanished()) && q.imaginary < 0 && q.real < 0
        };
        if !ok {
            failures.push(format!("{n:?} / {d:?} = {q:?}, want {want}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn scalar_divide_circle_wide_wrap_edge() {
    // -1 / (16385 × 2^(k-14)): the floored reciprocal overshoots and the real product lands at -2^31 - 2^18, past i32::MIN, so the quotient came out positive. Random sweeps almost never draw a power-of-two numerator, so this is pinned directly.
    let mut failures = Vec::new();
    for k in -100..=100 {
        let d = C {
            real: 0x4001,
            imaginary: 0,
            exponent: stored(k),
        };
        for s in [S::from(-1), S::from(-0.5f64), S::from(-1024)] {
            let q = s / d;
            let want = Complex::new(s.to_f64(), 0.0) / raw_c(&d);
            if !(q.is_normal() && (raw_c(&q) - want).norm() / want.norm() < 2f64.powi(-10)) {
                failures.push(format!("{s:?} / {d:?} = {q:?}, want {want}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn circle_divide_scalar_non_canonical_numerator() {
    // A de-normalized numerator pair kept only its own few bits thru `a << FRAC / s`: 1.4% error here before the pair was canonicalized.
    let n = C {
        real: 0xfff8u16 as i16,
        imaginary: 0x0007,
        exponent: 0x68,
    };
    let s = S {
        fraction: 0x57c4,
        exponent: 0x66,
    };
    let q = n / s;
    let want = raw_c(&n) / s.to_f64();
    assert!(
        q.is_normal() && (raw_c(&q) - want).norm() / want.norm() < 2f64.powi(-12),
        "{q:?} want {want}"
    );
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Random Circle/Circle, Circle/Scalar, Scalar/Circle and reciprocal over the full exponent range against Complex<f64>.
/// Normal results must match within `tol`; results the oracle puts clearly past either end of the range must escape. Operand fractions are raw random bits, so non-canonical pairs are included.
macro_rules! division_sweep {
    ($name:ident, $f:ty, $tol:expr, $count:expr) => {
        #[test]
        fn $name() {
            type Cx = Circle<$f, i8>;
            type Sx = Scalar<$f, i8>;
            let frac_bits = (std::mem::size_of::<$f>() * 8) as i32;
            let raw = |c: &Cx| -> Complex<f64> {
                let k = (c.exponent ^ i8::MIN) as i32;
                let scale = 2f64.powi(k - frac_bits + 2);
                Complex::new(c.real as f64 * scale, c.imaginary as f64 * scale)
            };
            let mut rng = Rng(0x9E3779B97F4A7C15);
            let circle = |rng: &mut Rng| -> Cx {
                loop {
                    let c = Cx {
                        real: rng.next() as $f,
                        imaginary: rng.next() as $f,
                        exponent: (rng.next() % 255 + 1) as u8 as i8,
                    };
                    if c.real != 0 || c.imaginary != 0 {
                        return c;
                    }
                }
            };
            let mut failures = Vec::new();
            let mut check = |what: String, got: Cx, want: Complex<f64>| {
                let norm = want.norm();
                let ok = if norm > MAX_NORMAL * 4.0 {
                    got.exploded()
                } else if norm < MIN_NORMAL / 4.0 {
                    got.vanished()
                } else if norm > MAX_NORMAL / 2.0 || norm < MIN_NORMAL * 2.0 {
                    true
                } else {
                    got.is_normal() && (raw(&got) - want).norm() / norm < $tol
                };
                if !ok {
                    failures.push(format!("{what} = {got:?}, want {want}"));
                }
            };
            for _ in 0..$count {
                let a = circle(&mut rng);
                let b = circle(&mut rng);
                let s = Sx {
                    fraction: rng.next() as $f,
                    exponent: (rng.next() % 255 + 1) as u8 as i8,
                };
                let (wa, wb, ws) = (raw(&a), raw(&b), s.to_f64());
                check(format!("{a:?} / {b:?}"), a / b, wa / wb);
                check(format!("{a:?} / {s:?}"), a / s, wa / ws);
                check(format!("{s:?} / {a:?}"), s / a, Complex::new(ws, 0.0) / wa);
                check(format!("1 / {a:?}"), a.reciprocal(), wa.inv());
            }
            assert!(
                failures.is_empty(),
                "{} failures, first: {:#?}",
                failures.len(),
                &failures[..failures.len().min(5)]
            );
        }
    };
}

division_sweep!(division_sweep_f4e3, i16, 2f64.powi(-10), 200_000);
division_sweep!(division_sweep_f5e3, i32, 2f64.powi(-26), 200_000);

#[test]
fn scalar_reciprocal_bottom_binade_repro() {
    // -(2^127 - 2^112): the true reciprocal -2^-127 × (1 + 2^-15 + ...) is representable, but the truncating quotient landed on -2^-127 exactly, which only exists one binade lower, so it vanished.
    let x = S {
        fraction: 0x0002,
        exponent: 0xfeu8 as i8,
    };
    let r = x.reciprocal();
    let want = 1.0 / x.to_f64();
    assert!(r.is_normal() && r.is_negative(), "{r:?}");
    assert!(
        ((r.to_f64() - want) / want).abs() < 2f64.powi(-13),
        "{r:?} want {want:e}"
    );
}

#[test]
fn scalar_reciprocal_top_binades_sweep() {
    // Every fraction at the two top exponents: 1/x must stay normal and match within a couple of ulps unless it is below 2^-127 in magnitude, or exactly -2^-127, which N0 can only store one binade lower.
    let mut failures = Vec::new();
    for e in [0xfeu8, 0xff] {
        for f in 0..=u16::MAX {
            let x = S {
                fraction: f as i16,
                exponent: e as i8,
            };
            let want = 1.0 / x.to_f64();
            let r = x.reciprocal();
            let ok = if want.abs() < MIN_NORMAL || want == -MIN_NORMAL {
                r.vanished() && r.is_negative() == (want < 0.0)
            } else {
                r.is_normal() && ((r.to_f64() - want) / want).abs() < 2f64.powi(-13)
            };
            if !ok {
                failures.push(format!("1 / {x:?} = {r:?}, want {want:e}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn scalar_division_floors() {
    // Division rounds toward negative infinity like multiply: the stored quotient never exceeds the exact one.
    let mut rng = Rng(0x2545F4914F6CDD1D);
    let mut failures = Vec::new();
    for _ in 0..200_000 {
        let a = S {
            fraction: rng.next() as i16,
            exponent: stored((rng.next() % 101) as i32 - 50),
        };
        let b = S {
            fraction: rng.next() as i16,
            exponent: stored((rng.next() % 101) as i32 - 50),
        };
        let want = a.to_f64() / b.to_f64();
        let q = a / b;
        let got = q.to_f64();
        if !(q.is_normal() && got <= want && (want - got) / want.abs() < 2f64.powi(-13)) {
            failures.push(format!("{a:?} / {b:?} = {q:?} ({got:e}), want {want:e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn frac_of_tiny_negative_stays_below_one() {
    // frac(-1e-10) = 1 - 1e-10 rounded onto exactly 1.0, outside [0, 1); it now gives the largest value below one, like the negative-vanished case.
    let got = S::from(-1e-10f64).frac();
    assert!(got == S::EFFECTIVELY_POS_ONE, "{got:?}");
    let mut failures = Vec::new();
    for k in -127..=-1 {
        for f in [0x0000u16, 0x0001, 0x5555, 0x7fff] {
            let x = S {
                fraction: f as i16,
                exponent: stored(k),
            };
            let got = x.frac();
            if !(got >= 0 && got < 1) {
                failures.push(format!("frac({x:?}) = {got:?}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

/// Random F4E3 Circle with raw-bit components (non-canonical pairs included) at true exponent `k`.
fn raw_circle(rng: &mut Rng, k: i32) -> C {
    loop {
        let c = C {
            real: rng.next() as i16,
            imaginary: rng.next() as i16,
            exponent: stored(k),
        };
        if c.real != 0 || c.imaginary != 0 {
            return c;
        }
    }
}

#[test]
fn circle_modulus_translation_sweep() {
    // Circle % Circle is ((ac + bd) mod p) + ((bc - ad) mod p)i with p = c² + d², homogeneous of degree 2: scaling both operands by 2^k scales the result by 2^(2k), not 2^k.
    // The old path built the cross products and p at the stored exponents, so the period escaped once |den| passed 2^64 or fell below 2^-64 at E3 and every such result came out undefined.
    let mut rng = Rng(0x0DDB1A5E5BAD5EED);
    let mut failures = Vec::new();
    let mut checked = 0u32;
    for _ in 0..400 {
        let base_a = (rng.next() % 41) as i32 - 20;
        let base_b = (rng.next() % 41) as i32 - 20;
        let a = raw_circle(&mut rng, base_a);
        let b = raw_circle(&mut rng, base_b);
        let base = a % b;
        if !base.is_normal() {
            continue;
        }
        let base_k = (base.exponent ^ i8::MIN) as i32;
        for k in -127..=127 {
            let (ka, kb) = (base_a + k, base_b + k);
            if !(-127..=127).contains(&ka) || !(-127..=127).contains(&kb) {
                continue;
            }
            let (mut sa, mut sb) = (a, b);
            sa.exponent = stored(ka);
            sb.exponent = stored(kb);
            let got = sa % sb;
            let target = base_k + 2 * k;
            let want = if target > 127 {
                C {
                    real: base.real,
                    imaginary: base.imaginary,
                    exponent: 0,
                }
            } else if target < -127 {
                C {
                    real: base.real >> 1,
                    imaginary: base.imaginary >> 1,
                    exponent: 0,
                }
            } else {
                C {
                    real: base.real,
                    imaginary: base.imaginary,
                    exponent: stored(target),
                }
            };
            checked += 1;
            if (got.real, got.imaginary, got.exponent) != (want.real, want.imaginary, want.exponent)
            {
                failures.push(format!("({sa:?}) % ({sb:?}) = {got:?}, want {want:?}"));
            }
        }
    }
    assert!(checked > 10_000, "only {checked} translated cases");
    assert!(
        failures.is_empty(),
        "{} of {checked} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}

#[test]
fn circle_modulus_vs_oracle() {
    // Documented semantics: Re = (ac + bd) mod p and Im = (bc - ad) mod p, floored, so each lies in [0, p) with p = c² + d².
    // The cross products round to FRAC bits before the exact modulus, so the error grows with |x / p|; quotients up to 2^6 keep the oracle meaningful, and distances are taken around the period so a result just under p matches one just over 0.
    let mut rng = Rng(0xC0FFEE0DDF00D123);
    let mut failures = Vec::new();
    let mut checked = 0u32;
    for _ in 0..300_000 {
        let ka = (rng.next() % 255) as i32 - 127;
        let kb = (rng.next() % 255) as i32 - 127;
        let a = raw_circle(&mut rng, ka);
        let b = raw_circle(&mut rng, kb);
        let (wa, wb) = (raw_c(&a), raw_c(&b));
        let p = wb.norm_sqr();
        let x = wa.re * wb.re + wa.im * wb.im;
        let y = wa.im * wb.re - wa.re * wb.im;
        if !(MIN_NORMAL * 2f64.powi(20)..MAX_NORMAL / 4.0).contains(&p)
            || x.abs().max(y.abs()) > 64.0 * p
        {
            continue;
        }
        checked += 1;
        let got = a % b;
        let tolerance = p * ((x.abs().max(y.abs()) / p) + 2.0) * 2f64.powi(-12);
        let around = |g: f64, w: f64| -> f64 {
            let d = (g - w).rem_euclid(p);
            d.min(p - d)
        };
        let want = Complex::new(x - p * (x / p).floor(), y - p * (y / p).floor());
        let ok = if got.is_zero() {
            around(0.0, want.re) <= tolerance && around(0.0, want.im) <= tolerance
        } else if got.is_normal() {
            let g = raw_c(&got);
            let in_period = |v: f64| v >= 0.0 && v < p * (1.0 + 2f64.powi(-10));
            in_period(g.re)
                && in_period(g.im)
                && around(g.re, want.re) <= tolerance
                && around(g.im, want.im) <= tolerance
        } else {
            // A remainder below the smallest normal escapes as vanished even when p is normal.
            got.vanished() && want.re.max(want.im) < 4.0 * MIN_NORMAL
        };
        if !ok {
            failures.push(format!(
                "({a:?}) % ({b:?}) = {got:?}, want {want} (p {p:e})"
            ));
        }
    }
    assert!(checked > 50_000, "only {checked} cases in range");
    assert!(
        failures.is_empty(),
        "{} of {checked} failures, first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(5)]
    );
}
