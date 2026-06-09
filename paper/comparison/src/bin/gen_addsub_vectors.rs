//! Generate test vectors for spirix_addsub vs IEEE binary32.
//!
//! Output format (one test per line, decimal/hex mix for iverilog $fscanf): <a_hex> <a_exp_dec> <b_hex> <b_exp_dec> <sub> <gold_hex> <gold_exp_dec> <gold_state>
//!
//! gold_state: 0=Normal, 1=Zero, 2=PosVan, 3=NegVan, 4=PosExp, 5=NegExp, 6=Inf, 7=Undefined.
//!
//! Exponents are unsigned u8 in the AMBIG=0 convention. Internal (true) exponent = stored - BIAS where BIAS = 127. Stored=0x80=128 holds the binade containing +1.0, stored=0x7F=127 holds the binade containing -1.0 (via N0 canonicalization).
//!
//! Usage: cargo run --release --bin gen_addsub_vectors -- <n> <out_path>

use spirix_paper_comparison::*;
use std::io::{BufWriter, Write};
use std::fs::File;

fn state_code(s: SpirixState) -> u8 {
    match s {
        SpirixState::Normal => 0,
        SpirixState::Zero => 1,
        SpirixState::PosVanished => 2,
        SpirixState::NegVanished => 3,
        SpirixState::PosExploded => 4,
        SpirixState::NegExploded => 5,
        SpirixState::Infinity => 6,
        SpirixState::Undefined => 7,
    }
}

/// Compute one gold vector: take Spirix inputs, convert to f64, do IEEE arithmetic, convert IEEE result back to Spirix.
fn gold(a_st: u32, a_e: u8, b_st: u32, b_e: u8, sub: bool) -> (u32, u8, SpirixState) {
    let a_state = classify(a_st, a_e);
    let b_state = classify(b_st, b_e);

    use SpirixState::*;
    let result_via_truth_table: Option<(u32, u8, SpirixState)> = match (a_state, b_state) {
        (Undefined, _) => Some((a_st, a_e, Undefined)),
        (_, Undefined) => Some((b_st, b_e, Undefined)),
        (Zero, _) => {
            if !sub { Some((b_st, b_e, b_state)) }
            else { Some(spirix_negate_state(b_st, b_e, b_state)) }
        }
        (_, Zero) => Some((a_st, a_e, a_state)),
        (PosExploded | NegExploded | Infinity, PosExploded | NegExploded | Infinity)
            => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosVanished | NegVanished, PosVanished | NegVanished)
            => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosExploded | NegExploded | Infinity, _)
            | (_, PosExploded | NegExploded | Infinity)
            => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosVanished | NegVanished, Normal) => {
            if !sub { Some((b_st, b_e, b_state)) }
            else { Some(spirix_negate_state(b_st, b_e, b_state)) }
        }
        (Normal, PosVanished | NegVanished) => Some((a_st, a_e, a_state)),
        (Normal, Normal) => None,
    };

    if let Some(r) = result_via_truth_table { return r; }

    let af = spirix_to_f64(a_st, a_e);
    let bf = spirix_to_f64(b_st, b_e);
    let rf = if sub { af - bf } else { af + bf };
    f64_to_spirix(rf)
}

/// Spirix-rule negation for a (possibly non-normal) operand.
fn spirix_negate_state(storage: u32, exp: u8, state: SpirixState) -> (u32, u8, SpirixState) {
    use SpirixState::*;
    match state {
        Zero | Infinity | Undefined => (storage, exp, state),
        PosExploded => (NEG_ONE_EXPLODED, AMBIG_EXP, NegExploded),
        NegExploded => (POS_ONE_EXPLODED, AMBIG_EXP, PosExploded),
        PosVanished => (NEG_ONE_VANISHED, AMBIG_EXP, NegVanished),
        NegVanished => (POS_ONE_VANISHED, AMBIG_EXP, PosVanished),
        Normal => {
            let s24 = storage & 0x00FF_FFFF;
            if s24 == POS_ONE_NORMAL {
                // +0.5·2^exp → -0.5·2^exp = -1.0·2^(exp-1); canonical form is NEG_ONE_NORMAL at exp-1.
                let new_exp = exp as i32 - 1;
                if new_exp < MIN_EXP as i32 {
                    (NEG_ONE_VANISHED, AMBIG_EXP, NegVanished)
                } else {
                    (NEG_ONE_NORMAL, new_exp as u8, Normal)
                }
            } else if s24 == NEG_ONE_NORMAL {
                // -1.0·2^exp → +1.0·2^exp = +0.5·2^(exp+1); canonical form is POS_ONE_NORMAL at exp+1.
                let new_exp = exp as i32 + 1;
                if new_exp > MAX_EXP as i32 {
                    (POS_ONE_EXPLODED, AMBIG_EXP, PosExploded)
                } else {
                    (POS_ONE_NORMAL, new_exp as u8, Normal)
                }
            } else {
                let neg_storage = (0u32.wrapping_sub(s24)) & 0x00FF_FFFF;
                (neg_storage, exp, Normal)
            }
        }
    }
}

fn emit<W: Write>(w: &mut W, a: u32, ae: u8, b: u32, be: u8, sub: bool) -> std::io::Result<()> {
    let (g, ge, gs) = gold(a, ae, b, be, sub);
    writeln!(w, "{:06x} {} {:06x} {} {} {:06x} {} {}",
             a & 0xFFFFFF, ae as u32,
             b & 0xFFFFFF, be as u32,
             sub as u8,
             g & 0xFFFFFF, ge as u32,
             state_code(gs))
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).map(|s| s.parse().unwrap_or(100_000)).unwrap_or(100_000);
    let path = args.get(2).cloned().unwrap_or_else(|| "verilog/vectors.txt".to_string());

    let f = File::create(&path)?;
    let mut w = BufWriter::new(f);

    // Exponent constants in new u8 convention. BIAS=127, so: stored=128 (0x80) → internal_exp=1 (binade containing +1.0) stored=127 (0x7F) → internal_exp=0 (binade containing -1.0)
    let bias: u8 = BIAS as u8;       // 127
    let exp_p1: u8 = bias + 1;        // 128: binade with +1.0
    let exp_0:  u8 = bias;            // 127: binade with -1.0
    let exp_p5: u8 = bias + 5;        // 132
    let exp_n100: u8 = bias - 100;    // 27
    let exp_p3: u8 = bias + 3;        // 130
    let exp_p12: u8 = bias + 12;      // 139
    let exp_p50: u8 = bias + 50;      // 177
    let exp_p10: u8 = bias + 10;      // 137
    let exp_n50: u8 = bias - 50;      // 77

    // ── Edge-case grid: every pair of state bit patterns × {add, sub} ──
    let edges: &[(u32, u8, &str)] = &[
        (NEG_ONE_NORMAL, exp_p1, "-1.0·2^1 = -2"),
        (POS_ONE_NORMAL, exp_p1, "+0.5·2^1 = +1"),
        (POS_ONE_NORMAL, exp_0,  "+0.5·2^0 = +0.5"),
        (NEG_ONE_NORMAL, exp_0,  "-1.0·2^0 = -1"),
        (POS_ONE_NORMAL, MAX_EXP, "max +"),
        (NEG_ONE_NORMAL, MAX_EXP, "max -"),
        (POS_ONE_NORMAL, MIN_EXP, "min +"),
        (NEG_ONE_NORMAL, MIN_EXP, "min -"),
        (0xFFFFFF, exp_0, "near +1 large m"),
        (0x000001, exp_0, "near -1 large m"),
        // Specials
        (0x000000, AMBIG_EXP, "Zero"),
        (0xFFFFFF, AMBIG_EXP, "Infinity"),
        (POS_ONE_EXPLODED, AMBIG_EXP, "PosExploded"),
        (NEG_ONE_EXPLODED, AMBIG_EXP, "NegExploded"),
        (POS_ONE_VANISHED, AMBIG_EXP, "PosVanished"),
        (NEG_ONE_VANISHED, AMBIG_EXP, "NegVanished"),
        (UNDEF_CANONICAL, AMBIG_EXP, "Undefined"),
    ];
    for &(a, ae, _) in edges {
        for &(b, be, _) in edges {
            for sub in [false, true] {
                emit(&mut w, a, ae, b, be, sub)?;
            }
        }
    }

    // ── Cancellation row: a + (-a), a - a near boundaries ──
    let cancellation: &[(u32, u8)] = &[
        (POS_ONE_NORMAL, exp_0), (POS_ONE_NORMAL, exp_p5), (POS_ONE_NORMAL, exp_n100),
        (NEG_ONE_NORMAL, exp_0), (NEG_ONE_NORMAL, exp_p5), (NEG_ONE_NORMAL, exp_n100),
        (0xC00000, exp_p3), (0x400000, exp_p3), (0xABCDEF, exp_p12),
    ];
    for &(a, ae) in cancellation {
        let (na, nae, _) = spirix_negate_state(a, ae, classify(a, ae));
        emit(&mut w, a, ae, a, ae, true)?;
        emit(&mut w, a, ae, na, nae, false)?;
        emit(&mut w, na, nae, a, ae, false)?;
    }

    // ── Random pairs from f32 space, mapped to Spirix ──
    let mut lfsr = Lfsr64::new(0xDEAD_BEEF_CAFE_F00D);
    for _ in 0..n {
        let af = lfsr.next_f32();
        let bf = lfsr.next_f32();
        let sub = (lfsr.next() & 1) == 1;
        let (a, ae, _) = f32_to_spirix(af);
        let (b, be, _) = f32_to_spirix(bf);
        emit(&mut w, a, ae, b, be, sub)?;
    }

    // ── Close-path-focused: random pairs forced to |exp_diff| ≤ 1 ──
    let close_n = n / 2;
    for _ in 0..close_n {
        let raw_a = lfsr.next();
        let raw_b = lfsr.next();
        let a_storage = (raw_a as u32) & 0x00FF_FFFF;
        let b_storage = (raw_b as u32) & 0x00FF_FFFF;
        // Random exp in [MIN_EXP+1, MAX_EXP-1] = [2, 254] (room for ±1 offset).
        let a_exp_u32 = (raw_a >> 24) % ((MAX_EXP - MIN_EXP - 1) as u64);
        let a_exp = (a_exp_u32 as u8).wrapping_add(MIN_EXP + 1);
        let exp_offset: i32 = match (raw_b >> 24) & 0x3 {
            0 => -1,
            1 => 0,
            _ => 1,
        };
        let b_exp_i32 = (a_exp as i32) + exp_offset;
        let b_exp = b_exp_i32.clamp(MIN_EXP as i32, MAX_EXP as i32) as u8;
        let sub = (lfsr.next() & 1) == 1;
        emit(&mut w, a_storage, a_exp, b_storage, b_exp, sub)?;
    }

    // ── Massive cancellation: a vs (a ± 1 ULP) at close path ──
    let cancellation_seeds: &[(u32, u8)] = &[
        (POS_ONE_NORMAL, exp_0), (POS_ONE_NORMAL, exp_p50), (POS_ONE_NORMAL, MIN_EXP + 1),
        (POS_ONE_NORMAL, MAX_EXP - 1),
        (NEG_ONE_NORMAL, exp_0), (NEG_ONE_NORMAL, exp_p50), (NEG_ONE_NORMAL, MIN_EXP + 1),
        (NEG_ONE_NORMAL, MAX_EXP - 1),
        (0xC00000, exp_0), (0x400000, exp_0), (0xFFFFFF, exp_0), (0x000001, exp_0),
        (0x800001, exp_p10), (0x7FFFFF, exp_p10), (0xABCDEF, exp_n50),
    ];
    for &(a, ae) in cancellation_seeds {
        for delta in [1u32, 2, 3, 0xFF, 0xFFFF, 0x800000, 0xC00000] {
            let b = (a.wrapping_add(delta)) & 0x00FF_FFFF;
            emit(&mut w, a, ae, b, ae, true)?;
            emit(&mut w, a, ae, b, ae, false)?;
            let be1 = (ae as i32 + 1).clamp(MIN_EXP as i32, MAX_EXP as i32) as u8;
            let bem1 = (ae as i32 - 1).clamp(MIN_EXP as i32, MAX_EXP as i32) as u8;
            emit(&mut w, a, ae, b, be1, true)?;
            emit(&mut w, a, ae, b, bem1, true)?;
        }
    }

    // ── Power-of-two boundaries (every 7th exp, both signs) ──
    let mut exp = MIN_EXP;
    while exp <= MAX_EXP {
        for &frac_a in &[POS_ONE_NORMAL, NEG_ONE_NORMAL, 0xFFFFFF, 0x000001,
                         0xC00000, 0x400000] {
            for &frac_b in &[POS_ONE_NORMAL, NEG_ONE_NORMAL, 0xFFFFFF, 0x000001] {
                emit(&mut w, frac_a, exp, frac_b, exp, false)?;
                emit(&mut w, frac_a, exp, frac_b, exp, true)?;
            }
        }
        if exp > MAX_EXP - 7 { break; }
        exp += 7;
    }

    w.flush()?;
    let pow2_iters = ((MAX_EXP as u32 - MIN_EXP as u32) / 7 + 1) as usize;
    let total = edges.len() * edges.len() * 2
              + cancellation.len() * 3
              + n as usize
              + close_n as usize
              + cancellation_seeds.len() * 7 * 4
              + pow2_iters * 6 * 4 * 2;
    eprintln!("Wrote {} vectors to {}", total, path);
    Ok(())
}
