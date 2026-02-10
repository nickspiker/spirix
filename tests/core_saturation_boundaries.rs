use spirix::ScalarF4E4;

#[test]
fn test_u8_saturation_comprehensive() {
    // Test from -512 to +512, incrementing by 1
    for i in -512..=512 {
        let scalar = ScalarF4E4::from(i);
        let result: u8 = scalar.into();

        // Expected behavior:
        // - Negative values → 0
        // - 0-255 → exact value
        // - 256+ → 255 (saturate to MAX)
        let expected = if i < 0 {
            0
        } else if i > 255 {
            255
        } else {
            i as u8
        };

        assert_eq!(
            result, expected,
            "Failed for i={}: got {}, expected {}",
            i, result, expected
        );
    }
}

#[test]
fn test_i8_saturation_comprehensive() {
    // Test from -512 to +512, incrementing by 1
    for i in -512..=512 {
        let scalar = ScalarF4E4::from(i);
        let result: i8 = scalar.into();

        // Expected behavior:
        // - Values < -128 → -128 (saturate to MIN)
        // - -128 to 127 → exact value
        // - Values > 127 → 127 (saturate to MAX)
        let expected = if i < -128 {
            -128
        } else if i > 127 {
            127
        } else {
            i as i8
        };

        assert_eq!(
            result, expected,
            "Failed for i={}: got {}, expected {}",
            i, result, expected
        );
    }
}

#[test]
fn test_u8_boundary_critical() {
    // Critical boundary: 2^8 = 256 must saturate to 255
    let s256 = ScalarF4E4::from(256);
    let result: u8 = s256.into();
    assert_eq!(result, u8::MAX, "2^8 (256) must saturate to u8::MAX (255)");

    // Just below boundary: 255 should convert exactly
    let s255 = ScalarF4E4::from(255);
    let result: u8 = s255.into();
    assert_eq!(result, 255, "255 must convert exactly to 255");

    // Just below: 254 should convert exactly
    let s254 = ScalarF4E4::from(254);
    let result: u8 = s254.into();
    assert_eq!(result, 254, "254 must convert exactly to 254");
}

#[test]
fn test_i8_boundary_critical() {
    // Critical upper boundary: 128 must saturate to 127
    let s128 = ScalarF4E4::from(128);
    let result: i8 = s128.into();
    assert_eq!(result, i8::MAX, "128 must saturate to i8::MAX (127)");

    // Just below: 127 should convert exactly
    let s127 = ScalarF4E4::from(127);
    let result: i8 = s127.into();
    assert_eq!(result, 127, "127 must convert exactly to 127");

    // Critical lower boundary: -129 must saturate to -128
    let s_neg129 = ScalarF4E4::from(-129);
    let result: i8 = s_neg129.into();
    assert_eq!(result, i8::MIN, "-129 must saturate to i8::MIN (-128)");

    // Just above: -128 should convert exactly
    let s_neg128 = ScalarF4E4::from(-128);
    let result: i8 = s_neg128.into();
    assert_eq!(result, -128, "-128 must convert exactly to -128");
}

#[test]
fn test_u16_boundary() {
    // 2^16 = 65536 must saturate to 65535
    let s = ScalarF4E4::from(65536);
    let result: u16 = s.into();
    assert_eq!(result, u16::MAX, "2^16 (65536) must saturate to u16::MAX (65535)");

    // Just below: 65535 should convert exactly
    let s = ScalarF4E4::from(65535);
    let result: u16 = s.into();
    assert_eq!(result, 65535, "65535 must convert exactly");
}

#[test]
fn test_u32_boundary() {
    // 2^32 = 4294967296 must saturate to 4294967295
    let s = ScalarF4E4::from(4294967296i64);
    let result: u32 = s.into();
    assert_eq!(result, u32::MAX, "2^32 must saturate to u32::MAX");

    // Just below: should convert exactly
    let s = ScalarF4E4::from(4294967295u32);
    let result: u32 = s.into();
    assert_eq!(result, 4294967295, "2^32-1 must convert exactly");
}

#[test]
fn test_negative_to_unsigned() {
    // All negative values should convert to 0 for unsigned types
    for i in -100..0 {
        let scalar = ScalarF4E4::from(i);
        let u8_result: u8 = scalar.into();
        let u16_result: u16 = scalar.into();
        let u32_result: u32 = scalar.into();

        assert_eq!(u8_result, 0, "Negative {} to u8 should be 0", i);
        assert_eq!(u16_result, 0, "Negative {} to u16 should be 0", i);
        assert_eq!(u32_result, 0, "Negative {} to u32 should be 0", i);
    }
}

#[test]
fn test_explicit_to_u8_method() {
    // Test explicit to_u8() method (not just Into trait)
    let s = ScalarF4E4::from(256);
    assert_eq!(s.to_u8(), 255, "to_u8() must saturate 256 to 255");

    let s = ScalarF4E4::from(255);
    assert_eq!(s.to_u8(), 255, "to_u8() must convert 255 exactly");

    let s = ScalarF4E4::from(-1);
    assert_eq!(s.to_u8(), 0, "to_u8() must convert negative to 0");
}

#[test]
fn test_explicit_to_i8_method() {
    // Test explicit to_i8() method (not just Into trait)
    let s = ScalarF4E4::from(128);
    assert_eq!(s.to_i8(), 127, "to_i8() must saturate 128 to 127");

    let s = ScalarF4E4::from(127);
    assert_eq!(s.to_i8(), 127, "to_i8() must convert 127 exactly");

    let s = ScalarF4E4::from(-129);
    assert_eq!(s.to_i8(), -128, "to_i8() must saturate -129 to -128");

    let s = ScalarF4E4::from(-128);
    assert_eq!(s.to_i8(), -128, "to_i8() must convert -128 exactly");
}
