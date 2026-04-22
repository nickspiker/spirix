//! Exhaustive F3E3 verification: add/sub/mul/div class against f64 gold.
//! Flags suspicious bugs: cases where spirix claims exploded when f64 says tiny,
//! or claims vanished when f64 says huge, etc.

use spirix::*;
use std::collections::BTreeMap;

type S = ScalarF3E3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class { Zero, Vanished, Normal, Exploded, Infinity, Undefined }

fn classify(s: S) -> Class {
    if s.is_undefined() { Class::Undefined }
    else if s.is_zero() { Class::Zero }
    else if s.is_infinite() { Class::Infinity }
    else if s.exploded() { Class::Exploded }
    else if s.vanished() { Class::Vanished }
    else { Class::Normal }
}

fn name(c: Class) -> &'static str {
    match c {
        Class::Zero => "[0]", Class::Vanished => "[↓]", Class::Normal => "[#]",
        Class::Exploded => "[↑]", Class::Infinity => "[∞]", Class::Undefined => "[℘?]",
    }
}

#[derive(Copy, Clone, Debug)]
enum Op { Add, Sub, Mul, Div }

fn op_name(o: Op) -> &'static str {
    match o { Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/" }
}

fn spirix_op(o: Op, a: S, b: S) -> S {
    match o { Op::Add => a + b, Op::Sub => a - b, Op::Mul => a * b, Op::Div => a / b }
}

fn f64_op(o: Op, a: f64, b: f64) -> f64 {
    match o { Op::Add => a + b, Op::Sub => a - b, Op::Mul => a * b, Op::Div => a / b }
}

/// What class SHOULD spirix return given the f64 result magnitude?
/// Returns None when both magnitudes are acceptable (e.g. at a boundary).
fn expected_class_from_f64(r: f64) -> Option<Class> {
    if r.is_nan() { return Some(Class::Undefined); }
    if r.is_infinite() { return Some(Class::Infinity); }
    let abs = r.abs();
    if abs == 0.0 { return Some(Class::Zero); }
    // F3E3 normal range: 2^MIN_EXP (≈ 5.9e-39) to 2^MAX_EXP (≈ 1.7e38). We use safe
    // margins that both directions can land in [↓] or [↑] only if clearly outside.
    const MIN_NORMAL: f64 = 1e-38;   // below this, expect vanished
    const MAX_NORMAL: f64 = 1e38;    // above this, expect exploded
    if abs < MIN_NORMAL { return Some(Class::Vanished); }
    if abs > MAX_NORMAL { return Some(Class::Exploded); }
    Some(Class::Normal)
}

fn run(o: Op) {
    let mut confusions: BTreeMap<(Class, Class), usize> = BTreeMap::new();
    let mut example: BTreeMap<(Class, Class), (S, S)> = BTreeMap::new();
    let mut total = 0usize;
    let mut flagged = 0usize;

    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let s1 = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !s1.is_normal() { continue; }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let s2 = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !s2.is_normal() { continue; }
                    // Skip div by near-zero that f64 would treat as zero
                    if matches!(o, Op::Div) {
                        let f2f: f64 = s2.into();
                        if f2f == 0.0 { continue; }
                    }
                    total += 1;
                    let spirix_result = classify(spirix_op(o, s1, s2));
                    let a_f64: f64 = s1.into();
                    let b_f64: f64 = s2.into();
                    let f64_result = f64_op(o, a_f64, b_f64);
                    let expected = match expected_class_from_f64(f64_result) {
                        Some(c) => c, None => continue,
                    };

                    // The bug pattern: spirix says exploded when f64 says vanished/zero
                    // (tiny real result misclassified as huge). Or the reverse.
                    let suspicious = (spirix_result == Class::Exploded
                                      && (expected == Class::Vanished || expected == Class::Zero))
                                   || (spirix_result == Class::Vanished
                                       && (expected == Class::Exploded || expected == Class::Infinity));
                    if suspicious {
                        flagged += 1;
                        let key = (spirix_result, expected);
                        *confusions.entry(key).or_insert(0) += 1;
                        example.entry(key).or_insert((s1, s2));
                    }
                }
            }
        }
    }
    println!("\n=== {} ({} pairs tested, {} class-sign errors) ===",
             op_name(o), total, flagged);
    for ((spirix_c, expected_c), count) in &confusions {
        let (s1, s2) = example[&(*spirix_c, *expected_c)];
        let f1: f64 = s1.into();
        let f2: f64 = s2.into();
        println!("  spirix says {}, f64 says {}  ({} cases)  e.g.  {} {} {} = {} (spirix) / f64={:.3e}",
                 name(*spirix_c), name(*expected_c), count,
                 f1, op_name(o), f2,
                 { let r: f64 = spirix_op(o, s1, s2).into(); format!("{:.3e}", r) },
                 f64_op(o, f1, f2));
    }
}

fn main() {
    for op in [Op::Add, Op::Sub, Op::Mul, Op::Div] {
        run(op);
    }
}
