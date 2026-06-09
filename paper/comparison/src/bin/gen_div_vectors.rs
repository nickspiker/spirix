//! Test vectors for spirix_divide vs IEEE binary32 divide.

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
        // Undefined dominates.
        (Undefined, _) => Some((a_st, a_e, Undefined)),
        (_, Undefined) => Some((b_st, b_e, Undefined)),

        // Divide by zero: 0/0 = undef, anything/0 = inf.
        (Zero, Zero) => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (_, Zero) => Some((0xFFFFFF, AMBIG_EXP, Infinity)),

        // 0/anything (non-zero) = 0.
        (Zero, _) => Some((0, AMBIG_EXP, Zero)),

        // inf/inf = undef.
        (Infinity, Infinity) => Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),

        // inf/normal = inf, normal/inf = 0.
        (Infinity, _) => Some((0xFFFFFF, AMBIG_EXP, Infinity)),
        (_, Infinity) => Some((0, AMBIG_EXP, Zero)),

        // exploded/exploded or vanished/vanished = undef.
        (PosExploded | NegExploded, PosExploded | NegExploded) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        (PosVanished | NegVanished, PosVanished | NegVanished) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),

        // All other non-Normal × Anything combinations: Spirix's divide truth table returns Undefined for these (no IEEE equivalent). Gold expects Undefined-state; tb accepts any LSBC≥3 storage at AMBIG_EXP.
        (PosExploded | NegExploded, _) | (_, PosExploded | NegExploded) |
        (PosVanished  | NegVanished,  _) | (_, PosVanished  | NegVanished) =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),

        (Normal, Normal) => None,
    };

    if let Some(r) = truth { return r; }

    // Normal / Normal via f64.
    let af = spirix_to_f64(a_st, a_e);
    let bf = spirix_to_f64(b_st, b_e);
    let rf = af / bf;
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
    let path = args.get(2).cloned().unwrap_or_else(|| "verilog/div_vectors.txt".to_string());

    let f = File::create(&path)?;
    let mut w = BufWriter::new(f);

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

    let mut lfsr = Lfsr64::new(0x1234_5678_9ABC_DEF0);
    for _ in 0..n {
        let af = lfsr.next_f32();
        let bf = lfsr.next_f32();
        let (a, ae, _) = f32_to_spirix(af);
        let (b, be, _) = f32_to_spirix(bf);
        emit(&mut w, a, ae, b, be)?;
    }

    w.flush()?;
    let total = edges.len() * edges.len() + n as usize;
    eprintln!("Wrote {} divide vectors to {}", total, path);
    Ok(())
}
