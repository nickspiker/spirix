//! Exhaustive F3E3 verification of AND, OR, XOR, NOT against the README tables.
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

#[derive(Copy, Clone)]
enum Op {
    And,
    Or,
    Xor,
}

fn op_name(o: Op) -> &'static str {
    match o {
        Op::And => "&",
        Op::Or => "|",
        Op::Xor => "⊻",
    }
}

fn apply(o: Op, a: S, b: S) -> S {
    match o {
        Op::And => a & b,
        Op::Or => a | b,
        Op::Xor => a ^ b,
    }
}

fn expected(o: Op, a: Class, b: Class) -> BTreeSet<Class> {
    use Class::*;
    let mut s = BTreeSet::new();
    match o {
        Op::And => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Infinity, x) | (x, Infinity) => {
                s.insert(x);
            }
            (Zero, _) | (_, Zero) => {
                s.insert(Zero);
            }
            (Vanished, Vanished)
            | (Vanished, Normal)
            | (Normal, Vanished)
            | (Exploded, Exploded)
            | (Exploded, Normal)
            | (Normal, Exploded) => {
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
        },
        Op::Or => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Infinity, _) | (_, Infinity) => {
                s.insert(Infinity);
            }
            (Zero, x) | (x, Zero) => {
                s.insert(x);
            }
            (Vanished, Vanished)
            | (Vanished, Normal)
            | (Normal, Vanished)
            | (Exploded, Normal)
            | (Normal, Exploded)
            | (Exploded, Exploded) => {
                s.insert(Undefined);
            }
            (Vanished, Exploded) | (Exploded, Vanished) => {
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Normal, Normal) => {
                s.insert(Normal);
                s.insert(Vanished);
            }
        },
        Op::Xor => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Infinity, Zero) | (Zero, Infinity) => {
                s.insert(Infinity);
            }
            (Infinity, Vanished) | (Vanished, Infinity) => {
                s.insert(Vanished);
            }
            (Infinity, Normal) | (Normal, Infinity) => {
                s.insert(Normal);
            }
            (Infinity, Exploded) | (Exploded, Infinity) => {
                s.insert(Exploded);
            }
            (Infinity, Infinity) => {
                s.insert(Zero);
            }
            (Zero, x) | (x, Zero) => {
                s.insert(x);
            }
            (Vanished, Exploded) | (Exploded, Vanished) => {
                s.insert(Exploded);
            }
            (Normal, Normal) => {
                s.insert(Normal);
                s.insert(Zero);
                s.insert(Vanished);
            }
            (Vanished, Vanished)
            | (Vanished, Normal)
            | (Normal, Vanished)
            | (Exploded, Normal)
            | (Normal, Exploded)
            | (Exploded, Exploded) => {
                s.insert(Undefined);
            }
        },
    }
    s
}

fn reps() -> Vec<(Class, Vec<S>)> {
    let mut by_class: BTreeMap<Class, Vec<S>> = BTreeMap::new();
    for f in -128i8..=127 {
        for e in -128i8..=127 {
            let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
            by_class.entry(classify(s)).or_default().push(s);
        }
    }
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

fn run_op(o: Op, reps: &[(Class, Vec<S>)]) {
    let mut mismatches: BTreeMap<(Class, Class), BTreeSet<Class>> = BTreeMap::new();
    let mut observed: BTreeMap<(Class, Class), BTreeSet<Class>> = BTreeMap::new();
    let mut total = 0usize;
    let mut fails = 0usize;
    for (ca, va) in reps {
        for (cb, vb) in reps {
            let exp = expected(o, *ca, *cb);
            let seen = observed.entry((*ca, *cb)).or_default();
            for a in va {
                for b in vb {
                    total += 1;
                    let r = apply(o, *a, *b);
                    let rc = classify(r);
                    seen.insert(rc);
                    if !exp.contains(&rc) {
                        fails += 1;
                        mismatches.entry((*ca, *cb)).or_default().insert(rc);
                    }
                }
            }
        }
    }
    println!(
        "\n=== {} ({} pairs, {} mismatches) ===",
        op_name(o),
        total,
        fails
    );
    if fails > 0 {
        for ((ca, cb), got) in &mismatches {
            let exp = expected(o, *ca, *cb);
            let exp_s: Vec<_> = exp.iter().map(|c| name(*c)).collect();
            let got_s: Vec<_> = got.iter().map(|c| name(*c)).collect();
            println!(
                "  OUT-OF-SET {} {} {} — expected {:?}, also got {:?}",
                name(*ca),
                op_name(o),
                name(*cb),
                exp_s,
                got_s
            );
        }
    }
    let mut any_missing = false;
    for (ca, _) in reps {
        for (cb, _) in reps {
            let exp = expected(o, *ca, *cb);
            let empty = BTreeSet::new();
            let seen = observed.get(&(*ca, *cb)).unwrap_or(&empty);
            let missing: Vec<Class> = exp.iter().filter(|c| !seen.contains(c)).copied().collect();
            if !missing.is_empty() {
                if !any_missing {
                    println!("  Coverage gaps:");
                    any_missing = true;
                }
                let exp_s: Vec<_> = exp.iter().map(|c| name(*c)).collect();
                let miss_s: Vec<_> = missing.iter().map(|c| name(*c)).collect();
                println!(
                    "    {} {} {} — expected {:?}, missing {:?}",
                    name(*ca),
                    op_name(o),
                    name(*cb),
                    exp_s,
                    miss_s
                );
            }
        }
    }
}

// NOT is unary — separate from the binary Op enum. Per README: class map is Zero↔Infinity, all others preserve class with sign-flip on preserved classes (Vanished/Normal/Exploded).
fn not_expected_class(c: Class) -> Class {
    match c {
        Class::Zero => Class::Infinity,
        Class::Infinity => Class::Zero,
        Class::Vanished => Class::Vanished,
        Class::Normal => Class::Normal,
        Class::Exploded => Class::Exploded,
        Class::Undefined => Class::Undefined,
    }
}

fn run_not(reps: &[(Class, Vec<S>)]) {
    let mut class_fails = 0usize;
    let mut sign_fails = 0usize;
    let mut total = 0usize;
    let mut class_mismatch_samples: BTreeMap<Class, (S, S)> = BTreeMap::new();
    let mut sign_mismatch_samples: BTreeMap<Class, (S, S)> = BTreeMap::new();

    for (ca, va) in reps {
        let expected_class = not_expected_class(*ca);
        for a in va {
            total += 1;
            let r = !*a;
            let rc = classify(r);
            if rc != expected_class {
                class_fails += 1;
                class_mismatch_samples.entry(*ca).or_insert((*a, r));
            }
            // Sign-flip check: only meaningful on preserved classes where both input and output have a defined sign (not Zero/Infinity which are signless, not Undefined which has no sign contract).
            if matches!(*ca, Class::Vanished | Class::Normal | Class::Exploded)
                && rc == expected_class
            {
                let a_neg = a.is_negative();
                let r_neg = r.is_negative();
                if a_neg == r_neg {
                    sign_fails += 1;
                    sign_mismatch_samples.entry(*ca).or_insert((*a, r));
                }
            }
        }
    }

    println!(
        "\n=== ~ ({} inputs, {} class mismatches, {} sign mismatches) ===",
        total, class_fails, sign_fails
    );
    for (c, (a, r)) in &class_mismatch_samples {
        let ab = unsafe { std::mem::transmute::<S, [i8; 2]>(*a) };
        let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
        println!(
            "  class: ~{} [{:#04x},{}] → classified as {} [{:#04x},{}] (expected {})",
            name(*c),
            ab[0] as u8,
            ab[1],
            name(classify(*r)),
            rb[0] as u8,
            rb[1],
            name(not_expected_class(*c))
        );
    }
    for (c, (a, r)) in &sign_mismatch_samples {
        let ab = unsafe { std::mem::transmute::<S, [i8; 2]>(*a) };
        let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
        println!(
            "  sign:  ~{} [{:#04x},{}] sign={} → [{:#04x},{}] sign={} (expected flip)",
            name(*c),
            ab[0] as u8,
            ab[1],
            if a.is_negative() { "-" } else { "+" },
            rb[0] as u8,
            rb[1],
            if r.is_negative() { "-" } else { "+" }
        );
    }
}

fn main() {
    let reps = reps();
    run_op(Op::And, &reps);
    run_op(Op::Or, &reps);
    run_op(Op::Xor, &reps);
    run_not(&reps);
}
