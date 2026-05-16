//! Exhaustive F3E3 verification of +, -, ×, /, % against their README truth tables.
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
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

fn op_name(o: Op) -> &'static str {
    match o {
        Op::Add => "+",
        Op::Sub => "-",
        Op::Mul => "×",
        Op::Div => "/",
        Op::Mod => "%",
    }
}

fn apply(o: Op, a: S, b: S) -> S {
    match o {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Div => a / b,
        Op::Mod => a % b,
    }
}

fn expected(o: Op, a: Class, b: Class) -> BTreeSet<Class> {
    use Class::*;
    let mut s = BTreeSet::new();
    match o {
        Op::Add => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Infinity, _) | (_, Infinity) | (Exploded, _) | (_, Exploded) => {
                s.insert(Undefined);
            }
            (Zero, Zero) => {
                s.insert(Zero);
            }
            (Zero, Vanished) | (Vanished, Zero) => {
                s.insert(Vanished);
            }
            (Zero, Normal) | (Normal, Zero) => {
                s.insert(Normal);
            }
            (Vanished, Vanished) => {
                s.insert(Undefined);
            }
            (Vanished, Normal) | (Normal, Vanished) => {
                s.insert(Normal);
            }
            (Normal, Normal) => {
                s.insert(Zero);
                s.insert(Vanished);
                s.insert(Normal);
                s.insert(Exploded);
            }
        },
        Op::Sub => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Infinity, _) | (_, Infinity) | (Exploded, _) | (_, Exploded) => {
                s.insert(Undefined);
            }
            (Zero, Zero) => {
                s.insert(Zero);
            }
            (Zero, Vanished) | (Vanished, Zero) => {
                s.insert(Vanished);
            }
            (Zero, Normal) => {
                s.insert(Normal);
            }
            (Normal, Zero) => {
                s.insert(Normal);
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Vanished, Vanished) => {
                s.insert(Undefined);
            }
            (Vanished, Normal) => {
                s.insert(Normal);
            }
            (Normal, Vanished) => {
                s.insert(Normal);
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Normal, Normal) => {
                s.insert(Zero);
                s.insert(Vanished);
                s.insert(Normal);
                s.insert(Exploded);
            }
        },
        Op::Mul => match (a, b) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            (Zero, Infinity) | (Infinity, Zero) => {
                s.insert(Undefined);
            }
            (Zero, _) | (_, Zero) => {
                s.insert(Zero);
            }
            (Vanished, Exploded) | (Exploded, Vanished) => {
                s.insert(Undefined);
            }
            (Vanished, Infinity) | (Infinity, Vanished) => {
                s.insert(Infinity);
            }
            (Vanished, Vanished) | (Vanished, Normal) | (Normal, Vanished) => {
                s.insert(Vanished);
            }
            (Normal, Normal) => {
                s.insert(Normal);
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Normal, Exploded) | (Exploded, Normal) | (Exploded, Exploded) => {
                s.insert(Exploded);
            }
            (Normal, Infinity) | (Infinity, Normal) => {
                s.insert(Infinity);
            }
            (Exploded, Infinity) | (Infinity, Exploded) => {
                s.insert(Infinity);
            }
            (Infinity, Infinity) => {
                s.insert(Infinity);
            }
        },
        // Division truth table from README. Convention: in run_op, a=row, b=col, and the operation computed is `b OP a` (col is first operand). So for division the match reads (a /* denominator */, b /* numerator */).
        Op::Div => match (a /* denom */, b /* numer */) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            // Indeterminate: 0/0, ↓/↓, ↑/↑, ∞/∞
            (Zero, Zero) | (Vanished, Vanished) | (Exploded, Exploded) | (Infinity, Infinity) => {
                s.insert(Undefined);
            }
            // X / 0 = ∞ for any non-zero X.
            (Zero, _) => {
                s.insert(Infinity);
            }
            // X / ∞ = 0 for any non-infinity X.
            (Infinity, _) => {
                s.insert(Zero);
            }
            // 0 / X = 0 for any non-zero X (denom non-zero, handled above).
            (_, Zero) => {
                s.insert(Zero);
            }
            // ∞ / X = ∞ for any non-infinity X.
            (_, Infinity) => {
                s.insert(Infinity);
            }
            // Remaining: denom is Vanished/Normal/Exploded, numer is Vanished/Normal/Exploded.
            (Vanished, Normal) => {
                s.insert(Exploded);
            } // finite/tiny = huge
            (Vanished, Exploded) => {
                s.insert(Exploded);
            } // huge/tiny = huge
            (Normal, Vanished) => {
                s.insert(Vanished);
            } // tiny/finite = tiny
            (Normal, Normal) => {
                s.insert(Normal);
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Normal, Exploded) => {
                s.insert(Exploded);
            }
            (Exploded, Vanished) => {
                s.insert(Vanished);
            } // tiny/huge = tiny
            (Exploded, Normal) => {
                s.insert(Vanished);
            } // finite/huge = tiny
        },
        // Modulus truth table from README. Same convention as Div: in run_op, a=row, b=col, and the operation computed is `b OP a`. For %, the row is the PERIOD (divisor) and col is the NUMERATOR. Cell entries with `X / Y` notation (sign-dependent) contribute both to the set.
        Op::Mod => match (a /* period */, b /* numer */) {
            (Undefined, _) | (_, Undefined) => {
                s.insert(Undefined);
            }
            // Row [0] (period=Zero): always 0. Also col [0] numer=Zero → 0.
            (Zero, _) | (_, Zero) => {
                s.insert(Zero);
            }
            // Row [↓] (period=Vanished): all undefined (after Zero handled).
            (Vanished, _) => {
                s.insert(Undefined);
            }
            // Row [∞] (period=Infinity): all undefined (after Zero handled).
            (Infinity, _) => {
                s.insert(Undefined);
            }
            // Transfinite numerator (col [↑] or [∞]) with Normal/Exploded period (Vanished/Infinity handled above) → rule 3: undefined.
            (Normal, Exploded)
            | (Normal, Infinity)
            | (Exploded, Exploded)
            | (Exploded, Infinity) => {
                s.insert(Undefined);
            }
            // Row [#] (period=Normal): Vanished numer → [↓] or [#] depending on signs. Normal numer → [0], [↓], [#] depending on magnitudes.
            (Normal, Vanished) => {
                s.insert(Vanished);
                s.insert(Normal);
            }
            (Normal, Normal) => {
                s.insert(Zero);
                s.insert(Vanished);
                s.insert(Normal);
            }
            // Row [↑] (period=Exploded): Vanished numer → [↓] / [↑] signs. Normal numer → [#] / [℘%⬆] signs.
            (Exploded, Vanished) => {
                s.insert(Vanished);
                s.insert(Exploded);
            }
            (Exploded, Normal) => {
                s.insert(Normal);
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
    let mut first_bad: BTreeMap<(Class, Class, Class), (S, S, S)> = BTreeMap::new();
    let mut total = 0usize;
    let mut fails = 0usize;
    for (ca, va) in reps {
        for (cb, vb) in reps {
            let exp = expected(o, *ca, *cb);
            let seen = observed.entry((*ca, *cb)).or_default();
            for a in va {
                for b in vb {
                    total += 1;
                    // Table convention is `col OP row`: col is first operand. Here ca=row, cb=col, so compute b OP a (col first, row second).
                    let r = apply(o, *b, *a);
                    let rc = classify(r);
                    seen.insert(rc);
                    if !exp.contains(&rc) {
                        fails += 1;
                        mismatches.entry((*ca, *cb)).or_default().insert(rc);
                        first_bad.entry((*ca, *cb, rc)).or_insert((*b, *a, r));
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
        for rc in got {
            if let Some((lhs, rhs, res)) = first_bad.get(&(*ca, *cb, *rc)) {
                let lb = unsafe { std::mem::transmute::<S, [i8; 2]>(*lhs) };
                let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*rhs) };
                let rs = unsafe { std::mem::transmute::<S, [i8; 2]>(*res) };
                let lf: f64 = (*lhs).into();
                let rf: f64 = (*rhs).into();
                let res_f: f64 = (*res).into();
                println!(
                    "    -> got {}: lhs=[{:#04x},{}]={} rhs=[{:#04x},{}]={} result=[{:#04x},{}]={}",
                    name(*rc),
                    lb[0] as u8,
                    lb[1],
                    lf,
                    rb[0] as u8,
                    rb[1],
                    rf,
                    rs[0] as u8,
                    rs[1],
                    res_f
                );
            }
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

fn main() {
    let reps = reps();
    for op in [Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Mod] {
        run_op(op, &reps);
    }
}
