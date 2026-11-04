//! SPX-01 and SPX-02 (security audit, 2026-11-03): an integer exponent of transfinite magnitude must not spin the squaring loop, and `Display` of a normal `Circle` at the exponent floor must return.
//! Every case runs on its own thread under a deadline, so a regression fails the test instead of hanging the suite.

use spirix::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

/// Run `f` on a thread; no result within `secs` seconds is a hang.
fn within<T: Send + 'static>(secs: u64, f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    rx.recv_timeout(Duration::from_secs(secs)).ok()
}

/// SPX-01, the auditor's input: the exponent is an even integer of magnitude about 2^(2^31 - 1), so the loop would run 2^31 times; the answer is decided by |base| against 1.
#[test]
fn pow_with_a_transfinite_integer_exponent_returns() {
    let base = Scalar::<i32, i32> { fraction: -1, exponent: i32::MAX };
    let exp = Scalar::<i32, i32> { fraction: -1, exponent: -1 };
    assert!(base.is_normal() && base.is_positive() && base < ScalarF5E5::ONE, "{base:?}");
    assert!(exp.is_integer() && exp.is_positive(), "{exp:?}");
    let r = within(10, move || base.pow(exp)).expect("pow hung on the auditor's input");
    // A base below 1 to a huge even power vanishes with a positive sign.
    assert!(r.vanished() && r.is_positive(), "{r:?}");
    // The negated exponent goes thru the reciprocal first: this base is the largest 32-bit value below 1, whose reciprocal rounds to exactly 1, so the answer is 1 — what matters here is that it returns.
    let r = within(10, move || base.pow(-exp)).expect("pow hung on the negated input");
    assert!(!r.is_undefined(), "{r:?}");
    // A base whose reciprocal is exact explodes thru the same path.
    let r = within(10, move || ScalarF5E5::HALF.pow(-exp)).expect("pow hung on the negated input");
    assert!(r.exploded() && r.is_positive(), "{r:?}");
}

/// Past the cutoff |n| ≥ 2^(FRAC + EXP) every integer power resolves by dominance (and parity, which is always even there); just below it the loop still runs and agrees.
#[test]
fn huge_integer_powers_resolve_by_dominance() {
    type S = ScalarF5E5;
    let n = S::TWO.pow(100isize);
    assert!(n.is_integer() && n.is_normal(), "{n:?}");
    let two = S::from(2);
    assert!(two.pow(n).exploded() && two.pow(n).is_positive());
    assert!(two.pow(-n).vanished() && two.pow(-n).is_positive());
    let minus_two = S::from(-2);
    assert!(minus_two.pow(n).exploded() && minus_two.pow(n).is_positive(), "an even power of a negative base is positive");
    assert!(S::HALF.pow(n).vanished() && S::HALF.pow(n).is_positive());
    assert!(S::ONE.pow(n) == S::ONE);
    assert!(S::from(-1).pow(n) == S::ONE);
    assert!(S::ZERO.pow(n).is_zero());
    let below = S::TWO.pow(63isize);
    assert!(two.pow(below).exploded() && two.pow(below).is_positive());
    assert!(S::HALF.pow(below).vanished());
    // The Circle twin: the direction n·θ is unknowable at that scale, so the honest answer is undefined — returned, not spun for.
    let z = CircleF5E5::from((2, 1));
    let p = within(10, move || z.pow(n)).expect("circle pow hung");
    assert!(p.is_undefined(), "{p:?}");
}

/// SPX-02, the auditor's inputs: canonical circles at the exponent floor (both component views vanish) and a non-canonical pattern built thru the public fields.
#[test]
fn display_of_a_circle_at_the_exponent_floor_returns() {
    let a = within(10, || format!("{}", Circle::<i8, i8> { real: 52, imaginary: -1, exponent: 1 }))
        .expect("F3E3 floor circle hung");
    let b = within(10, || format!("{}", Circle::<i32, i32> { real: 0x34000000, imaginary: -1, exponent: 1 }))
        .expect("F5E5 floor circle hung");
    let c = within(10, || format!("{}", Circle::<i8, i8> { real: 1, imaginary: 1, exponent: 1 }))
        .expect("non-canonical floor circle hung");
    let d = within(10, || format!("{}", Circle::<i128, i128> { real: 1 << 125, imaginary: -1, exponent: 1 }))
        .expect("F7E7 floor circle hung");
    for s in [&a, &b, &c] {
        // Widened one exponent step, the value prints in scientific notation with its (negative) exponent.
        assert!(s.starts_with('⦇') && s.contains("⦈×A^-"), "{s}");
    }
    assert!(a.starts_with("⦇+"), "{a}");
    assert!(a.contains(",-"), "{a}");
    // Nothing is wider than a 128-bit exponent: each component prints as the vanished Scalar its view is.
    assert_eq!(d, "⦇⦉+↓⦊,⦉-↓⦊⦈");
    // Away from the floor the fixed and scientific paths are untouched.
    assert_eq!(format!("{}", CircleF5E3::from((15_i32, 31_i32))), "⦇+15,+31⦈");
}

/// Every F3E3 exponent, every real component, a spread of imaginary components: `Display` returns for each (the fuzz suite covers the operators but never formatted a `Circle`).
#[test]
fn display_returns_across_the_f3e3_exponent_range() {
    let progress = Arc::new(AtomicU32::new(0));
    let seen = progress.clone();
    let imaginaries: [i8; 8] = [0, 1, -1, 52, 127, -128, 64, -64];
    let done = within(300, move || {
        let mut count = 0u32;
        for exponent in 0u8..=255 {
            for real in 0u8..=255 {
                for imaginary in imaginaries {
                    let z = Circle::<i8, i8> { real: real as i8, imaginary, exponent: exponent as i8 };
                    seen.store((exponent as u32) << 16 | (real as u32) << 8 | imaginary as u8 as u32, Ordering::Relaxed);
                    let s = format!("{}", z);
                    assert!(!s.is_empty());
                    count += 1;
                }
            }
        }
        count
    });
    let at = progress.load(Ordering::Relaxed);
    let formatted = done.unwrap_or_else(|| {
        panic!(
            "Display hung at exponent {:#04x}, real {:#04x}, imaginary {:#04x}",
            at >> 16,
            (at >> 8) & 0xFF,
            at & 0xFF
        )
    });
    assert_eq!(formatted, 256 * 256 * 8);
}
