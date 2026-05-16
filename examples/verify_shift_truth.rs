//! Exhaustive F3E3 verification of << and >> against their expected truth table and sign-preservation rules. Shift by integer is "adjust exponent", which can keep a Normal in-range, push it to Exploded (overflow), or drop it to Vanished (underflow). Non-normals pass through unchanged.
use spirix::*;
use std::collections::BTreeMap;

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
    Shl,
    Shr,
}

fn op_name(o: Op) -> &'static str {
    match o {
        Op::Shl => "<<",
        Op::Shr => ">>",
    }
}

fn apply(o: Op, a: S, n: i32) -> S {
    match o {
        Op::Shl => a << n,
        Op::Shr => a >> n,
    }
}

// Expected class for Normal input shifted by true_delta (the unbounded exponent change: +n for <<, -n for >>). F3E3 has MIN_EXP=-127, MAX_EXP=127.
const MIN_EXP: i32 = -127;
const MAX_EXP: i32 = 127;

fn expected_normal_class(self_exp: i32, delta: i32) -> Class {
    // Compute unbounded true_new_exp; wrap around can't happen in i32.
    let true_new = self_exp + delta;
    if true_new > MAX_EXP {
        Class::Exploded
    } else if true_new < MIN_EXP {
        Class::Vanished
    } else {
        Class::Normal
    }
}

fn expected_class(o: Op, ca: Class, self_exp: i32, n: i32) -> Class {
    let delta = match o {
        Op::Shl => n,
        Op::Shr => -n,
    };
    match ca {
        Class::Zero | Class::Infinity | Class::Undefined => ca,
        Class::Vanished | Class::Exploded => ca, // design: non-normal passes through
        Class::Normal => expected_normal_class(self_exp, delta),
    }
}

fn main() {
    // Enumerate shift counts spanning i8 (native E on F3E3) so we hit every overflow/underflow corner. We test via the `i32` overload which saturate()s into E.
    let shifts: Vec<i32> = (-128..=127).collect();

    for op in [Op::Shl, Op::Shr] {
        let mut class_fails = 0usize;
        let mut sign_fails = 0usize;
        let mut total = 0usize;
        let mut class_bad: BTreeMap<(Class, Class), (S, i32, S)> = BTreeMap::new();
        let mut sign_bad: Vec<(S, i32, S)> = Vec::new();

        for f in -128i8..=127 {
            for e in -128i8..=127 {
                let s = unsafe { std::mem::transmute::<[i8; 2], S>([f, e]) };
                let ca = classify(s);
                for &n in &shifts {
                    total += 1;
                    let r = apply(op, s, n);
                    let rc = classify(r);
                    let exp_rc = expected_class(op, ca, e as i32, n);
                    if rc != exp_rc {
                        class_fails += 1;
                        class_bad.entry((ca, exp_rc)).or_insert((s, n, r));
                    }
                    // Sign-preservation check: for any input that has a sign (not Zero/Infinity/Undefined) and output that has one, shift must preserve sign.
                    let input_has_sign =
                        matches!(ca, Class::Vanished | Class::Normal | Class::Exploded);
                    let output_has_sign =
                        matches!(rc, Class::Vanished | Class::Normal | Class::Exploded);
                    if input_has_sign && output_has_sign {
                        if s.is_negative() != r.is_negative() {
                            sign_fails += 1;
                            if sign_bad.len() < 5 {
                                sign_bad.push((s, n, r));
                            }
                        }
                    }
                }
            }
        }
        println!(
            "\n=== {} ({} tests, {} class mismatches, {} sign mismatches) ===",
            op_name(op),
            total,
            class_fails,
            sign_fails
        );
        for ((ca, exp_rc), (s, n, r)) in class_bad.iter().take(12) {
            let sb = unsafe { std::mem::transmute::<S, [i8; 2]>(*s) };
            let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
            let sf: f64 = (*s).into();
            let rf: f64 = (*r).into();
            println!(
                "  {} {} {} [n={}] [{:#04x},{}]={} → [{:#04x},{}]={} (got {}, expected {})",
                name(*ca),
                op_name(op),
                "…",
                n,
                sb[0] as u8,
                sb[1],
                sf,
                rb[0] as u8,
                rb[1],
                rf,
                name(classify(*r)),
                name(*exp_rc)
            );
        }
        for (s, n, r) in sign_bad.iter().take(5) {
            let sb = unsafe { std::mem::transmute::<S, [i8; 2]>(*s) };
            let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(*r) };
            let sf: f64 = (*s).into();
            let rf: f64 = (*r).into();
            println!(
                "  sign {} [n={}]: [{:#04x},{}]={} (sign={}) → [{:#04x},{}]={} (sign={})",
                op_name(op),
                n,
                sb[0] as u8,
                sb[1],
                sf,
                if s.is_negative() { "-" } else { "+" },
                rb[0] as u8,
                rb[1],
                rf,
                if r.is_negative() { "-" } else { "+" }
            );
        }
    }
}
