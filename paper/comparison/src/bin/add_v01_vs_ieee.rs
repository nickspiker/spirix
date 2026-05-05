//! Bit-accurate Spirix v0.1 N0 add at FRAC=24, EXP=8, banker's rounding, compared against native IEEE 754 binary32 add.
//!
//! ## Format details
//!
//! Spirix v0.1 N0 stores a 24-bit fraction and an 8-bit exponent (32 bits total, bit-equivalent to binary32). The stored MSB encodes sign via implicit complement: a stored MSB of 1 reads as positive (conceptual pattern `01.xxx...`, magnitude in [1, 2)), and a stored MSB of 0 reads as negative (conceptual pattern `10.xxx...`, magnitude in [-2, -1) per two's complement).
//!
//! Internally this module operates on the full sign-extended 25-bit two's complement value (the "compute form" Q), not the 24-bit storage form, because the algorithm is cleaner when sign-extension and arithmetic are explicit. The conversions at the boundaries handle the storage-form / compute-form translation:
//!
//! - `f32_to_spirix` produces the compute form Q
//! - `spirix_add` consumes and produces the compute form Q
//! - `spirix_to_f32` consumes the compute form Q
//!
//! The 24-bit stored form is recoverable as `Q & 0xFF_FFFF` (lower 24 bits); the implicit complement of the stored MSB is bit 24 of Q.
//!
//! ## Ambiguous exponent
//!
//! `AMB_EXP = i8::MAX = 127` is the single sentinel exponent for non-normal states (zero, infinity, exploded, vanished, undefined). Both overflow past `i8::MAX - 1` and underflow past `i8::MIN` wrap (through the two's complement circle) to `AMB_EXP`, so the saturation check is a single equality test on the post-arithmetic exponent.

use spirix_paper_comparison::{Counters, Lcg, ulp_diff_same_sign};

const FRAC: i32 = 24;
const EXP_BITS: i32 = 8;
const _: () = assert!(EXP_BITS == 8); // keeps the linter happy and pins assumption
const AMB_EXP: i8 = i8::MAX; // 127

// ── f32 ↔ Spirix v0.1 N0 conversion ─────────────────────────────────────────

/// Convert a normal, finite f32 to Spirix v0.1 N0 compute form (Q, exp). Caller is expected to filter out subnormals, ±0, ±∞, NaN — this routine doesn't model those mappings.
///
/// IEEE binary32 normal value:   v = (-1)^sign * (1 + mantissa/2^23) * 2^(biased_exp - 127)
/// Spirix v0.1 normal value:    v = Q / 2^23 * 2^exp  with Q ∈ [+2^23, +2^24-1] ∪ [-2^24, -2^23-1]
///
/// For a positive f32, `Q = 2^23 + mantissa` (in [+2^23, +2^24-1]) and `exp = biased_exp - 127`.
///
/// For a negative f32 with `mantissa > 0`, `Q = -(2^23 + mantissa)` (in (-2^24, -2^23-1]) and `exp = biased_exp - 127`.
///
/// For a negative f32 with `mantissa = 0` (significand exactly 1.0): the magnitude -1.0 is **not** directly representable in Spirix's negative range [-2, -1), so we use the equivalent `(-2.0) * 2^(exp - 1) = -1.0 * 2^exp`. That is, `Q = -2^24`, `exp = biased_exp - 127 - 1`.
fn f32_to_spirix(v: f32) -> (i32, i8) {
    let bits = v.to_bits();
    let sign = (bits >> 31) != 0;
    let biased_exp = ((bits >> 23) & 0xFF) as i32;
    let mantissa = (bits & 0x7F_FFFF) as i32;

    debug_assert!(biased_exp != 0 && biased_exp != 255, "non-normal f32 passed to f32_to_spirix");

    let exp_unbiased = biased_exp - 127;

    if !sign {
        // Positive
        let q = (1 << 23) | mantissa; // in [+2^23, +2^24-1]
        (q, exp_unbiased as i8)
    } else if mantissa == 0 {
        // Negative significand = 1.0 exactly: shift down one exponent and use Q = -2^24
        (-(1 << 24), (exp_unbiased - 1) as i8)
    } else {
        // Negative significand > 1.0
        let q = -((1 << 23) | mantissa); // in (-2^24, -2^23-1]
        (q, exp_unbiased as i8)
    }
}

/// Convert Spirix v0.1 N0 compute form (Q, exp) back to f32. Handles normals; returns 0.0 for the zero pattern at AMB_EXP, ±∞ for the infinity pattern, NaN for any other AMB_EXP pattern (vanished, exploded, undefined — encoded as NaN here so the IEEE comparison classifier can detect them).
fn spirix_to_f32(q: i32, exp: i8) -> f32 {
    if exp == AMB_EXP {
        // Caller's harness inspects (q, exp) directly to classify non-normals. This conversion just returns a placeholder that's distinguishable from normal results.
        let stored = (q as u32) & 0xFF_FFFF;
        return if stored == 0 {
            0.0
        } else if stored == 0xFF_FFFF {
            f32::INFINITY
        } else {
            f32::NAN
        };
    }

    // Normal: Q ∈ [+2^23, +2^24-1] ∪ [-2^24, -2^23-1].
    let abs_q = q.unsigned_abs() as u64;
    let sign_neg = q < 0;

    // Position of the highest 1-bit in abs_q — for normals this is 23 (positive case)
    // or 24 (negative case where Q = -2^k).
    let leading_pos = 63 - abs_q.leading_zeros() as i32;
    let shift = leading_pos - 23; // align to IEEE's implicit-1 position

    let aligned = if shift >= 0 {
        abs_q >> shift
    } else {
        abs_q << (-shift)
    };

    let mantissa = (aligned & 0x7F_FFFF) as u32;
    let biased_exp = exp as i32 + 127 + shift;

    if biased_exp <= 0 {
        return if sign_neg { -0.0 } else { 0.0 };
    }
    if biased_exp >= 255 {
        return if sign_neg { f32::NEG_INFINITY } else { f32::INFINITY };
    }

    let bits = ((sign_neg as u32) << 31) | ((biased_exp as u32) << 23) | mantissa;
    f32::from_bits(bits)
}

// ── Bit-accurate Spirix v0.1 N0 add ─────────────────────────────────────────

/// Spirix v0.1 N0 add at FRAC=24, banker's rounding. Returns the compute-form result (Q, exp).
///
/// **Stub for now** — the algorithm port from the v0.0 N1 reference (`examples/ieee_add_f32.rs` in the parent crate) is the next phase of this binary's development.
#[allow(dead_code, unused_variables)]
fn spirix_add(a_q: i32, a_exp: i8, b_q: i32, b_exp: i8) -> (i32, i8) {
    // TODO: port the close/far split + banker's rounding + rovf detection algorithm from the v0.0 N1 bit-accurate Verilog port at examples/ieee_add_f32.rs in the parent spirix crate, adapted for FRAC=24 N0 storage and AMB_EXP=i8::MAX wrap-detection semantics.
    unimplemented!("spirix_add: v0.1 N0 algorithm port pending")
}

// ── Comparison harness ──────────────────────────────────────────────────────

fn main() {
    const N: u64 = 10_000_000;
    println!(
        "Spirix v0.1 N0 add (FRAC={FRAC}, EXP={EXP_BITS}, banker's rounding) vs IEEE 754 binary32 — {N} trials"
    );
    println!();

    let mut rng = Lcg::new(0xDEAD_BEEF_CAFE_1234);
    let mut counters = Counters::default();

    for _ in 0..N {
        let a = rng.next_normal_f32();
        let b = rng.next_normal_f32();
        let ieee_result = a + b;

        let (a_q, a_exp) = f32_to_spirix(a);
        let (b_q, b_exp) = f32_to_spirix(b);
        let (r_q, r_exp) = spirix_add(a_q, a_exp, b_q, b_exp);

        // Detect Spirix non-normal results before converting to f32, so we can classify architectural distinctions.
        if r_exp == AMB_EXP {
            let stored = (r_q as u32) & 0xFF_FFFF;
            let leading_same_bits = leading_same_bit_count(stored);

            if stored == 0 {
                // Spirix → exact zero. IEEE side: 0 is exact iff a + b == 0.
                if ieee_result == 0.0 {
                    counters.record_exact();
                } else {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
                continue;
            }
            if stored == 0xFF_FFFF {
                // Spirix → infinity. IEEE side: infinity iff overflow.
                if ieee_result.is_infinite() {
                    counters.record_spirix_exploded_ieee_inf();
                } else {
                    counters.record_spirix_exploded_ieee_finite();
                }
                continue;
            }
            if leading_same_bits == 1 {
                // Exploded — IEEE expected to have overflowed.
                if ieee_result.is_infinite() {
                    counters.record_spirix_exploded_ieee_inf();
                } else {
                    counters.record_spirix_exploded_ieee_finite();
                }
                continue;
            }
            if leading_same_bits == 2 {
                // Vanished — IEEE expected to be denormal or zero.
                if ieee_result == 0.0 {
                    counters.record_spirix_vanished_ieee_zero();
                } else if is_denormal(ieee_result) {
                    counters.record_spirix_vanished_ieee_denormal();
                } else {
                    // Spirix vanished but IEEE produced a normal result — unexpected.
                    counters.record_off_by_more(u64::MAX, a, b);
                }
                continue;
            }
            // Undefined or unexpected non-normal pattern.
            counters.record_off_by_more(u64::MAX, a, b);
            continue;
        }

        // Spirix normal result — convert and ULP-compare against IEEE.
        let spirix_result = spirix_to_f32(r_q, r_exp);

        if ieee_result == 0.0 && spirix_result == 0.0 {
            counters.record_exact();
            continue;
        }
        if ieee_result == 0.0 || spirix_result == 0.0 {
            // One is zero, one isn't — should be very rare for normal+normal, treat as off-by-more.
            counters.record_off_by_more(u64::MAX, a, b);
            continue;
        }

        let diff = ulp_diff_same_sign(ieee_result, spirix_result);
        if diff == 0 {
            counters.record_exact();
        } else if diff == 1 {
            counters.record_one_ulp();
        } else {
            counters.record_off_by_more(diff, a, b);
        }
    }

    counters.print_summary(N);
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Count the leading same-bit run length in the lower 24 bits of `stored`. Used to classify Spirix non-normal states at the ambiguous exponent: 1 = exploded, 2 = vanished, 3+ = undefined. Returns FRAC for the uniform patterns (zero, infinity).
fn leading_same_bit_count(stored: u32) -> i32 {
    let stored = stored & 0xFF_FFFF;
    if stored == 0 || stored == 0xFF_FFFF {
        return FRAC;
    }
    let msb = (stored >> (FRAC - 1)) & 1;
    let mut count = 1;
    for i in (0..FRAC - 1).rev() {
        let bit = (stored >> i) & 1;
        if bit == msb {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Returns true if `v` is a denormal (subnormal) IEEE 754 binary32 value.
fn is_denormal(v: f32) -> bool {
    let bits = v.to_bits();
    let biased_exp = (bits >> 23) & 0xFF;
    let mantissa = bits & 0x7F_FFFF;
    biased_exp == 0 && mantissa != 0
}
