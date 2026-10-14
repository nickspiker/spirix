//! Exponent-cycle edge regressions for code that missed the AMBIG=0 port.
//!
//! The main check is translation invariance: shifting both operands by k binades must shift the result by k binades (same fractions), or escape correctly once that falls off either end of the cycle (exploded keeps the fractions, vanished keeps them >> 1). Mid-range results are trusted as the reference, so the sweep tests the edges without an independent oracle.

use spirix::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

const OPS: [&str; 11] = [
    "C+C", "C-C", "C+S", "C-S", "S+C", "S-C", "C&C", "C|C", "C^C", "-C", "conj",
];

macro_rules! translation_sweep {
    ($test:ident, $C:ty, $S:ty, $E:ty, $ebits:expr) => {
        #[test]
        fn $test() {
            let top: i64 = (1i64 << $ebits) - 1;
            let apply = |op: &str, a: $C, b: $C, s: $S| -> $C {
                match op {
                    "C+C" => a + b,
                    "C-C" => a - b,
                    "C+S" => a + s,
                    "C-S" => a - s,
                    "S+C" => s + a,
                    "S-C" => s - a,
                    "C&C" => a & b,
                    "C|C" => a | b,
                    "C^C" => a ^ b,
                    "-C" => -a,
                    _ => a.conjugate(),
                }
            };
            let set = |mut c: $C, pos: i64| -> $C {
                c.exponent = (pos & top) as $E;
                c
            };
            let set_s = |mut c: $S, pos: i64| -> $S {
                c.exponent = (pos & top) as $E;
                c
            };
            let mut rng = Rng(0x9E3779B97F4A7C15);
            let mut failures = Vec::new();
            for _ in 0..1500 {
                let a = <$C>::from((rng.f(), rng.f()));
                let mut b = <$C>::from((rng.f(), rng.f()));
                let mut sb = <$S>::from(rng.f());
                // Near-cancellation half the time, which is what drives results toward the bottom of the cycle.
                if rng.next() & 1 == 0 {
                    b = -a + <$C>::from((rng.f() * 1e-3, rng.f() * 1e-3));
                    sb = -a.r() + <$S>::from(rng.f() * 1e-3);
                }
                if !a.is_normal() || !b.is_normal() || !sb.is_normal() {
                    continue;
                }
                let pa = (a.exponent as i64 & top) + (rng.next() % 9) as i64 - 4;
                let pb = pa + (rng.next() % 5) as i64 - 2;
                let (a, b, sb) = (set(a, pa), set(b, pb), set_s(sb, pb));
                for op in OPS {
                    let r0 = apply(op, a, b, sb);
                    if !r0.is_normal() {
                        continue;
                    }
                    let pr = r0.exponent as i64 & top;
                    let po = if op == "-C" || op == "conj" { pa } else { pb };
                    let (lo, hi) = (1 - pa.min(po), top - pa.max(po));
                    for k in [
                        lo,
                        lo + 1,
                        lo + 2,
                        lo + 5,
                        lo + 30,
                        hi - 30,
                        hi - 5,
                        hi - 2,
                        hi - 1,
                        hi,
                    ] {
                        if k < lo || k > hi {
                            continue;
                        }
                        let r = apply(op, set(a, pa + k), set(b, pb + k), set_s(sb, pb + k));
                        let np = pr + k;
                        let ok = if np > top {
                            r.is_exploded() && r.real == r0.real && r.imaginary == r0.imaginary
                        } else if np < 1 {
                            r.is_vanished()
                                && r.real == r0.real >> 1
                                && r.imaginary == r0.imaginary >> 1
                        } else {
                            r.is_normal()
                                && (r.exponent as i64 & top) == np
                                && r.real == r0.real
                                && r.imaginary == r0.imaginary
                        };
                        if !ok {
                            failures.push(format!(
                                "{op} a={:?} k={k} want_pos={np}",
                                (a.real, a.imaginary, pa)
                            ));
                        }
                    }
                }
            }
            assert!(
                failures.is_empty(),
                "{} failures, first: {:?}",
                failures.len(),
                &failures[..failures.len().min(8)]
            );
        }
    };
}

translation_sweep!(translation_f4e3, CircleF4E3, ScalarF4E3, i8, 8);
translation_sweep!(translation_f7e3, CircleF7E3, ScalarF7E3, i8, 8);
translation_sweep!(translation_f4e4, CircleF4E4, ScalarF4E4, i16, 16);

fn value(c: &CircleF4E3) -> (f64, f64) {
    (c.r().to_f64(), c.i().to_f64())
}

#[test]
fn circle_negate_conjugate_bottom_of_cycle() {
    // Negating 0x4000 gives 0xC000, which renormalizes one binade down; at stored exponent 2 that lands on 1, still normal.
    let x = CircleF4E3::new(0x4000, 0, 2);
    let n = -x;
    assert!(
        n.is_normal() && n.exponent == 1,
        "-x vanished at the bottom binade"
    );
    assert_eq!(value(&n), (-value(&x).0, 0.0));
    let y = CircleF4E3::new(0x100, 0x4000, 2);
    let c = y.conjugate();
    assert!(c.is_normal() && c.exponent == 1);
    assert_eq!(value(&c), (value(&y).0, -value(&y).1));
    // One more step down genuinely vanishes.
    assert!((-CircleF4E3::new(0x4000, 0, 1)).is_vanished());
}

#[test]
fn circle_negate_conjugate_non_canonical() {
    // All-ones (-1 raw) components renormalize by FRAC-1 like any other; the old special case dropped one binade too many and halved the value.
    for (re, im) in [
        (1i16, 0i16),
        (0, 1),
        (1, 1),
        (-1, 0),
        (0, -1),
        (1, -1),
        (2, 0),
    ] {
        let x = CircleF4E3::new(re, im, 0x80u8 as i8);
        let (xr, xi) = value(&x);
        assert_eq!(value(&-x), (-xr, -xi), "negate {re:#x},{im:#x}");
        assert_eq!(
            value(&x.conjugate()),
            (xr, -xi),
            "conjugate {re:#x},{im:#x}"
        );
    }
}

#[test]
fn circle_bitwise_bottom_of_cycle() {
    // XOR result needs one binade of renormalization; at stored exponent 2 the result sits at 1, not vanished.
    let r = CircleF4E3::new(0x5000, 0, 2) ^ CircleF4E3::new(0x6000, 0, 2);
    assert!(r.is_normal() && r.exponent == 1 && r.real == 0x6000);
}

#[test]
fn scalar_is_contiguous_integer_sweep() {
    for v in -70000i64..=70000 {
        let want = (-65535..=65535).contains(&v);
        assert_eq!(ScalarF4E3::from(v).is_contiguous(), want, "v={v}");
    }
    assert!(!(ScalarF4E3::MAX * 2i32).is_contiguous());
    assert!(!(ScalarF4E3::MIN * 2i32).is_contiguous());
    assert!(
        ScalarF4E3::MAX_CONTIGUOUS.is_contiguous() && ScalarF4E3::MIN_CONTIGUOUS.is_contiguous()
    );
}

#[test]
fn circle_contiguous_constants_scale_with_fraction() {
    // Exponent is FRAC-2 (Circle N1 carries FRAC-1 magnitude bits); it was 2*EXP-2, right only when FRAC = 2*EXP.
    assert_eq!(CircleF4E3::MAX_CONTIGUOUS.r().to_f64(), 32766.0);
    assert_eq!(CircleF4E4::MAX_CONTIGUOUS.r().to_f64(), 32766.0);
    assert_eq!(CircleF5E3::MAX_CONTIGUOUS.r().to_f64(), 2147483646.0);
    assert_eq!(CircleF5E4::MIN_CONTIGUOUS.r().to_f64(), -2147483647.0);
    assert_eq!(
        CircleF6E3::MAX_CONTIGUOUS.r().to_f64(),
        (i64::MAX - 1) as f64
    );
}

#[test]
fn scalar_trig_of_min_is_undefined() {
    // |MIN| explodes; its AMBIG exponent used to read as a tiny magnitude, so sin(MIN) = 0 and cos(MIN) = 1.
    assert!(ScalarF4E3::MIN.sin().is_undefined());
    assert!(ScalarF4E3::MIN.cos().is_undefined());
    assert!(ScalarF4E3::MIN.tan().is_undefined());
    assert!(ScalarF5E4::MIN.sin().is_undefined());
    assert!(ScalarF4E3::MAX.sin().is_undefined());
}

#[test]
#[allow(deprecated)]
fn scalar_normalize_is_identity() {
    for v in [1.5f64, -0.5, 1.0, -1.0, 3.0e-30, -7.25e20] {
        let mut x = ScalarF4E3::from(v);
        let before = (x.fraction, x.exponent);
        x.normalize();
        assert_eq!((x.fraction, x.exponent), before, "v={v}");
    }
}

#[test]
fn debug_exponent_colours() {
    // Integer-exponent colour (light yellow) for logical k >= 0, fractional (light cyan) for k < 0. Negative powers of two sit one binade low, so -1.0 is at k = -1.
    let integer = "38;2;255;255;128m";
    let fractional = "38;2;128;255;255m";
    for (v, want) in [
        (1.0f64, integer),
        (2.0, integer),
        (-1.0, fractional),
        (-3.0, integer),
        (0.5, fractional),
        (0.25, fractional),
    ] {
        let s = format!("{:#?}", ScalarF4E3::from(v));
        let tail = &s[s.rfind("*2^").unwrap()..];
        assert!(tail.contains(want), "v={v}: {tail:?}");
        let c = format!("{:#?}", CircleF4E3::from((v, 0.0f64)));
        let tail = &c[c.rfind("*2^").unwrap()..];
        assert!(tail.contains(want), "circle v={v}: {tail:?}");
    }
}

#[test]
fn circle_or_negligible_negative_operand() {
    // A far-smaller operand with one negative component fills that component with ones (-1 at big's resolution); the pair then renormalizes as a Circle. This branch used Scalar::normalize on one component, which flipped the other component's sign: (1 + 0.0156i) | tiny gave (-4.8e-7, -0.0156).
    let big = CircleF4E3::new(0x4000, 0x0100, 0x80u8 as i8);
    let small = CircleF4E3::new(-0x4000, 0x10, (0x80u8 - 40) as i8);
    let r = big | small;
    assert_eq!(value(&r), (-1.0 / 16384.0, 1.0 / 64.0));
    assert_eq!(
        (r.real, r.imaginary, r.exponent),
        (-64, 0x4000, (0x80u8 - 6) as i8),
        "canonical N1 pair"
    );
    let r = CircleF4E3::new(0x0100, 0x4000, 0x80u8 as i8)
        | CircleF4E3::new(0x10, -0x4000, (0x80u8 - 40) as i8);
    assert_eq!(value(&r), (1.0 / 64.0, -1.0 / 16384.0));
}
