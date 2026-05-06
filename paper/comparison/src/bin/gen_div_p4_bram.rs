//! BRAM init for the external-clock divide-P4 silicon test harness.
//!
//! Emits a `.mem` file with packed 128-bit entries (one per line, hex), where
//! each entry is one (a, b, expected_q, expected_state) test case for the
//! spirix_divide PARALLEL=4 DUT clocked from the RC oscillator.
//!
//! Bit layout (MSB → LSB, 128 bits per word):
//!   [127:104] a_frac      (24)
//!   [103: 96] a_exp       (8, signed two's complement)
//!   [ 95: 72] b_frac      (24)
//!   [ 71: 64] b_exp       (8)
//!   [ 63: 40] exp_q_frac  (24)
//!   [ 39: 32] exp_q_exp   (8)
//!   [ 31: 28] exp_state   (4: 0=Normal, 1=Zero, 2=PosVan, 3=NegVan,
//!                              4=PosExp, 5=NegExp, 6=Inf, 7=Undef)
//!   [ 27:  0] reserved    (28, zero)
//!
//! 256 entries: every edge × edge combination (19² = 361 → first 64 chosen
//! to span the truth-table cases), padded to 256 with LFSR-driven random
//! Normal × Normal pairs.

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
    let truth: Option<(u32, i8, SpirixState)> = match (a_state, b_state) {
        (Undefined, _) => Some((a_st, a_e, Undefined)),
        (_, Undefined) => Some((b_st, b_e, Undefined)),
        (Zero, Zero) => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (_, Zero) => Some((0xFFFFFF, AMBIG_EXP, Infinity)),
        (Zero, _) => Some((0, AMBIG_EXP, Zero)),
        (Infinity, Infinity) => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (Infinity, _) => Some((0xFFFFFF, AMBIG_EXP, Infinity)),
        (_, Infinity) => Some((0, AMBIG_EXP, Zero)),
        (PosExploded | NegExploded, PosExploded | NegExploded) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosVanished | NegVanished, PosVanished | NegVanished) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosExploded | NegExploded, _) | (_, PosExploded | NegExploded) |
        (PosVanished  | NegVanished,  _) | (_, PosVanished  | NegVanished) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (Normal, Normal) => None,
    };

    if let Some(r) = truth { return r; }

    let af = spirix_to_f64(a_st, a_e);
    let bf = spirix_to_f64(b_st, b_e);
    let rf = af / bf;
    f64_to_spirix(rf)
}

fn pack_entry(a: u32, ae: i8, b: u32, be: i8,
              g: u32, ge: i8, gs: u8) -> u128 {
    let a24 = ((a as u128) & 0xFFFFFF) << 104;
    let ae8 = ((ae as u8 as u128) & 0xFF) << 96;
    let b24 = ((b as u128) & 0xFFFFFF) << 72;
    let be8 = ((be as u8 as u128) & 0xFF) << 64;
    let g24 = ((g as u128) & 0xFFFFFF) << 40;
    let ge8 = ((ge as u8 as u128) & 0xFF) << 32;
    let gs4 = ((gs as u128) & 0xF) << 28;
    a24 | ae8 | b24 | be8 | g24 | ge8 | gs4
}

fn emit<W: Write>(w: &mut W, a: u32, ae: i8, b: u32, be: i8) -> std::io::Result<()> {
    let (g, ge, gs) = gold(a, ae, b, be);
    let packed = pack_entry(a, ae, b, be, g, ge, state_code(gs));
    writeln!(w, "{:032x}", packed)
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap_or_else(|| "div_p4_vectors.mem".to_string());
    let n_entries: usize = 256;

    let f = File::create(&path)?;
    let mut w = BufWriter::new(f);

    let edges: &[(u32, i8)] = &[
        (NEG_ONE_NORMAL, 1), (POS_ONE_NORMAL, 1),
        (POS_ONE_NORMAL, 0), (NEG_ONE_NORMAL, 0),
        (POS_ONE_NORMAL, MAX_EXP), (NEG_ONE_NORMAL, MAX_EXP),
        (POS_ONE_NORMAL, MIN_EXP), (NEG_ONE_NORMAL, MIN_EXP),
        (0xFFFFFF, 0), (0x000001, 0),
        (0x000000, AMBIG_EXP),                       // Zero
        (0xFFFFFF, AMBIG_EXP),                       // Infinity
        (POS_ONE_EXPLODED, AMBIG_EXP),
        (NEG_ONE_EXPLODED, AMBIG_EXP),
        (POS_ONE_VANISHED, AMBIG_EXP),
        (NEG_ONE_VANISHED, AMBIG_EXP),
    ];

    let mut count = 0usize;

    // First pass: a small representative spread of edge × edge cases.
    // Take every other (a, b) combination from a 16×16 grid, giving 128 entries.
    for (i, &(a, ae)) in edges.iter().enumerate() {
        for (j, &(b, be)) in edges.iter().enumerate() {
            if (i + j) & 1 == 0 && count < n_entries / 2 {
                emit(&mut w, a, ae, b, be)?;
                count += 1;
            }
        }
    }

    // Second pass: fill remainder with LFSR-driven random Normal × Normal.
    let mut lfsr = Lfsr64::new(0x1234_5678_9ABC_DEF0);
    while count < n_entries {
        let af = lfsr.next_f32();
        let bf = lfsr.next_f32();
        let (a, ae, _) = f32_to_spirix(af);
        let (b, be, _) = f32_to_spirix(bf);
        // Skip degenerate entries where input is non-Normal (gold shortcut).
        if classify(a, ae) == SpirixState::Normal &&
           classify(b, be) == SpirixState::Normal {
            emit(&mut w, a, ae, b, be)?;
            count += 1;
        }
    }

    w.flush()?;
    eprintln!("Wrote {} entries to {} ({} bits each, packed)", count, path, 128);
    Ok(())
}
