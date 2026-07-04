//! Conversion truth tables: the IEEE ↔ Spirix boundary map and value round-trips.
//! Pins the special-value correspondences in both directions, integer-cast saturation semantics, and precision of the f64 round trip — the behaviors a user hits the moment Spirix touches the outside world.

use spirix::*;

type S = ScalarF5E3;
type W = ScalarF6E5;

// ===================================================================== IEEE → Spirix =====

#[test]
fn ieee_specials_into_spirix() {
    // NaN → undefined (any NaN payload).
    assert!(S::from(f32::NAN).is_undefined());
    assert!(S::from(-f32::NAN).is_undefined());
    assert!(W::from(f64::NAN).is_undefined());
    // ±inf → signed exploded — IEEE's overflow sentinel maps to the class that still carries sign.
    let pi32 = S::from(f32::INFINITY);
    assert!(pi32.exploded() && pi32.is_positive());
    let ni32 = S::from(f32::NEG_INFINITY);
    assert!(ni32.exploded() && ni32.is_negative());
    let pi64 = W::from(f64::INFINITY);
    assert!(pi64.exploded() && pi64.is_positive());
    // ±0.0 → the single Zero (Spirix zero is signless).
    assert!(S::from(0.0f32).is_zero());
    assert!(S::from(-0.0f32).is_zero());
    assert!(W::from(-0.0f64).is_zero());
}

#[test]
fn ieee_subnormals_map_by_width() {
    // An f32 subnormal (~1e-40) is below F5E3's 2^-128 floor → vanished with sign; but F6E5's i32 exponent holds it comfortably → normal.
    let sub = 1e-40f32;
    let narrow = S::from(sub);
    assert!(narrow.vanished() && narrow.is_positive(), "subnormal → vanished at E3");
    let narrow_n = S::from(-sub);
    assert!(narrow_n.vanished() && narrow_n.is_negative());
    let wide = W::from(sub as f64);
    assert!(wide.is_normal(), "same value is normal at E5");
    let rel = (wide.to_f64() - sub as f64).abs() / sub as f64;
    assert!(rel < 1e-7, "subnormal value survives the wide conversion: rel {rel}");
}

// ===================================================================== Spirix → IEEE =====

#[test]
fn spirix_specials_into_ieee() {
    let und = S::ZERO / S::ZERO;
    let inf = S::INFINITY;
    let huge: S = S::MAX * 2;
    // NOTE: MAX_NEG is the negative value CLOSEST to zero (max of the negatives); the most-negative normal is MIN.
    let neg_huge: S = S::MIN * 2;
    let tiny: S = S::MIN_POS / 9;
    // Undefined → NaN; the singular unsigned ∞ ALSO → NaN (it is not IEEE's signed infinity — no sign to give it).
    assert!(und.to_f32().is_nan());
    assert!(inf.to_f32().is_nan());
    assert!(und.to_f64().is_nan());
    // Exploded → ±inf (magnitude beyond the format, sign preserved).
    assert_eq!(huge.to_f32(), f32::INFINITY);
    assert_eq!(neg_huge.to_f32(), f32::NEG_INFINITY);
    // Vanished → ±0.0 with the SIGN preserved in IEEE's signed zero.
    let vz = tiny.to_f32();
    assert!(vz == 0.0 && vz.is_sign_positive());
    let nvz = (-tiny).to_f32();
    assert!(nvz == 0.0 && nvz.is_sign_negative(), "negative vanished → -0.0");
    // Zero → +0.0.
    assert!(S::ZERO.to_f32() == 0.0);
}

#[test]
fn integer_casts_saturate_and_floor() {
    // NaN-like → 0 and out-of-range saturates (like Rust `as`), but in-range values FLOOR rather than truncate — consistent with Spirix's floor-based frac/division/rounding philosophy, and unlike Rust's toward-zero `as`.
    let und = S::ZERO / S::ZERO;
    let inf = S::INFINITY;
    let huge: S = S::MAX * 2;
    let neg_huge: S = S::MIN * 2;
    let tiny: S = S::MIN_POS / 9;
    assert_eq!(und.to_i32(), 0, "undefined → 0 (Rust NaN-cast convention)");
    assert_eq!(huge.to_i32(), i32::MAX);
    assert_eq!(neg_huge.to_i32(), i32::MIN);
    assert_eq!(inf.to_i32(), i32::MAX, "unsigned ∞ saturates high (documented quirk)");
    // Vanished of either sign casts to 0 (negligible short-circuit — NOTE: floor(-↓) is -1, but the int cast treats vanished as ≈0 before flooring).
    assert_eq!(tiny.to_i32(), 0);
    assert_eq!((-tiny).to_i32(), 0);
    // Floor semantics: 2.9 → 2, -2.9 → -3.
    assert_eq!(S::from(2.9f32).to_i32(), 2);
    assert_eq!(S::from(-2.9f32).to_i32(), -3);
    assert_eq!(S::from(42).to_i64(), 42);
    assert_eq!(S::from(-42).to_i64(), -42);
}

// ===================================================================== Round trips =====

#[test]
fn f64_round_trip_precision() {
    // f64 → ScalarF6E5 → f64 must hold ~2^-62 relative error (i64 fraction, one bit of slack for floor rounding on each leg).
    let cases = [
        1.0f64, -1.0, 0.5, -0.5, 2.0, 42.0, 1.0 / 42.0, 3.141592653589793, 2.718281828459045, 1e300, 1e-300, -6.62607015e-34, 4294967296.0, 2.3283064365386963e-10,
    ];
    for &x in &cases {
        let rt = W::from(x).to_f64();
        let rel = ((rt - x) / x).abs();
        assert!(rel < 1e-15, "round trip {x} → {rt}: rel {rel:.3e}");
    }
}

#[test]
fn f32_round_trip_sweep() {
    // Deterministic sweep across every f32 binade: value bits from a fixed LCG, all finite → convert thru ScalarF6E5 and back, requiring f32-exactness after rounding.
    let mut state = 0x1234_5678_9ABC_DEF0u64;
    let mut lcg = move || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state
    };
    let mut checked = 0usize;
    for _ in 0..50_000 {
        let bits = lcg() as u32;
        let x = f32::from_bits(bits);
        if !x.is_finite() || x == 0.0 {
            continue;
        }
        let rt = W::from(x).to_f32();
        // Subnormal f32 inputs stay in range at E5, so every finite value must round-trip to the same f32 (F6's 64-bit fraction dominates f32's 24).
        assert_eq!(rt.to_bits(), x.to_bits(), "f32 round trip {x:e} → {rt:e}");
        checked += 1;
    }
    assert!(checked > 40_000, "sweep degenerated: {checked}");
}

#[test]
fn primitive_ingest_matches_value() {
    // Integer and float ingestion agree with each other and the oracle.
    assert!(S::from(42) == S::from(42.0f32));
    assert!(S::from(-7) == S::from(-7.0f64));
    assert_eq!(W::from(u64::MAX).to_f64(), u64::MAX as f64);
    assert_eq!(W::from(i64::MIN).to_f64(), i64::MIN as f64);
    // u128/i128 extremes must classify normal at wide widths (no silent wrap).
    assert!(W::from(u128::MAX).is_normal());
    assert!(W::from(i128::MIN).is_normal());
}
