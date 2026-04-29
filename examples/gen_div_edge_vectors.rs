/// Generate div edge-case test vectors for F3E3 (FRAC=8, EXP=8). Mirrors the Verilog iterative divide edge case chain. Output: hex file with lines "a_frac a_exp b_frac b_exp r_frac r_exp"
use spirix::Scalar;

const AMBIG: i8 = -128;
const UNDEF_NEG_DIV_NEG: i8 = 0xE9u8 as i8;
const UNDEF_TF_DIV_TF: i8 = 0x16;
const UNDEF_GENERAL: i8 = 0xFEu8 as i8;

fn is_n1(f: i8) -> bool {
    let top2 = ((f as u8) >> 6) & 3;
    top2 == 0b01 || top2 == 0b10
}

fn is_n2(f: i8) -> bool {
    if is_n1(f) {
        return false;
    }
    let bit6 = ((f as u8) >> 6) & 1;
    let bit5 = ((f as u8) >> 5) & 1;
    bit6 != bit5
}

fn is_n0(f: i8) -> bool {
    f == 0 || f == -1 // 0x00 or 0xFF — all 8 bits same prefix
}

fn top3_same(f: i8) -> bool {
    let b7 = ((f as u8) >> 7) & 1;
    let b6 = ((f as u8) >> 6) & 1;
    let b5 = ((f as u8) >> 5) & 1;
    b7 == b6 && b6 == b5
}

fn is_undef(f: i8) -> bool {
    !is_n0(f) && top3_same(f)
}

/// Compute expected result matching the Verilog iterative divide's edge case chain
fn verilog_div_result(af: i8, ae: i8, bf: i8, be: i8) -> (i8, i8) {
    let a_is_ambig = ae == AMBIG;
    let b_is_ambig = be == AMBIG;

    if a_is_ambig || b_is_ambig {
        if is_undef(af) {
            return (af, ae);
        }
        if is_undef(bf) {
            return (bf, be);
        }
        if b_is_ambig && bf == 0 {
            if a_is_ambig && af == 0 {
                return (UNDEF_NEG_DIV_NEG, AMBIG);
            }
            return (-1, AMBIG); // infinity
        }
        if a_is_ambig && af == -1 {
            if b_is_ambig && bf == -1 {
                return (UNDEF_TF_DIV_TF, AMBIG);
            }
            return (-1, AMBIG);
        }
        if (a_is_ambig && af == 0) || (b_is_ambig && bf == -1) {
            return (0, AMBIG);
        }
        let a_exploded = a_is_ambig && is_n1(af);
        let b_exploded = b_is_ambig && is_n1(bf);
        let a_vanished = is_n2(af);
        let b_vanished = is_n2(bf);
        if a_exploded && b_exploded {
            return (UNDEF_TF_DIV_TF, AMBIG);
        }
        if a_vanished && b_vanished {
            return (UNDEF_NEG_DIV_NEG, AMBIG);
        }
        if a_vanished || b_vanished {
            return (UNDEF_GENERAL, AMBIG);
        }
        // exploded || exploded, or remaining AMBIG → GENERAL
        return (UNDEF_GENERAL, AMBIG);
    }
    if bf == 0 {
        return (-1, AMBIG);
    }
    // Normal — skip
    (0x7F, 0x7F) // sentinel
}

fn main() {
    let mut count = 0u64;

    // All pairs where both have AMBIG exponent (65536 pairs)
    for af in -128i8..=127 {
        for bf in -128i8..=127 {
            let (rf, re) = verilog_div_result(af, AMBIG, bf, AMBIG);
            if rf == 0x7F && re == 0x7F {
                continue;
            }
            println!(
                "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                af as u8, AMBIG as u8, bf as u8, AMBIG as u8, rf as u8, re as u8
            );
            count += 1;
        }
    }

    // AMBIG a × normal b (N1-normalized fracs)
    let sample_exps: Vec<i8> = vec![-127, -64, -1, 0, 1, 64, 126, 127];
    for af in -128i8..=127 {
        for &be in &sample_exps {
            for bf in (-128i8..=127).filter(|&f| is_n1(f)) {
                let (rf, re) = verilog_div_result(af, AMBIG, bf, be);
                if rf == 0x7F && re == 0x7F {
                    continue;
                }
                println!(
                    "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                    af as u8, AMBIG as u8, bf as u8, be as u8, rf as u8, re as u8
                );
                count += 1;
            }
        }
    }

    // Normal a × AMBIG b
    for bf in -128i8..=127 {
        for &ae in &sample_exps {
            for af in (-128i8..=127).filter(|&f| is_n1(f)) {
                let (rf, re) = verilog_div_result(af, ae, bf, AMBIG);
                if rf == 0x7F && re == 0x7F {
                    continue;
                }
                println!(
                    "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                    af as u8, ae as u8, bf as u8, AMBIG as u8, rf as u8, re as u8
                );
                count += 1;
            }
        }
    }

    eprintln!("Generated {} vectors", count);
}
