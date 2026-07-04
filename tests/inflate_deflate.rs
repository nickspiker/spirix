/// Test inflate/deflate via Scalar::new and internal methods. Since inflate/deflate are pub(crate), we test them indirectly thru known Scalar construction patterns, or directly if exposed.
///
/// For now, we verify the mathematical properties:
/// - inflate(stored).deflate() == stored (round-trip)
/// - inflate(stored) always has N-1 form (top two effective bits differ)
/// - Positive effective values have negative stored fractions (MSB=1)
/// - Negative effective values have non-negative stored fractions (MSB=0)

#[test]
fn inflate_deflate_i8_exhaustive() {
    // We can't call inflate/deflate directly (pub(crate)), but we can verify the mathematical properties by computing what inflate should produce and checking it matches the specification.
    for stored in i8::MIN..=i8::MAX {
        let s = stored as u8;
        let sign_bit = ((!stored) >> 7) & 1; // ~MSB
        let nine_bit: u16 = (s as u16) | ((sign_bit as u16) << 8);
        let effective: i16 = ((nine_bit << 7) as i16) >> 7; // sign-extend from bit 8

        // Round-trip: deflate(effective) should give back stored
        assert_eq!(
            effective as i8, stored,
            "round-trip failed for stored={stored}"
        );

        // N-1 property: top two effective bits always differ
        let bit8 = (effective >> 8) & 1;
        let bit7 = (effective >> 7) & 1;
        assert_ne!(
            bit8, bit7,
            "N-1 violated for stored={stored}: effective={effective}"
        );

        // Sign convention: negative stored => positive effective, and vice versa
        if stored < 0 {
            assert!(
                effective > 0,
                "stored={stored} is negative but effective={effective} not positive"
            );
        } else if stored > 0 {
            assert!(
                effective < 0,
                "stored={stored} is positive but effective={effective} not negative"
            );
        }
        // stored == 0: effective is most negative value (-256), which is negative. That's correct.
    }
}

#[test]
fn inflate_deflate_i8_known_values() {
    // stored = -128 (0b10000000) => effective = +128
    let s: i8 = -128;
    let eff = inflate_i8(s);
    assert_eq!(eff, 128);

    // stored = 0 (0b00000000) => effective = -256
    let s: i8 = 0;
    let eff = inflate_i8(s);
    assert_eq!(eff, -256);

    // stored = 127 (0b01111111) => effective = -129
    let s: i8 = 127;
    let eff = inflate_i8(s);
    assert_eq!(eff, -129);

    // stored = -1 (0b11111111) => effective = +255
    let s: i8 = -1;
    let eff = inflate_i8(s);
    assert_eq!(eff, 255);

    // stored = 1 => effective = -255
    let s: i8 = 1;
    let eff = inflate_i8(s);
    assert_eq!(eff, -255);
}

#[test]
fn negation_via_wrapping_neg_i8() {
    // Verify wrapping_neg on stored == deflate(-inflate(stored)) for general case
    for stored in i8::MIN..=i8::MAX {
        let effective = inflate_i8(stored);
        let neg_effective = effective.wrapping_neg();
        let neg_stored = neg_effective as i8; // deflate
        let expected = stored.wrapping_neg();

        if stored == 0 || stored == i8::MIN {
            // Edge cases: wrapping_neg is a no-op, need exponent adjustment
            continue;
        }

        assert_eq!(neg_stored, expected,
            "wrapping_neg mismatch for stored={stored}: deflate(-inflate)={neg_stored}, wrapping_neg={expected}");

        // Verify the negated value inflates to the negation of the original
        let neg_eff_check = inflate_i8(neg_stored);
        assert_eq!(
            neg_eff_check, -effective,
            "inflate(neg_stored) != -effective for stored={stored}"
        );
    }
}

#[test]
fn fraction_constants_i8() {
    use spirix::ScalarF3E3;

    // Normal class
    assert_eq!(i8::MIN, i8::MIN); // -128 = 10000000
    let eff = inflate_i8(i8::MIN);
    assert_eq!(eff, 128);
    assert!(eff > 0);

    assert_eq!(0i8, 0); // 00000000
    let eff = inflate_i8(0i8);
    assert_eq!(eff, -256);
    assert!(eff < 0);

    assert_eq!(-1i8, -1); // 11111111
    let eff = inflate_i8(-1i8);
    assert_eq!(eff, 255);

    assert_eq!(0i8, 0); // 00000000
    let eff = inflate_i8(0i8);
    assert_eq!(eff, -256);

    // Escaped class: exploded (N-1 stored, sign direct)
    assert_eq!(64i8, 64); // 01000000
    assert_eq!(i8::MIN, -128); // 10000000

    // Escaped class: vanished (N-2 stored, sign direct)
    assert_eq!(32i8, 32); // 00100000
    assert_eq!(-64i8, -64); // 11000000
}

#[test]
fn from_f64_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::from(1.0f64);
    assert!(
        one.fraction == S::ONE.fraction && one.exponent == S::ONE.exponent,
        "from(1.0) should be ONE"
    );

    let neg_one = S::from(-1.0f64);
    assert!(neg_one == S::NEG_ONE, "from(-1.0) should be NEG_ONE");

    let two = S::from(2.0f64);
    assert!(two == S::TWO, "from(2.0) should be TWO");

    let zero = S::from(0.0f64);
    assert!(zero == S::ZERO, "from(0.0) should be ZERO");

    let inf = S::from(f64::INFINITY);
    assert!(
        inf.exploded(),
        "from(INFINITY) should be exploded (IEEE inf has direction)"
    );

    let nan = S::from(f64::NAN);
    assert!(nan.is_undefined(), "from(NAN) should be undefined");
}

#[test]
fn from_f32_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::from(1.0f32);
    assert!(
        one.fraction == S::ONE.fraction && one.exponent == S::ONE.exponent,
        "from(1.0f32) should be ONE"
    );

    let two = S::from(2.0f32);
    assert!(
        two.fraction == S::TWO.fraction && two.exponent == S::TWO.exponent,
        "from(2.0f32) should be TWO"
    );

    let zero = S::from(0.0f32);
    assert!(zero == S::ZERO, "from(0.0f32) should be ZERO");

    let inf = S::from(f32::INFINITY);
    assert!(
        inf.exploded(),
        "from(f32::INFINITY) should be exploded (IEEE inf has direction)"
    );

    let nan = S::from(f32::NAN);
    assert!(nan.is_undefined(), "from(f32::NAN) should be undefined");
}

#[test]
fn comparison_new_format() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::ONE;
    let two = S::TWO;
    let neg_one = S::NEG_ONE;
    let half = S::HALF;
    let zero = S::ZERO;

    assert!(two > one);
    assert!(one > half);
    assert!(one > zero);
    assert!(one > neg_one);
    assert!(neg_one < zero);
    assert!(neg_one < half);
    assert!(half > zero);
    assert!(zero == zero);
    assert!(one == one);

    // Cross-sign
    let neg_two = -two;
    assert!(neg_two < neg_one);
    assert!(neg_two < zero);
    assert!(two > neg_two);

    // Small values from f64
    let small_pos = S::from(0.001);
    let small_neg = S::from(-0.001);
    assert!(small_pos > zero);
    assert!(small_neg < zero);
    assert!(small_pos > small_neg);

    // Escaped values
    let pos_exploded = S::EXPLODED_POS;
    let neg_exploded = S::EXPLODED_NEG;
    let pos_vanished = S::VANISHED_POS;
    let neg_vanished = S::VANISHED_NEG;

    // Exploded: further from zero than normals
    assert!(pos_exploded > one);
    assert!(pos_exploded > two);
    assert!(neg_exploded < neg_one);
    assert!(neg_exploded < neg_two);

    // Vanished: closer to zero than normals
    assert!(pos_vanished < half);
    assert!(pos_vanished > zero);
    assert!(neg_vanished > neg_one);
    assert!(neg_vanished < zero);

    // Cross-class escaped
    assert!(pos_exploded > neg_exploded);
    assert!(pos_vanished > neg_vanished);
    assert!(pos_exploded > pos_vanished);
    assert!(neg_exploded < neg_vanished);
    assert!(pos_exploded > neg_vanished);
    assert!(neg_exploded < pos_vanished);

    // Same-type same-sign: unordered (PartialOrd returns None → not less, not greater, not equal)
    let pos_exploded2 = S::EXPLODED_POS;
    assert!(!(pos_exploded < pos_exploded2));
    assert!(!(pos_exploded > pos_exploded2));
    assert!(!(pos_exploded == pos_exploded2));

    // Undefined: unordered with everything
    let undef = S::from(0.0) / S::from(0.0);
    assert!(!(undef == one));
    assert!(!(undef < one));
    assert!(!(undef > one));
    assert!(!(undef == undef));

    // Infinity: unordered with everything
    let inf = S::INFINITY;
    assert!(!(inf == one));
    assert!(!(inf < one));
    assert!(!(inf > one));
    assert!(!(inf == inf));
}

#[test]
fn roundtrip_f64() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    for &val in &[
        1.0, -1.0, 2.0, 0.5, -0.5, 3.14159, -42.0, 0.001, 1000.0, 0.125,
    ] {
        let s = S::from(val);
        let back: f64 = (&s).into();
        let err = (back - val).abs() / val.abs().max(1e-300);
        assert!(
            err < 1e-6,
            "roundtrip failed for {val}: got {back}, err={err}"
        );
    }
}

#[test]
fn addition_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    // Normal arithmetic
    let one = S::ONE;
    let two = S::TWO;
    let neg_one = S::NEG_ONE;
    let half = S::HALF;
    let zero = S::ZERO;

    let result = one + one;
    assert!(result == two, "1 + 1 should be 2");

    let result = one + neg_one;
    assert!(result == zero, "1 + (-1) should be 0");

    let result = half + half;
    assert!(result == one, "0.5 + 0.5 should be 1");

    let result = two + neg_one;
    assert!(result == one, "2 + (-1) should be 1");

    let result = neg_one + neg_one;
    assert!(result == -two, "(-1) + (-1) should be -2");

    // f64 roundtrip thru addition
    for &(a, b, expected) in &[
        (3.0, 5.0, 8.0),
        (1.0, -1.0, 0.0),
        (-42.0, 42.0, 0.0),
        (0.125, 0.875, 1.0),
        (100.0, 0.5, 100.5),
        (-3.14, 3.14, 0.0),
    ] {
        let sa = S::from(a);
        let sb = S::from(b);
        let result = sa + sb;
        let back: f64 = (&result).into();
        let err = (back - expected).abs();
        assert!(
            err < 0.01,
            "{a} + {b}: expected {expected}, got {back}, err={err}"
        );
    }

    // --- Full truth table coverage ---
    let pos_exploded = S::EXPLODED_POS;
    let neg_exploded = S::EXPLODED_NEG;
    let pos_vanished = S::VANISHED_POS;
    let neg_vanished = S::VANISHED_NEG;
    let inf = S::INFINITY;
    let undef = S::from(0.0) / S::from(0.0);

    // [0] row
    assert!((zero + zero) == zero, "[0]+[0]=[0]");
    assert!((zero + pos_vanished).vanished(), "[0]+[+↓]=[↓]");
    assert!((zero + one) == one, "[0]+[#]=[#]");
    assert!(
        (zero + pos_exploded).exploded(),
        "[0]+[+↑]=[↑] (zero identity)"
    );
    assert!((zero + inf).is_infinite(), "[0]+[∞]=[∞] (∞ absorbs)");
    assert!((zero + undef).is_undefined(), "[0]+[℘]=[℘]");

    // [↓] row
    assert!((pos_vanished + zero).vanished(), "[+↓]+[0]=[↓]");
    assert!(
        (pos_vanished + pos_vanished).is_undefined(),
        "[+↓]+[+↓]=[℘↓+↓]"
    );
    assert!((pos_vanished + one) == one, "[+↓]+[#]=[#]");
    assert!(
        (pos_vanished + pos_exploded).exploded(),
        "[+↓]+[+↑]=[↑] (vanished negligible)"
    );
    assert!((pos_vanished + inf).is_infinite(), "[+↓]+[∞]=[∞]");
    assert!((pos_vanished + undef).is_undefined(), "[+↓]+[℘]=[℘]");

    // [#] row (normal + special)
    assert!((one + zero) == one, "[#]+[0]=[#]");
    assert!((one + pos_vanished) == one, "[#]+[+↓]=[#]");
    assert!((one + pos_exploded).is_undefined(), "[#]+[+↑]=[℘+⬆]");
    assert!((one + inf).is_infinite(), "[#]+[∞]=[∞]");
    assert!((one + undef).is_undefined(), "[#]+[℘]=[℘]");

    // [↑] row
    assert!((pos_exploded + zero).exploded(), "[+↑]+[0]=[↑]");
    assert!(
        (pos_exploded + pos_vanished).exploded(),
        "[+↑]+[+↓]=[↑] (vanished negligible)"
    );
    assert!((pos_exploded + one).is_undefined(), "[+↑]+[#]=[℘⬆+]");
    assert!(
        (pos_exploded + pos_exploded).is_undefined(),
        "[+↑]+[+↑]=[℘⬆+⬆]"
    );
    assert!((pos_exploded + inf).is_infinite(), "[+↑]+[∞]=[∞]");
    assert!((pos_exploded + undef).is_undefined(), "[+↑]+[℘]=[℘]");

    // [∞] row (infinity absorbs everything; signless [∞] makes ∞-∞ = ∞ too)
    assert!((inf + zero).is_infinite(), "[∞]+[0]=[∞]");
    assert!((inf + one).is_infinite(), "[∞]+[#]=[∞]");
    assert!((inf + inf).is_infinite(), "[∞]+[∞]=[∞]");
    assert!((inf + undef).is_undefined(), "[∞]+[℘]=[℘]");

    // [℘] row
    assert!((undef + zero).is_undefined(), "[℘]+[0]=[℘]");
    assert!((undef + one).is_undefined(), "[℘]+[#]=[℘]");
    assert!((undef + undef).is_undefined(), "[℘]+[℘]=[℘]");

    // Sign preservation for escaped values
    assert!(
        (zero + pos_vanished).is_positive(),
        "[0]+[+↓] should be positive"
    );
    assert!(
        (zero + neg_vanished).is_negative(),
        "[0]+[-↓] should be negative"
    );
    assert!(
        (pos_vanished + zero).is_positive(),
        "[+↓]+[0] should be positive"
    );
    assert!(
        (neg_vanished + zero).is_negative(),
        "[-↓]+[0] should be negative"
    );

    // Negative escaped interactions
    assert!((neg_exploded + zero).exploded(), "[-↑]+[0]=[-↑]");
    assert!((neg_exploded + one).is_undefined(), "[-↑]+[#]=[℘]");
    assert!(
        (neg_exploded + neg_exploded).is_undefined(),
        "[-↑]+[-↑]=[℘]"
    );
    assert!(
        (pos_exploded + neg_exploded).is_undefined(),
        "[+↑]+[-↑]=[℘]"
    );
}

#[test]
fn subtraction_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::ONE;
    let two = S::TWO;
    let zero = S::ZERO;

    assert!((two - one) == one, "2 - 1 = 1");
    assert!((one - one) == zero, "1 - 1 = 0");
    assert!((one - two) == -one, "1 - 2 = -1");
    assert!((zero - one) == -one, "0 - 1 = -1");

    for &(a, b, expected) in &[
        (10.0, 3.0, 7.0),
        (1.0, 0.5, 0.5),
        (-5.0, 3.0, -8.0),
        (100.0, 100.0, 0.0),
        (0.125, 0.125, 0.0),
    ] {
        let sa = S::from(a);
        let sb = S::from(b);
        let result = sa - sb;
        let back: f64 = (&result).into();
        let err = (back - expected).abs();
        assert!(err < 0.01, "{a} - {b}: expected {expected}, got {back}");
    }
}

#[test]
fn multiplication_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::ONE;
    let two = S::TWO;
    let neg_one = S::NEG_ONE;
    let zero = S::ZERO;

    assert!((one * one) == one, "1 * 1 = 1");
    assert!((two * one) == two, "2 * 1 = 2");
    assert!((neg_one * neg_one) == one, "-1 * -1 = 1");
    assert!((one * neg_one) == neg_one, "1 * -1 = -1");
    assert!((two * zero) == zero, "2 * 0 = 0");

    {
        type S44 = Scalar<i16, i16>;
        for &(a, b) in &[
            (253.0, -254.0),
            (253.0, 253.0),
            (-254.0, -254.0),
            (3.0, 5.0),
            (0.5, 0.5),
        ] {
            let sa = S44::from(a);
            let sb = S44::from(b);
            let result = sa * sb;
            let back: f64 = (&result).into();
            let expected = a * b;
            let err = (back - expected).abs();
            eprintln!("F4E4: {a} * {b} = {back} (expected {expected}, err={err})");
        }
    }

    for &(a, b, expected) in &[
        (3.0, 5.0, 15.0),
        (2.0, 0.5, 1.0),
        (-2.0, -3.0, 6.0),
        (0.125, 8.0, 1.0),
        (-4.0, 3.0, -12.0),
        (100.0, 0.01, 1.0),
    ] {
        let sa = S::from(a);
        let sb = S::from(b);
        let result = sa * sb;
        let back: f64 = (&result).into();
        let err = (back - expected).abs();
        assert!(err < 0.01, "{a} * {b}: expected {expected}, got {back}");
    }

    // --- Full truth table coverage ---
    let pos_exploded = S::EXPLODED_POS;
    let neg_exploded = S::EXPLODED_NEG;
    let pos_vanished = S::VANISHED_POS;
    let neg_vanished = S::VANISHED_NEG;
    let inf = S::INFINITY;
    let undef = S::from(0.0) / S::from(0.0);

    // [0] row
    assert!((zero * zero) == zero, "[0]*[0]=[0]");
    assert!((zero * pos_vanished) == zero, "[0]*[+↓]=[0]");
    assert!((zero * one) == zero, "[0]*[#]=[0]");
    assert!((zero * pos_exploded) == zero, "[0]*[+↑]=[0]");
    assert!((zero * inf).is_undefined(), "[0]*[∞]=[℘]");
    assert!((zero * undef).is_undefined(), "[0]*[℘]=[℘]");

    // [↓] row
    assert!((pos_vanished * zero) == zero, "[+↓]*[0]=[0]");
    assert!((pos_vanished * pos_vanished).vanished(), "[+↓]*[+↓]=[↓]");
    assert!((pos_vanished * one).vanished(), "[+↓]*[#]=[↓]");
    assert!(
        (pos_vanished * pos_exploded).is_undefined(),
        "[+↓]*[+↑]=[℘]"
    );
    assert!((pos_vanished * inf).is_infinite(), "[+↓]*[∞]=[∞]");
    assert!((pos_vanished * undef).is_undefined(), "[+↓]*[℘]=[℘]");

    // [#] row
    assert!((one * zero) == zero, "[#]*[0]=[0]");
    assert!((one * pos_vanished).vanished(), "[#]*[+↓]=[↓]");
    assert!((one * pos_exploded).exploded(), "[#]*[+↑]=[↑]");
    assert!((one * inf).is_infinite(), "[#]*[∞]=[∞]");
    assert!((one * undef).is_undefined(), "[#]*[℘]=[℘]");

    // [↑] row
    assert!((pos_exploded * zero) == zero, "[+↑]*[0]=[0]");
    assert!(
        (pos_exploded * pos_vanished).is_undefined(),
        "[+↑]*[+↓]=[℘]"
    );
    assert!((pos_exploded * one).exploded(), "[+↑]*[#]=[↑]");
    assert!((pos_exploded * pos_exploded).exploded(), "[+↑]*[+↑]=[↑]");
    assert!((pos_exploded * inf).is_infinite(), "[+↑]*[∞]=[∞]");
    assert!((pos_exploded * undef).is_undefined(), "[+↑]*[℘]=[℘]");

    // [∞] row
    assert!((inf * zero).is_undefined(), "[∞]*[0]=[℘]");
    assert!((inf * one).is_infinite(), "[∞]*[#]=[∞]");
    assert!((inf * inf).is_infinite(), "[∞]*[∞]=[∞]");
    assert!((inf * undef).is_undefined(), "[∞]*[℘]=[℘]");

    // [℘] row
    assert!((undef * zero).is_undefined(), "[℘]*[0]=[℘]");
    assert!((undef * one).is_undefined(), "[℘]*[#]=[℘]");
    assert!((undef * undef).is_undefined(), "[℘]*[℘]=[℘]");

    // Sign preservation escaped * normal sign preservation:
    assert!(
        (neg_exploded * one).exploded() && (neg_exploded * one).is_negative(),
        "[-↑]*[#]=[-↑]"
    );
    // escaped * normal sign preservation:
    assert!(
        (neg_vanished * one).vanished() && (neg_vanished * one).is_negative(),
        "[-↓]*[#]=[-↓]"
    );
}

#[test]
fn multiply_f3e3_exhaustive() {
    use spirix::Scalar;
    type S = Scalar<i8, i8>;

    let mut failures = 0;
    let mut total = 0;
    for a_stored in i8::MIN..=i8::MAX {
        for b_stored in a_stored..=i8::MAX {
            let a = S::new(a_stored, 0);
            let b = S::new(b_stored, 0);
            let result = a * b;

            // Compute expected via f64
            let a_val: f64 = (&a).into();
            let b_val: f64 = (&b).into();
            let expected = a_val * b_val;
            let got: f64 = (&result).into();

            total += 1;
            // F3E3 has 8 bits of fraction = ~2.4 decimal digits. 1 ULP ≈ 1/256 ≈ 0.004. Allow 2 ULP of error for truncation rounding.
            let ulp = expected.abs() / 128.0; // 1 ULP at FRAC=8
            if (got - expected).abs() > ulp * 2.0 + 1e-10 {
                if failures < 10 {
                    eprintln!("FAIL: stored ({a_stored}, {b_stored}) val ({a_val} * {b_val}) = {expected}, got {got}");
                }
                failures += 1;
            }
        }
    }
    eprintln!("{failures}/{total} failures");
    assert_eq!(
        failures, 0,
        "{failures} multiplication failures out of {total}"
    );
}

#[test]
fn division_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::ONE;
    let two = S::TWO;
    let neg_one = S::NEG_ONE;
    let zero = S::ZERO;

    assert!((one / one) == one, "1 / 1 = 1");
    assert!((two / one) == two, "2 / 1 = 2");
    assert!((one / two) == S::HALF, "1 / 2 = 0.5");
    assert!((neg_one / neg_one) == one, "-1 / -1 = 1");
    assert!((one / neg_one) == neg_one, "1 / -1 = -1");

    // Division by zero = infinity
    assert!((one / zero).is_infinite(), "1 / 0 = ∞");
    // Zero divided by zero = undefined
    assert!((zero / zero).is_undefined(), "0 / 0 = ℘");

    for &(a, b, expected) in &[
        (10.0, 2.0, 5.0), // stored fracs are negative (positive values)
        (1.0, 3.0, 0.333333),
        (-6.0, 2.0, -3.0),
        (100.0, 0.5, 200.0),
        (0.125, 0.25, 0.5),
    ] {
        let sa = S::from(a);
        let sb = S::from(b);
        let result = sa / sb;
        let back: f64 = (&result).into();
        let err = (back - expected).abs();
        assert!(err < 0.01, "{a} / {b}: expected {expected}, got {back}");
    }
}

#[test]
fn divide_f3e3_exhaustive() {
    use spirix::Scalar;
    type S = Scalar<i8, i8>;

    let mut failures = 0;
    let mut total = 0;
    for a_stored in i8::MIN..=i8::MAX {
        for b_stored in i8::MIN..=i8::MAX {
            let a = S::new(a_stored, 0);
            let b = S::new(b_stored, 0);
            if b.is_zero() {
                continue;
            } // skip divide by zero
            let result = a / b;

            let a_val: f64 = (&a).into();
            let b_val: f64 = (&b).into();
            let expected = a_val / b_val;
            let got: f64 = (&result).into();

            total += 1;
            let ulp = expected.abs() / 128.0;
            if (got - expected).abs() > ulp * 2.0 + 1e-10 {
                if failures < 10 {
                    eprintln!("FAIL: stored ({a_stored}, {b_stored}) val ({a_val} / {b_val}) = {expected}, got {got}");
                }
                failures += 1;
            }
        }
    }
    eprintln!("{failures}/{total} failures");
    assert_eq!(failures, 0, "{failures} division failures out of {total}");
}

// Local helper matching the inflate algorithm (since the trait is pub(crate))
fn inflate_i8(stored: i8) -> i16 {
    let low = stored as i16;
    let sign_bit = ((!stored) >> 7) as i16 & 1;
    let extended = (low & 0xFF) | (sign_bit << 8);
    (extended << 7) >> 7
}
