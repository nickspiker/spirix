//! Exhaustive truth-table tests for basic arithmetic operations. Verifies every value-class combination produces the expected result class per the README truth tables (col OP row convention).

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

// ============================================================ Addition truth table (col + row) ============================================================
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
    // [↑]+[0] = [↑] (zero is additive identity)
    check("+", ep, z, ep + z, &[Exploded]);
    check("+", en, z, en + z, &[Exploded]);
    // [↑]+[↓] = [↑] (vanished is negligible against exploded)
    check("+", ep, vp, ep + vp, &[Exploded]);
    check("+", ep, vn, ep + vn, &[Exploded]);
    // [↑]+[#] = [℘] (could partially cancel back into normal range)
    check("+", ep, np, ep + np, &[Undefined]);
    // [0]+[↑] = [↑]
    check("+", z, ep, z + ep, &[Exploded]);
    // [↓]+[↑] = [↑]
    check("+", vp, ep, vp + ep, &[Exploded]);
    // [#]+[↑] = [℘]
    check("+", np, ep, np + ep, &[Undefined]);
    // [↑]+[↑] = [℘] (opposing phases could cancel)
    check("+", ep, ep, ep + ep, &[Undefined]);
    check("+", ep, en, ep + en, &[Undefined]);
    // [∞]+anything = [∞] (infinity absorbs everything; −∞ no-op so ∞−∞ = ∞ too)
    check("+", inf, z, inf + z, &[Infinity]);
    check("+", inf, vp, inf + vp, &[Infinity]);
    check("+", inf, np, inf + np, &[Infinity]);
    check("+", inf, ep, inf + ep, &[Infinity]);
    check("+", inf, inf, inf + inf, &[Infinity]);
    // anything+[∞] = [∞]
    check("+", z, inf, z + inf, &[Infinity]);
    check("+", vp, inf, vp + inf, &[Infinity]);
    check("+", np, inf, np + inf, &[Infinity]);
    check("+", ep, inf, ep + inf, &[Infinity]);
    // [℘]+anything = [℘] (undefined propagates, but is checked before infinity)
    check("+", und, z, und + z, &[Undefined]);
    check("+", und, np, und + np, &[Undefined]);
    check("+", und, inf, und + inf, &[Undefined]);
    // anything+[℘] = [℘]
    check("+", z, und, z + und, &[Undefined]);
    check("+", np, und, np + und, &[Undefined]);
    check("+", inf, und, inf + und, &[Undefined]);
}

// ============================================================ Subtraction truth table (col - row) ============================================================
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
    // [↑]-[0] = [↑] (zero identity)
    check("-", ep, z, ep - z, &[Exploded]);
    // [↑]-[↓] = [↑] (vanished negligible)
    check("-", ep, vp, ep - vp, &[Exploded]);
    check("-", ep, vn, ep - vn, &[Exploded]);
    // [↑]-[#] = [℘]
    check("-", ep, np, ep - np, &[Undefined]);
    // [0]-[↑] = [↑] (negation of exploded is exploded; signed exploded retains class)
    check("-", z, ep, z - ep, &[Exploded]);
    // [↓]-[↑] = [↑]
    check("-", vp, ep, vp - ep, &[Exploded]);
    // [#]-[↑] = [℘]
    check("-", np, ep, np - ep, &[Undefined]);
    // [↑]-[↑] = [℘]
    check("-", ep, ep, ep - ep, &[Undefined]);
    // [∞]-anything = [∞] (infinity absorbs; −∞ no-op so ∞−∞ = ∞)
    check("-", inf, z, inf - z, &[Infinity]);
    check("-", inf, vp, inf - vp, &[Infinity]);
    check("-", inf, np, inf - np, &[Infinity]);
    check("-", inf, ep, inf - ep, &[Infinity]);
    check("-", inf, inf, inf - inf, &[Infinity]);
    // anything-[∞] = [∞]
    check("-", z, inf, z - inf, &[Infinity]);
    check("-", vp, inf, vp - inf, &[Infinity]);
    check("-", np, inf, np - inf, &[Infinity]);
    check("-", ep, inf, ep - inf, &[Infinity]);
    // [℘]-anything = [℘]
    check("-", und, np, und - np, &[Undefined]);
    check("-", und, inf, und - inf, &[Undefined]);
    // anything-[℘] = [℘]
    check("-", np, und, np - und, &[Undefined]);
    check("-", inf, und, inf - und, &[Undefined]);
}

// ============================================================ Multiplication truth table (col × row) ============================================================
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

// ============================================================ Division truth table (col ÷ row) ============================================================
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

    // [0]÷[0] = [℘] (can't test directly since 0/0 IS our undefined source) [0]÷[#] = [0]
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

// ============================================================ Modulus truth table (col % row) ============================================================
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
    check(
        "%",
        S::from(6),
        S::from(3),
        S::from(6) % S::from(3),
        &[Zero],
    );
    check(
        "%",
        S::from(7),
        S::from(3),
        S::from(7) % S::from(3),
        &[Zero, Vanished, Normal],
    );

    // [#]%[↓] = [℘] (vanished period)
    check("%", np, vp, np % vp, &[Undefined]);

    // [↓]%[↓] = [℘]
    check("%", vp, vp, vp % vp, &[Undefined]);
    check("%", vp, vn, vp % vn, &[Undefined]);

    // [↓]%[#] same sign = [↓], diff sign = [#]
    check("%", vp, np, vp % np, &[Vanished]); // same sign (both pos)
    check("%", vn, np, vn % np, &[Normal]); // diff sign

    // [#]%[↑] same sign = [#], diff sign = [℘]
    check("%", np, ep, np % ep, &[Normal]); // same sign
    check("%", np, en, np % en, &[Undefined]); // diff sign

    // [↓]%[↑] same sign = [↓], diff sign = [↑]
    check("%", vp, ep, vp % ep, &[Vanished]); // same sign
    check("%", vn, ep, vn % ep, &[Exploded]); // diff sign (vanished absorbed)

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

// ============================================================ Unary operation truth tables: sqrt, lb, ln, exp, powb, square ============================================================

/// Verify the result class of a unary op matches the expected set.
fn check_unary(op: &str, input_name: &str, x: S, result: S, expected: &[Class]) {
    let rc = classify(&result);
    if !expected.contains(&rc) {
        panic!(
            "{}({}) = {:?} (class {} = {})\n  expected one of {:?}\n  input  : {:?}\n  result : {:?}",
            op, input_name, rc, rc as u8, class_name(rc),
            expected.iter().map(|c| class_name(*c)).collect::<Vec<_>>(),
            x, result
        );
    }
}

#[test]
fn sqrt_unary_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(4); // sqrt(4) = 2
    let nn = S::from(-4); // sqrt(-4) = undefined
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("sqrt", "[0]", z, z.sqrt(), &[Zero]);
    check_unary("sqrt", "[+↓]", vp, vp.sqrt(), &[Undefined]);
    check_unary("sqrt", "[-↓]", vn, vn.sqrt(), &[Undefined]);
    check_unary("sqrt", "[+#]", np, np.sqrt(), &[Normal]);
    check_unary("sqrt", "[-#]", nn, nn.sqrt(), &[Undefined]);
    check_unary("sqrt", "[+↑]", ep, ep.sqrt(), &[Undefined]);
    check_unary("sqrt", "[-↑]", en, en.sqrt(), &[Undefined]);
    check_unary("sqrt", "[∞]", inf, inf.sqrt(), &[Infinity]);
    check_unary("sqrt", "[℘]", und, und.sqrt(), &[Undefined]);
}

#[test]
fn lb_unary_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(2); // lb(2) = 1
    let nn = S::from(-2); // lb(-2) = undefined
    let one = S::ONE; // lb(1) = 0
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("lb", "[0]", z, z.lb(), &[Infinity]);
    check_unary("lb", "[+↓]", vp, vp.lb(), &[Undefined]);
    check_unary("lb", "[-↓]", vn, vn.lb(), &[Undefined]);
    check_unary("lb", "[+# >1]", np, np.lb(), &[Normal]);
    check_unary("lb", "[+# =1]", one, one.lb(), &[Zero, Normal]); // lb(1) = 0
    check_unary("lb", "[-#]", nn, nn.lb(), &[Undefined]);
    check_unary("lb", "[+↑]", ep, ep.lb(), &[Undefined]);
    check_unary("lb", "[-↑]", en, en.lb(), &[Undefined]);
    check_unary("lb", "[∞]", inf, inf.lb(), &[Infinity]);
    check_unary("lb", "[℘]", und, und.lb(), &[Undefined]);
}

#[test]
fn ln_unary_truth_table() {
    // ln behaves identically to lb on classes
    let z = S::ZERO;
    let np = S::from(2);
    let one = S::ONE;
    let nn = S::from(-2);
    let und = S::ZERO / S::ZERO;
    check_unary("ln", "[0]", z, z.ln(), &[Infinity]);
    check_unary("ln", "[+#]", np, np.ln(), &[Normal]);
    check_unary("ln", "[+# =1]", one, one.ln(), &[Zero, Normal]);
    check_unary("ln", "[-#]", nn, nn.ln(), &[Undefined]);
    check_unary("ln", "[℘]", und, und.ln(), &[Undefined]);
}

#[test]
fn exp_unary_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(2); // exp(2) ≈ 7.39
    let nn = S::from(-2); // exp(-2) ≈ 0.135
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("exp", "[0]", z, z.exp(), &[Normal]); // e^0 = 1
    check_unary("exp", "[+↓]", vp, vp.exp(), &[Normal]); // ≈ 1
    check_unary("exp", "[-↓]", vn, vn.exp(), &[Normal]); // ≈ 1
    check_unary("exp", "[+#]", np, np.exp(), &[Normal, Exploded]);
    check_unary("exp", "[-#]", nn, nn.exp(), &[Normal, Vanished, Zero]);
    check_unary(
        "exp",
        "[+↑]",
        ep,
        ep.exp(),
        &[Exploded, Infinity, Undefined],
    );
    check_unary("exp", "[-↑]", en, en.exp(), &[Zero]); // e^-∞ = 0
    check_unary("exp", "[∞]", inf, inf.exp(), &[Infinity]);
    check_unary("exp", "[℘]", und, und.exp(), &[Undefined]);
}

#[test]
fn not_unary_truth_table() {
    // Bitwise NOT: 1:1 class except [0] ↔ [∞] swap. Sign flips within preserved classes.
    let z = S::ZERO;
    let inf = S::INFINITY;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(2);
    let nn = S::from(-2);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let und = S::ZERO / S::ZERO;

    // ZERO ↔ INFINITY swap
    let not_z = (!z).into();
    let not_inf = (!inf).into();
    assert_eq!(classify(&not_z), Infinity, "NOT([0]) should be [∞]");
    assert_eq!(classify(&not_inf), Zero, "NOT([∞]) should be [0]");

    // Class preserved, sign flipped
    let cases: &[(&str, S, Class)] = &[
        ("[+↓]", vp, Vanished),
        ("[-↓]", vn, Vanished),
        ("[+#]", np, Normal),
        ("[-#]", nn, Normal),
        ("[+↑]", ep, Exploded),
        ("[-↑]", en, Exploded),
    ];
    for (name, x, expected_class) in cases {
        let r: S = (!*x).into();
        assert_eq!(
            classify(&r),
            *expected_class,
            "NOT({}) class should be {:?}",
            name,
            expected_class
        );
        assert_ne!(
            x.is_negative(),
            r.is_negative(),
            "NOT({}) should flip sign",
            name
        );
    }

    // Undefined preserved
    assert_eq!(classify(&!und), Undefined);
}

// ============================================================ Algebraic unary truth tables: neg, abs, sign, recip, floor, ceil, round, frac ============================================================

/// Common representatives for the unary class tables. `np`/`nn` are non-boundary normals so negation/abs don't hit the exponent-edge escape cases (those are their own tests).
fn unary_reps() -> (S, S, S, S, S, S, S, S, S) {
    (
        S::ZERO,
        S::VANISHED_POS,
        S::VANISHED_NEG,
        S::from(2),
        S::from(-2),
        S::EXPLODED_POS,
        S::EXPLODED_NEG,
        S::INFINITY,
        S::ZERO / S::ZERO,
    )
}

#[test]
fn neg_unary_truth_table() {
    let (z, vp, vn, np, nn, ep, en, inf, und) = unary_reps();
    check_unary("neg", "[0]", z, -z, &[Zero]);
    check_unary("neg", "[+↓]", vp, -vp, &[Vanished]);
    check_unary("neg", "[-↓]", vn, -vn, &[Vanished]);
    check_unary("neg", "[+#]", np, -np, &[Normal]);
    check_unary("neg", "[-#]", nn, -nn, &[Normal]);
    check_unary("neg", "[+↑]", ep, -ep, &[Exploded]);
    check_unary("neg", "[-↑]", en, -en, &[Exploded]);
    check_unary("neg", "[∞]", inf, -inf, &[Infinity]);
    check_unary("neg", "[℘]", und, -und, &[Undefined]);
}

#[test]
fn abs_unary_truth_table() {
    let (z, vp, vn, np, nn, ep, en, inf, und) = unary_reps();
    check_unary("abs", "[0]", z, z.magnitude(), &[Zero]);
    check_unary("abs", "[+↓]", vp, vp.magnitude(), &[Vanished]);
    check_unary("abs", "[-↓]", vn, vn.magnitude(), &[Vanished]);
    check_unary("abs", "[+#]", np, np.magnitude(), &[Normal]);
    check_unary("abs", "[-#]", nn, nn.magnitude(), &[Normal]);
    check_unary("abs", "[+↑]", ep, ep.magnitude(), &[Exploded]);
    check_unary("abs", "[-↑]", en, en.magnitude(), &[Exploded]);
    check_unary("abs", "[∞]", inf, inf.magnitude(), &[Infinity]);
    check_unary("abs", "[℘]", und, und.magnitude(), &[Undefined]);
    // abs must be non-negative for every signed input.
    for x in [vn, nn, en] {
        assert!(!x.magnitude().is_negative(), "abs should be non-negative");
    }
}

#[test]
fn sign_unary_truth_table() {
    // sign(0) and sign(∞) are directionless → undefined (℘±∅); everything with a definite orientation (vanished/normal/exploded) yields ±1 (Normal).
    let (z, vp, vn, np, nn, ep, en, inf, und) = unary_reps();
    check_unary("sign", "[0]", z, z.sign(), &[Undefined]);
    check_unary("sign", "[+↓]", vp, vp.sign(), &[Normal]);
    check_unary("sign", "[-↓]", vn, vn.sign(), &[Normal]);
    check_unary("sign", "[+#]", np, np.sign(), &[Normal]);
    check_unary("sign", "[-#]", nn, nn.sign(), &[Normal]);
    check_unary("sign", "[+↑]", ep, ep.sign(), &[Normal]);
    check_unary("sign", "[-↑]", en, en.sign(), &[Normal]);
    check_unary("sign", "[∞]", inf, inf.sign(), &[Undefined]);
    check_unary("sign", "[℘]", und, und.sign(), &[Undefined]);
}

#[test]
fn recip_unary_truth_table() {
    // 1/x inverts the magnitude class: 0↔∞, vanished↔exploded, normal↔normal.
    let (z, vp, vn, np, nn, ep, en, inf, und) = unary_reps();
    check_unary("recip", "[0]", z, z.reciprocal(), &[Infinity]);
    check_unary("recip", "[+↓]", vp, vp.reciprocal(), &[Exploded]);
    check_unary("recip", "[-↓]", vn, vn.reciprocal(), &[Exploded]);
    check_unary("recip", "[+#]", np, np.reciprocal(), &[Normal]); // 1/2
    check_unary("recip", "[-#]", nn, nn.reciprocal(), &[Normal]);
    check_unary("recip", "[+↑]", ep, ep.reciprocal(), &[Vanished]);
    check_unary("recip", "[-↑]", en, en.reciprocal(), &[Vanished]);
    check_unary("recip", "[∞]", inf, inf.reciprocal(), &[Zero]);
    check_unary("recip", "[℘]", und, und.reciprocal(), &[Undefined]);
}

#[test]
fn floor_ceil_round_unary_truth_tables() {
    let (z, _vp, _vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let half = S::from(1) / S::from(2); // 0.5
    let neg_half = S::from(-1) / S::from(2); // -0.5
    let two = S::from(2);

    // floor: 0.5→0, -0.5→-1, integer 2→2. Escaped/∞/℘ pass thru class.
    check_unary("floor", "0.5", half, half.floor(), &[Zero]);
    check_unary("floor", "-0.5", neg_half, neg_half.floor(), &[Normal]); // -1
    check_unary("floor", "2", two, two.floor(), &[Normal]);
    check_unary("floor", "[+↑]", ep, ep.floor(), &[Exploded]);
    check_unary("floor", "[-↑]", en, en.floor(), &[Exploded]);
    check_unary("floor", "[∞]", inf, inf.floor(), &[Infinity]);
    check_unary("floor", "[℘]", und, und.floor(), &[Undefined]);

    // ceil: 0.5→1, -0.5→0.
    check_unary("ceil", "0.5", half, half.ceil(), &[Normal]); // 1
    check_unary("ceil", "-0.5", neg_half, neg_half.ceil(), &[Zero]);
    check_unary("ceil", "[+↑]", ep, ep.ceil(), &[Exploded]);
    check_unary("ceil", "[∞]", inf, inf.ceil(), &[Infinity]);
    check_unary("ceil", "[℘]", und, und.ceil(), &[Undefined]);

    // round: banker's — round(0.5)=0, round(2.5)=2 (both to even).
    check_unary("round", "0.5", half, half.round(), &[Zero]);
    let two_half = S::from(5) / S::from(2); // 2.5 → 2 (even)
    check_unary("round", "2.5", two_half, two_half.round(), &[Normal]);
    check_unary("round", "[+↑]", ep, ep.round(), &[Exploded]);
    check_unary("round", "[∞]", inf, inf.round(), &[Infinity]);
    check_unary("round", "0", z, z.round(), &[Zero]);
}

#[test]
fn frac_unary_truth_table() {
    // frac = x - floor(x) ∈ [0,1). Integer/escaped → 0; ∞ → undefined (℘⨅∞).
    let (z, vp, _vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let half = S::from(1) / S::from(2);
    let two = S::from(2);
    check_unary("frac", "0", z, z.frac(), &[Zero]);
    check_unary("frac", "2", two, two.frac(), &[Zero]); // integer → 0
    check_unary("frac", "0.5", half, half.frac(), &[Normal]);
    check_unary("frac", "[+↓]", vp, vp.frac(), &[Zero, Vanished]); // ≈0
    check_unary("frac", "[+↑]", ep, ep.frac(), &[Zero]); // huge integer → 0
    check_unary("frac", "[-↑]", en, en.frac(), &[Zero]);
    check_unary("frac", "[∞]", inf, inf.frac(), &[Undefined]); // ℘⨅∞
    check_unary("frac", "[℘]", und, und.frac(), &[Undefined]);
}

// ============================================================ Trigonometric / hyperbolic class tables (class edges only — accuracy is checked in tests/numeric_reference.rs) ============================================================

#[test]
fn sin_cos_tan_truth_tables() {
    // sin/cos/tan output bounded ranges (tan unbounded near poles), so escaped/∞ inputs lose phase and go undefined.
    // Near-zero inputs track x (sin), or → 1 (cos).
    let (z, vp, vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let half = S::from(1) / S::from(2);
    let one = S::ONE;
    let inrange = &[Zero, Vanished, Normal]; // any value in [-1, 1]

    check_unary("sin", "[0]", z, z.sin(), &[Zero]);
    check_unary("sin", "[+↓]", vp, vp.sin(), &[Vanished]);
    check_unary("sin", "[-↓]", vn, vn.sin(), &[Vanished]);
    check_unary("sin", "0.5", half, half.sin(), inrange);
    check_unary("sin", "1", one, one.sin(), inrange);
    check_unary("sin", "[+↑]", ep, ep.sin(), &[Undefined]);
    check_unary("sin", "[-↑]", en, en.sin(), &[Undefined]);
    check_unary("sin", "[∞]", inf, inf.sin(), &[Undefined]);
    check_unary("sin", "[℘]", und, und.sin(), &[Undefined]);

    check_unary("cos", "[0]", z, z.cos(), &[Normal]); // = 1
    check_unary("cos", "[+↓]", vp, vp.cos(), &[Normal]); // ≈ 1
    check_unary("cos", "0.5", half, half.cos(), inrange);
    check_unary("cos", "[+↑]", ep, ep.cos(), &[Undefined]);
    check_unary("cos", "[∞]", inf, inf.cos(), &[Undefined]);
    check_unary("cos", "[℘]", und, und.cos(), &[Undefined]);

    check_unary("tan", "[0]", z, z.tan(), &[Zero]);
    check_unary("tan", "[+↓]", vp, vp.tan(), &[Vanished]);
    check_unary(
        "tan",
        "1",
        one,
        one.tan(),
        &[Zero, Vanished, Normal, Exploded, Infinity],
    );
    check_unary("tan", "[+↑]", ep, ep.tan(), &[Undefined]);
    check_unary("tan", "[∞]", inf, inf.tan(), &[Undefined]);
    check_unary("tan", "[℘]", und, und.tan(), &[Undefined]);
}

#[test]
fn asin_acos_atan_truth_tables() {
    // asin/acos: hard domain |x| ≤ 1 (out of range → undefined).
    // atan: all reals, ±↑ → ±π/2, but ∞ is directionless → undefined.
    let (z, vp, vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let half = S::from(1) / S::from(2);
    let neg_half = S::from(-1) / S::from(2);
    let two = S::from(2); // out of asin/acos domain

    check_unary("asin", "[0]", z, z.asin(), &[Zero]);
    check_unary("asin", "[+↓]", vp, vp.asin(), &[Vanished]);
    check_unary("asin", "[-↓]", vn, vn.asin(), &[Vanished]);
    check_unary("asin", "0.5", half, half.asin(), &[Normal]);
    check_unary("asin", "-0.5", neg_half, neg_half.asin(), &[Normal]);
    check_unary("asin", "2 (>1)", two, two.asin(), &[Undefined]);
    check_unary("asin", "[+↑]", ep, ep.asin(), &[Undefined]);
    check_unary("asin", "[∞]", inf, inf.asin(), &[Undefined]);
    check_unary("asin", "[℘]", und, und.asin(), &[Undefined]);

    check_unary("acos", "[0]", z, z.acos(), &[Normal]); // π/2
    check_unary("acos", "0.5", half, half.acos(), &[Normal]);
    check_unary("acos", "2 (>1)", two, two.acos(), &[Undefined]);
    check_unary("acos", "[+↑]", ep, ep.acos(), &[Undefined]);
    check_unary("acos", "[∞]", inf, inf.acos(), &[Undefined]);
    check_unary("acos", "[℘]", und, und.acos(), &[Undefined]);

    check_unary("atan", "[0]", z, z.atan(), &[Zero]);
    check_unary("atan", "[+↓]", vp, vp.atan(), &[Vanished]);
    check_unary("atan", "0.5", half, half.atan(), &[Normal]);
    check_unary("atan", "[+↑]", ep, ep.atan(), &[Normal]); // → +π/2
    check_unary("atan", "[-↑]", en, en.atan(), &[Normal]); // → -π/2
    check_unary("atan", "[∞]", inf, inf.atan(), &[Undefined]); // direction undetermined
    check_unary("atan", "[℘]", und, und.atan(), &[Undefined]);
}

#[test]
fn sinh_cosh_tanh_truth_tables() {
    // Hyperbolics grow like exp (sinh/cosh unbounded), tanh saturates to ±1.
    let (z, vp, vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let two = S::from(2);
    let neg_two = S::from(-2);

    check_unary("sinh", "[0]", z, z.sinh(), &[Zero]);
    check_unary("sinh", "[+↓]", vp, vp.sinh(), &[Vanished]);
    check_unary("sinh", "[-↓]", vn, vn.sinh(), &[Vanished]);
    check_unary("sinh", "2", two, two.sinh(), &[Normal, Exploded]);
    check_unary("sinh", "[+↑]", ep, ep.sinh(), &[Exploded]);
    check_unary("sinh", "[-↑]", en, en.sinh(), &[Exploded]);
    check_unary("sinh", "[∞]", inf, inf.sinh(), &[Infinity]);
    check_unary("sinh", "[℘]", und, und.sinh(), &[Undefined]);

    check_unary("cosh", "[0]", z, z.cosh(), &[Normal]); // = 1
    check_unary("cosh", "[+↓]", vp, vp.cosh(), &[Normal]);
    check_unary("cosh", "2", two, two.cosh(), &[Normal, Exploded]);
    check_unary("cosh", "-2", neg_two, neg_two.cosh(), &[Normal, Exploded]);
    check_unary("cosh", "[+↑]", ep, ep.cosh(), &[Exploded]);
    check_unary("cosh", "[-↑]", en, en.cosh(), &[Exploded]);
    check_unary("cosh", "[∞]", inf, inf.cosh(), &[Infinity]);
    check_unary("cosh", "[℘]", und, und.cosh(), &[Undefined]);
    // cosh ≥ 1 always → never negative.
    assert!(!neg_two.cosh().is_negative());

    check_unary("tanh", "[0]", z, z.tanh(), &[Zero]);
    check_unary("tanh", "[+↓]", vp, vp.tanh(), &[Vanished]);
    check_unary("tanh", "2", two, two.tanh(), &[Normal]); // in (-1,1)
    check_unary("tanh", "[+↑]", ep, ep.tanh(), &[Normal]); // → +1
    check_unary("tanh", "[-↑]", en, en.tanh(), &[Normal]); // → -1
    check_unary("tanh", "[℘]", und, und.tanh(), &[Undefined]);
}

// ============================================================ Binary: log (base), pow, shifts, min/max/clamp ============================================================

#[test]
fn log_truth_table() {
    // log_b(a) = lb(a)/lb(b). Definite transfinite limits resolve; escaped/negative operands are undefined with which-operand reason tags. (Locks the scalar_logarithm_scalar fix.)
    let two = S::from(2);
    let four = S::from(4);
    let five = S::from(5);
    let z = S::ZERO;
    let inf = S::INFINITY;
    let ep = S::EXPLODED_POS;
    let und = z / z;

    check("$", four, two, four.log(two), &[Normal]); // log2(4) = 2
    check("$", S::ONE, two, S::ONE.log(two), &[Zero]); // log2(1) = 0
    check("$", inf, two, inf.log(two), &[Infinity]); // log(∞) = ∞
    check("$", z, two, z.log(two), &[Infinity]); // log(0) = ∞
    check("$", five, inf, five.log(inf), &[Zero]); // log base ∞ = 0
    check("$", five, z, five.log(z), &[Zero]); // log base 0 = 0
    check("$", five, S::ONE, five.log(S::ONE), &[Undefined]); // base 1 = ℘@1
    check("$", S::from(-4), two, S::from(-4).log(two), &[Undefined]); // neg value
    check("$", four, S::from(-2), four.log(S::from(-2)), &[Undefined]); // neg base
    check("$", ep, two, ep.log(two), &[Undefined]); // escaped value
    check("$", five, ep, five.log(ep), &[Undefined]); // escaped base
    check("$", inf, inf, inf.log(inf), &[Undefined]); // ∞/∞
    check("$", und, two, und.log(two), &[Undefined]); // ℘ propagates
    check("$", two, und, two.log(und), &[Undefined]);
}

#[test]
fn pow_truth_table() {
    let two = S::from(2);
    let three = S::from(3);
    let half = S::from(1) / S::from(2);
    let z = S::ZERO;
    let inf = S::INFINITY;
    let und = z / z;

    // Definite normals.
    check("^", two, three, two.pow(three), &[Normal, Exploded]); // 2^3 = 8
    check("^", three, two, three.pow(two), &[Normal]); // 3^2 = 9
    // x^0 = 1 for any finite x (identity).
    check("^", three, z, three.pow(z), &[Normal]);
    check("^", z, z, z.pow(z), &[Normal]); // 0^0 = 1 (Spirix convention)
    // 0^positive = 0, 0^negative = ∞ (only the exponent's sign matters for a zero base).
    check("^", z, two, z.pow(two), &[Zero]);
    check("^", z, half, z.pow(half), &[Zero]);
    check("^", z, S::from(-2), z.pow(S::from(-2)), &[Infinity]);
    // Large exponent escapes at F3E3 (i8 exponent, ceiling 2^127).
    check("^", two, S::from(200), two.pow(S::from(200)), &[Exploded]);
    check("^", two, S::from(-200), two.pow(S::from(-200)), &[Vanished]);
    // Negative base to a non-integer power → undefined (℘-^).
    check("^", S::from(-2), half, S::from(-2).pow(half), &[Undefined]);
    // Undefined propagates.
    check("^", und, two, und.pow(two), &[Undefined]);
    check("^", two, und, two.pow(und), &[Undefined]);
    // Infinite base: pure sign dominance for ANY nonzero exponent — the old stored-exponent test sent ∞^0.5 to ZERO.
    let inf = S::INFINITY;
    check("^", inf, two, inf.pow(two), &[Infinity]);
    check("^", inf, half, inf.pow(half), &[Infinity]);
    check("^", inf, S::from(-2), inf.pow(S::from(-2)), &[Zero]);
    check("^", inf, half, inf.pow(-half), &[Zero]);
    check("^", inf, z, inf.pow(z), &[Normal]); // ∞^0 = 1
    check("^", inf, inf, inf.pow(S::EXPLODED_POS), &[Infinity]);
    // Zero base with a VANISHED exponent still resolves by sign: 0^(+↓) = 0, 0^(-↓) = ∞.
    check("^", z, z, z.pow(S::VANISHED_POS), &[Zero]);
    check("^", z, z, z.pow(S::VANISHED_NEG), &[Infinity]);
}

#[test]
fn pow_escaped_base_truth_table() {
    // Escaped base m·2^E: integer exponents resolve fully (class + parity sign + phase, via the multiply chain); non-integer |p| > 1 resolves class only (canonical positive escaped); non-integer |p| < 1 is class-indeterminate (tiny^0.01 can re-enter normal range) → ℘.
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let one = S::ONE;
    let two = S::from(2);
    let three = S::from(3);
    let half = S::from(1) / S::from(2);
    let five_halves = S::from(5) / S::from(2);
    let inf = S::INFINITY;

    // x^1 = x exactly — identity holds for escaped values (phase included).
    let r = vp.pow(one);
    assert_eq!(
        (r.fraction, r.exponent),
        (vp.fraction, vp.exponent),
        "↓^1 must be the identity"
    );

    // Integer exponents: class + parity sign.
    check("^", vp, two, vp.pow(two), &[Vanished]); // ↓² = ↓
    check("^", vn, two, vn.pow(two), &[Vanished]);
    assert!(!vn.pow(two).is_negative(), "(-↓)² is positive");
    check("^", vn, three, vn.pow(three), &[Vanished]);
    assert!(vn.pow(three).is_negative(), "(-↓)³ is negative");
    check("^", vp, S::from(-2), vp.pow(S::from(-2)), &[Exploded]); // ↓⁻² = ↑
    check("^", ep, two, ep.pow(two), &[Exploded]); // ↑² = ↑
    check("^", en, three, en.pow(three), &[Exploded]);
    assert!(en.pow(three).is_negative(), "(-↑)³ is negative");
    check("^", ep, S::from(-2), ep.pow(S::from(-2)), &[Vanished]); // ↑⁻² = ↓

    // Non-integer |p| > 1: class resolves, canonical positive (phase honestly lost).
    check("^", vp, five_halves, vp.pow(five_halves), &[Vanished]);
    check("^", ep, five_halves, ep.pow(five_halves), &[Exploded]);
    check("^", vp, -five_halves, vp.pow(-five_halves), &[Exploded]);
    check("^", ep, -five_halves, ep.pow(-five_halves), &[Vanished]);
    // Negative escaped base, non-integer p → ℘-^ like normals.
    check("^", vn, five_halves, vn.pow(five_halves), &[Undefined]);
    check("^", en, half, en.pow(half), &[Undefined]);

    // Non-integer |p| < 1: class indeterminate → ℘.
    check("^", vp, half, vp.pow(half), &[Undefined]);
    check("^", ep, half, ep.pow(half), &[Undefined]);

    // Escaped exponent: positive base resolves by dominance; negative base can't commit parity.
    check("^", vp, ep, vp.pow(ep), &[Vanished]); // tiny^huge = tinier
    check("^", vp, en, vp.pow(en), &[Exploded]); // tiny^-huge = huge
    check("^", ep, ep, ep.pow(ep), &[Exploded]);
    check("^", ep, en, ep.pow(en), &[Vanished]);
    check("^", vn, ep, vn.pow(ep), &[Undefined]); // parity of ↑ unknown
    // Vanished / infinite exponent stays undefined (0·∞ form / directionless ∞).
    check("^", vp, vp, vp.pow(vp), &[Undefined]);
    check("^", ep, inf, ep.pow(inf), &[Undefined]);
    // x^0 = 1 still holds for escaped bases.
    check("^", vp, S::ZERO, vp.pow(S::ZERO), &[Normal]);
    check("^", ep, S::ZERO, ep.pow(S::ZERO), &[Normal]);
}

#[test]
fn shift_truth_table() {
    // `<<` = ×2ⁿ, `>>` = ÷2ⁿ by an integer amount.
    // NOTE: the shift amount is E-typed, so at F3E3 (E = i8) it saturates at ±127 — a shift of 200 becomes 127, which is a width limit, not an escape. Escapes here use amounts within i8 range that still cross the boundary.
    let two = S::from(2);
    let z = S::ZERO;
    let inf = S::INFINITY;
    check("<<", two, S::ZERO, two << 0, &[Normal]); // ×1
    check("<<", two, S::ZERO, two << 1, &[Normal]); // = 4
    check("<<", two, S::ZERO, two << 127, &[Exploded]); // 2^128 over the ceiling → escapes
    check(">>", two, S::ZERO, two >> 1, &[Normal]); // = 1
    // A right shift escapes to vanished symmetrically once it crosses the floor; a small value reaches it within i8's shift range where `2` (exp +1) cannot.
    let tiny_normal = S::ONE >> 120; // 2^-120, still normal
    check(">>", tiny_normal, S::ZERO, tiny_normal >> 20, &[Vanished]);
    // Zero and infinity are fixed points of scaling.
    check("<<", z, S::ZERO, z << 5, &[Zero]);
    check("<<", inf, S::ZERO, inf << 5, &[Infinity]);
}

#[test]
fn min_max_clamp_truth_table() {
    let two = S::from(2);
    let five = S::from(5);
    let neg_three = S::from(-3);
    let inf = S::INFINITY;
    let z = S::ZERO;
    let und = z / z;

    // Ordered normals.
    check("min", two, five, two.min(five), &[Normal]); // → 2
    check("max", two, five, two.max(five), &[Normal]); // → 5
    // Against infinity: Spirix's ∞ is the UNSIGNED point-at-infinity, so it is not ordered relative to a finite value — min/max return undefined (℘⌊ / ℘⌈), not the finite operand.
    // This is a design choice (not a bug): a signless ∞ sits at "both ends" of the line.
    check("min", two, inf, two.min(inf), &[Undefined]);
    check("max", two, inf, two.max(inf), &[Undefined]);
    // Undefined operand → undefined (non-orderable).
    check("min", two, und, two.min(und), &[Undefined]);
    check("max", two, und, two.max(und), &[Undefined]);

    // clamp(x, -3, 5): below → -3, inside → x, above → 5.
    assert_eq!(
        classify(&neg_three.clamp(z, five)),
        Zero,
        "clamp(-3, [0,5]) → 0"
    );
    assert_eq!(classify(&two.clamp(neg_three, five)), Normal, "clamp inside");
    // clamp against ∞ inherits the same unsigned-∞ non-ordering as min/max → undefined (℘∩).
    assert_eq!(
        classify(&inf.clamp(neg_three, five)),
        Undefined,
        "clamp(∞, [-3,5]) is non-orderable → ℘∩"
    );
}

#[test]
fn powb_unary_truth_table() {
    // 2^x — same class table as exp (different base doesn't change it).
    let (z, vp, vn, _np, _nn, ep, en, inf, und) = unary_reps();
    let two = S::from(2);
    let neg_two = S::from(-2);
    check_unary("powb", "[0]", z, z.powb(), &[Normal]); // = 1
    check_unary("powb", "[+↓]", vp, vp.powb(), &[Normal]);
    check_unary("powb", "[-↓]", vn, vn.powb(), &[Normal]);
    check_unary("powb", "2", two, two.powb(), &[Normal]); // = 4
    check_unary("powb", "-2", neg_two, neg_two.powb(), &[Normal]); // = 0.25
    check_unary("powb", "[+↑]", ep, ep.powb(), &[Exploded, Infinity, Undefined]);
    check_unary("powb", "[-↑]", en, en.powb(), &[Zero]); // 2^-∞ = 0
    check_unary("powb", "[∞]", inf, inf.powb(), &[Infinity]);
    check_unary("powb", "[℘]", und, und.powb(), &[Undefined]);
}

#[test]
fn square_unary_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(2);
    let nn = S::from(-2);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("square", "[0]", z, z.square(), &[Zero]);
    check_unary("square", "[+↓]", vp, vp.square(), &[Vanished, Zero]);
    check_unary("square", "[-↓]", vn, vn.square(), &[Vanished, Zero]);
    check_unary(
        "square",
        "[+#]",
        np,
        np.square(),
        &[Normal, Vanished, Exploded],
    );
    check_unary(
        "square",
        "[-#]",
        nn,
        nn.square(),
        &[Normal, Vanished, Exploded],
    );
    check_unary("square", "[+↑]", ep, ep.square(), &[Exploded]);
    check_unary("square", "[-↑]", en, en.square(), &[Exploded]);
    check_unary("square", "[∞]", inf, inf.square(), &[Infinity]);
    check_unary("square", "[℘]", und, und.square(), &[Undefined]);

    // All squares are non-negative (can't be < 0)
    for &x in &[np, nn, ep, en] {
        let sq = x.square();
        if sq.is_normal() {
            assert!(
                !sq.is_negative(),
                "square({:?}) should be non-negative, got negative",
                x
            );
        }
    }
}

// ============================================================ Specific edge case: subtraction with MIN producing exploded ============================================================
#[test]
fn subtraction_min_boundary_exploded() {
    // NEG_ONE at MIN_EXPONENT: negating requires exp+1 which wraps → exploded
    let min_neg = S::MAX_NEG; // smallest magnitude negative normal
    let result = S::ZERO - min_neg;
    // 0 - (tiny negative) = tiny positive, could be normal or edge-case But the interesting case: negate MIN at exp boundary
    let at_boundary = S::ONE; // fraction = POS_ONE_NORMAL (MIN stored), exp=1
    let neg_boundary = S::NEG_ONE; // fraction = NEG_ONE_NORMAL (0 stored), exp=0
                                   // These should negate cleanly
    let r1 = S::ZERO - at_boundary;
    assert!(r1.is_negative(), "0-1 should be negative");
    let r2 = S::ZERO - neg_boundary;
    assert!(r2.is_positive(), "0-(-1) should be positive");
}

// ============================================================ Bitwise AND truth table (col & row) ============================================================
#[test]
fn and_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0] is absorber: [0] & X = [0]
    check("&", z, z, z & z, &[Zero]);
    check("&", z, vp, z & vp, &[Zero]);
    check("&", z, vn, z & vn, &[Zero]);
    check("&", z, np, z & np, &[Zero]);
    check("&", z, nn, z & nn, &[Zero]);
    check("&", z, ep, z & ep, &[Zero]);
    check("&", z, en, z & en, &[Zero]);
    check("&", z, inf, z & inf, &[Zero]);
    check("&", vp, z, vp & z, &[Zero]);
    check("&", np, z, np & z, &[Zero]);
    check("&", ep, z, ep & z, &[Zero]);
    check("&", inf, z, inf & z, &[Zero]);

    // [∞] is identity: [∞] & X = X
    check("&", inf, vp, inf & vp, &[Vanished]);
    check("&", inf, np, inf & np, &[Normal]);
    check("&", inf, ep, inf & ep, &[Exploded]);
    check("&", inf, inf, inf & inf, &[Infinity]);
    check("&", vp, inf, vp & inf, &[Vanished]);
    check("&", np, inf, np & inf, &[Normal]);
    check("&", ep, inf, ep & inf, &[Exploded]);

    // Escape & escape (same rank or cross-rank with shared ambig frame). [↓] & [↓] → [℘&]
    check("&", vp, vp, vp & vp, &[Undefined]);
    check("&", vp, vn, vp & vn, &[Undefined]);
    // [↑] & [↑] → [℘&]
    check("&", ep, ep, ep & ep, &[Undefined]);
    check("&", ep, en, ep & en, &[Undefined]);
    // [↓] & [↑] → [0] or [↑]
    check("&", vp, ep, vp & ep, &[Zero, Exploded]);
    check("&", vp, en, vp & en, &[Zero, Exploded]);
    check("&", vn, ep, vn & ep, &[Zero, Exploded]);
    check("&", ep, vp, ep & vp, &[Zero, Exploded]);

    // Escape & normal → [℘&] (ambig exp can't align with a real one)
    check("&", vp, np, vp & np, &[Undefined]);
    check("&", vn, np, vn & np, &[Undefined]);
    check("&", np, vp, np & vp, &[Undefined]);
    check("&", ep, np, ep & np, &[Undefined]);
    check("&", en, np, en & np, &[Undefined]);
    check("&", np, ep, np & ep, &[Undefined]);

    // [#] & [#] can land in [#], [0], or [↓]
    check("&", np, np, np & np, &[Normal, Zero, Vanished]);
    check("&", np, nn, np & nn, &[Normal, Zero, Vanished]);
    check("&", nn, nn, nn & nn, &[Normal, Zero, Vanished]);

    // [℘?] propagates
    check("&", und, z, und & z, &[Undefined]);
    check("&", und, vp, und & vp, &[Undefined]);
    check("&", und, np, und & np, &[Undefined]);
    check("&", und, ep, und & ep, &[Undefined]);
    check("&", und, inf, und & inf, &[Undefined]);
    check("&", und, und, und & und, &[Undefined]);
    check("&", z, und, z & und, &[Undefined]);
    check("&", np, und, np & und, &[Undefined]);
    check("&", inf, und, inf & und, &[Undefined]);
}

// ============================================================ Bitwise OR truth table (col | row) ============================================================
#[test]
fn or_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0] is identity: [0] | X = X
    check("|", z, z, z | z, &[Zero]);
    check("|", z, vp, z | vp, &[Vanished]);
    check("|", z, np, z | np, &[Normal]);
    check("|", z, ep, z | ep, &[Exploded]);
    check("|", z, inf, z | inf, &[Infinity]);
    check("|", vp, z, vp | z, &[Vanished]);
    check("|", np, z, np | z, &[Normal]);
    check("|", ep, z, ep | z, &[Exploded]);
    check("|", inf, z, inf | z, &[Infinity]);

    // [∞] is absorber: [∞] | X = [∞]
    check("|", inf, vp, inf | vp, &[Infinity]);
    check("|", inf, np, inf | np, &[Infinity]);
    check("|", inf, ep, inf | ep, &[Infinity]);
    check("|", inf, inf, inf | inf, &[Infinity]);
    check("|", vp, inf, vp | inf, &[Infinity]);
    check("|", np, inf, np | inf, &[Infinity]);
    check("|", ep, inf, ep | inf, &[Infinity]);

    // Escape | escape (same-rank → [℘|], cross-rank → [↓] or [↑])
    check("|", vp, vp, vp | vp, &[Undefined]);
    check("|", vp, vn, vp | vn, &[Undefined]);
    check("|", ep, ep, ep | ep, &[Undefined]);
    check("|", ep, en, ep | en, &[Undefined]);
    // [↓] | [↑] → [↓] or [↑]
    check("|", vp, ep, vp | ep, &[Vanished, Exploded]);
    check("|", vp, en, vp | en, &[Vanished, Exploded]);
    check("|", vn, ep, vn | ep, &[Vanished, Exploded]);
    check("|", ep, vp, ep | vp, &[Vanished, Exploded]);

    // Escape | normal → [℘|]
    check("|", vp, np, vp | np, &[Undefined]);
    check("|", np, vp, np | vp, &[Undefined]);
    check("|", ep, np, ep | np, &[Undefined]);
    check("|", np, ep, np | ep, &[Undefined]);

    // [#] | [#] can land in [#] or [↓]
    check("|", np, np, np | np, &[Normal, Vanished]);
    check("|", np, nn, np | nn, &[Normal, Vanished]);
    check("|", nn, nn, nn | nn, &[Normal, Vanished]);

    // [℘?] propagates (except [∞] absorbs ∞ row — but [℘] vs [∞] still [℘])
    check("|", und, z, und | z, &[Undefined]);
    check("|", und, vp, und | vp, &[Undefined]);
    check("|", und, np, und | np, &[Undefined]);
    check("|", und, ep, und | ep, &[Undefined]);
    check("|", und, und, und | und, &[Undefined]);
    check("|", z, und, z | und, &[Undefined]);
    check("|", np, und, np | und, &[Undefined]);
}

// ============================================================ Bitwise XOR truth table (col ^ row) ============================================================
#[test]
fn xor_truth_table() {
    let z = S::ZERO;
    let vp = S::VANISHED_POS;
    let vn = S::VANISHED_NEG;
    let np = S::from(7);
    let nn = S::from(-7);
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    // [0] is identity: [0] ⊻ X = X
    check("^", z, z, z ^ z, &[Zero]);
    check("^", z, vp, z ^ vp, &[Vanished]);
    check("^", z, np, z ^ np, &[Normal]);
    check("^", z, ep, z ^ ep, &[Exploded]);
    check("^", z, inf, z ^ inf, &[Infinity]);

    // [∞] inverts: [∞] ⊻ X = ~X (NOT table)
    check("^", inf, z, inf ^ z, &[Infinity]);
    check("^", inf, vp, inf ^ vp, &[Vanished]);
    check("^", inf, np, inf ^ np, &[Normal]);
    check("^", inf, ep, inf ^ ep, &[Exploded]);
    check("^", inf, inf, inf ^ inf, &[Zero]); // self-XOR cancels

    // Self-XOR cancels to [0] only when magnitudes are deterministic (normals, infinity). For escapes the magnitudes are ambiguous even if bit-identical, so the table reports [℘⊻].
    check("^", np, np, np ^ np, &[Zero]);
    // Same-class escape pairs (incl. bit-identical) → [℘⊻].
    check("^", vp, vp, vp ^ vp, &[Undefined]);
    check("^", vp, vn, vp ^ vn, &[Undefined]);
    check("^", ep, ep, ep ^ ep, &[Undefined]);
    check("^", ep, en, ep ^ en, &[Undefined]);

    // [↓] ⊻ [↑] → [↑] (opposite-rank bit patterns XOR to N-1)
    check("^", vp, ep, vp ^ ep, &[Exploded]);
    check("^", vp, en, vp ^ en, &[Exploded]);
    check("^", vn, ep, vn ^ ep, &[Exploded]);
    check("^", ep, vp, ep ^ vp, &[Exploded]);

    // Escape ⊻ normal → [℘⊻]
    check("^", vp, np, vp ^ np, &[Undefined]);
    check("^", np, vp, np ^ vp, &[Undefined]);
    check("^", ep, np, ep ^ np, &[Undefined]);
    check("^", np, ep, np ^ ep, &[Undefined]);

    // [#] ⊻ [#] can land in [0], [↓], or [#]
    check("^", np, nn, np ^ nn, &[Zero, Vanished, Normal]);

    // [℘?] propagates
    check("^", und, z, und ^ z, &[Undefined]);
    check("^", und, vp, und ^ vp, &[Undefined]);
    check("^", und, np, und ^ np, &[Undefined]);
    check("^", und, ep, und ^ ep, &[Undefined]);
    check("^", und, inf, und ^ inf, &[Undefined]);
    check("^", und, und, und ^ und, &[Undefined]);
    check("^", z, und, z ^ und, &[Undefined]);
    check("^", np, und, np ^ und, &[Undefined]);
    check("^", inf, und, inf ^ und, &[Undefined]);
}

// ============================================================ Circle arithmetic class-table tests ============================================================
// Circle ops should follow the same class-level truth tables as Scalar (the Spirix README defines them at the class level, not Scalar-specific). Here we spot-check the cases where a bug was likely or actually found, especially around INFINITY absorbing (it has to beat is_zero / is_normal etc.).

type C = CircleF3E3;

fn classify_c(c: &C) -> Class {
    if c.is_undefined() {
        Class::Undefined
    } else if c.is_zero() {
        Class::Zero
    } else if c.is_infinite() {
        Class::Infinity
    } else if c.exploded() {
        Class::Exploded
    } else if c.vanished() {
        Class::Vanished
    } else {
        Class::Normal
    }
}

fn check_c(op: &str, a: C, b: C, result: C, expected: &[Class]) {
    let rc = classify_c(&result);
    if !expected.contains(&rc) {
        panic!(
            "Circle {} {} {} = {} (class {}), expected one of {:?}\n  a = {:?}, b = {:?}, result = {:?}",
            class_name(classify_c(&a)),
            op,
            class_name(classify_c(&b)),
            class_name(rc),
            rc as u8,
            expected.iter().map(|c| class_name(*c)).collect::<Vec<_>>(),
            a,
            b,
            result
        );
    }
}

#[test]
fn circle_multiplication_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // INFINITY absorbs (the regression that motivated this test):
    check_c("×", inf, np, inf * np, &[Infinity]);
    check_c("×", np, inf, np * inf, &[Infinity]);
    check_c("×", inf, ep, inf * ep, &[Infinity]);
    check_c("×", ep, inf, ep * inf, &[Infinity]);
    check_c("×", inf, vp, inf * vp, &[Infinity]);
    check_c("×", vp, inf, vp * inf, &[Infinity]);
    check_c("×", inf, inf, inf * inf, &[Infinity]);

    // INFINITY × 0 = undefined (math indeterminate)
    check_c("×", inf, z, inf * z, &[Undefined]);
    check_c("×", z, inf, z * inf, &[Undefined]);

    // Undefined dominates infinity
    check_c("×", und, inf, und * inf, &[Undefined]);
    check_c("×", inf, und, inf * und, &[Undefined]);

    // Standard cases
    check_c("×", z, z, z * z, &[Zero]);
    check_c("×", z, np, z * np, &[Zero]);
    check_c("×", np, ep, np * ep, &[Exploded]);
    check_c("×", ep, ep, ep * ep, &[Exploded]);
    check_c("×", vp, vp, vp * vp, &[Vanished]);
    check_c("×", vp, np, vp * np, &[Vanished]);
    check_c("×", vp, ep, vp * ep, &[Undefined]);
    check_c("×", ep, vp, ep * vp, &[Undefined]);
}

#[test]
fn circle_addition_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // INFINITY absorbs in addition
    check_c("+", inf, z, inf + z, &[Infinity]);
    check_c("+", inf, np, inf + np, &[Infinity]);
    check_c("+", inf, ep, inf + ep, &[Infinity]);
    check_c("+", inf, vp, inf + vp, &[Infinity]);
    check_c("+", inf, inf, inf + inf, &[Infinity]);
    check_c("+", z, inf, z + inf, &[Infinity]);
    check_c("+", ep, inf, ep + inf, &[Infinity]);

    // Undefined propagates (beats infinity)
    check_c("+", und, inf, und + inf, &[Undefined]);
    check_c("+", inf, und, inf + und, &[Undefined]);

    // Exploded + finite is undefined (could cancel back into normal range)
    check_c("+", ep, np, ep + np, &[Undefined]);
    check_c("+", np, ep, np + ep, &[Undefined]);
    check_c("+", ep, ep, ep + ep, &[Undefined]);

    // Standard
    check_c("+", z, z, z + z, &[Zero]);
    check_c("+", z, np, z + np, &[Normal]);
    check_c("+", ep, z, ep + z, &[Exploded]);
    check_c("+", ep, vp, ep + vp, &[Exploded]);
}

#[test]
fn circle_division_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // INFINITY in numerator absorbs (except /0 and /undef)
    check_c("/", inf, np, inf / np, &[Infinity]);
    check_c("/", inf, ep, inf / ep, &[Infinity]);
    check_c("/", inf, vp, inf / vp, &[Infinity]);
    check_c("/", inf, z, inf / z, &[Infinity]);
    check_c("/", inf, inf, inf / inf, &[Undefined]);

    // INFINITY in denominator → zero
    check_c("/", np, inf, np / inf, &[Zero]);
    check_c("/", ep, inf, ep / inf, &[Zero]);
    check_c("/", vp, inf, vp / inf, &[Zero]);
    check_c("/", z, inf, z / inf, &[Zero]);

    // Division by zero
    check_c("/", np, z, np / z, &[Infinity]);
    check_c("/", ep, z, ep / z, &[Infinity]);
    check_c("/", vp, z, vp / z, &[Infinity]);
    check_c("/", z, z, z / z, &[Undefined]);

    // Same-class escape divisions are undefined
    check_c("/", ep, ep, ep / ep, &[Undefined]);
    check_c("/", vp, vp, vp / vp, &[Undefined]);
}

#[test]
fn circle_modulus_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // Zero anywhere → zero (matches Scalar rule 2)
    check_c("%", z, z, z % z, &[Zero]);
    check_c("%", z, np, z % np, &[Zero]);
    check_c("%", np, z, np % z, &[Zero]);
    check_c("%", ep, z, ep % z, &[Zero]);

    // Undefined propagates (rule 1)
    check_c("%", und, np, und % np, &[Undefined]);
    check_c("%", np, und, np % und, &[Undefined]);

    // Transfinite numerator → undefined (rule 3); the earlier escape block returned numerator-unchanged for [#]%[∞] and [↓]%[∞].
    check_c("%", inf, np, inf % np, &[Undefined]);
    check_c("%", inf, ep, inf % ep, &[Undefined]);
    check_c("%", inf, inf, inf % inf, &[Undefined]);
    check_c("%", ep, np, ep % np, &[Undefined]);
    check_c("%", ep, vp, ep % vp, &[Undefined]);
    check_c("%", ep, ep, ep % ep, &[Undefined]);

    // Infinite period → undefined (rule 5) — the regression that motivated this test.
    check_c("%", np, inf, np % inf, &[Undefined]);
    check_c("%", vp, inf, vp % inf, &[Undefined]);

    // Vanished period → undefined (rule 4)
    check_c("%", np, vp, np % vp, &[Undefined]);
    check_c("%", vp, vp, vp % vp, &[Undefined]);

    // Same-sign positive exploded period → numerator (rule 6 same-sign)
    check_c("%", np, ep, np % ep, &[Normal]);
}

#[test]
fn circle_and_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // [0] absorbs.
    check_c("&", z, z, z & z, &[Zero]);
    check_c("&", z, np, z & np, &[Zero]);
    check_c("&", z, inf, z & inf, &[Zero]);
    check_c("&", inf, z, inf & z, &[Zero]);

    // [∞] is identity — the regression: earlier block lumped infinity with same-class escape, returning [℘&].
    check_c("&", inf, np, inf & np, &[Normal]);
    check_c("&", inf, vp, inf & vp, &[Vanished]);
    check_c("&", inf, ep, inf & ep, &[Exploded]);
    check_c("&", inf, inf, inf & inf, &[Infinity]);
    check_c("&", np, inf, np & inf, &[Normal]);

    // Escape ↔ normal (rather than [0]) — earlier escape block fell thru to per-component AND returning [0].
    check_c("&", vp, np, vp & np, &[Undefined]);
    check_c("&", np, vp, np & vp, &[Undefined]);
    check_c("&", ep, np, ep & np, &[Undefined]);
    check_c("&", np, ep, np & ep, &[Undefined]);

    // Same-class escape → [℘&]
    check_c("&", vp, vp, vp & vp, &[Undefined]);
    check_c("&", ep, ep, ep & ep, &[Undefined]);

    // Undefined propagates
    check_c("&", und, np, und & np, &[Undefined]);
    check_c("&", inf, und, inf & und, &[Undefined]);
}

#[test]
fn circle_or_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // [0] is identity
    check_c("|", z, np, z | np, &[Normal]);
    check_c("|", np, z, np | z, &[Normal]);
    check_c("|", z, inf, z | inf, &[Infinity]);

    // [∞] is absorber — the regression.
    check_c("|", inf, np, inf | np, &[Infinity]);
    check_c("|", inf, vp, inf | vp, &[Infinity]);
    check_c("|", inf, ep, inf | ep, &[Infinity]);
    check_c("|", inf, inf, inf | inf, &[Infinity]);
    check_c("|", np, inf, np | inf, &[Infinity]);

    // Escape | normal → [℘|]
    check_c("|", vp, np, vp | np, &[Undefined]);
    check_c("|", ep, np, ep | np, &[Undefined]);
    check_c("|", np, vp, np | vp, &[Undefined]);
    check_c("|", np, ep, np | ep, &[Undefined]);

    // Same-class escape → [℘|]
    check_c("|", vp, vp, vp | vp, &[Undefined]);
    check_c("|", ep, ep, ep | ep, &[Undefined]);

    // Undefined propagates
    check_c("|", und, np, und | np, &[Undefined]);
}

#[test]
fn circle_xor_truth_table() {
    let z = C::ZERO;
    let vp = C::MIN_POS / 4u16;
    let np = C::from((3.0f32, 4.0));
    let ep = C::MAX * C::from((2.0f32, 0.0));
    let inf = C::INFINITY;
    let und = z / z;

    // [0] is identity
    check_c("^", z, np, z ^ np, &[Normal]);
    check_c("^", np, z, np ^ z, &[Normal]);
    check_c("^", z, ep, z ^ ep, &[Exploded]);
    check_c("^", z, inf, z ^ inf, &[Infinity]);

    // [∞] inverts: ∞ ⊻ X = ~X.
    check_c("^", inf, np, inf ^ np, &[Normal]);
    check_c("^", inf, vp, inf ^ vp, &[Vanished]);
    check_c("^", inf, ep, inf ^ ep, &[Exploded]);
    check_c("^", inf, inf, inf ^ inf, &[Zero]);

    // Escape ⊻ normal → [℘⊻]
    check_c("^", vp, np, vp ^ np, &[Undefined]);
    check_c("^", ep, np, ep ^ np, &[Undefined]);
    check_c("^", np, vp, np ^ vp, &[Undefined]);
    check_c("^", np, ep, np ^ ep, &[Undefined]);

    // Same-class escape pairs → [℘⊻] (even bit-identical: escape magnitudes ambiguous)
    check_c("^", vp, vp, vp ^ vp, &[Undefined]);
    check_c("^", ep, ep, ep ^ ep, &[Undefined]);

    // Undefined propagates
    check_c("^", und, np, und ^ np, &[Undefined]);
}

// ============================================================ Circle: pow, exp, ln, sqrt class tables + escaped-orientation preservation (F4E3 for angle precision) ============================================================

type C4 = CircleF4E3;
type S4 = ScalarF4E3;

fn classify_c4(c: &C4) -> Class {
    if c.is_undefined() {
        Class::Undefined
    } else if c.is_zero() {
        Class::Zero
    } else if c.is_infinite() {
        Class::Infinity
    } else if c.exploded() {
        Class::Exploded
    } else if c.vanished() {
        Class::Vanished
    } else {
        Class::Normal
    }
}

fn check_c4(what: &str, result: C4, expected: &[Class]) {
    let rc = classify_c4(&result);
    assert!(expected.contains(&rc), "{what}: got {rc:?}, expected {expected:?} (result {result:?})");
}

/// Angle (degrees) read straight from a Circle's fraction pair — for escaped values this IS the stored orientation.
fn angle_deg(c: &C4) -> f64 {
    (c.imaginary as f64).atan2(c.real as f64).to_degrees()
}

/// Assert an angle within half a degree (16-bit fractions resolve ~0.002°, so 0.5° allows for the atan2→rotate→cos/sin round trip).
fn assert_angle(what: &str, c: &C4, want: f64) {
    let got = angle_deg(c);
    let mut d = (got - want).abs() % 360.0;
    if d > 180.0 {
        d = 360.0 - d;
    }
    assert!(d < 0.5, "{what}: angle {got:.2}° != {want:.2}° (Δ {d:.2}°)");
}

/// Escaped 3-4-5 representatives: exploded and vanished Circles at +53.13°, built by walking a normal across the range boundary so the stored orientation is genuine.
fn escaped_circles() -> (C4, C4) {
    let base = C4::from((3, 4));
    let mut e = base;
    for _ in 0..600 {
        if e.exploded() {
            break;
        }
        e = e + e;
    }
    let mut t = base;
    let half = S4::from(1) / S4::from(2);
    for _ in 0..600 {
        if t.vanished() {
            break;
        }
        t = t * half;
    }
    assert!(e.exploded() && t.vanished(), "representative construction failed");
    (e, t)
}

#[test]
fn circle_pow_scalar_truth_table() {
    let (e, t) = escaped_circles();
    let z = C4::ZERO;
    let inf = C4::INFINITY;
    let two = S4::from(2);
    let neg_two = S4::from(-2);
    let two5 = S4::from(5) / S4::from(2);
    let half = S4::from(1) / S4::from(2);

    // Zero base resolves by exponent sign; 0^0 = 1.
    check_c4("0^0", z.pow(S4::ZERO), &[Normal]);
    check_c4("0^2", z.pow(two), &[Zero]);
    check_c4("0^-2", z.pow(neg_two), &[Infinity]);
    // ∞ base: sign dominance, ∞^0 = 1.
    check_c4("∞^2", inf.pow(two), &[Infinity]);
    check_c4("∞^-2", inf.pow(neg_two), &[Zero]);
    check_c4("∞^0", inf.pow(S4::ZERO), &[Normal]);

    // Escaped base, integer exponent: class + ORIENTATION thru the multiply chain; z^1 ≡ z.
    let e1 = e.pow(S4::from(1));
    assert!(e1.exploded() && e1.real == e.real && e1.imaginary == e.imaginary, "↑^1 must be the identity");
    let e2 = e.pow(two);
    assert!(e2.exploded());
    assert_angle("↑^2", &e2, 106.26);
    let em1 = e.pow(S4::from(-1));
    assert!(em1.vanished());
    assert_angle("↑^-1", &em1, -53.13);
    let t2 = t.pow(two);
    assert!(t2.vanished());
    assert_angle("↓^2", &t2, 106.26);
    let tm2 = t.pow(neg_two);
    assert!(tm2.exploded());
    assert_angle("↓^-2", &tm2, -106.26);

    // Escaped base, non-integer |p| > 1: class by dominance, orientation rotates to p·θ.
    let e25 = e.pow(two5);
    assert!(e25.exploded());
    assert_angle("↑^2.5", &e25, 132.83);
    let t25 = t.pow(two5);
    assert!(t25.vanished());
    assert_angle("↓^2.5", &t25, 132.83);
    let em25 = e.pow(-two5);
    assert!(em25.vanished());
    assert_angle("↑^-2.5", &em25, -132.83);

    // Non-integer |p| < 1: class indeterminate (huge^0.5 can re-enter normal range).
    check_c4("↑^0.5", e.pow(half), &[Undefined]);
    check_c4("↓^0.5", t.pow(half), &[Undefined]);
}

#[test]
fn circle_pow_circle_truth_table() {
    let (e, _t) = escaped_circles();
    let z = C4::ZERO;
    let inf = C4::INFINITY;
    let i = C4::POS_I;

    // Real-axis complex exponents reroute thru the scalar path.
    check_c4("0^(2+0i)", z.pow(C4::from((2, 0))), &[Zero]);
    check_c4("0^(-2+0i)", z.pow(C4::from((-2, 0))), &[Infinity]);
    check_c4("0^(0)", z.pow(C4::ZERO), &[Normal]); // 0^0 = 1 (was ℘ before the exp.is_zero guard)
    let e2 = e.pow(C4::from((2, 0)));
    assert!(e2.exploded());
    assert_angle("↑^(2+0i)", &e2, 106.26);

    // Truly complex exponents: zero/∞ base by Re(w) sign; escaped base unknowable → ℘.
    check_c4("0^i", z.pow(i), &[Undefined]); // oscillates
    check_c4("∞^(2+i)", inf.pow(C4::from((2, 1))), &[Infinity]);
    check_c4("∞^(-2+i)", inf.pow(C4::from((-2, 1))), &[Zero]);
    check_c4("↑^(1+i)", e.pow(C4::from((1, 1))), &[Undefined]);

    // Normal-path sanity: (1+i)² = 2i, i^i = e^(-π/2) ≈ 0.2079 (real).
    let sq = C4::from((1, 1)).pow(C4::from((2, 0)));
    assert!(sq.is_normal());
    let ii = i.pow(i);
    assert!(ii.is_normal() && ii.i().is_negligible(), "i^i must be real");
}

#[test]
fn circle_exp_ln_sqrt_truth_tables() {
    let (e, t) = escaped_circles();
    let z = C4::ZERO;
    let inf = C4::INFINITY;
    let und = z / z;

    // exp: vanished ≈ 1; exploded resolves by the SIGN of Re (stored in the orientation): Re<0 → 0, Re≥0 → ℘; ∞ → ∞ (Scalar convention).
    check_c4("exp(0)", z.exp(), &[Normal]); // = 1
    check_c4("exp(↓)", t.exp(), &[Normal]); // ≈ 1
    let mut west = C4::from((-3, 1));
    for _ in 0..600 {
        if west.exploded() {
            break;
        }
        west = west + west;
    }
    check_c4("exp(↑ Re<0)", west.exp(), &[Zero]); // e^(-huge·dir) = 0
    check_c4("exp(↑ Re>0)", e.exp(), &[Undefined]); // angle unknowable
    check_c4("exp(∞)", inf.exp(), &[Infinity]);
    check_c4("exp(℘)", und.exp(), &[Undefined]);

    // ln: 0 and ∞ → singular ∞ (Scalar convention); escaped → ℘ (ln|z| unrepresentable poisons the pair).
    check_c4("ln(0)", z.ln(), &[Infinity]);
    check_c4("ln(∞)", inf.ln(), &[Infinity]);
    check_c4("ln(↑)", e.ln(), &[Undefined]);
    check_c4("ln(↓)", t.ln(), &[Undefined]);
    check_c4("ln(℘)", und.ln(), &[Undefined]);

    // sqrt: escaped is class-indeterminate (sqrt can re-enter normal range) → ℘√; 0/∞ pass thru.
    check_c4("sqrt(0)", z.sqrt(), &[Zero]);
    check_c4("sqrt(∞)", inf.sqrt(), &[Infinity]);
    check_c4("sqrt(↑)", e.sqrt(), &[Undefined]);
    check_c4("sqrt(↓)", t.sqrt(), &[Undefined]);

    // square: escaped preserves class AND doubles the orientation.
    let esq = e.square();
    assert!(esq.exploded());
    assert_angle("square(↑)", &esq, 106.26);
}

#[test]
fn circle_escaped_orientation_thru_ops() {
    // The Circle promise: an escaped value's angle survives every orientation-preserving op.
    let (e, t) = escaped_circles();
    assert_angle("↑ base", &e, 53.13);
    assert_angle("↑ × i", &(e * C4::POS_I), 143.13);
    assert_angle("-↑", &(-e), -126.87);
    assert_angle("conj ↑", &e.conjugate(), -53.13);
    assert_angle("↑ × ↑", &(e * e), 106.26);
    let r = e.reciprocal();
    assert!(r.vanished());
    assert_angle("1/↑", &r, -53.13);
    assert_angle("↓ × i", &(t * C4::POS_I), 143.13);
    let rv = t.reciprocal();
    assert!(rv.exploded());
    assert_angle("1/↓", &rv, -53.13);
    // Scalar scale keeps direction; a negative scalar rotates by 180°.
    assert_angle("↑ × 3", &(e * S4::from(3)), 53.13);
    assert_angle("↑ × -3", &(e * S4::from(-3)), -126.87);
    // Addition dominance: ↑ + ↓ keeps the exploded orientation; ↓ + normal yields the normal.
    assert_angle("↑ + ↓", &(e + t), 53.13);
    let vn = t + C4::from((1, 1));
    assert!(vn.is_normal());
}

// ============================================================ Equality / ordering semantics (0.1.0 decision: STRICT) ============================================================

#[test]
fn equality_ordering_semantics() {
    // DECISION (0.1.0): equality and ordering are STRICT. ℘, ∞, ↑, ↓ never compare equal to anything — including themselves — because equality between magnitudes the representation has lost would be a claim Spirix can't back. Users wanting bit identity compare the pub fields directly.
    let und = S::ZERO / S::ZERO;
    let inf = S::INFINITY;
    let ep = S::EXPLODED_POS;
    let vp = S::VANISHED_POS;
    let two = S::from(2);

    // Reflexive equality fails for every non-value class (NaN-style, but broader).
    assert!(und != und);
    assert!(inf != inf);
    assert!(ep != ep, "↑ == ↑ must be false: identical phase does not mean identical magnitude");
    assert!(vp != vp);
    // Normal equality is exact.
    assert!(two == S::from(2));
    assert!(S::ZERO == S::ZERO, "Zero is a definite value and equals itself");

    // Ordering: escaped values DO order against normals (the answer is knowable)...
    assert!(ep > two);
    assert!(vp < two);
    assert!(-vp < vp);
    // ...but ℘ and the unsigned point-at-infinity are unordered against everything.
    assert!(!(und < two) && !(two < und));
    assert!(!(inf < two) && !(two < inf) && !(inf > two));
    // partial_cmp on non-value classes is None — sort_by(partial_cmp().unwrap()) WILL panic on escaped data; that is deliberate, not an oversight.
    assert!(two.partial_cmp(&inf).is_none());
    assert!(ep.partial_cmp(&ep).is_none());
}
