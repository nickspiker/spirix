//! Verify `aligned_and` against the README truth table, class-level. Uses F3E3 with representative values from each class + every normal value in the "clean middle" of the exponent range.

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

fn name(c: Class) -> &'static str {
    match c {
        Class::Zero => "[0]",
        Class::Vanished => "[↓]",
        Class::Normal => "[#]",
        Class::Exploded => "[↑]",
        Class::Infinity => "[∞]",
        Class::Undefined => "[℘?]",
    }
}

/// Expected output classes per (row, col), from the README AND table.
fn expected(a: Class, b: Class) -> BTreeSet<Class> {
    use Class::*;
    let mut s = BTreeSet::new();
    match (a, b) {
        (Undefined, _) | (_, Undefined) => {
            s.insert(Undefined);
        }
        (Zero, _) | (_, Zero) => {
            s.insert(Zero);
        }
        (Infinity, x) | (x, Infinity) => {
            s.insert(x);
        }
        (Vanished, Vanished) => {
            s.insert(Undefined);
        }
        (Vanished, Normal) | (Normal, Vanished) => {
            s.insert(Undefined);
        }
        (Vanished, Exploded) | (Exploded, Vanished) => {
            s.insert(Zero);
            s.insert(Exploded);
        }
        (Normal, Normal) => {
            s.insert(Normal);
            s.insert(Zero);
            s.insert(Vanished);
        }
        (Normal, Exploded) | (Exploded, Normal) => {
            s.insert(Undefined);
        }
        (Exploded, Exploded) => {
            s.insert(Undefined);
        }
    }
    s
}

fn reps() -> Vec<(Class, Vec<S>)> {
    // Exhaustively enumerate every F3E3 value, classify, and bucket by class.
    let mut by_class: BTreeMap<Class, Vec<S>> = BTreeMap::new();
    for f in -128i8..=127 {
        for e in -128i8..=127 {
            let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
            by_class.entry(classify(s)).or_default().push(s);
        }
    }
    // Include edge cases and don't cap — every Scalar value is a rep.
    [
        Class::Zero,
        Class::Vanished,
        Class::Normal,
        Class::Exploded,
        Class::Infinity,
        Class::Undefined,
    ]
    .iter()
    .map(|c| (*c, by_class.remove(c).unwrap_or_default()))
    .collect()
}

fn main() {
    let reps = reps();
    let mut mismatches: BTreeMap<(Class, Class), Vec<(S, S, Class)>> = BTreeMap::new();
    let mut observed: BTreeMap<(Class, Class), BTreeSet<Class>> = BTreeMap::new();
    let mut total = 0usize;
    let mut fails = 0usize;

    for (ca, va) in &reps {
        for (cb, vb) in &reps {
            let exp = expected(*ca, *cb);
            let seen = observed.entry((*ca, *cb)).or_default();
            for a in va {
                for b in vb {
                    total += 1;
                    let r = *a & *b;
                    let rc = classify(r);
                    seen.insert(rc);
                    if !exp.contains(&rc) {
                        fails += 1;
                        mismatches.entry((*ca, *cb)).or_default().push((*a, *b, rc));
                    }
                }
            }
        }
    }

    println!("Tested {} pairs, {} class-mismatch(es).", total, fails);

    // Report any OUT-OF-SET outputs (got something not in the table)
    for ((ca, cb), cases) in &mismatches {
        let exp = expected(*ca, *cb);
        let exp_str: Vec<&str> = exp.iter().map(|c| name(*c)).collect();
        println!(
            "\nOUT-OF-SET: {} & {} — expected {:?}, got:",
            name(*ca),
            name(*cb),
            exp_str
        );
        let mut seen: BTreeSet<Class> = BTreeSet::new();
        for (a, b, rc) in cases.iter().take(200) {
            if seen.insert(*rc) {
                let af: f64 = (*a).into();
                let bf: f64 = (*b).into();
                println!("  {} from {} & {}", name(*rc), af, bf);
            }
        }
        println!("  ({} cases total)", cases.len());
    }

    // Report COVERAGE: any class listed in the table that was NOT observed
    let mut missing_anywhere = false;
    println!("\nCoverage check (table lists classes but test didn't observe them):");
    for (ca, _) in &reps {
        for (cb, _) in &reps {
            let exp = expected(*ca, *cb);
            let empty = BTreeSet::new();
            let seen = observed.get(&(*ca, *cb)).unwrap_or(&empty);
            let missing: Vec<Class> = exp.iter().filter(|c| !seen.contains(c)).copied().collect();
            if !missing.is_empty() {
                missing_anywhere = true;
                let miss_str: Vec<&str> = missing.iter().map(|c| name(*c)).collect();
                let exp_str: Vec<&str> = exp.iter().map(|c| name(*c)).collect();
                println!(
                    "  {} & {} — expected {:?}, missing {:?}",
                    name(*ca),
                    name(*cb),
                    exp_str,
                    miss_str
                );
            }
        }
    }
    if !missing_anywhere {
        println!("  (none — every class listed in the table was produced by at least one sample)");
    }
}
