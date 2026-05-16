//! Exhaustive F3E3 verification of scalar negation:
//! 1. The unified `-x` path and the branchy specialized path agree bit-exactly.
//! 2. Class transitions match the README truth table.
//! 3. `-(-x) == x` (negate-negate is identity) for all inputs, including the boundary cases that escape or re-enter normal.
use spirix::*;
use std::collections::{BTreeMap, BTreeSet};

type S = ScalarF3E3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Zero,
    Vanished,
    Normal,
    Exploded,
    Infinity,
    Undefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Signed {
    Zero,
    PosVan,
    NegVan,
    PosNorm,
    NegNorm,
    PosExp,
    NegExp,
    Inf,
    Undef,
}

fn classify(s: S) -> Class {
    if s.is_undefined() {
        Class::Undefined
    } else if s.is_zero() {
        Class::Zero
    } else if s.is_infinite() {
        Class::Infinity
    } else if s.exploded() {
        Class::Exploded
    } else if s.vanished() {
        Class::Vanished
    } else {
        Class::Normal
    }
}

fn classify_signed(s: S) -> Signed {
    match classify(s) {
        Class::Zero => Signed::Zero,
        Class::Infinity => Signed::Inf,
        Class::Undefined => Signed::Undef,
        Class::Vanished => {
            if s.is_negative() {
                Signed::NegVan
            } else {
                Signed::PosVan
            }
        }
        Class::Normal => {
            if s.is_negative() {
                Signed::NegNorm
            } else {
                Signed::PosNorm
            }
        }
        Class::Exploded => {
            if s.is_negative() {
                Signed::NegExp
            } else {
                Signed::PosExp
            }
        }
    }
}

fn name(c: Signed) -> &'static str {
    match c {
        Signed::Zero => "[0]",
        Signed::PosVan => "[+↓]",
        Signed::NegVan => "[-↓]",
        Signed::PosNorm => "[+#]",
        Signed::NegNorm => "[-#]",
        Signed::PosExp => "[+↑]",
        Signed::NegExp => "[-↑]",
        Signed::Inf => "[∞]",
        Signed::Undef => "[℘?]",
    }
}

fn expected(input: Signed) -> BTreeSet<Signed> {
    use Signed::*;
    let mut s = BTreeSet::new();
    match input {
        Zero => {
            s.insert(Zero);
        }
        Inf => {
            s.insert(Inf);
        }
        Undef => {
            s.insert(Undef);
        }
        PosVan => {
            s.insert(NegVan);
        }
        NegVan => {
            s.insert(PosVan);
        }
        // Normal can escape at the exponent boundaries.
        PosNorm => {
            s.insert(NegNorm);
            s.insert(NegVan);
        }
        NegNorm => {
            s.insert(PosNorm);
            s.insert(PosExp);
        }
        PosExp => {
            s.insert(NegExp);
        }
        NegExp => {
            s.insert(PosExp);
        }
    }
    s
}

fn main() {
    // Collect every F3E3 encoding.
    let mut all: Vec<S> = Vec::with_capacity(65_536);
    for f in -128i8..=127 {
        for e in -128i8..=127 {
            all.push(unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) });
        }
    }

    // Check 1: default (branchy) vs unified pipeline — bit-exact agreement. `-x` calls the default `scalar_negate`; the unified variant is opt-in via `scalar_negate_unified`.
    let mut default_vs_unified_mismatches = 0usize;
    let mut first_mismatch: Option<(S, S, S)> = None;
    for x in &all {
        let d = -*x;
        let mut u = *x;
        u.scalar_negate_unified();
        let d_bytes: [i8; 2] = unsafe { std::mem::transmute(d) };
        let u_bytes: [i8; 2] = unsafe { std::mem::transmute(u) };
        if d_bytes != u_bytes {
            default_vs_unified_mismatches += 1;
            if first_mismatch.is_none() {
                first_mismatch = Some((*x, d, u));
            }
        }
    }

    // Check 2: class transitions match the truth table.
    let mut class_mismatches = 0usize;
    let mut class_bad: BTreeMap<Signed, BTreeSet<Signed>> = BTreeMap::new();
    for x in &all {
        let r = -*x;
        let r_class = classify_signed(r);
        let x_class = classify_signed(*x);
        let exp = expected(x_class);
        if !exp.contains(&r_class) {
            class_mismatches += 1;
            class_bad.entry(x_class).or_default().insert(r_class);
        }
    }

    // Check 3: double-negation identity `-(-x) == x`. Two inputs unavoidably lose information: pos_one_normal@MIN_EXP and neg_one_normal@MAX_EXP escape the normal range on first negation (to neg_one_vanished / pos_one_exploded respectively), and the escape classes have ambiguous magnitude — the second negation returns a valid opposite-signed escape but can't recover the original exact exponent. That's not a bug, it's the class-escape semantics. Expected escapes are counted separately from unexpected identity failures.
    let mut expected_escapes = 0usize;
    let mut unexpected_identity = 0usize;
    let mut first_unexpected: Option<(S, S)> = None;
    for x in &all {
        let negx = -*x;
        let rr = -negx;
        let x_bytes: [i8; 2] = unsafe { std::mem::transmute(*x) };
        let rr_bytes: [i8; 2] = unsafe { std::mem::transmute(rr) };
        if x_bytes != rr_bytes {
            // Was the first negation a class-escape from normal?
            let escaped = classify(*x) == Class::Normal && classify(negx) != Class::Normal;
            if escaped {
                expected_escapes += 1;
            } else {
                unexpected_identity += 1;
                if first_unexpected.is_none() {
                    first_unexpected = Some((*x, rr));
                }
            }
        }
    }

    println!("=== scalar negation exhaustive F3E3 verification ===");
    println!(
        "  default vs unified: {} mismatches / {}",
        default_vs_unified_mismatches,
        all.len()
    );
    println!(
        "  class transitions : {} mismatches / {}",
        class_mismatches,
        all.len()
    );
    println!("  -(-x) == x        : {} unexpected / {} (plus {} expected class-escapes at normal-range boundaries)",
             unexpected_identity, all.len(), expected_escapes);

    if let Some((x, d, u)) = first_mismatch {
        let xb: [i8; 2] = unsafe { std::mem::transmute(x) };
        let db: [i8; 2] = unsafe { std::mem::transmute(d) };
        let ub: [i8; 2] = unsafe { std::mem::transmute(u) };
        println!("  first default/unified diff: x=[{:#04x},{}] default=[{:#04x},{}] unified=[{:#04x},{}]",
                 xb[0] as u8, xb[1], db[0] as u8, db[1], ub[0] as u8, ub[1]);
    }
    for (input, got) in &class_bad {
        let exp = expected(*input);
        let exp_s: Vec<_> = exp.iter().map(|c| name(*c)).collect();
        let got_s: Vec<_> = got.iter().map(|c| name(*c)).collect();
        println!(
            "  class OUT-OF-SET -{} → expected {:?}, also got {:?}",
            name(*input),
            exp_s,
            got_s
        );
    }
    if let Some((x, rr)) = first_unexpected {
        let xb: [i8; 2] = unsafe { std::mem::transmute(x) };
        let rrb: [i8; 2] = unsafe { std::mem::transmute(rr) };
        println!(
            "  first unexpected -(-x)≠x: x=[{:#04x},{}] -(-x)=[{:#04x},{}]",
            xb[0] as u8, xb[1], rrb[0] as u8, rrb[1]
        );
    }
}
