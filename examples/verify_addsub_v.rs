//! Exhaustive F3E3 verification of the Verilog spirix_addsub algorithm
//! (ported here to Rust) against Rust's native scalar_add_scalar /
//! scalar_subtract_scalar.
//!
//! Purpose: prove the Verilog RTL matches the Rust reference for every one
//! of the 65,536 × 65,536 × 2 (add + sub) pairs without needing to run
//! iverilog on all of them.  A small spot-check in iverilog is enough once
//! this passes.
use spirix::*;

type S = ScalarF3E3;
const FRAC: i32 = 8;
const EXP_BITS: i32 = 8;
const INT_BITS: i32 = 2 * FRAC;
const AMBIG_EXP: i8 = i8::MIN;
const MIN_EXP: i16 = AMBIG_EXP as i16 + 1;
const MAX_EXP: i16 = i8::MAX as i16;

// Bit patterns shared with Rust undefined.rs (match src/core/undefined.rs).
const POS_ONE_NORMAL: i8 = i8::MIN;                // 0x80
const NEG_ONE_NORMAL: i8 = 0;                      // 0x00
const POS_ONE_EXPLODED: i8 = 0x40;
const NEG_ONE_EXPLODED: i8 = i8::MIN;              // 0x80
const POS_ONE_VANISHED: i8 = 0x20;
const NEG_ONE_VANISHED: i8 = 0xC0u8 as i8;
const UNDEF_TF_P_TF: i8    = 0x1A;
const UNDEF_TF_M_TF: i8    = 0xE5u8 as i8;
const UNDEF_VAN_P_VAN: i8  = 0x1D;
const UNDEF_VAN_M_VAN: i8  = 0xE2u8 as i8;
const UNDEF_TF_P_FIN: i8   = 0x1C;
const UNDEF_TF_M_FIN: i8   = 0xE3u8 as i8;
const UNDEF_FIN_P_TF: i8   = 0x1B;
const UNDEF_FIN_M_TF: i8   = 0xE4u8 as i8;

// State detection (purely on the stored (frac, exp) pair).
fn is_ambig(e: i8) -> bool { e == AMBIG_EXP }
fn frac_zero(f: i8) -> bool { f == 0 }
fn frac_neg1(f: i8) -> bool { f == -1i8 }
fn n0(f: i8) -> bool { frac_zero(f) || frac_neg1(f) }
fn bit(f: i8, p: i32) -> bool { ((f as u8 >> p) & 1) != 0 }
fn top_same(f: i8, n: i32) -> bool {
    let msb = bit(f, 7);
    (1..n).all(|k| bit(f, 7 - k) == msb)
}
fn is_n1(f: i8) -> bool { bit(f, 7) != bit(f, 6) }
fn is_n2(f: i8) -> bool { !is_n1(f) && !n0(f) && bit(f, 7) != bit(f, 5) }
fn is_top3(f: i8) -> bool { !n0(f) && top_same(f, 3) }

fn is_zero(f: i8, e: i8)     -> bool { is_ambig(e) && frac_zero(f) }
fn is_inf(f: i8, e: i8)      -> bool { is_ambig(e) && frac_neg1(f) }
fn is_exploded(f: i8, e: i8) -> bool { is_ambig(e) && is_n1(f) }
fn is_transf(f: i8, e: i8)   -> bool { is_inf(f, e) || is_exploded(f, e) }
fn is_vanished(f: i8, e: i8) -> bool { is_ambig(e) && is_n2(f) }
fn is_undef(f: i8, e: i8)    -> bool { is_ambig(e) && is_top3(f) }
fn is_normal(e: i8)          -> bool { !is_ambig(e) }

// N0 negation (mirrors the Verilog neg_b logic).
fn neg_normal(f: i8, e: i8) -> (i8, i8) {
    if f == POS_ONE_NORMAL {
        let em1 = e.wrapping_sub(1);
        if em1 == AMBIG_EXP { (NEG_ONE_VANISHED, AMBIG_EXP) } else { (NEG_ONE_NORMAL, em1) }
    } else if f == NEG_ONE_NORMAL {
        let ep1 = e.wrapping_add(1);
        if ep1 == AMBIG_EXP { (POS_ONE_EXPLODED, AMBIG_EXP) } else { (POS_ONE_NORMAL, ep1) }
    } else {
        (f.wrapping_neg(), e)
    }
}

fn neg_nonnormal(f: i8, e: i8) -> (i8, i8) {
    // Rust scalar_negate: signless (zero/inf/undef) → no-op;
    // escape poles → canonical swap; anything else → wrapping_neg.
    if frac_zero(f) || frac_neg1(f) || is_top3(f) { (f, e) }
    else if f == POS_ONE_EXPLODED { (NEG_ONE_EXPLODED, e) }
    else if f == NEG_ONE_EXPLODED { (POS_ONE_EXPLODED, e) }
    else if f == POS_ONE_VANISHED { (NEG_ONE_VANISHED, e) }
    else if f == NEG_ONE_VANISHED { (POS_ONE_VANISHED, e) }
    else { (f.wrapping_neg(), e) }
}

fn neg_spirix(f: i8, e: i8) -> (i8, i8) {
    if is_ambig(e) { neg_nonnormal(f, e) } else { neg_normal(f, e) }
}

// N0 inflate: wide = {{FRAC{~f[FRAC-1]}}, f}
fn inflate(f: i8) -> i16 {
    let wide = f as i16;
    let mask = (-1i16) << FRAC;
    wide ^ mask
}

// Leading-same count on 16-bit signed.
fn leading_same(v: i16) -> i32 {
    let u = v as u16;
    (u.leading_zeros().max((!u).leading_zeros())) as i32
}

// Main addsub function, mirror of Verilog spirix_addsub.
fn addsub_v(a_f: i8, a_e: i8, b_f: i8, b_e: i8, sub: bool) -> (i8, i8) {
    let a_undef = is_undef(a_f, a_e);
    let b_undef = is_undef(b_f, b_e);
    let a_trans = is_transf(a_f, a_e);
    let b_trans = is_transf(b_f, b_e);
    let a_van = is_vanished(a_f, a_e);
    let b_van = is_vanished(b_f, b_e);
    let a_zero = is_zero(a_f, a_e);
    let a_norm = is_normal(a_e);
    let b_norm = is_normal(b_e);
    let any_non_normal = !a_norm || !b_norm;

    // Negated b
    let (neg_b_f, neg_b_e) = neg_spirix(b_f, b_e);

    if any_non_normal {
        // Edge-case priority (matches Rust + Verilog shortcut order).
        if a_undef { return (a_f, a_e); }
        if b_undef { return (b_f, b_e); }
        // Zero early: X ± [0] = X, [0] ± X = ±X. Checked before transfinite so
        // the 4 zero-plus-transfinite cells pass through instead of becoming
        // transfinite-plus-finite undefined.
        if a_zero { return if sub { (neg_b_f, neg_b_e) } else { (b_f, b_e) }; }
        let b_zero_ = is_zero(b_f, b_e);
        if b_zero_ { return (a_f, a_e); }
        if a_trans && b_trans {
            let p = if sub { UNDEF_TF_M_TF } else { UNDEF_TF_P_TF };
            return (p, AMBIG_EXP);
        }
        if a_van && b_van {
            let p = if sub { UNDEF_VAN_M_VAN } else { UNDEF_VAN_P_VAN };
            return (p, AMBIG_EXP);
        }
        if a_trans {
            let p = if sub { UNDEF_TF_M_FIN } else { UNDEF_TF_P_FIN };
            return (p, AMBIG_EXP);
        }
        if b_trans {
            let p = if sub { UNDEF_FIN_M_TF } else { UNDEF_FIN_P_TF };
            return (p, AMBIG_EXP);
        }
        if a_van { return if sub { (neg_b_f, neg_b_e) } else { (b_f, b_e) }; }
        if b_van { return (a_f, a_e); }
        return (a_f, a_e); // fallback
    }

    // Both normal
    let raw_diff = (a_e as i16) - (b_e as i16);
    let a_is_big = raw_diff >= 0;
    let (big_f, big_e, small_f, small_e) = if a_is_big {
        (a_f, a_e, b_f, b_e)
    } else {
        (b_f, b_e, a_f, a_e)
    };
    let exp_diff = raw_diff.abs();
    let negate_small = sub && a_is_big;
    let negate_big = sub && !a_is_big;

    // Negligible bypass (shift ≥ FRAC-1).
    if exp_diff >= FRAC as i16 - 1 {
        if negate_big {
            let (nbf, nbe) = neg_normal(big_f, big_e);
            return (nbf, nbe);
        }
        return (big_f, big_e);
    }

    let shift = exp_diff as i32;
    let big_inf = inflate(big_f);
    let small_inf = inflate(small_f);
    let big_shifted = big_inf << shift;

    // Add/sub with carry chain.
    let big_op = if negate_big { big_shifted.wrapping_neg() } else { big_shifted };
    let small_op = if negate_small { small_inf.wrapping_neg() } else { small_inf };
    let sum = big_op.wrapping_add(small_op);

    if sum == 0 { return (0, AMBIG_EXP); }

    let leading = leading_same(sum);
    let shl_amount = leading - FRAC;
    let canonical = if shl_amount >= 0 { sum << shl_amount } else { sum >> (-shl_amount) };
    let out_frac = canonical as i8;
    let exp_calc = (small_e as i32) - shl_amount;

    if exp_calc > MAX_EXP as i32 {
        // Overflow → exploded (N-1 shape, input sign preserved).
        let exp_shl = leading - 1 - FRAC;
        let w = if exp_shl >= 0 { sum << exp_shl } else { sum >> (-exp_shl) };
        return (w as i8, AMBIG_EXP);
    }
    if exp_calc < MIN_EXP as i32 {
        // Underflow → vanished (N-2 shape).
        let van_shl = leading - 2 - FRAC;
        let w = if van_shl >= 0 { sum << van_shl } else { sum >> (-van_shl) };
        return (w as i8, AMBIG_EXP);
    }

    (out_frac, exp_calc as i8)
}

fn main() {
    let mut add_mismatches = 0usize;
    let mut sub_mismatches = 0usize;
    let mut first_bad_add: Option<(S, S, S, (i8, i8))> = None;
    let mut first_bad_sub: Option<(S, S, S, (i8, i8))> = None;

    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let a = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let b = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };

                    let (add_f, add_e) = addsub_v(f1, e1, f2, e2, false);
                    let rust_add = a + b;
                    let rust_add_bytes = unsafe { std::mem::transmute::<S, [i8; 2]>(rust_add) };
                    if (add_f, add_e) != (rust_add_bytes[0], rust_add_bytes[1]) {
                        add_mismatches += 1;
                        if first_bad_add.is_none() {
                            first_bad_add = Some((a, b, rust_add, (add_f, add_e)));
                        }
                    }

                    let (sub_f, sub_e) = addsub_v(f1, e1, f2, e2, true);
                    let rust_sub = a - b;
                    let rust_sub_bytes = unsafe { std::mem::transmute::<S, [i8; 2]>(rust_sub) };
                    if (sub_f, sub_e) != (rust_sub_bytes[0], rust_sub_bytes[1]) {
                        sub_mismatches += 1;
                        if first_bad_sub.is_none() {
                            first_bad_sub = Some((a, b, rust_sub, (sub_f, sub_e)));
                        }
                    }
                }
            }
        }
    }

    println!("=== spirix_addsub Verilog model vs Rust on F3E3 (4.29B pairs each) ===");
    println!("  + mismatches: {}", add_mismatches);
    println!("  - mismatches: {}", sub_mismatches);

    if let Some((a, b, r, v)) = first_bad_add {
        let ab = unsafe { std::mem::transmute::<S, [i8; 2]>(a) };
        let bb = unsafe { std::mem::transmute::<S, [i8; 2]>(b) };
        let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
        println!("  first bad + : a=[{:#04x},{}] b=[{:#04x},{}] rust=[{:#04x},{}] verilog=[{:#04x},{}]",
                 ab[0] as u8, ab[1], bb[0] as u8, bb[1],
                 rb[0] as u8, rb[1], v.0 as u8, v.1);
    }
    if let Some((a, b, r, v)) = first_bad_sub {
        let ab = unsafe { std::mem::transmute::<S, [i8; 2]>(a) };
        let bb = unsafe { std::mem::transmute::<S, [i8; 2]>(b) };
        let rb = unsafe { std::mem::transmute::<S, [i8; 2]>(r) };
        println!("  first bad - : a=[{:#04x},{}] b=[{:#04x},{}] rust=[{:#04x},{}] verilog=[{:#04x},{}]",
                 ab[0] as u8, ab[1], bb[0] as u8, bb[1],
                 rb[0] as u8, rb[1], v.0 as u8, v.1);
    }
}
