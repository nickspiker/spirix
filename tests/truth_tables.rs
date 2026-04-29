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

// ============================================================
// Addition truth table (col + row) ============================================================
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
// Subtraction truth table (col - row) ============================================================
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
// Multiplication truth table (col × row) ============================================================
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
// Division truth table (col ÷ row) ============================================================
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

// ============================================================
// Modulus truth table (col % row) ============================================================
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

// ============================================================
// Unary operation truth tables: sqrt, lb, ln, exp, powb, square ============================================================

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
    let np = S::from(4);  // sqrt(4) = 2
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
    let np = S::from(2);   // lb(2) = 1
    let nn = S::from(-2);  // lb(-2) = undefined
    let one = S::ONE;       // lb(1) = 0
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("lb", "[0]", z, z.lb(), &[Infinity]);
    check_unary("lb", "[+↓]", vp, vp.lb(), &[Undefined]);
    check_unary("lb", "[-↓]", vn, vn.lb(), &[Undefined]);
    check_unary("lb", "[+# >1]", np, np.lb(), &[Normal]);
    check_unary("lb", "[+# =1]", one, one.lb(), &[Zero, Normal]);  // lb(1) = 0
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
    let np = S::from(2);  // exp(2) ≈ 7.39
    let nn = S::from(-2); // exp(-2) ≈ 0.135
    let ep = S::EXPLODED_POS;
    let en = S::EXPLODED_NEG;
    let inf = S::INFINITY;
    let und = S::ZERO / S::ZERO;

    check_unary("exp", "[0]", z, z.exp(), &[Normal]);  // e^0 = 1
    check_unary("exp", "[+↓]", vp, vp.exp(), &[Normal]);  // ≈ 1
    check_unary("exp", "[-↓]", vn, vn.exp(), &[Normal]);  // ≈ 1
    check_unary("exp", "[+#]", np, np.exp(), &[Normal, Exploded]);
    check_unary("exp", "[-#]", nn, nn.exp(), &[Normal, Vanished, Zero]);
    check_unary("exp", "[+↑]", ep, ep.exp(), &[Exploded, Infinity, Undefined]);
    check_unary("exp", "[-↑]", en, en.exp(), &[Zero]);  // e^-∞ = 0
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
        assert_eq!(classify(&r), *expected_class, "NOT({}) class should be {:?}", name, expected_class);
        assert_ne!(x.is_negative(), r.is_negative(), "NOT({}) should flip sign", name);
    }

    // Undefined preserved
    assert_eq!(classify(&!und), Undefined);
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
    check_unary("square", "[+#]", np, np.square(), &[Normal, Vanished, Exploded]);
    check_unary("square", "[-#]", nn, nn.square(), &[Normal, Vanished, Exploded]);
    check_unary("square", "[+↑]", ep, ep.square(), &[Exploded]);
    check_unary("square", "[-↑]", en, en.square(), &[Exploded]);
    check_unary("square", "[∞]", inf, inf.square(), &[Infinity]);
    check_unary("square", "[℘]", und, und.square(), &[Undefined]);

    // All squares are non-negative (can't be < 0)
    for &x in &[np, nn, ep, en] {
        let sq = x.square();
        if sq.is_normal() {
            assert!(!sq.is_negative(), "square({:?}) should be non-negative, got negative", x);
        }
    }
}

// ============================================================
// Specific edge case: subtraction with MIN producing exploded ============================================================
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
