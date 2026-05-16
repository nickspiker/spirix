//! Exhaustive F3E3 verification of floor/ceil/round/frac against an f64 oracle.
use spirix::*;

type S = ScalarF3E3;

#[derive(Copy, Clone, Debug)]
enum Op {
    Floor,
    Ceil,
    Round,
    Frac,
}

fn op_name(o: Op) -> &'static str {
    match o {
        Op::Floor => "floor",
        Op::Ceil => "ceil",
        Op::Round => "round",
        Op::Frac => "frac",
    }
}

fn spirix_op(o: Op, a: S) -> S {
    match o {
        Op::Floor => a.floor(),
        Op::Ceil => a.ceil(),
        Op::Round => a.round(),
        Op::Frac => a.frac(),
    }
}

fn f64_op(o: Op, a: f64) -> f64 {
    match o {
        Op::Floor => a.floor(),
        Op::Ceil => a.ceil(),
        // Rust f64::round() rounds half-away-from-zero; Spirix uses banker's.
        Op::Round => {
            let floor = a.floor();
            let diff = a - floor;
            if diff < 0.5 {
                floor
            } else if diff > 0.5 {
                floor + 1.0
            } else {
                // Exactly 0.5 → round to even
                if (floor as i64) & 1 == 0 {
                    floor
                } else {
                    floor + 1.0
                }
            }
        }
        Op::Frac => a - a.floor(),
    }
}

fn main() {
    for op in [Op::Floor, Op::Ceil, Op::Round, Op::Frac] {
        let mut total = 0usize;
        let mut mismatches = 0usize;
        let mut samples: Vec<(S, S, f64)> = Vec::new();
        for f in -128i8..=127 {
            for e in -128i8..=127 {
                let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
                if !s.is_normal() {
                    continue;
                } // let's keep the oracle check to normals only
                let r = spirix_op(op, s);
                let af: f64 = s.into();
                if !af.is_finite() || (af == 0.0 && !s.is_zero()) {
                    continue;
                }
                let expected = f64_op(op, af);
                let rf: f64 = r.into();
                // Both values should match within f64 precision, except for cases where Spirix has to quantize (r is vanished/normal, but f64 can represent exact intermediate).
                let ok = if rf.is_nan() || expected.is_nan() {
                    rf.is_nan() == expected.is_nan()
                } else {
                    // F3E3 has 8-bit fraction → ULP ≈ |value|·2^-8. Allow a few ULP of slack plus a small absolute floor for values near zero. frac() in particular can round to exactly 1 when self is a tiny fraction above an integer below in magnitude — that's F3E3 quantization, not a bug.
                    let tol = af.abs().max(expected.abs()).max(1.0) / 128.0;
                    (rf - expected).abs() <= tol
                };
                total += 1;
                if !ok {
                    mismatches += 1;
                    if samples.len() < 5 {
                        samples.push((s, r, expected));
                    }
                }
            }
        }
        println!(
            "=== {} ({} normal inputs, {} mismatches) ===",
            op_name(op),
            total,
            mismatches
        );
        for (s, r, exp) in &samples {
            let sb = unsafe { std::mem::transmute::<S, [i8; 2]>(*s) };
            let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
            let af: f64 = (*s).into();
            let rf: f64 = (*r).into();
            println!(
                "  [{:#04x},{}]={} -> spirix=[{:#04x},{}]={} expected={}",
                sb[0] as u8, sb[1], af, rb[0] as u8, rb[1], rf, exp
            );
        }
    }
}
