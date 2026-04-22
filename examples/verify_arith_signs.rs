//! Exhaustive F3E3 sign check: for every non-zero result, confirm the spirix
//! sign agrees with f64's. Catches sign regressions that truth-table and
//! class-membership tests don't (since both collapse magnitude classes and
//! ignore sign).
use spirix::*;
use std::collections::BTreeMap;

type S = ScalarF3E3;

#[derive(Copy, Clone, Debug)]
enum Op { Add, Sub, Mul }

fn op_name(o: Op) -> &'static str {
    match o { Op::Add => "+", Op::Sub => "-", Op::Mul => "*" }
}

fn spirix_op(o: Op, a: S, b: S) -> S {
    match o { Op::Add => a + b, Op::Sub => a - b, Op::Mul => a * b }
}

fn f64_op(o: Op, a: f64, b: f64) -> f64 {
    match o { Op::Add => a + b, Op::Sub => a - b, Op::Mul => a * b }
}

fn run(o: Op) {
    let mut total = 0usize;
    let mut sign_errors = 0usize;
    let mut first_bad: Option<(S, S, S, f64)> = None;
    let mut by_pattern: BTreeMap<(i8, i8), usize> = BTreeMap::new();

    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !s1.is_normal() { continue; }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !s2.is_normal() { continue; }
                    let r = spirix_op(o, s1, s2);
                    let a_f64: f64 = s1.into();
                    let b_f64: f64 = s2.into();
                    let rf = f64_op(o, a_f64, b_f64);
                    // Skip if f64 result is 0 or NaN (sign undefined).
                    if rf == 0.0 || rf.is_nan() { continue; }
                    // Skip if spirix says undefined or zero (no meaningful sign to compare).
                    if r.is_undefined() || r.is_zero() || r.is_infinite() { continue; }
                    total += 1;
                    let spirix_neg = r.is_negative();
                    let f64_neg = rf < 0.0;
                    if spirix_neg != f64_neg {
                        sign_errors += 1;
                        if first_bad.is_none() {
                            first_bad = Some((s1, s2, r, rf));
                        }
                        let b1 = unsafe { std::mem::transmute::<S, [i8; 2]>(s1) };
                        *by_pattern.entry((b1[0], b1[1])).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    println!("\n=== {} ({} pairs checked, {} sign errors) ===",
             op_name(o), total, sign_errors);
    if let Some((s1, s2, r, rf)) = first_bad {
        let b1 = unsafe { std::mem::transmute::<S, [i8; 2]>(s1) };
        let b2 = unsafe { std::mem::transmute::<S, [i8; 2]>(s2) };
        let br = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
        let a_f64: f64 = s1.into();
        let b_f64: f64 = s2.into();
        let r_f64: f64 = r.into();
        println!("  first bad:");
        println!("    s1 = [{:#04x},{}] = {:e}", b1[0] as u8, b1[1], a_f64);
        println!("    s2 = [{:#04x},{}] = {:e}", b2[0] as u8, b2[1], b_f64);
        println!("    spirix = [{:#04x},{}] = {:e} (is_negative = {})",
                 br[0] as u8, br[1], r_f64, r.is_negative());
        println!("    f64    = {:e} (neg = {})", rf, rf < 0.0);
    }
}

fn main() {
    for op in [Op::Add, Op::Sub, Op::Mul] {
        run(op);
    }
}
