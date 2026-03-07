/// Generate sqrt edge-case test vectors for F3E3 (FRAC=8, EXP=8).
/// Mirrors the Verilog iterative sqrt edge case chain.
/// Output: hex file with lines "a_frac a_exp r_frac r_exp"

const AMBIG: i8 = -128;
const UNDEF_SQRT_NEG: i8 = 0xF6u8 as i8;
const UNDEF_SQRT_EXPLOD: i8 = 0x08;
const UNDEF_SQRT_VANISH: i8 = 0xF7u8 as i8;

fn is_n1(f: i8) -> bool {
    let top2 = ((f as u8) >> 6) & 3;
    top2 == 0b01 || top2 == 0b10
}

fn is_n2(f: i8) -> bool {
    if is_n1(f) { return false; }
    let b6 = ((f as u8) >> 6) & 1;
    let b5 = ((f as u8) >> 5) & 1;
    b6 != b5
}

fn is_n0(f: i8) -> bool {
    f == 0 || f == -1
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

fn verilog_sqrt_result(af: i8, ae: i8) -> (i8, i8) {
    if ae == AMBIG {
        if is_undef(af) { return (af, ae); }
        if is_n0(af) { return (af, ae); }
        if is_n2(af) { return (UNDEF_SQRT_VANISH, AMBIG); }
        return (UNDEF_SQRT_EXPLOD, AMBIG);
    }
    if af < 0 { return (UNDEF_SQRT_NEG, AMBIG); }
    // Positive normal — skip
    (0x7F, 0x7F)
}

fn main() {
    let mut count = 0u64;

    // All 256 fracs with AMBIG exponent
    for af in -128i8..=127 {
        let (rf, re) = verilog_sqrt_result(af, AMBIG);
        if rf == 0x7F && re == 0x7F { continue; }
        println!(
            "{:02x} {:02x} {:02x} {:02x}",
            af as u8, AMBIG as u8, rf as u8, re as u8
        );
        count += 1;
    }

    // Negative normals with sampled exponents
    let sample_exps: Vec<i8> = vec![-127, -64, -1, 0, 1, 64, 126, 127];
    for &ae in &sample_exps {
        for af in -128i8..=-1 {
            if !is_n1(af) { continue; }
            let (rf, re) = verilog_sqrt_result(af, ae);
            if rf == 0x7F && re == 0x7F { continue; }
            println!(
                "{:02x} {:02x} {:02x} {:02x}",
                af as u8, ae as u8, rf as u8, re as u8
            );
            count += 1;
        }
    }

    eprintln!("Generated {} vectors", count);
}
