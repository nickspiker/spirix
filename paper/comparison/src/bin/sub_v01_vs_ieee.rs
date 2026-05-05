//! Bit-accurate Spirix v0.1 N0 sub at FRAC=24, EXP=8, banker's rounding, compared against native IEEE 754 binary32 sub.
//!
//! Subtraction is implemented as `a + (-b)` using the bit-accurate add algorithm and the lib's `Neg` impl on `Scalar<i32, i8>` to negate the second operand correctly across all Spirix states (normal sign-flip with renormalization at boundary; signless zero stays signless; phase-flip for exploded/vanished; undefined propagates).
//!
//! The add algorithm is duplicated here rather than imported from add_v01_vs_ieee.rs because Cargo doesn't share `[[bin]]` source files. Both binaries embed the same bit-accurate add for the same reason — paper readers can verify the algorithm in one file without crossing module boundaries.

use spirix_paper_comparison::{
    run_phase_a, run_phase_b, spirix_negate, Operand, SpirixState, AMB_EXP, COMPUTE_FRAC, FRAC,
    INT_BITS,
};

const _: () = assert!(FRAC == 24 && COMPUTE_FRAC == 25 && INT_BITS == 28);

/// Spirix v0.1 N0 subtract: `a - b = a + (-b)`. Negation handled by the lib's Neg impl via spirix_negate, which correctly handles all states (normal, zero, vanished, exploded, infinity, undefined).
fn spirix_sub(a: Operand, b: Operand) -> (i32, i8) {
    spirix_add(a, spirix_negate(b))
}

// ── Bit-accurate add (same as add_v01_vs_ieee.rs — see truth-table notes there) ───

fn spirix_add(a: Operand, b: Operand) -> (i32, i8) {
    use SpirixState::*;
    match (a.state, b.state) {
        (Normal, Normal) => spirix_add_normal_normal(a.q, a.exp, b.q, b.exp),
        (Undefined, _) => (a.q, a.exp),
        (_, Undefined) => (b.q, b.exp),
        (Zero, _) => (b.q, b.exp),
        (_, Zero) => (a.q, a.exp),
        (Exploded | Infinity, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        (Vanished, Vanished) => (UNDEFINED_CANONICAL, AMB_EXP),
        (Exploded | Infinity, _) | (_, Exploded | Infinity) => (UNDEFINED_CANONICAL, AMB_EXP),
        (Vanished, _) => (b.q, b.exp),
        (_, Vanished) => (a.q, a.exp),
    }
}

const UNDEFINED_CANONICAL: i32 = 0x10_0000;

fn spirix_add_normal_normal(a_q: i32, a_exp: i8, b_q: i32, b_exp: i8) -> (i32, i8) {
    let raw_diff = (a_exp as i16) - (b_exp as i16);
    let a_is_big = raw_diff >= 0;
    let (big_q, big_exp, small_q) = if a_is_big {
        (a_q, a_exp, b_q)
    } else {
        (b_q, b_exp, a_q)
    };
    let exp_diff = raw_diff.unsigned_abs() as i32;

    if exp_diff >= COMPUTE_FRAC {
        return (big_q, big_exp);
    }

    let is_close = exp_diff <= 1;

    let big_ext = (big_q as i64) << 2;
    let small_ext = (small_q as i64) << 2;

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

    let (out_q, exp_wide) = if is_close {
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

    if exp_wide >= AMB_EXP as i16 {
        let frac = if out_q < 0 { 0x80_0000 } else { 0x40_0000 };
        return (frac, AMB_EXP);
    }
    if exp_wide < i8::MIN as i16 {
        let frac = if out_q < 0 { 0xC0_0000_u32 as i32 } else { 0x20_0000 };
        return (frac, AMB_EXP);
    }

    (out_q, exp_wide as i8)
}

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

fn rovf_detect(out_q_raw: i32, round_up: bool) -> (bool, bool) {
    let pos_all_ones = (out_q_raw & ((1 << (COMPUTE_FRAC - 1)) - 1)) == ((1 << (COMPUTE_FRAC - 1)) - 1);
    let rovf_pos = (out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 0 && pos_all_ones && round_up;
    let rovf_neg = ((out_q_raw >> (COMPUTE_FRAC - 1)) & 1 == 1)
        && ((out_q_raw >> (COMPUTE_FRAC - 2)) & 1 == 0)
        && ((out_q_raw & ((1 << (COMPUTE_FRAC - 2)) - 1)) == (1 << (COMPUTE_FRAC - 2)) - 1)
        && round_up;
    (rovf_pos, rovf_neg)
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    const N: u64 = 10_000_000;
    println!(
        "Spirix v0.1 N0 sub (FRAC={FRAC}, EXP=8, banker's rounding, sub = a + (-b)) vs IEEE 754 binary32"
    );
    println!();

    run_phase_a("sub", N, 0x5AB5_5AB5_5AB5_5AB5, |a, b| a - b, spirix_sub);
    println!();
    run_phase_b("sub", N, 0xBABE_F00D_DEAD_5AB1, |a, b| a - b, spirix_sub);
}
