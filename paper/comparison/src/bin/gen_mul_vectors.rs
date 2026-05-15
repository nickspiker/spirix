//! Generate test vectors for spirix_multiply vs IEEE binary32 multiply.
//!
//! Output format (one test per line):
//!   <a_hex> <a_exp_dec> <b_hex> <b_exp_dec> <gold_hex> <gold_exp_dec> <gold_state>

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

fn gold(a_st: u32, a_e: i8, b_st: u32, b_e: i8) -> (u32, i8, SpirixState) {
    let a_state = classify(a_st, a_e);
    let b_state = classify(b_st, b_e);

    use SpirixState::*;
    let truth_table: Option<(u32, i8, SpirixState)> = match (a_state, b_state) {
        // Undefined dominates.
        (Undefined, _) => Some((a_st, a_e, Undefined)),
        (_, Undefined) => Some((b_st, b_e, Undefined)),

        // Spirix-specific: only literal Infinity × Zero is undefined; exploded
        // (= "value beyond max representable, but not literally infinity") ×
        // zero gives zero, because zero is exact and "very large but finite"
        // times zero is still zero in the limit. IEEE collapses inf and
        // overflow-saturated values together, so this is a Spirix-side
        // refinement of the IEEE × semantics.
        (Zero, Infinity) | (Infinity, Zero) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),

        // 0 × anything (incl. exploded) = 0.
        (Zero, _) | (_, Zero) => Some((0, AMBIG_EXP, Zero)),

        // inf × anything (non-zero) = inf (signless).
        (Infinity, _) | (_, Infinity) => Some((0xFFFFFF, AMBIG_EXP, Infinity)),

        // exploded × vanished or vanished × exploded → undefined.
        (PosExploded | NegExploded, PosVanished | NegVanished) |
        (PosVanished | NegVanished, PosExploded | NegExploded) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),

        // exploded × exploded = exploded with combined sign.
        (PosExploded, PosExploded) | (NegExploded, NegExploded) =>
            Some((POS_ONE_EXPLODED, AMBIG_EXP, PosExploded)),
        (PosExploded, NegExploded) | (NegExploded, PosExploded) =>
            Some((NEG_ONE_EXPLODED, AMBIG_EXP, NegExploded)),

        // vanished × vanished = vanished (combined sign).
        (PosVanished, PosVanished) | (NegVanished, NegVanished) =>
            Some((POS_ONE_VANISHED, AMBIG_EXP, PosVanished)),
        (PosVanished, NegVanished) | (NegVanished, PosVanished) =>
            Some((NEG_ONE_VANISHED, AMBIG_EXP, NegVanished)),

        // exploded × normal or normal × exploded = exploded with combined sign.
        (PosExploded, Normal) | (Normal, PosExploded) => {
            let other = if matches!(a_state, Normal) { (a_st, a_e) } else { (b_st, b_e) };
            // Check sign of normal: storage MSB=1 means positive in N0.
            let normal_pos = (other.0 >> (FRAC - 1)) & 1 == 1;
            if normal_pos { Some((POS_ONE_EXPLODED, AMBIG_EXP, PosExploded)) }
            else { Some((NEG_ONE_EXPLODED, AMBIG_EXP, NegExploded)) }
        }
        (NegExploded, Normal) | (Normal, NegExploded) => {
            let other = if matches!(a_state, Normal) { (a_st, a_e) } else { (b_st, b_e) };
            let normal_pos = (other.0 >> (FRAC - 1)) & 1 == 1;
            if normal_pos { Some((NEG_ONE_EXPLODED, AMBIG_EXP, NegExploded)) }
            else { Some((POS_ONE_EXPLODED, AMBIG_EXP, PosExploded)) }
        }

        // vanished × normal or normal × vanished = vanished (combined sign).
        (PosVanished, Normal) | (Normal, PosVanished) => {
            let other = if matches!(a_state, Normal) { (a_st, a_e) } else { (b_st, b_e) };
            let normal_pos = (other.0 >> (FRAC - 1)) & 1 == 1;
            if normal_pos { Some((POS_ONE_VANISHED, AMBIG_EXP, PosVanished)) }
            else { Some((NEG_ONE_VANISHED, AMBIG_EXP, NegVanished)) }
        }
        (NegVanished, Normal) | (Normal, NegVanished) => {
            let other = if matches!(a_state, Normal) { (a_st, a_e) } else { (b_st, b_e) };
            let normal_pos = (other.0 >> (FRAC - 1)) & 1 == 1;
            if normal_pos { Some((NEG_ONE_VANISHED, AMBIG_EXP, NegVanished)) }
            else { Some((POS_ONE_VANISHED, AMBIG_EXP, PosVanished)) }
        }

        (Normal, Normal) => None,
    };

    if let Some(r) = truth_table { return r; }

    // Normal × Normal via f64 arithmetic.
    let af = spirix_to_f64(a_st, a_e);
    let bf = spirix_to_f64(b_st, b_e);
    let rf = af * bf;
    f64_to_spirix(rf)
}

fn emit<W: Write>(w: &mut W, a: u32, ae: i8, b: u32, be: i8) -> std::io::Result<()> {
    let (g, ge, gs) = gold(a, ae, b, be);
    writeln!(w, "{:06x} {} {:06x} {} {:06x} {} {}",
             a & 0xFFFFFF, ae as i32,
             b & 0xFFFFFF, be as i32,
             g & 0xFFFFFF, ge as i32,
             state_code(gs))
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).map(|s| s.parse().unwrap_or(100_000)).unwrap_or(100_000);
    let path = args.get(2).cloned().unwrap_or_else(|| "verilog/mul_vectors.txt".to_string());

    let f = File::create(&path)?;
    let mut w = BufWriter::new(f);

    // Edge-case grid.
    let edges: &[(u32, i8)] = &[
        (NEG_ONE_NORMAL, 1), (POS_ONE_NORMAL, 1),
        (POS_ONE_NORMAL, 0), (NEG_ONE_NORMAL, 0),
        (POS_ONE_NORMAL, MAX_EXP), (NEG_ONE_NORMAL, MAX_EXP),
        (POS_ONE_NORMAL, MIN_EXP), (NEG_ONE_NORMAL, MIN_EXP),
        (0xFFFFFF, 0), (0x000001, 0), (0xC00000, 3), (0x400000, 3),
        (0x000000, AMBIG_EXP), (0xFFFFFF, AMBIG_EXP),
        (POS_ONE_EXPLODED, AMBIG_EXP), (NEG_ONE_EXPLODED, AMBIG_EXP),
        (POS_ONE_VANISHED, AMBIG_EXP), (NEG_ONE_VANISHED, AMBIG_EXP),
        (UNDEF_CANONICAL, AMBIG_EXP),
    ];
    for &(a, ae) in edges {
        for &(b, be) in edges {
            emit(&mut w, a, ae, b, be)?;
        }
    }

    // Random pairs from f32.
    let mut lfsr = Lfsr64::new(0xCAFE_BEEF_DEAD_F00D);
    for _ in 0..n {
        let af = lfsr.next_f32();
        let bf = lfsr.next_f32();
        let (a, ae, _) = f32_to_spirix(af);
        let (b, be, _) = f32_to_spirix(bf);
        emit(&mut w, a, ae, b, be)?;
    }

    w.flush()?;
    let total = edges.len() * edges.len() + n as usize;
    eprintln!("Wrote {} multiply vectors to {}", total, path);
    Ok(())
}
