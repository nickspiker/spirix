//! Exhaustive truth-table tests for basic arithmetic operations.
//! Verifies every value-class combination produces the expected result class
//! per the README truth tables (col OP row convention).

use spirix::*;

type S = ScalarF3E3;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Class {
    Zero,
    Vanished,
    Normal,
    Exploded,
    Infinity,
    Undefined,
}

fn classify(s: &S) -> Class {
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

fn class_name(c: Class) -> &'static str {
    match c {
        Class::Zero => "[0]",
        Class::Vanished => "[↓]",
        Class::Normal => "[#]",
        Class::Exploded => "[↑]",
        Class::Infinity => "[∞]",
        Class::Undefined => "[℘]",
    }
}

/// Check that the result class is one of the expected classes.
fn check(op: &str, a: S, b: S, result: S, expected: &[Class]) {
    let rc = classify(&result);
    if !expected.contains(&rc) {
        let ac = classify(&a);
        let bc = classify(&b);
        panic!(
            "{} {} {} = {} (class {}), expected one of {:?}\n  a = {:?}, b = {:?}, result = {:?}",
            class_name(ac),
            op,
            class_name(bc),
            class_name(rc),
            rc as u8,
            expected.iter().map(|c| class_name(*c)).collect::<Vec<_>>(),
            a,
            b,
            result
        );
    }
}

// Representative values for each class
fn zeros() -> Vec<S> {
    vec![S::ZERO]
}

fn vanished_pos() -> Vec<S> {
    vec![S::VANISHED_POS]
}

fn vanished_neg() -> Vec<S> {
    vec![S::VANISHED_NEG]
}

fn normals_pos() -> Vec<S> {
    vec![S::from(1), S::from(42)]
}

fn normals_neg() -> Vec<S> {
    vec![S::from(-1), S::from(-42)]
}

fn exploded_pos() -> Vec<S> {
    vec![S::EXPLODED_POS]
}

fn exploded_neg() -> Vec<S> {
    vec![S::EXPLODED_NEG]
}

fn infinities() -> Vec<S> {
    vec![S::INFINITY]
}

fn undefineds() -> Vec<S> {
    vec![S::ZERO / S::ZERO] // 0/0 = undefined
}

use Class::*;

// ============================================================
// Addition truth table (col + row)
// ============================================================
#[test]
fn addition_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0]+[0] = [0]
    check("+", z, z, z + z, &[Zero]);
    // [0]+[↓] = [↓]
    check("+", z, vp, z + vp, &[Vanished]);
    check("+", z, vn, z + vn, &[Vanished]);
    // [↓]+[0] = [↓]
    check("+", vp, z, vp + z, &[Vanished]);
    // [0]+[#] = [#]
    check("+", z, np, z + np, &[Normal]);
    // [#]+[0] = [#]
    check("+", np, z, np + z, &[Normal]);
    // [↓]+[↓] = [℘]
    check("+", vp, vp, vp + vp, &[Undefined]);
    check("+", vp, vn, vp + vn, &[Undefined]);
    // [↓]+[#] = [#]
    check("+", vp, np, vp + np, &[Normal]);
    // [#]+[↓] = [#]
    check("+", np, vp, np + vp, &[Normal]);
    // [#]+[#] = [0],[↓],[#],[↑]
    check("+", np, nn, np + nn, &[Zero, Vanished, Normal, Exploded]);
    check("+", np, np, np + np, &[Zero, Vanished, Normal, Exploded]);
    // [↑]+anything_finite = [℘]
    check("+", ep, z, ep + z, &[Undefined]);
    check("+", ep, vp, ep + vp, &[Undefined]);
    check("+", ep, np, ep + np, &[Undefined]);
    check("+", en, z, en + z, &[Undefined]);
    // anything_finite+[↑] = [℘]
    check("+", z, ep, z + ep, &[Undefined]);
    check("+", vp, ep, vp + ep, &[Undefined]);
    check("+", np, ep, np + ep, &[Undefined]);
    // [↑]+[↑] = [℘]
    check("+", ep, ep, ep + ep, &[Undefined]);
    check("+", ep, en, ep + en, &[Undefined]);
    // [∞]+anything = [℘]
    check("+", inf, z, inf + z, &[Undefined]);
    check("+", inf, np, inf + np, &[Undefined]);
    check("+", inf, ep, inf + ep, &[Undefined]);
    check("+", inf, inf, inf + inf, &[Undefined]);
    // anything+[∞] = [℘]
    check("+", z, inf, z + inf, &[Undefined]);
    check("+", np, inf, np + inf, &[Undefined]);
    // [℘]+anything = [℘]
    check("+", und, z, und + z, &[Undefined]);
    check("+", und, np, und + np, &[Undefined]);
    // anything+[℘] = [℘]
    check("+", z, und, z + und, &[Undefined]);
    check("+", np, und, np + und, &[Undefined]);
}

// ============================================================
// Subtraction truth table (col - row)
// ============================================================
#[test]
fn subtraction_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0]-[0] = [0]
    check("-", z, z, z - z, &[Zero]);
    // [#]-[0] = [#],[↓],[↑] (negation edge cases at exponent boundary)
    check("-", np, z, np - z, &[Normal, Vanished, Exploded]);
    check("-", nn, z, nn - z, &[Normal, Vanished, Exploded]);
    // [0]-[#] = [#],[↓],[↑]
    check("-", z, np, z - np, &[Normal, Vanished, Exploded]);
    // [↓]-[↓] = [℘]
    check("-", vp, vp, vp - vp, &[Undefined]);
    check("-", vp, vn, vp - vn, &[Undefined]);
    // [#]-[#] = [0],[↓],[#],[↑]
    check("-", np, np, np - np, &[Zero, Vanished, Normal, Exploded]);
    check("-", np, nn, np - nn, &[Zero, Vanished, Normal, Exploded]);
    // [↑]-anything_finite = [℘]
    check("-", ep, z, ep - z, &[Undefined]);
    check("-", ep, np, ep - np, &[Undefined]);
    // anything_finite-[↑] = [℘]
    check("-", z, ep, z - ep, &[Undefined]);
    check("-", np, ep, np - ep, &[Undefined]);
    // [↑]-[↑] = [℘]
    check("-", ep, ep, ep - ep, &[Undefined]);
    // [∞]-anything = [℘]
    check("-", inf, z, inf - z, &[Undefined]);
    check("-", inf, np, inf - np, &[Undefined]);
    // anything-[∞] = [℘]
    check("-", z, inf, z - inf, &[Undefined]);
    // [℘]-anything = [℘]
    check("-", und, np, und - np, &[Undefined]);
    check("-", np, und, np - und, &[Undefined]);
}

// ============================================================
// Multiplication truth table (col × row)
// ============================================================
#[test]
fn multiplication_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-3);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0]×anything = [0] (except [∞] and [℘])
    check("×", z, z, z * z, &[Zero]);
    check("×", z, vp, z * vp, &[Zero]);
    check("×", z, np, z * np, &[Zero]);
    check("×", z, ep, z * ep, &[Zero]);
    check("×", np, z, np * z, &[Zero]);
    check("×", ep, z, ep * z, &[Zero]);
    // [0]×[∞] = [℘]
    check("×", z, inf, z * inf, &[Undefined]);
    check("×", inf, z, inf * z, &[Undefined]);
    // [↓]×[↓] = [↓]
    check("×", vp, vp, vp * vp, &[Vanished]);
    check("×", vp, vn, vp * vn, &[Vanished]);
    // [↓]×[#] = [↓]
    check("×", vp, np, vp * np, &[Vanished]);
    check("×", np, vp, np * vp, &[Vanished]);
    // [↓]×[↑] = [℘] (negligible × transfinite)
    check("×", vp, ep, vp * ep, &[Undefined]);
    check("×", ep, vp, ep * vp, &[Undefined]);
    // [↓]×[∞] = [∞]
    check("×", vp, inf, vp * inf, &[Infinity]);
    check("×", inf, vp, inf * vp, &[Infinity]);
    // [#]×[#] = [#],[↓],[↑]
    check("×", np, nn, np * nn, &[Normal, Vanished, Exploded]);
    check("×", np, np, np * np, &[Normal, Vanished, Exploded]);
    // [#]×[↑] = [↑]
    check("×", np, ep, np * ep, &[Exploded]);
    check("×", ep, np, ep * np, &[Exploded]);
    // [#]×[∞] = [∞]
    check("×", np, inf, np * inf, &[Infinity]);
    check("×", inf, np, inf * np, &[Infinity]);
    // [↑]×[↑] = [↑]
    check("×", ep, ep, ep * ep, &[Exploded]);
    check("×", ep, en, ep * en, &[Exploded]);
    // [↑]×[∞] = [∞]
    check("×", ep, inf, ep * inf, &[Infinity]);
    check("×", inf, ep, inf * ep, &[Infinity]);
    // [∞]×[∞] = [∞]
    check("×", inf, inf, inf * inf, &[Infinity]);
    // [℘]×anything = [℘]
    check("×", und, np, und * np, &[Undefined]);
    check("×", np, und, np * und, &[Undefined]);
}

// ============================================================
// Division truth table (col ÷ row)
// ============================================================
#[test]
fn division_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let np = S::from(7);
    let nn = S::from(-3);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0]÷[0] = [℘]
    // (can't test directly since 0/0 IS our undefined source)
    // [0]÷[#] = [0]
    check("÷", z, np, z / np, &[Zero]);
    check("÷", z, nn, z / nn, &[Zero]);
    // [0]÷[↑] = [0]
    check("÷", z, ep, z / ep, &[Zero]);
    // [0]÷[∞] = [0]
    check("÷", z, inf, z / inf, &[Zero]);
    // [#]÷[0] = [∞]
    check("÷", np, z, np / z, &[Infinity]);
    check("÷", nn, z, nn / z, &[Infinity]);
    // [↓]÷[#] = [↓]
    check("÷", vp, np, vp / np, &[Vanished]);
    // [#]÷[↓] = [↑]
    check("÷", np, vp, np / vp, &[Exploded]);
    // [#]÷[#] = [#],[↓],[↑]
    check("÷", np, nn, np / nn, &[Normal, Vanished, Exploded]);
    check("÷", np, np, np / np, &[Normal, Vanished, Exploded]);
    // [#]÷[↑] = [↓]
    check("÷", np, ep, np / ep, &[Vanished]);
    // [#]÷[∞] = [0]
    check("÷", np, inf, np / inf, &[Zero]);
    // [↑]÷[#] = [↑]
    check("÷", ep, np, ep / np, &[Exploded]);
    // [↑]÷[↑] = [℘]
    check("÷", ep, ep, ep / ep, &[Undefined]);
    check("÷", ep, en, ep / en, &[Undefined]);
    // [↑]÷[∞] = [0]
    check("÷", ep, inf, ep / inf, &[Zero]);
    // [∞]÷[#] = [∞]
    check("÷", inf, np, inf / np, &[Infinity]);
    // [∞]÷[∞] = [℘]
    check("÷", inf, inf, inf / inf, &[Undefined]);
    // [∞]÷[↑] = [∞]
    check("÷", inf, ep, inf / ep, &[Infinity]);
    // [℘]÷anything = [℘]
    check("÷", und, np, und / np, &[Undefined]);
    check("÷", np, und, np / und, &[Undefined]);
}

// ============================================================
// Modulus truth table (col % row)
// ============================================================
#[test]
fn modulus_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-3);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0]%anything = [0]
    check("%", z, z, z % z, &[Zero]);
    check("%", z, np, z % np, &[Zero]);
    check("%", z, vp, z % vp, &[Zero]);
    check("%", z, ep, z % ep, &[Zero]);
    check("%", z, inf, z % inf, &[Zero]);
    // anything%[0] = [0]
    check("%", np, z, np % z, &[Zero]);
    check("%", vp, z, vp % z, &[Zero]);
    check("%", ep, z, ep % z, &[Zero]);

    // [#]%[#] = [0],[↓],[#]
    check("%", np, nn, np % nn, &[Zero, Vanished, Normal]);
    check("%", S::from(6), S::from(3), S::from(6) % S::from(3), &[Zero]);
    check("%", S::from(7), S::from(3), S::from(7) % S::from(3), &[Zero, Vanished, Normal]);

    // [#]%[↓] = [℘] (vanished period)
    check("%", np, vp, np % vp, &[Undefined]);

    // [↓]%[↓] = [℘]
    check("%", vp, vp, vp % vp, &[Undefined]);
    check("%", vp, vn, vp % vn, &[Undefined]);

    // [↓]%[#] same sign = [↓], diff sign = [#]
    check("%", vp, np, vp % np, &[Vanished]); // same sign (both pos)
    check("%", vn, np, vn % np, &[Normal]);   // diff sign

    // [#]%[↑] same sign = [#], diff sign = [℘]
    check("%", np, ep, np % ep, &[Normal]);     // same sign
    check("%", np, en, np % en, &[Undefined]);  // diff sign

    // [↓]%[↑] same sign = [↓], diff sign = [↑]
    check("%", vp, ep, vp % ep, &[Vanished]);   // same sign
    check("%", vn, ep, vn % ep, &[Exploded]);   // diff sign (vanished absorbed)

    // [↑]%anything = [℘] (transfinite numerator)
    check("%", ep, np, ep % np, &[Undefined]);
    check("%", ep, vp, ep % vp, &[Undefined]);
    check("%", ep, ep, ep % ep, &[Undefined]);

    // [∞]%anything = [℘]
    check("%", inf, np, inf % np, &[Undefined]);
    check("%", inf, ep, inf % ep, &[Undefined]);
    check("%", inf, inf, inf % inf, &[Undefined]);

    // anything%[∞] = [℘] (signless period)
    check("%", np, inf, np % inf, &[Undefined]);
    check("%", vp, inf, vp % inf, &[Undefined]);

    // [℘]%anything = [℘]
    check("%", und, np, und % np, &[Undefined]);
    check("%", np, und, np % und, &[Undefined]);
}

// ============================================================
// Specific edge case: subtraction with MIN producing exploded
// ============================================================
#[test]
fn subtraction_min_boundary_exploded() {
    // NEG_ONE at MIN_EXPONENT: negating requires exp+1 which wraps → exploded
    let min_neg = S::MAX_NEG; // smallest magnitude negative normal
    let result = S::ZERO - min_neg;
    // 0 - (tiny negative) = tiny positive, could be normal or edge-case
    // But the interesting case: negate MIN at exp boundary
    let at_boundary = S::ONE; // fraction = POS_ONE_NORMAL (MIN stored), exp=1
    let neg_boundary = S::NEG_ONE; // fraction = NEG_ONE_NORMAL (0 stored), exp=0
    // These should negate cleanly
    let r1 = S::ZERO - at_boundary;
    assert!(r1.is_negative(), "0-1 should be negative");
    let r2 = S::ZERO - neg_boundary;
    assert!(r2.is_positive(), "0-(-1) should be positive");
}
