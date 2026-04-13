/// Test inflate/deflate via Scalar::new and internal methods.
/// Since inflate/deflate are pub(crate), we test them indirectly
/// through known Scalar construction patterns, or directly if exposed.
///
/// For now, we verify the mathematical properties:
/// - inflate(stored).deflate() == stored (round-trip)
/// - inflate(stored) always has N-1 form (top two effective bits differ)
/// - Positive effective values have negative stored fractions (MSB=1)
/// - Negative effective values have non-negative stored fractions (MSB=0)

#[test]
fn inflate_deflate_i8_exhaustive() {
    // We can't call inflate/deflate directly (pub(crate)), but we can verify
    // the mathematical properties by computing what inflate should produce
    // and checking it matches the specification.
    for stored in i8::MIN..=i8::MAX {
        let s = stored as u8;
        let sign_bit = ((!stored) >> 7) & 1; // ~MSB
        let nine_bit: u16 = (s as u16) | ((sign_bit as u16) << 8);
        let effective: i16 = ((nine_bit << 7) as i16) >> 7; // sign-extend from bit 8

        // Round-trip: deflate(effective) should give back stored
        assert_eq!(effective as i8, stored, "round-trip failed for stored={stored}");

        // N-1 property: top two effective bits always differ
        let bit8 = (effective >> 8) & 1;
        let bit7 = (effective >> 7) & 1;
        assert_ne!(bit8, bit7, "N-1 violated for stored={stored}: effective={effective}");

        // Sign convention: negative stored => positive effective, and vice versa
        if stored < 0 {
            assert!(effective > 0, "stored={stored} is negative but effective={effective} not positive");
        } else if stored > 0 {
            assert!(effective < 0, "stored={stored} is positive but effective={effective} not negative");
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
        assert_eq!(neg_eff_check, -effective,
            "inflate(neg_stored) != -effective for stored={stored}");
    }
}

#[test]
fn fraction_constants_i8() {
    use spirix::FractionConstants;

    // Normal class
    assert_eq!(i8::POS_ONE_NORMAL_FRACTION, i8::MIN); // -128 = 10000000
    let eff = inflate_i8(i8::POS_ONE_NORMAL_FRACTION);
    assert_eq!(eff, 128);
    assert!(eff > 0);

    assert_eq!(i8::NEG_ONE_NORMAL_FRACTION, 0); // 00000000
    let eff = inflate_i8(i8::NEG_ONE_NORMAL_FRACTION);
    assert_eq!(eff, -256);
    assert!(eff < 0);

    assert_eq!(i8::MAX_FRACTION, -1); // 11111111
    let eff = inflate_i8(i8::MAX_FRACTION);
    assert_eq!(eff, 255);

    assert_eq!(i8::MIN_FRACTION, 0); // 00000000
    let eff = inflate_i8(i8::MIN_FRACTION);
    assert_eq!(eff, -256);

    // Escaped class: exploded (N-1 stored, sign direct)
    assert_eq!(i8::POS_ONE_EXPLODED_FRACTION, 64);  // 01000000
    assert_eq!(i8::NEG_ONE_EXPLODED_FRACTION, -128); // 10000000

    // Escaped class: vanished (N-2 stored, sign direct)
    assert_eq!(i8::POS_ONE_VANISHED_FRACTION, 32);  // 00100000
    assert_eq!(i8::NEG_ONE_VANISHED_FRACTION, -64);  // 11000000
}

#[test]
fn is_integer_new_format() {
    use spirix::Scalar;

    // 42 = stored 10101000 (-88), exponent 6
    // effective = 168, value = 168/256 * 2^6 = 42.0
    let forty_two = Scalar::<i8, i8>::new(-88, 6);
    assert!(forty_two.is_integer(), "42 should be integer");

    // 42.5: stored 10101010 (-86), exponent 6
    // effective = 170, value = 170/256 * 2^6 = 42.5
    let forty_two_point_five = Scalar::<i8, i8>::new(-86, 6);
    assert!(!forty_two_point_five.is_integer(), "42.5 should not be integer");

    // 1.0: stored = MIN (-128), exponent = 1
    let one = Scalar::<i8, i8>::ONE;
    assert!(one.is_integer(), "1 should be integer");

    // 0.5: stored = MIN (-128), exponent = 0
    let half = Scalar::<i8, i8>::HALF;
    assert!(!half.is_integer(), "0.5 should not be integer");

    // Zero
    let zero = Scalar::<i8, i8>::ZERO;
    assert!(zero.is_integer(), "0 should be integer");

    // 2.0: stored = MIN (-128), exponent = 2
    let two = Scalar::<i8, i8>::TWO;
    assert!(two.is_integer(), "2 should be integer");

    // -1.0: stored = 0, exponent = 0
    // effective = -256, value = -256/256 * 2^0 = -1.0
    let neg_one = Scalar::<i8, i8>::NEG_ONE;
    assert!(neg_one.is_integer(), "-1 should be integer");

    // 3.0: stored = 10010000 (-112), exponent 6... wait let me compute
    // 3 = effective 192, value = 192/256 * 2^2 = 3.0
    // stored = deflate(192) = 192 as i8 = -64
    let three = Scalar::<i8, i8>::new(-64, 2);
    assert!(three.is_integer(), "3 should be integer");

    // PI should not be integer
    let pi = Scalar::<i8, i8>::PI;
    assert!(!pi.is_integer(), "PI should not be integer");
}

#[test]
fn floor_new_format() {
    use spirix::Scalar;
    type S = Scalar<i8, i8>;

    // -1.5: effective=-192, stored=deflate(-192)=-192 as i8=64, exponent=1
    let neg_1_5 = S::new(64, 1);
    let f = neg_1_5.floor();
    // floor(-1.5) = -2: stored=0, exponent=1 → effective=-256, value=-256/256*2=-2
    assert_eq!(f.fraction, 0);
    assert_eq!(f.exponent, 1);

    // 1.5: effective=192, stored=deflate(192)=-64, exponent=1
    let pos_1_5 = S::new(-64, 1);
    let f = pos_1_5.floor();
    // floor(1.5) = 1: stored=MIN(-128), exponent=1
    assert_eq!(f.fraction, i8::MIN);
    assert_eq!(f.exponent, 1);

    // 2.75: effective=176, stored=-80, exponent=2
    let pos_2_75 = S::new(-80, 2);
    let f = pos_2_75.floor();
    // floor(2.75) = 2: stored=MIN(-128), exponent=2
    assert_eq!(f.fraction, i8::MIN);
    assert_eq!(f.exponent, 2);

    // -2.75: effective=-176, stored=80, exponent=2
    let neg_2_75 = S::new(80, 2);
    let f = neg_2_75.floor();
    // floor(-2.75) = -3: effective=-192, stored=64, exponent=2
    assert_eq!(f.fraction, 64);
    assert_eq!(f.exponent, 2);

    // 0.5: stored=MIN, exponent=0 → floor = 0
    assert!(S::HALF.floor() == S::ZERO);

    // -1.0: stored=0, exponent=0 → floor = -1
    assert!(S::NEG_ONE.floor() == S::NEG_ONE);

    // Integer values floor to themselves
    assert!(S::ONE.floor() == S::ONE);
    assert!(S::TWO.floor() == S::TWO);
}

#[test]
fn from_f64_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::from(1.0f64);
    assert!(one.fraction == S::ONE.fraction && one.exponent == S::ONE.exponent, "from(1.0) should be ONE");

    let neg_one = S::from(-1.0f64);
    assert!(neg_one == S::NEG_ONE, "from(-1.0) should be NEG_ONE");

    let two = S::from(2.0f64);
    assert!(two == S::TWO, "from(2.0) should be TWO");

    let zero = S::from(0.0f64);
    assert!(zero == S::ZERO, "from(0.0) should be ZERO");

    let inf = S::from(f64::INFINITY);
    assert!(inf.is_infinite(), "from(INFINITY) should be infinite");

    let nan = S::from(f64::NAN);
    assert!(nan.is_undefined(), "from(NAN) should be undefined");
}

#[test]
fn from_f32_basic() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    let one = S::from(1.0f32);
    assert!(one.fraction == S::ONE.fraction && one.exponent == S::ONE.exponent, "from(1.0f32) should be ONE");

    let two = S::from(2.0f32);
    assert!(two.fraction == S::TWO.fraction && two.exponent == S::TWO.exponent, "from(2.0f32) should be TWO");

    let zero = S::from(0.0f32);
    assert!(zero == S::ZERO, "from(0.0f32) should be ZERO");

    let inf = S::from(f32::INFINITY);
    assert!(inf.is_infinite(), "from(f32::INFINITY) should be infinite");

    let nan = S::from(f32::NAN);
    assert!(nan.is_undefined(), "from(f32::NAN) should be undefined");
}

#[test]
fn roundtrip_f64() {
    use spirix::Scalar;
    type S = Scalar<i32, i8>;

    for &val in &[1.0, -1.0, 2.0, 0.5, -0.5, 3.14159, -42.0, 0.001, 1000.0, 0.125] {
        let s = S::from(val);
        let back: f64 = (&s).into();
        let err = (back - val).abs() / val.abs().max(1e-300);
        assert!(err < 1e-6, "roundtrip failed for {val}: got {back}, err={err}");
    }
}

// Local helper matching the inflate algorithm (since the trait is pub(crate))
fn inflate_i8(stored: i8) -> i16 {
    let low = stored as i16;
    let sign_bit = ((!stored) >> 7) as i16 & 1;
    let extended = (low & 0xFF) | (sign_bit << 8);
    (extended << 7) >> 7
}
