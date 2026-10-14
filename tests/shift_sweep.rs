//! Exhaustive `<<` / `>>` sweep over every normal stored exponent × every E3 shift amount, both types.
//!
//! Regression for the Circle shift bug where circle_shl_integer / circle_shr_integer kept the pre-AMBIG=0 sign-flip overflow tests: crossing 1.0 upward exploded, landing at or below [1, 2) from above vanished (including `x >> 0` for x in [1, 2)), and real overflow/underflow past the cycle edge wrapped silently to the far end of the range.
//!
//! Expected result is computed directly from the cyclic exponent: stored position p in 1..=255, new position p ± shift; above 255 explodes, below 1 vanishes, otherwise same fraction at the new position.

use spirix::*;

#[derive(Debug, PartialEq)]
enum Expect {
    Normal(i32),
    Exploded,
    Vanished,
}

fn expect(stored: i32, delta: i32) -> Expect {
    let np = stored + delta;
    if np > 255 {
        Expect::Exploded
    } else if np < 1 {
        Expect::Vanished
    } else {
        Expect::Normal(np)
    }
}

#[test]
fn circle_shift_sweep_f4e3() {
    let bases = [
        CircleF4E3::from((1.0f64, 0.5f64)),
        CircleF4E3::from((-0.75f64, 0.3f64)),
        CircleF4E3::from((0.1f64, -0.9f64)),
        CircleF4E3::from((-1.0f64, 0.0f64)),
        CircleF4E3::from((0.0f64, 1.0f64)),
    ];
    let mut failures = Vec::new();
    for b in bases.iter() {
        for stored in 1i32..=255 {
            let mut c = *b;
            c.exponent = stored as u8 as i8;
            for s in -128i32..=127 {
                for (op, r, delta) in [("<<", c << s, s), (">>", c >> s, -s)] {
                    let ok = match expect(stored, delta) {
                        Expect::Normal(np) => {
                            r.is_normal()
                                && r.exponent as u8 as i32 == np
                                && r.real == c.real
                                && r.imaginary == c.imaginary
                        }
                        // Escapes keep the phase: exploded carries the fractions as-is, vanished carries them >> 1.
                        Expect::Exploded => {
                            r.is_exploded() && r.real == c.real && r.imaginary == c.imaginary
                        }
                        Expect::Vanished => {
                            r.is_vanished()
                                && r.real == c.real >> 1
                                && r.imaginary == c.imaginary >> 1
                        }
                    };
                    if !ok {
                        failures.push(format!(
                            "({}, {}) * 2^{} {} {}",
                            c.real,
                            c.imaginary,
                            stored - 128,
                            op,
                            s
                        ));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(8)]
    );
}

#[test]
fn scalar_shift_sweep_f4e3() {
    let bases = [
        ScalarF4E3::from(1.0f64),
        ScalarF4E3::from(-0.75f64),
        ScalarF4E3::from(0.1f64),
        ScalarF4E3::from(-1.0f64),
    ];
    let mut failures = Vec::new();
    for b in bases.iter() {
        for stored in 1i32..=255 {
            let mut c = *b;
            c.exponent = stored as u8 as i8;
            let neg = c < 0;
            for s in -128i32..=127 {
                for (op, r, delta) in [("<<", c << s, s), (">>", c >> s, -s)] {
                    let ok = match expect(stored, delta) {
                        Expect::Normal(np) => {
                            r.is_normal()
                                && r.exponent as u8 as i32 == np
                                && r.fraction == c.fraction
                        }
                        Expect::Exploded => r.is_exploded() && (r < 0) == neg,
                        Expect::Vanished => r.is_vanished() && (r < 0) == neg,
                    };
                    if !ok {
                        failures.push(format!("{} * 2^{} {} {}", c.fraction, stored - 128, op, s));
                    }
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures, first: {:?}",
        failures.len(),
        &failures[..failures.len().min(8)]
    );
}

#[test]
fn circle_shift_spot_checks() {
    let x = CircleF5E3::from((1.25f64, 0.5f64));
    assert!(x >> 0i32 == x, "x >> 0 must be x for x in [1, 2)");
    assert!((CircleF5E3::from((0.75f64, 0.25f64)) << 1i32) == CircleF5E3::from((1.5f64, 0.5f64)));
    assert!((CircleF5E3::from((3.0f64, -1.0f64)) >> 1i32) == CircleF5E3::from((1.5f64, -0.5f64)));
    assert!((CircleF5E3::from((3.0f64, -1.0f64)) << -1i32) == CircleF5E3::from((1.5f64, -0.5f64)));
    let mut a = CircleF5E3::from((0.6f64, 0.2f64));
    a <<= 2i32;
    a >>= 2i32;
    assert!(a == CircleF5E3::from((0.6f64, 0.2f64)));
    // Wider exponent: crossing 1.0 and both cycle edges at E4.
    let y = CircleF4E4::from((0.5f64, 0.5f64));
    assert!((y << 3i32) == CircleF4E4::from((4.0f64, 4.0f64)));
    // Shift amounts saturate to the E range, so a single shift from near 1.0 cannot leave the cycle; chain two.
    assert!(((y << 20000i32) << 20000i32).is_exploded());
    assert!(((y >> 20000i32) >> 20000i32).is_vanished());
    assert!(((y << -20000i32) << -20000i32).is_vanished());
}
