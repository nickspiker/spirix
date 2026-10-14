//! Exponent-range sweeps over the cross-width, Scalar/Circle and Rust-primitive conversions.
//!
//! Regression for the conversions that still carried the pre-AMBIG=0 signed exponent encoding: Circle cross-width From (escapes stamped with ED::MIN, sign-extending exponent casts, signed range tests), Circle::from((Scalar, Scalar)) aligning by a signed wrapping exponent difference, Scalar narrowing storing the raw N0 fraction as its escape pattern, Circle::from(Scalar) shifting escape prefixes down a class, -2^-127 from f64/f32 landing on ZERO, i128::MIN exploding at E3, and Scalar to integer shifting by an unbounded (wrapping) amount at E >= 16 bits.
//!
//! Every expectation is computed independently of the library from the true signed exponent k (stored = k XOR E::MIN; stored 0 is AMBIG, so the representable k range is -E::MAX..=E::MAX), with exact f64 values where those exist.

use spirix::*;

type S43 = Scalar<i16, i8>;
type S44 = Scalar<i16, i16>;
type S54 = Scalar<i32, i16>;
type C43 = Circle<i16, i8>;
type C44 = Circle<i16, i16>;
type C54 = Circle<i32, i16>;

fn e8(k: i32) -> i8 {
    (k as i8) ^ i8::MIN
}

fn e16(k: i32) -> i16 {
    (k as i16) ^ i16::MIN
}

fn report(name: &str, failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{}: {} failures, first: {:#?}",
        name,
        failures.len(),
        &failures[..failures.len().min(8)]
    );
}

/// Positive N0 significand `1mmm` (as u16) of a normal F4 fraction: positive normals store it directly, negatives are its two's complement, and the zero fraction (-2^(k+1)) is the 1.0 pole one binade up.
fn pos_sig16(f: i16) -> u16 {
    if f < 0 {
        f as u16
    } else if f == 0 {
        0x8000
    } else {
        (f as u16).wrapping_neg()
    }
}

/// Scalar F4 escape pattern: positive exploded = sig >> 1 (`01mmm`), positive vanished = sig >> 2 (`001mm`); negatives are the two's complement except at the canonical poles, which swap (matching scalar_negate).
fn scalar_escape16(sig: u16, exploded: bool, negative: bool) -> i16 {
    let p = if exploded { sig >> 1 } else { sig >> 2 };
    if !negative {
        return p as i16;
    }
    match (exploded, p) {
        (true, 0x4000) => i16::MIN,
        (false, 0x2000) => i16::MIN >> 1,
        _ => p.wrapping_neg() as i16,
    }
}

const F4_SAMPLES: [i16; 12] = [
    i16::MIN,         // +1.0
    0xC000u16 as i16, // +1.5
    -1,               // +1.99997
    0x8001u16 as i16, // +1.00003
    0xA5A5u16 as i16, // positive, mixed phase
    0x0000,           // -2.0 (pole)
    0x4000,           // -1.5
    0x7FFF,           // -1.00003
    0x0001,           // -1.99997
    0x5A5A,           // negative, mixed phase
    0x2000,           // -1.75
    0x6000,           // -1.25
];

// ───────────────────────────── Bug 3: Scalar cross-width ─────────────────────────────

#[test]
fn scalar_narrow_f4e4_to_f4e3_sweep() {
    let mut failures = Vec::new();
    for k in -32767i32..=32767 {
        for &f in F4_SAMPLES.iter() {
            let src = S44 {
                fraction: f,
                exponent: e16(k),
            };
            let dst = S43::from(&src);
            let negative = f >= 0;
            let expected = if (-127..=127).contains(&k) {
                S43 {
                    fraction: f,
                    exponent: e8(k),
                }
            } else {
                S43 {
                    fraction: scalar_escape16(pos_sig16(f), k > 127, negative),
                    exponent: 0,
                }
            };
            let class_ok = if (-127..=127).contains(&k) {
                dst.is_normal()
            } else if k > 127 {
                dst.exploded() && dst.is_negative() == negative
            } else {
                dst.vanished() && dst.is_negative() == negative
            };
            if dst.fraction != expected.fraction || dst.exponent != expected.exponent || !class_ok {
                failures.push(format!(
                    "k={} f={:#06x}: got ({:#06x}, {:#04x}) expected ({:#06x}, {:#04x})",
                    k, f, dst.fraction, dst.exponent, expected.fraction, expected.exponent
                ));
                continue;
            }
            // Phase consistency: when both x and -x escape, narrow(-x) must equal -narrow(x).
            let neg_src = -src;
            let neg_dst = S43::from(&neg_src);
            let flipped = -dst;
            if !dst.is_normal()
                && !neg_dst.is_normal()
                && (neg_dst.fraction != flipped.fraction || neg_dst.exponent != flipped.exponent)
            {
                failures.push(format!(
                    "k={} f={:#06x}: narrow(-x)=({:#06x}, {:#04x}) but -narrow(x)=({:#06x}, {:#04x})",
                    k, f, neg_dst.fraction, neg_dst.exponent, flipped.fraction, flipped.exponent
                ));
            }
        }
    }
    report("scalar_narrow_f4e4_to_f4e3_sweep", &failures);
}

#[test]
fn scalar_narrow_repros() {
    let big = S44 {
        fraction: i16::MIN,
        exponent: e16(200),
    };
    let r = S43::from(&big);
    assert!(r.exploded() && r.is_positive(), "2^200 -> {:?}", r);
    let r = S43::from(&-big);
    assert!(r.exploded() && r.is_negative(), "-2^200 -> {:?}", r);
    let r = S43::from(&S44 {
        fraction: 0xC000u16 as i16,
        exponent: e16(200),
    });
    assert!(
        r.exploded() && r.is_positive() && r.fraction == 0x6000,
        "1.5*2^200 -> {:?}",
        r
    );
    let r = S43::from(&S44 {
        fraction: 0xC000u16 as i16,
        exponent: e16(-200),
    });
    assert!(
        r.vanished() && r.is_positive() && r.fraction == 0x3000,
        "1.5*2^-200 -> {:?}",
        r
    );
    let r = S43::from(&S44 {
        fraction: 0,
        exponent: e16(-201),
    });
    assert!(r.vanished() && r.is_negative(), "-2^-200 -> {:?}", r);
}

#[test]
fn scalar_narrow_f5e4_to_f4e3_sweep() {
    // In range the signed fraction truncates (sa, floor); out of range the escape phase is the full-width magnitude truncated, then signed, so narrow(-x) == -narrow(x) as in multiply's escape path.
    let samples: [i32; 6] = [
        i32::MIN,
        0xC0001234u32 as i32,
        -1,
        0,
        0x4000_0001,
        0x5A5A_5A5A,
    ];
    let pos_sig32 = |f: i32| -> u16 {
        let m: u32 = if f < 0 {
            f as u32
        } else if f == 0 {
            0x8000_0000
        } else {
            (f as u32).wrapping_neg()
        };
        (m >> 16) as u16
    };
    let mut failures = Vec::new();
    for k in -32767i32..=32767 {
        for &f in samples.iter() {
            let src = S54 {
                fraction: f,
                exponent: e16(k),
            };
            let dst = S43::from(&src);
            let f16 = (f >> 16) as i16;
            let expected = if (-127..=127).contains(&k) {
                (f16, e8(k))
            } else {
                (scalar_escape16(pos_sig32(f), k > 127, f >= 0), 0)
            };
            if (dst.fraction, dst.exponent) != expected {
                failures.push(format!(
                    "k={} f={:#010x}: got ({:#06x}, {}) expected ({:#06x}, {})",
                    k, f, dst.fraction, dst.exponent, expected.0, expected.1
                ));
            }
        }
    }
    report("scalar_narrow_f5e4_to_f4e3_sweep", &failures);
}

#[test]
fn scalar_widen_f4e3_sweep() {
    let mut failures = Vec::new();
    for k in -127i32..=127 {
        for &f in F4_SAMPLES.iter() {
            let src = S43 {
                fraction: f,
                exponent: e8(k),
            };
            let w44 = S44::from(&src);
            let w54 = S54::from(&src);
            if (w44.fraction, w44.exponent) != (f, e16(k)) {
                failures.push(format!(
                    "F4E4 k={} f={:#06x}: got ({:#06x}, {:#06x})",
                    k, f, w44.fraction, w44.exponent
                ));
            }
            if (w54.fraction, w54.exponent) != ((f as i32) << 16, e16(k)) {
                failures.push(format!(
                    "F5E4 k={} f={:#06x}: got ({:#010x}, {:#06x})",
                    k, f, w54.fraction, w54.exponent
                ));
            }
            if src.to_f64() != w54.to_f64() {
                failures.push(format!(
                    "k={} f={:#06x}: value {} became {}",
                    k,
                    f,
                    src.to_f64(),
                    w54.to_f64()
                ));
            }
        }
    }
    report("scalar_widen_f4e3_sweep", &failures);
}

#[test]
fn scalar_escaped_inputs_keep_class_and_phase() {
    let mut failures = Vec::new();
    for raw in 0u32..=0xFFFF {
        let f = raw as u16 as i16;
        let src = S44 {
            fraction: f,
            exponent: 0,
        };
        let src_class = (
            src.is_zero(),
            src.is_infinite(),
            src.exploded(),
            src.vanished(),
            src.is_undefined(),
        );
        // Same fraction width: the fraction must pass thru bit-exact.
        let n = S43::from(&src);
        if (n.fraction, n.exponent) != (f, 0) {
            failures.push(format!(
                "F4E4->F4E3 escaped {:#06x}: got ({:#06x}, {})",
                f, n.fraction, n.exponent
            ));
        }
        // Widening the fraction appends zeros (infinity stays all-ones); class and sign must survive.
        let w = S54::from(&src);
        let w_class = (
            w.is_zero(),
            w.is_infinite(),
            w.exploded(),
            w.vanished(),
            w.is_undefined(),
        );
        let w_expected = if src.is_infinite() {
            -1
        } else {
            (f as i32) << 16
        };
        if w.exponent != 0
            || w.fraction != w_expected
            || w_class != src_class
            || w.is_negative() != src.is_negative()
        {
            failures.push(format!(
                "F4E4->F5E4 escaped {:#06x}: got ({:#010x}, {})",
                f, w.fraction, w.exponent
            ));
        }
    }
    report("scalar_escaped_inputs_keep_class_and_phase", &failures);
}

// ───────────────────────────── Bug 1: Circle cross-width ─────────────────────────────

const C4_SAMPLES: [(i16, i16); 6] = [
    (0x4000, 0),       // +1.0
    (0x6000, -0x3000), // 1.5 - 0.75i
    (i16::MIN, 0),     // -2.0 (N1 pole)
    (0x1000, 0x7FFF),  // imaginary dominant
    (-0x5000, 0x2000), // negative real dominant
    (0x0123, -0x6000), // negative imaginary dominant
];

fn circle_escape16(re: i16, im: i16, exploded: bool) -> (i16, i16) {
    if exploded {
        (re, im)
    } else {
        (re >> 1, im >> 1)
    }
}

#[test]
fn circle_narrow_f4e4_to_f4e3_sweep() {
    let mut failures = Vec::new();
    for k in -32767i32..=32767 {
        for &(re, im) in C4_SAMPLES.iter() {
            let src = C44 {
                real: re,
                imaginary: im,
                exponent: e16(k),
            };
            let dst = C43::from(&src);
            let (exp_re, exp_im, exp_e, class_ok) = if (-127..=127).contains(&k) {
                (re, im, e8(k), dst.is_normal())
            } else {
                let (r, i) = circle_escape16(re, im, k > 127);
                (
                    r,
                    i,
                    0,
                    if k > 127 {
                        dst.exploded()
                    } else {
                        dst.vanished()
                    },
                )
            };
            if (dst.real, dst.imaginary, dst.exponent) != (exp_re, exp_im, exp_e) || !class_ok {
                failures.push(format!(
                    "k={} ({:#06x}, {:#06x}): got ({:#06x}, {:#06x}, {:#04x}) expected ({:#06x}, {:#06x}, {:#04x})",
                    k, re, im, dst.real, dst.imaginary, dst.exponent, exp_re, exp_im, exp_e
                ));
            }
        }
    }
    report("circle_narrow_f4e4_to_f4e3_sweep", &failures);
}

#[test]
fn circle_narrow_f5e4_to_f4e3_sweep() {
    let mut failures = Vec::new();
    for k in -32767i32..=32767 {
        for &(re, im) in C4_SAMPLES.iter() {
            // Low 16 bits carry junk that narrowing must drop.
            let src = C54 {
                real: ((re as i32) << 16) | 0xBEEF,
                imaginary: ((im as i32) << 16) | 0x1234,
                exponent: e16(k),
            };
            let dst = C43::from(&src);
            let expected = if (-127..=127).contains(&k) {
                (re, im, e8(k))
            } else {
                let (r, i) = circle_escape16(re, im, k > 127);
                (r, i, 0)
            };
            if (dst.real, dst.imaginary, dst.exponent) != expected {
                failures.push(format!(
                    "k={} ({:#06x}, {:#06x}): got ({:#06x}, {:#06x}, {})",
                    k, re, im, dst.real, dst.imaginary, dst.exponent
                ));
            }
        }
    }
    report("circle_narrow_f5e4_to_f4e3_sweep", &failures);
}

#[test]
fn circle_widen_f4e3_sweep() {
    let mut failures = Vec::new();
    for k in -127i32..=127 {
        for &(re, im) in C4_SAMPLES.iter() {
            let src = C43 {
                real: re,
                imaginary: im,
                exponent: e8(k),
            };
            let w44 = C44::from(&src);
            if (w44.real, w44.imaginary, w44.exponent) != (re, im, e16(k)) {
                failures.push(format!(
                    "F4E4 k={}: got ({:#06x}, {:#06x}, {:#06x})",
                    k, w44.real, w44.imaginary, w44.exponent
                ));
            }
            let w54 = C54::from(&src);
            if (w54.real, w54.imaginary, w54.exponent)
                != ((re as i32) << 16, (im as i32) << 16, e16(k))
            {
                failures.push(format!(
                    "F5E4 k={}: got ({:#010x}, {:#010x}, {:#06x})",
                    k, w54.real, w54.imaginary, w54.exponent
                ));
            }
            // A small non-dominant component can sit below F4E3's Scalar range (its r()/i() vanishes) while F5E4 still holds it, so compare only components that are normal at the source width.
            if (src.r().is_normal() && src.r().to_f64() != w54.r().to_f64())
                || (src.i().is_normal() && src.i().to_f64() != w54.i().to_f64())
            {
                failures.push(format!(
                    "k={} ({:#06x}, {:#06x}): value changed on widening",
                    k, re, im
                ));
            }
            // Round trip back to F4E3 is exact.
            let back = C43::from(&w54);
            if (back.real, back.imaginary, back.exponent) != (re, im, e8(k)) {
                failures.push(format!(
                    "round trip k={}: got ({:#06x}, {:#06x}, {:#04x})",
                    k, back.real, back.imaginary, back.exponent
                ));
            }
        }
    }
    report("circle_widen_f4e3_sweep", &failures);
}

#[test]
fn circle_cross_width_repros() {
    let one = C54::from(&C43::ONE);
    assert_eq!(
        (one.real, one.imaginary, one.exponent),
        (C54::ONE.real, C54::ONE.imaginary, C54::ONE.exponent)
    );
    assert!(C54::from(&C43::ZERO).is_zero());
    let back = C43::from(&C54::ONE);
    assert_eq!(
        (back.real, back.imaginary, back.exponent),
        (C43::ONE.real, C43::ONE.imaginary, C43::ONE.exponent)
    );
    let thousand = C43::from(&C54::from(1000.0f64));
    assert_eq!(thousand.r().to_f64(), 1000.0);
    let neg_one = C43::from(&C54::NEG_ONE);
    assert_eq!(
        (neg_one.real, neg_one.imaginary, neg_one.exponent),
        (
            C43::NEG_ONE.real,
            C43::NEG_ONE.imaginary,
            C43::NEG_ONE.exponent
        )
    );
}

#[test]
fn circle_cross_width_escaped_inputs() {
    let mut failures = Vec::new();
    // Every escaped class on both axes, from the narrowing sweep's own outputs plus the singular values and an undefined prefix.
    let mut inputs: Vec<C43> = vec![C43::ZERO, C43::INFINITY];
    for &(re, im) in C4_SAMPLES.iter() {
        for &exploded in [true, false].iter() {
            let (r, i) = circle_escape16(re, im, exploded);
            inputs.push(C43 {
                real: r,
                imaginary: i,
                exponent: 0,
            });
        }
    }
    let general: i16 = (0x1Fi16) << 8;
    inputs.push(C43 {
        real: general,
        imaginary: general,
        exponent: 0,
    });
    for src in inputs.iter() {
        let sc = [
            src.is_zero(),
            src.is_infinite(),
            src.exploded(),
            src.vanished(),
            src.is_undefined(),
        ];
        let w = C54::from(src);
        let wc = [
            w.is_zero(),
            w.is_infinite(),
            w.exploded(),
            w.vanished(),
            w.is_undefined(),
        ];
        let n = C43::from(&w);
        let nc = [
            n.is_zero(),
            n.is_infinite(),
            n.exploded(),
            n.vanished(),
            n.is_undefined(),
        ];
        if sc != wc || sc != nc || w.exponent != 0 || n.exponent != 0 {
            failures.push(format!("{:?}: widened {:?}, narrowed back {:?}", src, w, n));
            continue;
        }
        if src.is_infinite() {
            if (w.real, w.imaginary) != (-1, -1) {
                failures.push(format!("infinity widened to {:?}", w));
            }
        } else if (w.real, w.imaginary) != ((src.real as i32) << 16, (src.imaginary as i32) << 16)
            || (n.real, n.imaginary) != (src.real, src.imaginary)
        {
            failures.push(format!(
                "{:?}: phase lost, widened {:?}, back {:?}",
                src, w, n
            ));
        }
    }
    report("circle_cross_width_escaped_inputs", &failures);
}

// ───────────────────────────── Bug 2: Circle::from((Scalar, Scalar)) ─────────────────────────────

/// N1 component of a normal N0 Scalar fraction at the same exponent.
fn n1_of(f: i16) -> i16 {
    (f >> 1) ^ i16::MIN
}

#[test]
fn circle_from_scalar_pair_alignment_sweep() {
    let pairs: [(i16, i16); 4] = [
        (i16::MIN, 0xC000u16 as i16), // (+1.0, +1.5)
        (0x4000, i16::MIN),           // (-1.5, +1.0)
        (0xA5A5u16 as i16, 0x5A5A),   // mixed signs and phases
        (0x0000, 0x0000),             // (-2.0, -2.0)
    ];
    let mut failures = Vec::new();
    for kr in -127i32..=127 {
        for ki in -127i32..=127 {
            for &(fr, fi) in pairs.iter() {
                let re = S43 {
                    fraction: fr,
                    exponent: e8(kr),
                };
                let im = S43 {
                    fraction: fi,
                    exponent: e8(ki),
                };
                let c = C43::from((re, im));
                let (rc, ic) = (n1_of(fr), n1_of(fi));
                let d = (kr - ki).abs();
                let expected = if kr >= ki {
                    (rc, if d >= 16 { 0 } else { ic >> d }, e8(kr))
                } else {
                    (if d >= 16 { 0 } else { rc >> d }, ic, e8(ki))
                };
                if (c.real, c.imaginary, c.exponent) != expected {
                    failures.push(format!(
                        "kr={} ki={} ({:#06x}, {:#06x}): got ({:#06x}, {:#06x}, {:#04x}) expected ({:#06x}, {:#06x}, {:#04x})",
                        kr, ki, fr, fi, c.real, c.imaginary, c.exponent, expected.0, expected.1, expected.2
                    ));
                }
            }
        }
    }
    report("circle_from_scalar_pair_alignment_sweep", &failures);
}

#[test]
fn circle_from_scalar_pair_repros() {
    let at = |k: i32| S43 {
        fraction: i16::MIN,
        exponent: e8(k),
    };
    let c = C43::from((at(64), at(-64)));
    assert_eq!(c.r().to_f64(), 2f64.powi(64), "(2^64, 2^-64) -> {:?}", c);
    assert!(c.i().is_zero(), "(2^64, 2^-64) -> {:?}", c);
    let c = C43::from((at(112), at(-112)));
    assert_eq!(c.r().to_f64(), 2f64.powi(112), "(2^112, 2^-112) -> {:?}", c);
    let c = C43::from((at(-112), at(112)));
    assert_eq!(c.i().to_f64(), 2f64.powi(112), "(2^-112, 2^112) -> {:?}", c);
    assert!(c.r().is_zero());
}

// ───────────────────────────── Bug 4: Circle::from(Scalar) ─────────────────────────────

#[test]
fn circle_from_scalar_matches_pair_form() {
    let mut failures = Vec::new();
    // Every escaped fraction at AMBIG that belongs to a class (Scalar ZERO and INFINITY are exact patterns; all-0s or all-1s prefixes with low junk bits belong to no Scalar class and are skipped).
    for raw in 0u32..=0xFFFF {
        let s = S43 {
            fraction: raw as u16 as i16,
            exponent: 0,
        };
        if !(s.is_zero() || s.is_infinite() || s.exploded() || s.vanished() || s.is_undefined()) {
            continue;
        }
        let c = C43::from(s);
        let p = C43::from((s, S43::ZERO));
        if (c.real, c.imaginary, c.exponent) != (p.real, p.imaginary, p.exponent) {
            failures.push(format!(
                "escaped {:#06x}: from(s)={:?} from((s, 0))={:?}",
                raw, c, p
            ));
            continue;
        }
        let expected_class = [
            s.is_zero(),
            s.is_infinite(),
            s.exploded(),
            s.vanished(),
            s.is_undefined(),
        ];
        let got_class = [
            c.is_zero(),
            c.is_infinite(),
            c.exploded(),
            c.vanished(),
            c.is_undefined(),
        ];
        if expected_class != got_class {
            failures.push(format!(
                "escaped {:#06x}: class {:?} became {:?}",
                raw, expected_class, got_class
            ));
            continue;
        }
        // Escape patterns are shared between Scalar and Circle: the real axis carries the Scalar fraction unchanged.
        if !s.is_infinite() && c.real != s.fraction {
            failures.push(format!(
                "escaped {:#06x}: real component {:#06x}",
                raw, c.real
            ));
        }
    }
    // Every normal exponent.
    for k in -127i32..=127 {
        for &f in F4_SAMPLES.iter() {
            let s = S43 {
                fraction: f,
                exponent: e8(k),
            };
            let c = C43::from(s);
            if (c.real, c.imaginary, c.exponent) != (n1_of(f), 0, e8(k)) {
                failures.push(format!("normal k={} f={:#06x}: got {:?}", k, f, c));
            }
        }
    }
    report("circle_from_scalar_matches_pair_form", &failures);
}

#[test]
fn circle_from_f64_escapes() {
    let c = C43::from(1e300f64);
    assert!(c.exploded() && c.real > 0, "1e300 -> {:?}", c);
    let c = C43::from(-1e300f64);
    assert!(c.exploded() && c.real < 0, "-1e300 -> {:?}", c);
    let c = C43::from(1e-300f64);
    assert!(c.vanished() && c.real > 0, "1e-300 -> {:?}", c);
    let c = C43::from(-1e-300f64);
    assert!(c.vanished() && c.real < 0, "-1e-300 -> {:?}", c);
    // IEEE infinities map to Scalar exploded (see rust_scalar module doc), so the Circle is exploded on the real axis.
    let c = C43::from(f64::INFINITY);
    assert!(c.exploded() && c.real > 0, "inf -> {:?}", c);
    let c = C43::from(f64::NEG_INFINITY);
    assert!(c.exploded() && c.real < 0, "-inf -> {:?}", c);
    let c = C43::from(f64::NAN);
    assert!(c.is_undefined(), "NaN -> {:?}", c);
    let s = S43::from(1e300f64);
    let c = C43::from(s);
    assert!(
        c.exploded() && c.real == s.fraction,
        "exploded scalar -> {:?}",
        c
    );
}

// ───────────────────────────── Bug 5: f64 / f32 into Scalar ─────────────────────────────

/// Expected class of ±m·2^k at F4E3: positives are normal for k in -127..=127; negatives store -1.mmm·2^k at k, except exact powers of two which store as -2·2^(k-1) one binade lower.
fn f_expect(m: f64, k: i32, negative: bool) -> &'static str {
    let stored = if negative && m == 1.0 { k - 1 } else { k };
    if stored > 127 {
        "exploded"
    } else if stored < -127 {
        "vanished"
    } else {
        "normal"
    }
}

fn check_float_sweep(name: &str, use_f32: bool) {
    let mantissas = [1.0f64, 1.5, 1.25, 1.0 + 1.0 / 32768.0, 2.0 - 1.0 / 32768.0];
    let mut failures = Vec::new();
    for k in -160i32..=160 {
        for &m in mantissas.iter() {
            for &negative in [false, true].iter() {
                let x = if negative { -m } else { m } * 2f64.powi(k);
                let (s, sc) = if use_f32 {
                    let xf = x as f32;
                    if xf as f64 != x {
                        continue;
                    }
                    (S43::from(xf), C43::from(xf))
                } else {
                    (S43::from(x), C43::from(x))
                };
                let want = f_expect(m, k, negative);
                let ok = match want {
                    // Circle N1 components carry one bit less than the N0 Scalar fraction, so the Circle is checked bit-wise against the Scalar and by value only where the mantissa fits 14 fractional bits.
                    "normal" => {
                        s.is_normal()
                            && s.to_f64() == x
                            && (sc.real, sc.imaginary, sc.exponent)
                                == (n1_of(s.fraction), 0, s.exponent)
                            && ((m * 16384.0).fract() != 0.0 || sc.r().to_f64() == x)
                    }
                    "exploded" => {
                        s.exploded()
                            && s.is_negative() == negative
                            && sc.exploded()
                            && (sc.real < 0) == negative
                    }
                    _ => {
                        s.vanished()
                            && s.is_negative() == negative
                            && sc.vanished()
                            && (sc.real < 0) == negative
                    }
                };
                if !ok {
                    failures.push(format!(
                        "{} x={:e} (m={} k={}): expected {}, got {:?} / {:?}",
                        name, x, m, k, want, s, sc
                    ));
                }
            }
        }
    }
    report(name, &failures);
}

#[test]
fn f64_to_scalar_exponent_sweep() {
    check_float_sweep("f64_to_scalar_exponent_sweep", false);
}

#[test]
fn f32_to_scalar_exponent_sweep() {
    check_float_sweep("f32_to_scalar_exponent_sweep", true);
}

#[test]
fn neg_two_pow_neg_127_vanishes() {
    let x = -(2f64.powi(-127));
    let s = S43::from(x);
    assert!(s.vanished() && s.is_negative(), "-2^-127 (f64) -> {:?}", s);
    let s = S43::from(x as f32);
    assert!(s.vanished() && s.is_negative(), "-2^-127 (f32) -> {:?}", s);
    let c = C43::from(x);
    assert!(c.vanished() && c.real < 0, "-2^-127 -> {:?}", c);
    // It is the same value scalar_negate produces from +2^-127.
    let pos = S43::from(-x);
    assert!(pos.is_normal());
    let neg = -pos;
    assert_eq!((s.fraction, s.exponent), (neg.fraction, neg.exponent));
}

// ───────────────────────────── Bug 6: integer MIN into Scalar ─────────────────────────────

#[test]
fn integer_min_and_powers_of_two() {
    type S73 = Scalar<i128, i8>;
    let s = S73::from(i128::MIN);
    assert!(s.is_normal(), "i128::MIN -> {:?}", s);
    assert_eq!(s.to_f64(), -(2f64.powi(127)));
    assert_eq!(s.to_i128(), i128::MIN);
    let s = S43::from(i128::MIN);
    assert!(
        s.is_normal() && s.to_f64() == -(2f64.powi(127)),
        "i128::MIN at F4E3 -> {:?}",
        s
    );
    assert_eq!(S43::from(i8::MIN).to_f64(), i8::MIN as f64);
    assert_eq!(S43::from(i16::MIN).to_f64(), i16::MIN as f64);
    assert_eq!(S43::from(i32::MIN).to_f64(), i32::MIN as f64);
    assert_eq!(S43::from(i64::MIN).to_f64(), i64::MIN as f64);
    assert_eq!(S43::from(isize::MIN).to_f64(), isize::MIN as f64);
    let mut failures = Vec::new();
    for j in 0..=127u32 {
        let v: i128 = if j == 127 { i128::MIN } else { -(1i128 << j) };
        for (name, got) in [
            ("F7E3", S73::from(v).to_f64()),
            ("F4E3", S43::from(v).to_f64()),
        ] {
            if got != v as f64 {
                failures.push(format!("{} -(2^{}) -> {}", name, j, got));
            }
        }
        if S73::from(v).to_i128() != v {
            failures.push(format!("F7E3 -(2^{}) round trip", j));
        }
    }
    report("integer_min_and_powers_of_two", &failures);
}

// ───────────────────────────── Bug 7: Scalar into integers ─────────────────────────────

/// Exact floor(v · 2^s) clamped to [lo, hi], computed in i128 with explicit saturation instead of shifting.
fn floor_scaled(v: i64, s: i64, lo: i128, hi: i128) -> i128 {
    if s >= 0 {
        if s >= 100 {
            return if v < 0 { lo } else { hi };
        }
        let wide = (v as i128)
            .checked_shl(s as u32)
            .filter(|w| w >> s == v as i128);
        match wide {
            Some(w) => w.clamp(lo, hi),
            None => {
                if v < 0 {
                    lo
                } else {
                    hi
                }
            }
        }
    } else if -s >= 100 {
        if v < 0 {
            -1
        } else {
            0
        }
    } else {
        ((v as i128) >> (-s)).clamp(lo, hi)
    }
}

/// Inflated N0 value of a normal F4 fraction: the stored bits with the implicit sign restored.
fn inflate16(f: i16) -> i64 {
    if f < 0 {
        f as u16 as i64
    } else {
        f as i64 - 0x10000
    }
}

#[test]
fn scalar_to_int_exponent_sweep_e4() {
    let mut failures = Vec::new();
    for stored in 1u32..=0xFFFF {
        let e = stored as u16 as i16;
        let k = (e ^ i16::MIN) as i64;
        for &f in F4_SAMPLES.iter() {
            let s = S44 {
                fraction: f,
                exponent: e,
            };
            let v = inflate16(f);
            let sh = k - 15;
            let want_i64 = floor_scaled(v, sh, i64::MIN as i128, i64::MAX as i128);
            let want_i8 = floor_scaled(v, sh, i8::MIN as i128, i8::MAX as i128);
            let want_u64 = if v < 0 {
                0
            } else {
                floor_scaled(v, sh, 0, u64::MAX as i128)
            };
            let got = (s.to_i64() as i128, s.to_i8() as i128, s.to_u64() as i128);
            if got != (want_i64, want_i8, want_u64) {
                failures.push(format!(
                    "k={} f={:#06x}: got {:?} expected {:?}",
                    k,
                    f,
                    got,
                    (want_i64, want_i8, want_u64)
                ));
            }
        }
    }
    report("scalar_to_int_exponent_sweep_e4", &failures);
}

#[test]
fn scalar_to_int_wide_exponent_boundaries() {
    // E5 and E7: sample the cycle edges and the true-exponent -1/0 crossing.
    let mut failures = Vec::new();
    let ks5: [i64; 10] = [
        -(i32::MAX as i64),
        -(i32::MAX as i64) + 1,
        -65536,
        -300,
        -1,
        0,
        1,
        62,
        63,
        i32::MAX as i64,
    ];
    for &k in ks5.iter() {
        for &f in F4_SAMPLES.iter() {
            let s = Scalar::<i16, i32> {
                fraction: f,
                exponent: (k as i32) ^ i32::MIN,
            };
            let want = floor_scaled(inflate16(f), k - 15, i64::MIN as i128, i64::MAX as i128);
            let want_u = if inflate16(f) < 0 {
                0
            } else {
                floor_scaled(inflate16(f), k - 15, 0, u64::MAX as i128)
            };
            if (s.to_i64() as i128, s.to_u64() as i128) != (want, want_u) {
                failures.push(format!(
                    "E5 k={} f={:#06x}: got ({}, {}) expected ({}, {})",
                    k,
                    f,
                    s.to_i64(),
                    s.to_u64(),
                    want,
                    want_u
                ));
            }
        }
    }
    let ks7: [i128; 9] = [
        -i128::MAX,
        -i128::MAX + 1,
        i64::MIN as i128,
        -300,
        -1,
        0,
        1,
        126,
        i128::MAX,
    ];
    for &k in ks7.iter() {
        for &f in F4_SAMPLES.iter() {
            let s = Scalar::<i16, i128> {
                fraction: f,
                exponent: k ^ i128::MIN,
            };
            let sh = k.clamp(-1000, 1000) as i64 - 15;
            let want = floor_scaled(inflate16(f), sh, i64::MIN as i128, i64::MAX as i128);
            let want_u = if inflate16(f) < 0 {
                0
            } else {
                floor_scaled(inflate16(f), sh, 0, u64::MAX as i128)
            };
            if (s.to_i64() as i128, s.to_u64() as i128) != (want, want_u) {
                failures.push(format!(
                    "E7 k={} f={:#06x}: got ({}, {}) expected ({}, {})",
                    k,
                    f,
                    s.to_i64(),
                    s.to_u64(),
                    want,
                    want_u
                ));
            }
        }
    }
    report("scalar_to_int_wide_exponent_boundaries", &failures);
}

#[test]
fn scalar_to_int_repro() {
    let tiny = S44 {
        fraction: 0xC000u16 as i16,
        exponent: 1,
    };
    assert_eq!(tiny.to_i64(), 0, "1.5*2^-32767");
    assert_eq!(tiny.to_u64(), 0, "1.5*2^-32767");
    // Floor semantics: a tiny negative normal floors to -1, as -0.5 does.
    let neg_tiny = -tiny;
    assert_eq!(neg_tiny.to_i64(), -1, "-1.5*2^-32767");
    assert_eq!(S43::from(-0.5f64).to_i64(), -1);
}
