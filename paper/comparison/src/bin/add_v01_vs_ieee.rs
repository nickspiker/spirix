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

/// State of a Spirix Scalar — every bit pattern maps to exactly one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpirixState {
    Normal,
    Zero,
    Vanished,
    Exploded,
    Infinity,
    Undefined,
}

/// Operand carried through spirix_add. For Normal state, `q` is the compute-form Q (sign-extended 25-bit) and `exp` is the unbiased exponent. For non-normal states (exp == AMB_EXP), `q` carries the storage pattern in its lower 24 bits.
#[derive(Debug, Clone, Copy)]
struct Operand {
    q: i32,
    exp: i8,
    state: SpirixState,
}

/// Classify a Spirix Scalar from its (q, exp) storage. Used after IEEE->Spirix conversion and at the output of arithmetic.
fn classify_state(q: i32, exp: i8) -> SpirixState {
    if exp != AMB_EXP {
        return SpirixState::Normal;
    }
    let stored = (q as u32) & 0xFF_FFFF;
    if stored == 0 {
        return SpirixState::Zero;
    }
    if stored == 0xFF_FFFF {
        return SpirixState::Infinity;
    }
    let lsbc = leading_same_bit_count(stored);
    if lsbc == 1 {
        return SpirixState::Exploded;
    }
    if lsbc == 2 {
        return SpirixState::Vanished;
    }
    SpirixState::Undefined
}

/// Canonical undefined storage pattern (LSBC ≥ 3). The specific cause-encoding of undefined doesn't matter for the architectural-match categorization in this comparison; any LSBC≥3 pattern reads as undefined.
const UNDEFINED_CANONICAL: i32 = 0x10_0000;

/// IEEE 754 binary32 result kind for categorization purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IeeeKind {
    Normal,
    Zero,
    Denormal,
    Inf,
    Nan,
}

fn classify_ieee(v: f32) -> IeeeKind {
    if v.is_nan() {
        IeeeKind::Nan
    } else if v.is_infinite() {
        IeeeKind::Inf
    } else if v == 0.0 {
        IeeeKind::Zero
    } else if is_denormal(v) {
        IeeeKind::Denormal
    } else {
        IeeeKind::Normal
    }
}

// ── f32 -> Spirix v0.1 N0 conversion via the spirix lib ─────────────────────

/// Convert an f32 to a Spirix Operand at FRAC=24, using the spirix lib for the IEEE→Scalar mapping (so denormal handling, ±∞ → exploded, NaN → undefined, ±0 → zero, and the high-end conversion-loss-to-exploded boundary all match the lib's authoritative behavior). Returns an Operand carrying the Spirix state plus the (q, exp) storage suitable for spirix_add's dispatcher.
///
/// The lib produces a `Scalar<i32, i8>` at FRAC=32. We bridge to our FRAC=24 compute form by truncating the lib's stored 32-bit fraction down 8 bits — bit-exact for f32 inputs because IEEE binary32 has only 24 effective fraction bits, so the lib's lower 8 stored bits are always zero for f32-sourced normals.
fn f32_to_spirix_via_lib(v: f32) -> Operand {
    let s: Scalar<i32, i8> = v.into();

    if s.exponent == AMB_EXP {
        // Lib mapped to a non-normal Spirix state. Classify from the 32-bit storage pattern, then translate the storage to our 24-bit form.
        let stored_32 = s.fraction as u32;
        let stored_24 = (stored_32 >> 8) as i32;
        let state = if stored_32 == 0 {
            SpirixState::Zero
        } else if stored_32 == 0xFFFF_FFFF {
            SpirixState::Infinity
        } else {
            let lsbc = leading_same_bit_count_u32(stored_32, 32);
            match lsbc {
                1 => SpirixState::Exploded,
                2 => SpirixState::Vanished,
                _ => SpirixState::Undefined,
            }
        };
        return Operand { q: stored_24, exp: AMB_EXP, state };
    }

    // Normal: truncate the lib's 32-bit stored fraction to our 24-bit storage form, then convert to compute Q.
    let stored_24 = (s.fraction as u32) >> 8;
    let msb = (stored_24 >> 23) & 1;
    let prefix = 1 - msb;
    let q_25bit = (prefix << 24) | stored_24;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    Operand { q, exp: s.exponent, state: SpirixState::Normal }
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

/// Convert Spirix v0.1 N0 compute form (Q, exp) back to f32 using the spirix lib's to_f32. Bridges our FRAC=24 form to the lib's Scalar<i32, i8> at FRAC=32 by left-shifting the 24-bit storage to occupy the upper 24 bits of the 32-bit fraction (zero-padding the lower 8 bits, which is bit-exact since Spirix at FRAC=24 has no information below those bits).
fn spirix_to_f32(q: i32, exp: i8) -> f32 {
    let stored_24 = if exp == AMB_EXP {
        // Non-normal: q already carries the storage pattern in its lower 24 bits.
        (q as u32) & 0xFF_FFFF
    } else {
        // Normal: extract the 24-bit storage form from compute Q (drop the implicit complement bit, keep lower 24).
        (q as u32) & 0xFF_FFFF
    };
    let stored_32 = if exp == AMB_EXP && stored_24 == 0xFF_FFFF {
        // Spirix infinity at FRAC=24 (uniform 24-bit ones) corresponds to lib's infinity at FRAC=32 (uniform 32-bit ones).
        0xFFFF_FFFFu32
    } else {
        // For all other patterns (normals + zero/exploded/vanished/undefined), left-shift-by-8 expands the 24-bit storage to 32-bit storage. The lower 8 bits stay zero (bit-exact: Spirix at FRAC=24 carries no info there).
        stored_24 << 8
    };
    let s = Scalar::<i32, i8> {
        fraction: stored_32 as i32,
        exponent: exp,
    };
    s.to_f32()
}

// ── Bit-accurate Spirix v0.1 N0 add ─────────────────────────────────────────

/// Spirix v0.1 N0 add at FRAC=24. Dispatches by operand state, then runs the bit-accurate normal+normal algorithm or the truth-table rule for non-normal combinations.
///
/// Truth table (matches src/implementations/addition/scalar_scalar.rs in the spirix crate):
///  - Both normal → bit-accurate add (close/far split, banker's rounding, rovf detection)
///  - Either undefined → propagate first undefined
///  - Either zero → other operand (zero is true additive identity, even for transfinite — matches the lib's note at scalar_scalar.rs:198)
///  - Both transfinite (exploded/infinity) → undefined "transfinite + transfinite"
///  - Both vanished → undefined "vanished + vanished"
///  - Transfinite + finite (normal) → undefined "transfinite + finite" (Spirix is more conservative than IEEE here: IEEE's ∞+5=∞, Spirix says undefined because the singular point doesn't admit relative arithmetic)
///  - Vanished + normal → return the normal (vanished passes through as below precision)
fn spirix_add(a: Operand, b: Operand) -> (i32, i8) {
    use SpirixState::*;
    match (a.state, b.state) {
        (Normal, Normal) => spirix_add_normal_normal(a.q, a.exp, b.q, b.exp),
        (Undefined, _) => (a.q, a.exp),
        (_, Undefined) => (b.q, b.exp),
        (Zero, _) => (b.q, b.exp),
        (_, Zero) => (a.q, a.exp),
        // Both transfinite (exploded ∪ infinity)
        (Exploded | Infinity, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        // Both vanished
        (Vanished, Vanished) => (UNDEFINED_CANONICAL, AMB_EXP),
        // One transfinite, one finite (normal)
        (Exploded | Infinity, _) | (_, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        // Vanished + normal: vanished passes through
        (Vanished, _) => (b.q, b.exp),
        (_, Vanished) => (a.q, a.exp),
    }
}

/// Bit-accurate Spirix v0.1 N0 add for normal+normal operands at FRAC=24, banker's rounding.
///
/// Algorithm structure:
///  1. Exponent difference + swap (so `big` has the larger exponent).
///  2. Negligible early exit if `exp_diff` exceeds compute precision.
///  3. Close path (exp_diff ≤ 1): align by 0 or 1 bit, sum, CLZ-normalize, round.
///  4. Far path (exp_diff ≥ 2): barrel-shift small with sticky, sum, bounded normalize (0/1/2 bits), round.
///  5. Banker's rounding and rounding-overflow detection.
///  6. v0.1 saturation: both overflow past i8::MAX-1 and underflow past i8::MIN map to AMB_EXP, encoded as exploded (LSBC=1) or vanished (LSBC=2) by phase.
fn spirix_add_normal_normal(a_q: i32, a_exp: i8, b_q: i32, b_exp: i8) -> (i32, i8) {
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
///
/// Bit layout below the LSB of the output fraction:
/// - bit (INT_BITS - 1 - COMPUTE_FRAC): guard
/// - bit (INT_BITS - 2 - COMPUTE_FRAC): round
/// - bits (INT_BITS - 2 - COMPUTE_FRAC - 1) .. 0: sticky region (OR of all)
///
/// Note: an earlier version of this routine (and the v0.0 N1 port at `examples/ieee_add_f32.rs` in the parent crate) used `INT_BITS - 3 - COMPUTE_FRAC` for the sticky mask shift, which is **off by one** — at FRAC=24 / INT_BITS=28 that mask captures zero bits and the LSB of `normalized` is silently dropped from the sticky calculation. The result is a systematic ~0.025% downward 1-ULP bias against IEEE for far-path same-sign adds. This was the source of the "1 ULP rounding choice" cases reported against IEEE binary32.
fn round_banker(normalized: i64, align_sticky: bool) -> (i32, bool) {
    let out_q_raw = (normalized >> (INT_BITS - COMPUTE_FRAC)) as i32;
    let guard = ((normalized >> (INT_BITS - 1 - COMPUTE_FRAC)) & 1) != 0;
    let lsb = ((normalized >> (INT_BITS - COMPUTE_FRAC)) & 1) != 0;
    let round_bit = ((normalized >> (INT_BITS - 2 - COMPUTE_FRAC)) & 1) != 0;
    let ext_sticky_mask = (1i64 << (INT_BITS - 2 - COMPUTE_FRAC)) - 1;
    let ext_sticky = (normalized & ext_sticky_mask) != 0;
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
        "Spirix v0.1 N0 add (FRAC={FRAC}, EXP={EXP_BITS}, banker's rounding) vs IEEE 754 binary32"
    );
    println!();

    // ── Phase A: IEEE-driven random ──────────────────────────────────────
    println!("== Phase A: IEEE-driven random — {N} trials ==");
    println!("Inputs: full random IEEE 754 binary32 bit patterns (every f32 bit pattern is a valid IEEE value). Lib's from(f32) maps each to its corresponding valid Spirix state.");
    println!();
    let mut rng_a = Lcg::new(0xDEAD_BEEF_CAFE_1234);
    let mut counters_a = Counters::default();
    for _ in 0..N {
        let a = f32::from_bits(rng_a.next_u32());
        let b = f32::from_bits(rng_a.next_u32());
        let a_op = f32_to_spirix_via_lib(a);
        let b_op = f32_to_spirix_via_lib(b);
        for op in [a_op, b_op] {
            match op.state {
                SpirixState::Exploded => counters_a.record_conversion_loss_to_exploded(),
                SpirixState::Vanished => counters_a.record_conversion_loss_to_vanished(),
                _ => {}
            }
        }
        let ieee_result = a + b;
        categorize_and_record(a, b, ieee_result, a_op, b_op, &mut counters_a);
    }
    counters_a.print_summary(N);

    println!();

    // ── Phase B: Spirix-driven random ────────────────────────────────────
    println!("== Phase B: Spirix-driven random — {N} trials ==");
    println!("Inputs: full random Spirix bit patterns (random 24-bit fraction + random i8 exponent — every bit pattern is a valid Spirix state by design). IEEE-side reference computed via the lib's to_f32 (signless infinity → NaN; vanished → ±0 with phase; undefined → NaN; exploded → ±∞ with phase; otherwise precise).");
    println!();
    let mut rng_b = Lcg::new(0xCAFE_BABE_DEAD_5678);
    let mut counters_b = Counters::default();
    for _ in 0..N {
        let a_op = random_spirix_operand(&mut rng_b);
        let b_op = random_spirix_operand(&mut rng_b);
        // Convert each Spirix operand to its f32 representative via the lib for the IEEE-side reference.
        let a = spirix_to_f32(a_op.q, a_op.exp);
        let b = spirix_to_f32(b_op.q, b_op.exp);
        let ieee_result = a + b;
        categorize_and_record(a, b, ieee_result, a_op, b_op, &mut counters_b);
    }
    counters_b.print_summary(N);
}

/// Generate a random valid Spirix operand. Random 24-bit fraction + random 8-bit exponent — every bit pattern is a valid Spirix state by design.
fn random_spirix_operand(rng: &mut Lcg) -> Operand {
    let stored_24 = (rng.next_u32() & 0xFF_FFFF) as i32;
    let exp_byte = rng.next_u32() as u8;
    let exp = exp_byte as i8;

    if exp == AMB_EXP {
        let state = classify_state(stored_24, AMB_EXP);
        return Operand { q: stored_24, exp: AMB_EXP, state };
    }

    // Normal Spirix: convert 24-bit storage to compute-form Q.
    let stored_24_u = stored_24 as u32 & 0xFF_FFFF;
    let msb = (stored_24_u >> 23) & 1;
    let prefix = 1 - msb;
    let q_25bit = (prefix << 24) | stored_24_u;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    Operand { q, exp, state: SpirixState::Normal }
}

/// Categorize a single (a, b) → result pair against IEEE and update counters. Used by both Phase A (IEEE-driven) and Phase B (Spirix-driven) — categorization is symmetric in input direction.
fn categorize_and_record(
    a: f32,
    b: f32,
    ieee_result: f32,
    a_op: Operand,
    b_op: Operand,
    counters: &mut Counters,
) {
    let (r_q, r_exp) = spirix_add(a_op, b_op);
    let r_state = classify_state(r_q, r_exp);
    let ieee_kind = classify_ieee(ieee_result);

    let inputs_had_arch_input = a_op.state != SpirixState::Normal
        || b_op.state != SpirixState::Normal
        || classify_ieee(a) != IeeeKind::Normal
        || classify_ieee(b) != IeeeKind::Normal;

    match r_state {
        SpirixState::Normal => {
            let spirix_result = spirix_to_f32(r_q, r_exp);
            match ieee_kind {
                IeeeKind::Normal => {
                    if ieee_result == 0.0 && spirix_result == 0.0 {
                        counters.record_exact();
                    } else if ieee_result == 0.0 || spirix_result == 0.0 {
                        counters.record_off_by_more(u64::MAX, a, b);
                    } else {
                        let diff = ulp_diff_same_sign(ieee_result, spirix_result);
                        if diff == 0 {
                            counters.record_exact();
                        } else if inputs_had_arch_input {
                            counters.record_spirix_normal_arch_drift();
                        } else if diff == 1 {
                            counters.record_one_ulp();
                        } else {
                            counters.record_off_by_more(diff, a, b);
                        }
                    }
                }
                IeeeKind::Zero | IeeeKind::Denormal => {
                    counters.record_spirix_normal_arch_drift();
                }
                IeeeKind::Inf | IeeeKind::Nan => {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
            }
        }
        SpirixState::Zero => match ieee_kind {
            IeeeKind::Zero => counters.record_spirix_zero_ieee_zero(),
            _ => counters.record_spirix_zero_ieee_other(),
        },
        SpirixState::Vanished => match ieee_kind {
            IeeeKind::Zero => counters.record_spirix_vanished_ieee_zero(),
            IeeeKind::Denormal => counters.record_spirix_vanished_ieee_denormal(),
            _ => counters.record_spirix_vanished_ieee_other(),
        },
        SpirixState::Exploded => match ieee_kind {
            IeeeKind::Inf => counters.record_spirix_exploded_ieee_inf(),
            IeeeKind::Normal => counters.record_spirix_exploded_ieee_finite(),
            _ => counters.record_spirix_exploded_ieee_other(),
        },
        SpirixState::Infinity => {
            // Spirix Infinity output arises only when one operand was Spirix Infinity AND the other was Spirix Zero (zero is identity per the truth table). Lib's to_f32 maps Spirix Infinity → IEEE NaN, so IEEE adds NaN+0=NaN. Categorize as architectural match.
            match ieee_kind {
                IeeeKind::Nan => counters.record_spirix_undefined_ieee_nan(),
                _ => counters.record_off_by_more(u64::MAX, a, b),
            }
        }
        SpirixState::Undefined => match ieee_kind {
            IeeeKind::Nan => counters.record_spirix_undefined_ieee_nan(),
            IeeeKind::Inf => counters.record_spirix_undefined_ieee_inf(),
            IeeeKind::Normal | IeeeKind::Denormal => counters.record_spirix_undefined_ieee_finite(),
            IeeeKind::Zero => counters.record_spirix_undefined_ieee_zero(),
        },
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
