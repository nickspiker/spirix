//! Bit-accurate Spirix v0.1 N0 add at FRAC=24, EXP=8, banker's rounding, compared against native IEEE 754 binary32 add.
//!
//! ## Format details
//!
//! Spirix v0.1 N0 stores a 24-bit fraction and an 8-bit exponent (32 bits total, bit-equivalent to binary32). The stored MSB encodes sign via implicit complement: a stored MSB of 1 reads as positive (conceptual pattern `01.xxx...`, magnitude in [1, 2)), and a stored MSB of 0 reads as negative (conceptual pattern `10.xxx...`, magnitude in [-2, -1) per two's complement).
//!
//! Internally this module operates on the full sign-extended 25-bit two's complement value (the "compute form" Q), not the 24-bit storage form. Internally Q is identical in both v0.0 N1 and v0.1 N0: the algorithm transfers verbatim, with only the AMB_EXP sentinel value and the wrap-detection direction differing. The 24-bit stored form is recoverable as `Q & 0xFF_FFFF` (lower 24 bits); the implicit complement of the stored MSB is bit 24 of Q.
//!
//! ## Ambiguous exponent
//!
//! `AMB_EXP = i8::MAX = 127` is the single sentinel exponent for non-normal states (zero, infinity, exploded, vanished, undefined). Both overflow past `i8::MAX - 1 = 126` and underflow past `i8::MIN = -128` saturate to AMB_EXP, so the saturation check is two range tests on the post-arithmetic exponent computed in a wider type. At AMB_EXP the storage convention switches from N0 normal (sign via implicit complement) to N-1-style explicit-sign-at-MSB classification: leading-same-bit count of 1 = exploded, 2 = vanished, 3+ = undefined; uniform `0` = zero, uniform `1` = infinity.

use spirix::Scalar;
use spirix_paper_comparison::{Counters, Lcg, ulp_diff_same_sign};

const FRAC: i32 = 24;             // storage fraction width (i24, "wonky" non-power-of-2)
const COMPUTE_FRAC: i32 = 25;     // compute width = FRAC + implicit complement bit; same as v0.0 N1's stored width
const EXP_BITS: i32 = 8;
const INT_BITS: i32 = COMPUTE_FRAC + 3; // 28: compute width + guard/round/sticky headroom
const AMB_EXP: i8 = i8::MAX;      // 127

const _: () = assert!(EXP_BITS == 8);

/// Outcome of an IEEE -> Spirix conversion.
#[derive(Debug, Clone, Copy)]
enum ConvOutcome {
    /// Clean conversion to a normal Spirix value: returns compute form (Q, exp).
    Normal { q: i32, exp: i8 },
    /// Conversion mapped to Spirix exploded (IEEE value too large for Spirix's normal range, e.g., IEEE biased_exp=254).
    Exploded,
    /// Conversion mapped to Spirix vanished (IEEE value too small, denormals below Spirix's smallest normal).
    Vanished,
    /// Conversion mapped to Spirix zero (IEEE ±0).
    Zero,
    /// Conversion mapped to Spirix undefined (IEEE NaN).
    Undefined,
}

// ── f32 -> Spirix v0.1 N0 conversion via the spirix lib ─────────────────────

/// Convert an f32 to Spirix v0.1 N0 form, using the production spirix lib for the IEEE->Scalar mapping (so denormal handling, ±∞ -> exploded, NaN -> undefined, ±0 -> zero, and the high-end conversion-loss-to-exploded boundary all match the lib's authoritative behavior).
///
/// The lib produces a `Scalar<i32, i8>` at FRAC=32. We bridge to our FRAC=24 compute form by truncating the lib's stored 32-bit fraction down to 24 bits via right-shift-by-8. This is bit-exact for f32 inputs because IEEE binary32 has only 24 effective fraction bits, so the lib's stored 32-bit fraction always has zeros in its lower 8 bits for any f32-sourced normal value.
fn f32_to_spirix_via_lib(v: f32) -> ConvOutcome {
    let s: Scalar<i32, i8> = v.into();

    if s.exponent == AMB_EXP {
        // Lib mapped to a non-normal Spirix state. Classify it from the storage pattern.
        let stored_32 = s.fraction as u32;
        if stored_32 == 0 {
            return ConvOutcome::Zero;
        }
        if stored_32 == 0xFFFF_FFFF {
            // Lib's infinity pattern at FRAC=32 — but f32 inputs cannot reach Spirix infinity (only arithmetic 1/0 etc. produces it), so this branch is theoretically unreachable from f32_to_spirix. Treat as zero just in case for safety.
            return ConvOutcome::Undefined;
        }
        // For non-uniform AMB patterns at FRAC=32, classify by leading-same-bit count on the 32-bit pattern.
        let lsbc = leading_same_bit_count_u32(stored_32, 32);
        if lsbc == 1 {
            return ConvOutcome::Exploded;
        }
        if lsbc == 2 {
            return ConvOutcome::Vanished;
        }
        return ConvOutcome::Undefined;
    }

    // Normal: truncate the lib's 32-bit stored fraction to our 24-bit storage form, then convert to compute Q.
    // Bit-exact for f32 inputs because IEEE binary32 has 24 effective bits -> lib's lower 8 bits are always zero.
    let stored_24 = (s.fraction as u32) >> 8;
    let msb = (stored_24 >> 23) & 1;
    let prefix = 1 - msb; // implicit complement of MSB reconstructs the (n+1)-bit two's complement value
    let q_25bit = (prefix << 24) | stored_24;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    ConvOutcome::Normal { q, exp: s.exponent }
}

/// Count the leading same-bit run length of `value`, examining the top `width` bits. Returns `width` for uniform patterns (all-zeros or all-ones).
fn leading_same_bit_count_u32(value: u32, width: u32) -> u32 {
    debug_assert!(width <= 32);
    // Construct mask carefully — `1u32 << 32` is undefined behavior on x86 (shift count masked to 5 bits, so equals `1u32 << 0 = 1`, NOT `0` as one might expect). Use u32::MAX for width=32 to sidestep.
    let mask = if width == 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    };
    let masked = value & mask;
    if masked == 0 || masked == mask {
        return width;
    }
    let msb = (masked >> (width - 1)) & 1;
    let mut count = 1u32;
    for i in (0..width - 1).rev() {
        let bit = (masked >> i) & 1;
        if bit == msb {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Convert Spirix v0.1 N0 compute form (Q, exp) back to f32. Handles normals; returns 0.0 for the zero pattern at AMB_EXP, ±∞ for the infinity pattern, NaN for any other AMB_EXP pattern (vanished, exploded, undefined — encoded as NaN here so the IEEE comparison classifier can detect them).
fn spirix_to_f32(q: i32, exp: i8) -> f32 {
    if exp == AMB_EXP {
        // Caller's harness inspects (q, exp) directly to classify non-normals. This conversion just returns a placeholder distinguishable from normal results.
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

/// Spirix v0.1 N0 add at FRAC=24, banker's rounding. Operates on compute form (Q, exp); returns same.
///
/// Algorithm structure:
///  1. Exponent difference + swap (so `big` has the larger exponent).
///  2. Negligible early exit if `exp_diff` exceeds compute precision.
///  3. Close path (exp_diff ≤ 1): align by 0 or 1 bit, sum, CLZ-normalize, round.
///  4. Far path (exp_diff ≥ 2): barrel-shift small with sticky, sum, bounded normalize (0/1/2 bits), round.
///  5. Banker's rounding (round-to-nearest-even) and rounding-overflow detection.
///  6. v0.1 saturation: both overflow past i8::MAX-1 and underflow past i8::MIN map to AMB_EXP, encoded as exploded (LSBC=1) or vanished (LSBC=2) by phase.
fn spirix_add(a_q: i32, a_exp: i8, b_q: i32, b_exp: i8) -> (i32, i8) {
    // Step 1: exponent difference + swap so `big` has the larger (or equal) exponent.
    let raw_diff = (a_exp as i16) - (b_exp as i16);
    let a_is_big = raw_diff >= 0;
    let (big_q, big_exp, small_q) = if a_is_big {
        (a_q, a_exp, b_q)
    } else {
        (b_q, b_exp, a_q)
    };
    let exp_diff = raw_diff.unsigned_abs() as i32;

    // Step 2: small operand contributes nothing (not even sticky) past compute precision.
    if exp_diff >= COMPUTE_FRAC {
        return (big_q, big_exp);
    }

    let is_close = exp_diff <= 1;

    // Step 3: extend to INT_BITS-bit signed integer with 2-bit left shift for binary-point headroom.
    let big_ext = (big_q as i64) << 2;
    let small_ext = (small_q as i64) << 2;

    let mask = (1i64 << INT_BITS) - 1;
    let sign_bit_int = 1i64 << (INT_BITS - 1);

    // Sign-extend a value to i64 from INT_BITS bits.
    let sext = |v: i64| -> i64 {
        let v = v & mask;
        if v & sign_bit_int != 0 {
            v | !mask
        } else {
            v
        }
    };

    let big_ext = sext(big_ext);
    let small_ext = sext(small_ext);

    // Constants for rovf-corrected fraction outputs.
    let pos_half: i32 = 1 << (COMPUTE_FRAC - 2);   // +0.5 in compute LSBs (post-rovf positive)
    let neg_one: i32 = -(1 << (COMPUTE_FRAC - 1)); // -1.0 in compute LSBs (post-rovf negative)

    // ── Path bodies ──────────────────────────────────────────────────────
    let (out_q, exp_wide) = if is_close {
        // Close path: |Δexp| ≤ 1, may totally cancel.
        let close_small = if (exp_diff & 1) != 0 {
            sext(small_ext >> 1)
        } else {
            small_ext
        };
        let close_align_sticky = (exp_diff & 1) != 0 && (small_ext & 1) != 0;
        let close_sum = sext(big_ext + close_small);

        if close_sum == 0 {
            // Exact cancellation → zero singularity.
            return (0, AMB_EXP);
        }

        // Count leading sign bits (positive: leading 0s; negative: leading 1s).
        let ubits = (close_sum & mask) as u32;
        let top_bit = (close_sum >> (INT_BITS - 1)) & 1;
        let leading = if top_bit != 0 {
            let inverted = (!ubits) & (mask as u32);
            inverted.leading_zeros() as i32 - (32 - INT_BITS)
        } else {
            ubits.leading_zeros() as i32 - (32 - INT_BITS)
        };

        let close_norm_shift = leading - 1;
        let close_normalized = sext(close_sum << close_norm_shift);

        let (out_q_raw, round_up) = round_banker(close_normalized, close_align_sticky);
        let (rovf_pos, rovf_neg) = rovf_detect(out_q_raw, round_up);

        let out_q = if rovf_pos {
            pos_half
        } else if rovf_neg {
            neg_one
        } else {
            out_q_raw + i32::from(round_up)
        };

        let exp_wide = (big_exp as i16) + 2 - (leading as i16)
            + i16::from(rovf_pos)
            - i16::from(rovf_neg);

        (out_q, exp_wide)
    } else {
        // Far path: |Δexp| ≥ 2, right-shift smaller operand with sticky tracking, then add.
        let far_shift = if exp_diff >= INT_BITS {
            (INT_BITS - 1) as u32
        } else {
            exp_diff as u32
        };

        let mut shifted = small_ext;
        let mut barrel_sticky = false;
        for i in 0..5u32 {
            if (far_shift >> i) & 1 != 0 {
                let amount = 1u32 << i;
                let lost_mask = (1i64 << amount) - 1;
                barrel_sticky = barrel_sticky || (shifted & lost_mask) != 0;
                shifted = sext(shifted >> amount);
            }
        }

        let far_aligned = shifted;
        let far_sum = sext(big_ext + far_aligned);

        if far_sum == 0 {
            return (0, AMB_EXP);
        }

        // Bounded normalize: 0, 1, or 2 bit left shift based on top three bits.
        let bit_n1 = (far_sum >> (INT_BITS - 1)) & 1;
        let bit_n2 = (far_sum >> (INT_BITS - 2)) & 1;
        let bit_n3 = (far_sum >> (INT_BITS - 3)) & 1;
        let far_d0 = bit_n1 != bit_n2;
        let far_d1 = bit_n2 != bit_n3;
        let far_norm_shift: i32 = if far_d0 {
            0
        } else if far_d1 {
            1
        } else {
            2
        };
        let far_leading = far_norm_shift + 1;

        let far_normalized = sext(far_sum << far_norm_shift);

        let (out_q_raw, round_up) = round_banker(far_normalized, barrel_sticky);
        let (rovf_pos, rovf_neg) = rovf_detect(out_q_raw, round_up);

        let out_q = if rovf_pos {
            pos_half
        } else if rovf_neg {
            neg_one
        } else {
            out_q_raw + i32::from(round_up)
        };

        let exp_wide = (big_exp as i16) + 2 - (far_leading as i16)
            + i16::from(rovf_pos)
            - i16::from(rovf_neg);

        (out_q, exp_wide)
    };

    // ── v0.1 saturation: both overflow and underflow → AMB_EXP ───────────
    if exp_wide >= AMB_EXP as i16 {
        // Overflow → exploded (LSBC=1, phase from sign of out_q).
        let frac = if out_q < 0 { 0x80_0000 } else { 0x40_0000 };
        return (frac, AMB_EXP);
    }
    if exp_wide < i8::MIN as i16 {
        // Underflow → vanished (LSBC=2, phase from sign of out_q).
        let frac = if out_q < 0 { 0xC0_0000_u32 as i32 } else { 0x20_0000 };
        return (frac, AMB_EXP);
    }

    (out_q, exp_wide as i8)
}

/// Banker's rounding (round-to-nearest-even). Returns the un-rounded fraction (extracted from `normalized`) and the round-up bit.
fn round_banker(normalized: i64, align_sticky: bool) -> (i32, bool) {
    let out_q_raw = (normalized >> (INT_BITS - COMPUTE_FRAC)) as i32;
    let guard = ((normalized >> (INT_BITS - 1 - COMPUTE_FRAC)) & 1) != 0;
    let lsb = ((normalized >> (INT_BITS - COMPUTE_FRAC)) & 1) != 0;
    let round_bit = ((normalized >> (INT_BITS - 2 - COMPUTE_FRAC)) & 1) != 0;
    let ext_sticky_mask = (1i64 << (INT_BITS - 3 - COMPUTE_FRAC)) - 1;
    let ext_sticky = if INT_BITS - 3 - COMPUTE_FRAC > 0 {
        (normalized & ext_sticky_mask) != 0
    } else {
        false
    };
    let sticky = ext_sticky || align_sticky;
    // Round up if guard is set AND (round_bit OR sticky OR lsb). The lsb term implements the "to-even" tie-break.
    let round_up = guard && (round_bit || sticky || lsb);
    (out_q_raw, round_up)
}

/// Detect rounding-overflow (rovf): the case where round-up would push the fraction past its representable range, requiring an exponent bump.
fn rovf_detect(out_q_raw: i32, round_up: bool) -> (bool, bool) {
    let pos_all_ones = (out_q_raw & ((1 << (COMPUTE_FRAC - 1)) - 1)) == ((1 << (COMPUTE_FRAC - 1)) - 1);
    let rovf_pos = (out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 0 && pos_all_ones && round_up;
    let rovf_neg = ((out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 1)
        && ((out_q_raw >> (COMPUTE_FRAC - 2)) & 1 == 0)
        && ((out_q_raw & ((1 << (COMPUTE_FRAC - 2)) - 1)) == (1 << (COMPUTE_FRAC - 2)) - 1)
        && round_up;
    (rovf_pos, rovf_neg)
}

// ── Comparison harness ──────────────────────────────────────────────────────

fn main() {
    const N: u64 = 10_000_000;
    println!(
        "Spirix v0.1 N0 add (FRAC={FRAC}, EXP={EXP_BITS}, banker's rounding) vs IEEE 754 binary32 — {N} trials"
    );
    println!("Inputs: full IEEE normal range (biased_exp in [1, 254]); non-normals (NaN, ±∞, ±0, denormals) are skipped pending non-normal-input support in spirix_add.");
    println!();

    let mut rng = Lcg::new(0xDEAD_BEEF_CAFE_1234);
    let mut counters = Counters::default();

    for _ in 0..N {
        // Generate two random f32 normals across the full IEEE normal range (biased_exp ∈ [1, 254]). Non-normals (NaN, ±∞, ±0, denormals) are filtered here because spirix_add at this iteration only handles normal compute-form (Q, exp) inputs.
        let a = next_normal_f32_full_range(&mut rng);
        let b = next_normal_f32_full_range(&mut rng);
        let ieee_result = a + b;

        // Convert via the spirix lib. If either input maps to a non-normal Spirix state during conversion (e.g., IEEE biased_exp=254 → Spirix exploded because Spirix's max normal exp is 126), record the conversion-loss category and skip — spirix_add doesn't yet handle non-normal inputs.
        let a_outcome = f32_to_spirix_via_lib(a);
        let b_outcome = f32_to_spirix_via_lib(b);
        let ((a_q, a_exp), (b_q, b_exp)) = match (a_outcome, b_outcome) {
            (ConvOutcome::Normal { q: aq, exp: ae }, ConvOutcome::Normal { q: bq, exp: be }) => {
                ((aq, ae), (bq, be))
            }
            _ => {
                for outcome in [a_outcome, b_outcome] {
                    match outcome {
                        ConvOutcome::Exploded => counters.record_conversion_loss_to_exploded(),
                        ConvOutcome::Vanished => counters.record_conversion_loss_to_vanished(),
                        ConvOutcome::Zero | ConvOutcome::Undefined | ConvOutcome::Normal { .. } => {}
                    }
                }
                continue;
            }
        };

        let (r_q, r_exp) = spirix_add(a_q, a_exp, b_q, b_exp);

        // Detect Spirix non-normal results before converting to f32, so we classify architectural distinctions explicitly.
        if r_exp == AMB_EXP {
            let stored = (r_q as u32) & 0xFF_FFFF;

            if stored == 0 {
                if ieee_result == 0.0 {
                    counters.record_exact();
                } else {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
                continue;
            }
            if stored == 0xFF_FFFF {
                if ieee_result.is_infinite() {
                    counters.record_spirix_exploded_ieee_inf();
                } else {
                    counters.record_spirix_exploded_ieee_finite();
                }
                continue;
            }

            let lsbc = leading_same_bit_count(stored);
            if lsbc == 1 {
                if ieee_result.is_infinite() {
                    counters.record_spirix_exploded_ieee_inf();
                } else {
                    counters.record_spirix_exploded_ieee_finite();
                }
                continue;
            }
            if lsbc == 2 {
                if ieee_result == 0.0 {
                    counters.record_spirix_vanished_ieee_zero();
                } else if is_denormal(ieee_result) {
                    counters.record_spirix_vanished_ieee_denormal();
                } else {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
                continue;
            }
            counters.record_off_by_more(u64::MAX, a, b);
            continue;
        }

        // Normal result — ULP-compare against IEEE.
        let spirix_result = spirix_to_f32(r_q, r_exp);

        if ieee_result == 0.0 && spirix_result == 0.0 {
            counters.record_exact();
            continue;
        }
        if ieee_result == 0.0 || spirix_result == 0.0 {
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

/// Generate a random normal f32 across the full IEEE normal range (biased_exp ∈ [1, 254]). Skips non-normals (biased_exp == 0 for ±0/denormals, biased_exp == 255 for ±∞/NaN).
fn next_normal_f32_full_range(rng: &mut Lcg) -> f32 {
    loop {
        let bits = rng.next_u32();
        let biased_exp = (bits >> 23) & 0xFF;
        if (1..=254).contains(&biased_exp) {
            return f32::from_bits(bits);
        }
    }
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
