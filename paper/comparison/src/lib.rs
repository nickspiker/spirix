//! Shared infrastructure for IEEE 754 binary32 comparison harnesses.
//!
//! Each per-operation binary in `src/bin/` provides a single bit-accurate Spirix algorithm; everything else (Spirix↔IEEE conversion, state classification, comparison categorization, phase orchestration) lives here. The bit-accurate algorithm sits in the binary so a paper reader can verify the implementation in one file without crossing module boundaries.

use spirix::Scalar;

// ── Format constants (Spirix v0.1 N0 at FRAC=24, EXP=8) ─────────────────────

/// Storage fraction width — 24 bits, "wonky" non-power-of-2 width chosen for bit-equivalence with IEEE 754 binary32 (32 total storage bits, 24 effective precision bits).
pub const FRAC: i32 = 24;
/// Compute fraction width — 25 bits = FRAC + implicit complement bit reconstructed at compute. Same as v0.0 N1's stored width; the algorithm transfers verbatim between v0.0 and v0.1, with only the conversion functions and AMB sentinel position differing.
pub const COMPUTE_FRAC: i32 = 25;
/// Internal arithmetic width — compute width + 3 bits of guard/round/sticky headroom = 28.
pub const INT_BITS: i32 = COMPUTE_FRAC + 3;
/// Ambiguous exponent sentinel under v0.1 — the single reserved exponent value used for all non-normal Spirix states (zero, infinity, exploded, vanished, undefined). Both overflow past i8::MAX-1 and underflow past i8::MIN saturate here.
pub const AMB_EXP: i8 = i8::MAX;

// ── Spirix state enum and operand ───────────────────────────────────────────

/// State of a Spirix Scalar — every bit pattern maps to exactly one of these by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpirixState {
    Normal,
    Zero,
    Vanished,
    Exploded,
    Infinity,
    Undefined,
}

/// Operand carried through bit-accurate Spirix algorithms. For Normal state, `q` is the compute-form Q (sign-extended 25-bit two's complement) and `exp` is the unbiased exponent. For non-normal states (exp == AMB_EXP), `q` carries the storage pattern in its lower 24 bits.
#[derive(Debug, Clone, Copy)]
pub struct Operand {
    pub q: i32,
    pub exp: i8,
    pub state: SpirixState,
}

/// IEEE 754 binary32 result kind for categorization purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IeeeKind {
    Normal,
    Zero,
    Denormal,
    Inf,
    Nan,
}

// ── Classifiers ─────────────────────────────────────────────────────────────

/// Classify a Spirix Scalar from its (q, exp) storage. Used after IEEE→Spirix conversion and at the output of arithmetic to decide which truth-table branch or category applies.
pub fn classify_state(q: i32, exp: i8) -> SpirixState {
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
    match lsbc {
        1 => SpirixState::Exploded,
        2 => SpirixState::Vanished,
        _ => SpirixState::Undefined,
    }
}

pub fn classify_ieee(v: f32) -> IeeeKind {
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

/// Returns true if `v` is a denormal (subnormal) IEEE 754 binary32 value.
pub fn is_denormal(v: f32) -> bool {
    let bits = v.to_bits();
    let biased_exp = (bits >> 23) & 0xFF;
    let mantissa = bits & 0x7F_FFFF;
    biased_exp == 0 && mantissa != 0
}

/// Count the leading same-bit run length in the lower 24 bits of `stored`. Used to classify Spirix non-normal states at the ambiguous exponent: 1 = exploded, 2 = vanished, 3+ = undefined. Returns FRAC for the uniform patterns (zero, infinity).
pub fn leading_same_bit_count(stored: u32) -> i32 {
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

/// Count the leading same-bit run length of `value`, examining the top `width` bits. Returns `width` for uniform patterns. Used during the IEEE→Spirix conversion bridge to classify the lib's 32-bit storage form.
pub fn leading_same_bit_count_u32(value: u32, width: u32) -> u32 {
    debug_assert!(width <= 32);
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

// ── Conversions ─────────────────────────────────────────────────────────────

/// Convert an f32 to a Spirix Operand at FRAC=24, using the spirix lib for the IEEE→Scalar mapping (so denormal handling, ±∞ → exploded, NaN → undefined, ±0 → zero, and the high-end conversion-loss-to-exploded boundary all match the lib's authoritative behavior).
///
/// The lib produces a `Scalar<i32, i8>` at FRAC=32. We bridge to our FRAC=24 compute form by truncating the lib's stored 32-bit fraction down 8 bits — bit-exact for f32 inputs because IEEE binary32 has only 24 effective fraction bits, so the lib's lower 8 stored bits are always zero for f32-sourced normals.
pub fn f32_to_spirix_via_lib(v: f32) -> Operand {
    let s: Scalar<i32, i8> = v.into();

    if s.exponent == AMB_EXP {
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

/// Convert Spirix v0.1 N0 (Q, exp) back to f32 using the lib's to_f32. Bridges our FRAC=24 form to the lib's Scalar<i32, i8> at FRAC=32 by left-shifting the 24-bit storage to occupy the upper 24 bits of the 32-bit fraction (bit-exact since FRAC=24 has no info below those bits).
///
/// **Spirix-design override**: per Spirix's "no signed zero" principle, any zero result from the lib (which would be -0.0 for negative-vanished inputs) is normalized to +0.0. This makes the IEEE-side reference consistent with Spirix's signless treatment of zero.
pub fn spirix_to_f32(q: i32, exp: i8) -> f32 {
    let stored_24 = (q as u32) & 0xFF_FFFF;
    let stored_32 = if exp == AMB_EXP && stored_24 == 0xFF_FFFF {
        0xFFFF_FFFFu32
    } else {
        stored_24 << 8
    };
    let s = Scalar::<i32, i8> {
        fraction: stored_32 as i32,
        exponent: exp,
    };
    let result = s.to_f32();
    if result == 0.0 {
        0.0
    } else {
        result
    }
}

/// Negate a Normal Spirix compute Q with boundary handling. The two boundary cases (where the result needs renormalization or saturates off the exp range):
///   `pos_one_normal` (Q=+2^23) at exp=k → `neg_one_normal` (Q=-2^24) at exp=k-1, with saturation to negative-vanished if exp underflows i8::MIN
///   `neg_one_normal` (Q=-2^24) at exp=k → `pos_one_normal` (Q=+2^23) at exp=k+1, with saturation to positive-exploded if exp overflows past AMB
/// All other normals: Q ↦ -Q at the same exponent (plain integer negation, in-range).
pub fn negate_normal_compute_q(q: i32, exp: i8) -> (i32, i8) {
    if q == (1 << 23) {
        let new_exp = (exp as i16) - 1;
        if new_exp < i8::MIN as i16 {
            // Underflow off the cliff → negative vanished (LSBC=2, negative phase)
            return (0xC0_0000_u32 as i32, AMB_EXP);
        }
        return (-(1 << 24), new_exp as i8);
    }
    if q == -(1 << 24) {
        let new_exp = (exp as i16) + 1;
        if new_exp >= AMB_EXP as i16 {
            // Overflow off the cliff → positive exploded (LSBC=1, positive phase)
            return (0x40_0000, AMB_EXP);
        }
        return (1 << 23, new_exp as i8);
    }
    (-q, exp)
}

/// Phase-flip negation for non-Normal Spirix states, used in the sub truth table for cases like `Zero - X`. Avoids the lib's full `scalar_negate` (which is branchy across all 6 states and has known boundary corners). Operates only on AMB-sentinel states:
///   Zero, Infinity, Undefined: signless/cause-preserving — no-op
///   Exploded: phase-flip top 2 bits (01... ↔ 10...)
///   Vanished: phase-flip top 3 bits (001... ↔ 110...)
/// Normal handled separately via `negate_normal_compute_q`.
pub fn negate_state_inline(op: Operand) -> (i32, i8) {
    match op.state {
        SpirixState::Normal => negate_normal_compute_q(op.q, op.exp),
        SpirixState::Zero | SpirixState::Infinity | SpirixState::Undefined => (op.q, op.exp),
        SpirixState::Exploded => {
            let stored_24 = (op.q as u32) & 0xFF_FFFF;
            let flipped = stored_24 ^ 0xC0_0000;
            (flipped as i32, AMB_EXP)
        }
        SpirixState::Vanished => {
            let stored_24 = (op.q as u32) & 0xFF_FFFF;
            let flipped = stored_24 ^ 0xE0_0000;
            (flipped as i32, AMB_EXP)
        }
    }
}

// ── Unified bit-accurate add/sub ────────────────────────────────────────────

/// Unified Spirix v0.1 N0 add/sub algorithm at FRAC=24, banker's rounding. The `sub` flag selects subtraction by integer-negating the second operand's compute Q at the alignment input — this mirrors the silicon Verilog's `subOp` pattern (single shared datapath, one bit flips effective sign of b). Because the negate is at the integer level (`wrapping_neg` on a 25-bit-range compute Q value), there are no state-level branches and no boundary edge cases — the algorithm's internal 28-bit signed arithmetic handles any value, and the result is renormalized to a valid Q.
///
/// Algorithm structure (same for add and sub):
///   1. Exponent difference and swap (so big has the larger exp, for precision-preserving alignment).
///   2. Negligible early exit if exp_diff exceeds compute precision.
///   3. Close path (exp_diff ≤ 1): align by 0/1 bits, sum, CLZ-normalize, round.
///   4. Far path (exp_diff ≥ 2): barrel-shift small with sticky tracking, sum, bounded normalize (0/1/2 bits), round.
///   5. Banker's rounding and rounding-overflow detection.
///   6. v0.1 saturation: overflow → exploded with phase, underflow → vanished with phase.
///
/// For sub, when we swapped (b had larger exp), the computed sum is `b - a` rather than `a - b`. The final out_q is integer-negated to correct, with the boundary-case-aware `negate_normal_compute_q` handling the pos_one/neg_one corners.
pub fn spirix_addsub_normal_normal(
    a_q: i32,
    a_exp: i8,
    b_q: i32,
    b_exp: i8,
    sub: bool,
) -> (i32, i8) {
    let raw_diff = (a_exp as i16) - (b_exp as i16);
    let a_is_big = raw_diff >= 0;
    let (big_q, big_exp, small_q) = if a_is_big {
        (a_q, a_exp, b_q)
    } else {
        (b_q, b_exp, a_q)
    };
    let exp_diff = raw_diff.unsigned_abs() as i32;

    // Negligible early exit: small contributes nothing past compute precision.
    // For add: result ≈ big regardless of swap.
    // For sub: result ≈ a - b. If a is big (no swap): a - 0 = a → return big. If b is big (swap): 0 - b = -b → negate big.
    if exp_diff >= COMPUTE_FRAC {
        if sub && !a_is_big {
            return negate_normal_compute_q(big_q, big_exp);
        }
        return (big_q, big_exp);
    }

    let is_close = exp_diff <= 1;

    let big_ext = (big_q as i64) << 2;
    let small_ext_raw = (small_q as i64) << 2;

    // Apply sub: integer-negate small at the alignment input (silicon's subOp pattern).
    let small_ext = if sub {
        small_ext_raw.wrapping_neg()
    } else {
        small_ext_raw
    };

    let mask = (1i64 << INT_BITS) - 1;
    let sign_bit_int = 1i64 << (INT_BITS - 1);

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

    let pos_half: i32 = 1 << (COMPUTE_FRAC - 2);
    let neg_one: i32 = -(1 << (COMPUTE_FRAC - 1));

    let (out_q_pre_swap_correction, exp_wide) = if is_close {
        let close_small = if (exp_diff & 1) != 0 {
            sext(small_ext >> 1)
        } else {
            small_ext
        };
        let close_align_sticky = (exp_diff & 1) != 0 && (small_ext & 1) != 0;
        let close_sum = sext(big_ext + close_small);

        if close_sum == 0 {
            return (0, AMB_EXP);
        }

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

    // For sub: if we swapped (b had larger exp), the computed value is `b - a` but we want `a - b`.
    // Negate the result Q. The integer negate has 2 boundary corners (pos_one ↔ neg_one with exp shift) handled by negate_normal_compute_q.
    let (out_q, exp_wide_corrected) = if sub && !a_is_big {
        // out_q_pre_swap_correction is in valid normal compute Q range; apply the boundary-aware negate.
        let (q, exp) = negate_normal_compute_q(out_q_pre_swap_correction, exp_wide.clamp(i8::MIN as i16, i8::MAX as i16) as i8);
        // negate_normal_compute_q may have already saturated to AMB; in that case return as-is.
        if exp == AMB_EXP {
            return (q, AMB_EXP);
        }
        (q, exp as i16)
    } else {
        (out_q_pre_swap_correction, exp_wide)
    };

    if exp_wide_corrected >= AMB_EXP as i16 {
        let frac = if out_q < 0 { 0x80_0000 } else { 0x40_0000 };
        return (frac, AMB_EXP);
    }
    if exp_wide_corrected < i8::MIN as i16 {
        let frac = if out_q < 0 { 0xC0_0000_u32 as i32 } else { 0x20_0000 };
        return (frac, AMB_EXP);
    }

    (out_q, exp_wide_corrected as i8)
}

/// Banker's rounding (round-to-nearest-even). Bit layout below the LSB of the output fraction:
///   bit (INT_BITS - 1 - COMPUTE_FRAC): guard
///   bit (INT_BITS - 2 - COMPUTE_FRAC): round
///   bits (INT_BITS - 2 - COMPUTE_FRAC - 1) .. 0: sticky region (OR of all)
fn round_banker(normalized: i64, align_sticky: bool) -> (i32, bool) {
    let out_q_raw = (normalized >> (INT_BITS - COMPUTE_FRAC)) as i32;
    let guard = ((normalized >> (INT_BITS - 1 - COMPUTE_FRAC)) & 1) != 0;
    let lsb = ((normalized >> (INT_BITS - COMPUTE_FRAC)) & 1) != 0;
    let round_bit = ((normalized >> (INT_BITS - 2 - COMPUTE_FRAC)) & 1) != 0;
    let ext_sticky_mask = (1i64 << (INT_BITS - 2 - COMPUTE_FRAC)) - 1;
    let ext_sticky = (normalized & ext_sticky_mask) != 0;
    let sticky = ext_sticky || align_sticky;
    let round_up = guard && (round_bit || sticky || lsb);
    (out_q_raw, round_up)
}

/// Detect rounding-overflow (rovf): when round-up would push the fraction past its representable range, requiring an exponent bump.
fn rovf_detect(out_q_raw: i32, round_up: bool) -> (bool, bool) {
    let pos_all_ones = (out_q_raw & ((1 << (COMPUTE_FRAC - 1)) - 1)) == ((1 << (COMPUTE_FRAC - 1)) - 1);
    let rovf_pos = (out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 0 && pos_all_ones && round_up;
    let rovf_neg = ((out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 1)
        && ((out_q_raw >> (COMPUTE_FRAC - 2)) & 1 == 0)
        && ((out_q_raw & ((1 << (COMPUTE_FRAC - 2)) - 1)) == (1 << (COMPUTE_FRAC - 2)) - 1)
        && round_up;
    (rovf_pos, rovf_neg)
}

/// Canonical undefined storage pattern (LSBC=3 positive). Specific cause-encoding doesn't affect the architectural-match categorization in this comparison; any LSBC≥3 pattern reads as Undefined.
pub const UNDEFINED_CANONICAL: i32 = 0x10_0000;

/// Truth-table dispatch for unified add/sub. Mirrors the spirix lib's scalar_add_scalar truth table, with sub-aware branches at the cases that need negation. Avoids `spirix_negate` entirely — uses inline phase-flips and boundary-aware compute-Q negation.
pub fn spirix_addsub(a: Operand, b: Operand, sub: bool) -> (i32, i8) {
    use SpirixState::*;
    match (a.state, b.state) {
        (Normal, Normal) => spirix_addsub_normal_normal(a.q, a.exp, b.q, b.exp, sub),
        (Undefined, _) => (a.q, a.exp),
        (_, Undefined) => (b.q, b.exp),
        // a + 0 = a; a - 0 = a (zero is identity, regardless of sub)
        (_, Zero) => (a.q, a.exp),
        // 0 + b = b; 0 - b = -b
        (Zero, _) => {
            if sub {
                negate_state_inline(b)
            } else {
                (b.q, b.exp)
            }
        }
        // Both transfinite: undefined
        (Exploded | Infinity, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        // Both vanished: undefined
        (Vanished, Vanished) => (UNDEFINED_CANONICAL, AMB_EXP),
        // Transfinite + finite (or transfinite - finite): undefined
        (Exploded | Infinity, _) | (_, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        // a is vanished, b is normal — vanished drops out:
        //   add: result is b
        //   sub: result is -b (b's negation, with boundary handling)
        (Vanished, _) => {
            if sub {
                negate_state_inline(b)
            } else {
                (b.q, b.exp)
            }
        }
        // a is normal, b is vanished — vanished drops out, result is a (regardless of sub)
        (_, Vanished) => (a.q, a.exp),
    }
}

// ── Random generators ───────────────────────────────────────────────────────

/// Deterministic LCG random number generator. Reproduces the same sequence across runs given the same seed, so reported numbers are auditable.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

/// Generate a random valid Spirix operand. Random 24-bit fraction + random 8-bit exponent — every bit pattern is a valid Spirix state by design.
pub fn random_spirix_operand(rng: &mut Lcg) -> Operand {
    let stored_24 = (rng.next_u32() & 0xFF_FFFF) as i32;
    let exp_byte = rng.next_u32() as u8;
    let exp = exp_byte as i8;

    if exp == AMB_EXP {
        let state = classify_state(stored_24, AMB_EXP);
        return Operand { q: stored_24, exp: AMB_EXP, state };
    }

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

// ── Counters and categorization ─────────────────────────────────────────────

/// Counters tabulating per-category comparison results.
///
/// Categories are organized by Spirix output state, then sub-divided by what IEEE produced. The "architectural" categories are honest-by-design mismatches between IEEE 754 and Spirix v0.1 semantics; "off_by_more" should always be zero for a correct implementation.
#[derive(Default, Debug)]
pub struct Counters {
    pub exact: u64,
    pub one_ulp: u64,
    pub spirix_normal_arch_drift: u64,
    pub off_by_more: u64,
    pub max_ulp_diff: u64,
    pub worst_a: f32,
    pub worst_b: f32,

    pub spirix_zero_ieee_zero: u64,
    pub spirix_zero_ieee_other: u64,

    pub spirix_vanished_ieee_zero: u64,
    pub spirix_vanished_ieee_denormal: u64,
    pub spirix_vanished_ieee_other: u64,

    pub spirix_exploded_ieee_inf: u64,
    pub spirix_exploded_ieee_finite: u64,
    pub spirix_exploded_ieee_other: u64,

    pub spirix_undefined_ieee_nan: u64,
    pub spirix_undefined_ieee_inf: u64,
    pub spirix_undefined_ieee_finite: u64,
    pub spirix_undefined_ieee_zero: u64,

    pub conversion_loss_to_exploded: u64,
    pub conversion_loss_to_vanished: u64,
}

impl Counters {
    pub fn record_off_by_more(&mut self, ulp_diff: u64, a: f32, b: f32) {
        self.off_by_more += 1;
        if ulp_diff > self.max_ulp_diff {
            self.max_ulp_diff = ulp_diff;
            self.worst_a = a;
            self.worst_b = b;
        }
    }

    pub fn print_summary(&self, n: u64) {
        let pct = |c: u64| c as f64 / n as f64 * 100.0;
        let row = |label: &str, count: u64| {
            println!("  {:42} {:>10}  ({:.4}%)", label, count, pct(count));
        };

        println!("Results ({} trials):", n);
        println!();
        println!("Spirix Normal output:");
        row("exact match", self.exact);
        row("1 ULP (valid rounding choice)", self.one_ulp);
        row("architectural drift (non-normal input or IEEE denormal/zero)", self.spirix_normal_arch_drift);
        row("off by more (REAL BUG — should be 0)", self.off_by_more);

        let zero_total = self.spirix_zero_ieee_zero + self.spirix_zero_ieee_other;
        if zero_total > 0 {
            println!();
            println!("Spirix Zero output:");
            row("IEEE -> 0 (match)", self.spirix_zero_ieee_zero);
            row("IEEE -> other (UNEXPECTED)", self.spirix_zero_ieee_other);
        }

        let vanished_total = self.spirix_vanished_ieee_zero
            + self.spirix_vanished_ieee_denormal
            + self.spirix_vanished_ieee_other;
        if vanished_total > 0 {
            println!();
            println!("Spirix Vanished output (architectural — Spirix preserves direction below precision):");
            row("IEEE -> 0", self.spirix_vanished_ieee_zero);
            row("IEEE -> denormal", self.spirix_vanished_ieee_denormal);
            row("IEEE -> other (UNEXPECTED)", self.spirix_vanished_ieee_other);
        }

        let exploded_total = self.spirix_exploded_ieee_inf
            + self.spirix_exploded_ieee_finite
            + self.spirix_exploded_ieee_other;
        if exploded_total > 0 {
            println!();
            println!("Spirix Exploded output (architectural — Spirix preserves direction beyond magnitude limit):");
            row("IEEE -> +/-inf", self.spirix_exploded_ieee_inf);
            row("IEEE -> finite (exp-range delta)", self.spirix_exploded_ieee_finite);
            row("IEEE -> other (UNEXPECTED)", self.spirix_exploded_ieee_other);
        }

        let undefined_total = self.spirix_undefined_ieee_nan
            + self.spirix_undefined_ieee_inf
            + self.spirix_undefined_ieee_finite
            + self.spirix_undefined_ieee_zero;
        if undefined_total > 0 {
            println!();
            println!("Spirix Undefined output (architectural — Spirix flags more cases as undefined than IEEE):");
            row("IEEE -> NaN  (architectural match)", self.spirix_undefined_ieee_nan);
            row("IEEE -> +/-inf  (Spirix more conservative)", self.spirix_undefined_ieee_inf);
            row("IEEE -> finite  (Spirix more conservative)", self.spirix_undefined_ieee_finite);
            row("IEEE -> 0  (Spirix more conservative)", self.spirix_undefined_ieee_zero);
        }

        if self.conversion_loss_to_exploded > 0 || self.conversion_loss_to_vanished > 0 {
            println!();
            println!("Input-side IEEE->Spirix conversion loss (informational, per input):");
            row("IEEE input -> Spirix exploded", self.conversion_loss_to_exploded);
            row("IEEE input -> Spirix vanished", self.conversion_loss_to_vanished);
        }

        if self.off_by_more > 0 || self.spirix_zero_ieee_other > 0 || self.spirix_vanished_ieee_other > 0 || self.spirix_exploded_ieee_other > 0 {
            println!();
            println!("  Max ULP error (Normal+Normal): {}", self.max_ulp_diff);
            println!("  Worst-case operand pair: a = {:e}, b = {:e}", self.worst_a, self.worst_b);
        }
    }
}

/// Compute ULP difference between two f32 values of the same sign. Returns u64::MAX for sign mismatches or NaN inputs.
pub fn ulp_diff_same_sign(a: f32, b: f32) -> u64 {
    if a.is_nan() || b.is_nan() {
        return u64::MAX;
    }
    let abits = a.to_bits();
    let bbits = b.to_bits();
    if (abits ^ bbits) >> 31 != 0 {
        return u64::MAX;
    }
    if abits > bbits {
        (abits - bbits) as u64
    } else {
        (bbits - abits) as u64
    }
}

/// Categorize a single (a, b) → result pair against IEEE and update counters. Used by both Phase A (IEEE-driven) and Phase B (Spirix-driven) — categorization is symmetric in input direction. Caller provides the Spirix operation result `(r_q, r_exp)` and the IEEE result `ieee_result`; this function classifies output states and buckets accordingly.
pub fn categorize_and_record(
    a: f32,
    b: f32,
    ieee_result: f32,
    a_op: Operand,
    b_op: Operand,
    r_q: i32,
    r_exp: i8,
    counters: &mut Counters,
) {
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
                        counters.exact += 1;
                    } else if ieee_result == 0.0 || spirix_result == 0.0 {
                        counters.record_off_by_more(u64::MAX, a, b);
                    } else {
                        let diff = ulp_diff_same_sign(ieee_result, spirix_result);
                        if diff == 0 {
                            counters.exact += 1;
                        } else if inputs_had_arch_input {
                            counters.spirix_normal_arch_drift += 1;
                        } else if diff == 1 {
                            counters.one_ulp += 1;
                        } else {
                            counters.record_off_by_more(diff, a, b);
                        }
                    }
                }
                IeeeKind::Zero | IeeeKind::Denormal => {
                    counters.spirix_normal_arch_drift += 1;
                }
                IeeeKind::Inf | IeeeKind::Nan => {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
            }
        }
        SpirixState::Zero => match ieee_kind {
            IeeeKind::Zero => counters.spirix_zero_ieee_zero += 1,
            _ => counters.spirix_zero_ieee_other += 1,
        },
        SpirixState::Vanished => match ieee_kind {
            IeeeKind::Zero => counters.spirix_vanished_ieee_zero += 1,
            IeeeKind::Denormal => counters.spirix_vanished_ieee_denormal += 1,
            _ => counters.spirix_vanished_ieee_other += 1,
        },
        SpirixState::Exploded => match ieee_kind {
            IeeeKind::Inf => counters.spirix_exploded_ieee_inf += 1,
            IeeeKind::Normal => counters.spirix_exploded_ieee_finite += 1,
            _ => counters.spirix_exploded_ieee_other += 1,
        },
        SpirixState::Infinity => match ieee_kind {
            IeeeKind::Nan => counters.spirix_undefined_ieee_nan += 1,
            _ => counters.record_off_by_more(u64::MAX, a, b),
        },
        SpirixState::Undefined => match ieee_kind {
            IeeeKind::Nan => counters.spirix_undefined_ieee_nan += 1,
            IeeeKind::Inf => counters.spirix_undefined_ieee_inf += 1,
            IeeeKind::Normal | IeeeKind::Denormal => counters.spirix_undefined_ieee_finite += 1,
            IeeeKind::Zero => counters.spirix_undefined_ieee_zero += 1,
        },
    }
}

// ── Phase orchestration ─────────────────────────────────────────────────────

/// Run a Phase A (IEEE-driven random) test. Generates random IEEE bit patterns, applies the IEEE op for the reference, and the Spirix op on the lib-converted operands.
pub fn run_phase_a<I, S>(
    op_name: &str,
    n: u64,
    seed: u64,
    ieee_op: I,
    spirix_op: S,
) -> Counters
where
    I: Fn(f32, f32) -> f32,
    S: Fn(Operand, Operand) -> (i32, i8),
{
    println!("== Phase A: IEEE-driven random — {n} trials ({op_name}) ==");
    println!("Inputs: full random IEEE 754 binary32 bit patterns (every f32 bit pattern is a valid IEEE value). Lib's from(f32) maps each to its corresponding valid Spirix state.");
    println!();

    let mut rng = Lcg::new(seed);
    let mut counters = Counters::default();
    for _ in 0..n {
        let a = f32::from_bits(rng.next_u32());
        let b = f32::from_bits(rng.next_u32());
        let a_op = f32_to_spirix_via_lib(a);
        let b_op = f32_to_spirix_via_lib(b);
        for op in [a_op, b_op] {
            match op.state {
                SpirixState::Exploded => counters.conversion_loss_to_exploded += 1,
                SpirixState::Vanished => counters.conversion_loss_to_vanished += 1,
                _ => {}
            }
        }
        let ieee_result = ieee_op(a, b);
        let (r_q, r_exp) = spirix_op(a_op, b_op);
        categorize_and_record(a, b, ieee_result, a_op, b_op, r_q, r_exp, &mut counters);
    }
    counters.print_summary(n);
    counters
}

/// Run a Phase B (Spirix-driven random) test. Generates random Spirix bit patterns directly, then converts each operand to f32 via the lib (with Spirix's signless-zero principle enforced) for the IEEE-side reference.
pub fn run_phase_b<I, S>(
    op_name: &str,
    n: u64,
    seed: u64,
    ieee_op: I,
    spirix_op: S,
) -> Counters
where
    I: Fn(f32, f32) -> f32,
    S: Fn(Operand, Operand) -> (i32, i8),
{
    println!("== Phase B: Spirix-driven random — {n} trials ({op_name}) ==");
    println!("Inputs: full random Spirix bit patterns (random 24-bit fraction + random i8 exponent — every bit pattern is a valid Spirix state by design). IEEE-side reference computed via the lib's to_f32, with Spirix's signless-zero principle enforced (vanished → +0; zero → +0; signless infinity → NaN; undefined → NaN; exploded → ±∞ with phase; otherwise precise).");
    println!();

    let mut rng = Lcg::new(seed);
    let mut counters = Counters::default();
    for _ in 0..n {
        let a_op = random_spirix_operand(&mut rng);
        let b_op = random_spirix_operand(&mut rng);
        let a = spirix_to_f32(a_op.q, a_op.exp);
        let b = spirix_to_f32(b_op.q, b_op.exp);
        let ieee_result = ieee_op(a, b);
        let (r_q, r_exp) = spirix_op(a_op, b_op);
        categorize_and_record(a, b, ieee_result, a_op, b_op, r_q, r_exp, &mut counters);
    }
    counters.print_summary(n);
    counters
}
