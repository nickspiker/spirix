//! Test vectors for spirix_sqrt vs IEEE binary32 sqrt.

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

fn gold(a_st: u32, a_e: i8) -> (u32, i8, SpirixState) {
    let state = classify(a_st, a_e);
    use SpirixState::*;
    let truth: Option<(u32, i8, SpirixState)> = match state {
        Undefined => Some((a_st, a_e, Undefined)),
        Zero => Some((0, AMBIG_EXP, Zero)),
        Infinity => Some((0xFFFFFF, AMBIG_EXP, Infinity)),
        // sqrt of exploded/vanished/negative → undefined.
        PosExploded | NegExploded | PosVanished | NegVanished | NegExploded =>
            Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined)),
        Normal => {
            // Check sign: N0 storage MSB=1 means positive.
            let pos = (a_st >> (FRAC - 1)) & 1 == 1;
            if !pos {
                Some((UNDEF_CANONICAL, AMBIG_EXP, Undefined))
            } else {
                None
            }
        }
    };

    if let Some(r) = truth { return r; }

    // Positive normal → IEEE sqrt.
    let af = spirix_to_f64(a_st, a_e);
    let rf = af.sqrt();
    f64_to_spirix(rf)
}

fn emit<W: Write>(w: &mut W, a: u32, ae: i8) -> std::io::Result<()> {
    let (g, ge, gs) = gold(a, ae);
    writeln!(w, "{:06x} {} {:06x} {} {}",
             a & 0xFFFFFF, ae as i32,
             g & 0xFFFFFF, ge as i32,
             state_code(gs))
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let n: u64 = args.get(1).map(|s| s.parse().unwrap_or(100_000)).unwrap_or(100_000);
    let path = args.get(2).cloned().unwrap_or_else(|| "verilog/sqrt_vectors.txt".to_string());

    let f = File::create(&path)?;
    let mut w = BufWriter::new(f);

    let edges: &[(u32, i8)] = &[
        (NEG_ONE_NORMAL, 1), (POS_ONE_NORMAL, 1),
        (POS_ONE_NORMAL, 0), (NEG_ONE_NORMAL, 0),
        (POS_ONE_NORMAL, MAX_EXP), (NEG_ONE_NORMAL, MAX_EXP),
        (POS_ONE_NORMAL, MIN_EXP), (NEG_ONE_NORMAL, MIN_EXP),
        (POS_ONE_NORMAL, 2), (POS_ONE_NORMAL, 4), (POS_ONE_NORMAL, -2),
        (0xFFFFFF, 0), (0xC00000, 3), (0x800000, 100), (0x800001, 0),
        (0x000000, AMBIG_EXP), (0xFFFFFF, AMBIG_EXP),
        (POS_ONE_EXPLODED, AMBIG_EXP), (NEG_ONE_EXPLODED, AMBIG_EXP),
        (POS_ONE_VANISHED, AMBIG_EXP), (NEG_ONE_VANISHED, AMBIG_EXP),
        (UNDEF_CANONICAL, AMBIG_EXP),
    ];
    for &(a, ae) in edges {
        emit(&mut w, a, ae)?;
    }

    let mut lfsr = Lfsr64::new(0x5A5A_BEEF_CAFE_5A5A);
    for _ in 0..n {
        let af = lfsr.next_f32();
        let (a, ae, _) = f32_to_spirix(af);
        emit(&mut w, a, ae)?;
    }

    w.flush()?;
    let total = edges.len() + n as usize;
    eprintln!("Wrote {} sqrt vectors to {}", total, path);
    Ok(())
}
