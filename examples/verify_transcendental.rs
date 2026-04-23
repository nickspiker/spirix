//! F3E3 sanity check of sqrt / exp / ln / sin / cos against f64 oracle.
//! Not exhaustive (transcendentals are inherently approximate and F3E3's
//! 8-bit fraction leaves only ~1/256 resolution, so bit-exact matches aren't
//! expected), but catches gross class/sign regressions across all normal
//! inputs in the range where f64 can serve as an oracle.
use spirix::*;

type S = ScalarF3E3;

#[derive(Copy, Clone, Debug)]
enum Op { Sqrt, Exp, Ln, Sin, Cos }

fn op_name(o: Op) -> &'static str {
    match o { Op::Sqrt => "sqrt", Op::Exp => "exp", Op::Ln => "ln", Op::Sin => "sin", Op::Cos => "cos" }
}

fn spirix_op(o: Op, a: S) -> S {
    match o {
        Op::Sqrt => a.sqrt(),
        Op::Exp  => a.exp(),
        Op::Ln   => a.ln(),
        Op::Sin  => a.sin(),
        Op::Cos  => a.cos(),
    }
}

fn f64_op(o: Op, a: f64) -> f64 {
    match o {
        Op::Sqrt => a.sqrt(),
        Op::Exp  => a.exp(),
        Op::Ln   => a.ln(),
        Op::Sin  => a.sin(),
        Op::Cos  => a.cos(),
    }
}

fn main() {
    for op in [Op::Sqrt, Op::Exp, Op::Ln, Op::Sin, Op::Cos] {
        let mut total = 0usize;
        let mut gross_sign = 0usize;
        let mut gross_class = 0usize;
        let mut samples: Vec<(S, f64, S, f64)> = Vec::new();
        for f in -128i8..=127 {
            for e in -128i8..=127 {
                let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
                if !s.is_normal() { continue; }
                // Restrict oracle's input range per op
                let af: f64 = s.into();
                if !af.is_finite() || af == 0.0 { continue; }
                match op {
                    Op::Sqrt => { if af < 0.0 { continue; } }
                    Op::Ln   => { if af <= 0.0 { continue; } }
                    Op::Exp  => { if af > 80.0 || af < -80.0 { continue; } }
                    Op::Sin | Op::Cos => { if af.abs() > 1e10 { continue; } }
                }
                let expected = f64_op(op, af);
                if !expected.is_finite() { continue; }
                let r = spirix_op(op, s);
                let _rf: f64 = r.into();
                total += 1;
                // Gross sign check: only if expected is clearly nonzero and
                // spirix result is normal/representable.
                if expected.abs() > 1e-3 && r.is_normal() {
                    let spirix_sign = r.is_negative();
                    let oracle_sign = expected < 0.0;
                    if spirix_sign != oracle_sign {
                        gross_sign += 1;
                        if samples.len() < 5 {
                            samples.push((s, af, r, expected));
                        }
                        continue;
                    }
                }
                // Gross class check: f64 says a regular nonzero finite,
                // but spirix says undefined/infinity.
                if expected.abs() > 1e-3 && expected.abs() < 1e30 {
                    if r.is_undefined() || r.is_infinite() {
                        gross_class += 1;
                        if samples.len() < 5 {
                            samples.push((s, af, r, expected));
                        }
                    }
                }
            }
        }
        println!("=== {} ({} inputs, {} gross sign errors, {} gross class errors) ===",
                 op_name(op), total, gross_sign, gross_class);
        for (s, af, r, ef) in &samples {
            let sb = unsafe { std::mem::transmute::<S, [i8; 2]>(*s) };
            let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
            let rf: f64 = (*r).into();
            println!("  [{:#04x},{}]={:e} -> spirix=[{:#04x},{}]={:e} expected={:e}",
                     sb[0] as u8, sb[1], af, rb[0] as u8, rb[1], rf, ef);
        }
    }
}
