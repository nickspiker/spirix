//! Exhaustive F3E3 verification of the Verilog spirix_addsub algorithm (ported here to Rust) against Rust's native scalar_add_scalar / scalar_subtract_scalar.
//!
//! Purpose: prove the Verilog RTL matches the Rust reference for every one of the 65,536 × 65,536 × 2 (add + sub) pairs without needing to run
//! iverilog on all of them.  A small spot-check in iverilog is enough once this passes.
use spirix::*;

type S = ScalarF3E3;
const FRAC: i32 = 8;
const EXP_BITS: i32 = 8;
const WORK_BITS: i32 = FRAC + 2;
const AMBIG_EXP: i8 = i8::MAX;          // = 0x7F = E::MAX
const MIN_EXP: i16 = i8::MIN as i16;    // = -0x80 (now a valid normal exp)
const MAX_EXP: i16 = AMBIG_EXP as i16 - 1; // = AMBIG - 1

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
    // Rust scalar_negate: signless (zero/inf/undef) → no-op; escape poles → canonical swap; anything else → wrapping_neg.
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

// N0 inflate: wide = {{FRAC{~f[FRAC-1]}}, f} — yields a FRAC+1-bit signed value.
fn inflate(f: i8) -> i16 {
    let wide = f as i16;
    let mask = (-1i16) << FRAC;
    wide ^ mask
}

// Sign-extend a value already in FRAC+1 bits to the full i16 representation
// (the upper bits are filled with the sign for clean arith downstream).
fn to_frac_plus_1(v: i16) -> i16 {
    // inflate already returns a sign-correct i16, but normalize anyway.
    let sign_bit = 1i16 << FRAC;
    let mask = (1i16 << (FRAC + 1)) - 1;
    let masked = v & mask;
    if (masked & sign_bit) != 0 { masked | !mask } else { masked }
}

// Leading-same count on a WORK_BITS-bit signed value held in i16.
// Returns the count of MSBs of `v` (interpreted as WORK_BITS-wide signed) that
// match the sign bit at position WORK_BITS-1.
fn leading_same(v: i16) -> i32 {
    let u = v as u16;
    let masked = u & ((1u16 << WORK_BITS) - 1);
    let sign = (masked >> (WORK_BITS - 1)) & 1;
    for k in 0..WORK_BITS {
        let bit = (masked >> (WORK_BITS - 1 - k)) & 1;
        if bit != sign { return k; }
    }
    WORK_BITS
}

// Main addsub function, mirror of Verilog spirix_addsub.
fn addsub_v(a_f: i8, a_e: i8, b_f: i8, b_e: i8, sub: bool) -> (i8, i8) {
    let a_undef = is_undef(a_f, a_e);
    let b_undef = is_undef(b_f, b_e);
    let a_inf = is_inf(a_f, a_e);
    let b_inf = is_inf(b_f, b_e);
    let a_zero_ = is_zero(a_f, a_e);
    let b_zero_ = is_zero(b_f, b_e);
    let a_exp_ = is_exploded(a_f, a_e);
    let b_exp_ = is_exploded(b_f, b_e);
    let a_van = is_vanished(a_f, a_e);
    let b_van = is_vanished(b_f, b_e);
    let a_norm = is_normal(a_e);
    let b_norm = is_normal(b_e);
    let any_non_normal = !a_norm || !b_norm;

    // Negated b
    let (neg_b_f, neg_b_e) = neg_spirix(b_f, b_e);

    if any_non_normal {
        // Edge-case priority (matches Verilog shortcut mux and Rust src):
        if a_undef { return (a_f, a_e); }
        if b_undef { return (b_f, b_e); }
        // [∞] absorbs everything (signless Riemann singularity).
        if a_inf || b_inf { return (-1i8, AMBIG_EXP); }
        // Zero identity.
        if a_zero_ { return if sub { (neg_b_f, neg_b_e) } else { (b_f, b_e) }; }
        if b_zero_ { return (a_f, a_e); }
        // Both same escape class → undefined.
        if a_exp_ && b_exp_ {
            let p = if sub { UNDEF_TF_M_TF } else { UNDEF_TF_P_TF };
            return (p, AMBIG_EXP);
        }
        if a_van && b_van {
            let p = if sub { UNDEF_VAN_M_VAN } else { UNDEF_VAN_P_VAN };
            return (p, AMBIG_EXP);
        }
        // Vanished negligible against exploded.
        if a_exp_ && b_van { return (a_f, a_e); }
        if a_van && b_exp_ { return if sub { (neg_b_f, neg_b_e) } else { (b_f, b_e) }; }
        // Exploded vs normal → undefined.
        if a_exp_ {
            let p = if sub { UNDEF_TF_M_FIN } else { UNDEF_TF_P_FIN };
            return (p, AMBIG_EXP);
        }
        if b_exp_ {
            let p = if sub { UNDEF_FIN_M_TF } else { UNDEF_FIN_P_TF };
            return (p, AMBIG_EXP);
        }
        // Vanished vs normal → normal.
        if a_van { return if sub { (neg_b_f, neg_b_e) } else { (b_f, b_e) }; }
        if b_van { return (a_f, a_e); }
        return (a_f, a_e); // fallback
    }

    // Both normal — shift-small-RIGHT in FRAC+2 working bits with a sticky flag.
    let raw_diff = (a_e as i16) - (b_e as i16);
    let a_is_big = raw_diff >= 0;
    let (big_f, big_e, small_f, _small_e) = if a_is_big {
        (a_f, a_e, b_f, b_e)
    } else {
        (b_f, b_e, a_f, a_e)
    };
    let exp_diff = raw_diff.abs();
    let negate_small = sub && a_is_big;
    let negate_big = sub && !a_is_big;

    // Negligible bypass (shift ≥ FRAC-1), matching Rust ref.
    if exp_diff >= FRAC as i16 - 1 {
        if negate_big {
            let (nbf, nbe) = neg_normal(big_f, big_e);
            return (nbf, nbe);
        }
        return (big_f, big_e);
    }

    let shift = exp_diff as i32;

    // N0 inflate to FRAC+1, sign-extend to WORK_BITS (= FRAC+2).
    let big_infl = to_frac_plus_1(inflate(big_f));
    let small_infl = to_frac_plus_1(inflate(small_f));

    // Sign-extend FRAC+1 → WORK_BITS = FRAC+2 (no-op in i16 since to_frac_plus_1
    // already sign-extended into the upper bits).
    let big_ext = big_infl;
    let small_ext = small_infl;

    // Conditional negation in WORK_BITS bits (avoids MIN_VALUE overflow).
    let big_eff = if negate_big { big_ext.wrapping_neg() } else { big_ext };
    let small_eff = if negate_small { small_ext.wrapping_neg() } else { small_ext };

    // Guard bit: bit at position (shift - 1) of small_eff. This is the highest
    // bit discarded by the arith-shr. For floor at canonical LSB it's exactly
    // what we need (lower bits floor away).
    let guard = if shift > 0 { ((small_eff >> (shift - 1)) & 1) != 0 } else { false };

    // Big never shifts. Small shifts right by `shift` (arith shift, sign-preserving).
    let small_aligned: i16 = small_eff >> shift;

    let sum = big_eff.wrapping_add(small_aligned);

    // Mask sum to WORK_BITS for consistent leading-same / shift logic, then
    // sign-extend back into i16 for arithmetic.
    let mask_work = (1i16 << WORK_BITS) - 1;
    let sign_bit_work = 1i16 << (WORK_BITS - 1);
    let sum_masked = sum & mask_work;
    let sum_sext: i16 = if (sum_masked & sign_bit_work) != 0 {
        sum_masked | !mask_work
    } else {
        sum_masked
    };
    let is_zero_sum = sum_sext == 0;

    // Extended sum: append guard bit at position -1. This puts us at the same
    // scale as Rust's shift-big-LEFT (small_exp scale for shift=1). Working
    // width is WORK_BITS+1 = FRAC+3 here, with target leading-same = 3.
    let extended_sum: i32 = ((sum_sext as i32) << 1) | (if guard { 1 } else { 0 });

    if extended_sum == 0 {
        return (0, AMBIG_EXP);  // exact zero
    }

    // Leading-same on extended_sum at WORK_BITS+1 bits.
    let work_ext = WORK_BITS + 1;
    let leading_ext: i32 = {
        let u = extended_sum as u32;
        let masked = u & ((1u32 << work_ext) - 1);
        let sign = (masked >> (work_ext - 1)) & 1;
        let mut k = 0;
        while k < work_ext {
            let bit = (masked >> (work_ext - 1 - k)) & 1;
            if bit != sign { break; }
            k += 1;
        }
        k
    };
    let shl_amount_ext = leading_ext - 3;

    let canonical = if shl_amount_ext >= 0 {
        extended_sum << shl_amount_ext
    } else {
        extended_sum >> (-shl_amount_ext)
    };
    let out_frac = canonical as i8;
    // Working at extended scale (= big_exp - 1), so out_exp shifts by one extra.
    let exp_calc = (big_e as i32) - 1 - shl_amount_ext;

    if exp_calc > MAX_EXP as i32 {
        let exp_shl = shl_amount_ext - 1;
        let w = if exp_shl >= 0 { extended_sum << exp_shl } else { extended_sum >> (-exp_shl) };
        return (w as i8, AMBIG_EXP);
    }
    if exp_calc < MIN_EXP as i32 {
        let van_shl = shl_amount_ext - 2;
        let w = if van_shl >= 0 { extended_sum << van_shl } else { extended_sum >> (-van_shl) };
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
