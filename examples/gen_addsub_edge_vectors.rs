/// Generate exhaustive add/sub edge-case test vectors for F3E3 (FRAC=8, EXP=8). Covers all pairs where at least one input has AMBIG exponent. Output: hex file with lines "a_frac a_exp b_frac b_exp sub r_frac r_exp"
use spirix::Scalar;

fn main() {
    let ambig: i8 = -128;
    let mut count = 0u64;

    // Test both add (sub=0) and sub (sub=1)
    for sub_flag in [false, true] {
        // All pairs where both have AMBIG exponent (65536 pairs × 2 ops)
        for af in -128i8..=127 {
            for bf in -128i8..=127 {
                let a = Scalar::<i8, i8>::new(af, ambig);
                let b = Scalar::<i8, i8>::new(bf, ambig);
                let r = if sub_flag { a - b } else { a + b };
                println!(
                    "{:02x} {:02x} {:02x} {:02x} {:01x} {:02x} {:02x}",
                    af as u8,
                    ambig as u8,
                    bf as u8,
                    ambig as u8,
                    sub_flag as u8,
                    r.fraction as u8,
                    r.exponent as u8
                );
                count += 1;
            }
        }

        // Pairs where a has AMBIG, b is normal (sampled exponents)
        for af in -128i8..=127 {
            for be in [-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for bf in -128i8..=127 {
                    let a = Scalar::<i8, i8>::new(af, ambig);
                    let b = Scalar::<i8, i8>::new(bf, be);
                    let r = if sub_flag { a - b } else { a + b };
                    println!(
                        "{:02x} {:02x} {:02x} {:02x} {:01x} {:02x} {:02x}",
                        af as u8,
                        ambig as u8,
                        bf as u8,
                        be as u8,
                        sub_flag as u8,
                        r.fraction as u8,
                        r.exponent as u8
                    );
                    count += 1;
                }
            }
        }

        // Pairs where b has AMBIG, a is normal (sampled exponents)
        for bf in -128i8..=127 {
            for ae in [-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for af in -128i8..=127 {
                    let a = Scalar::<i8, i8>::new(af, ae);
                    let b = Scalar::<i8, i8>::new(bf, ambig);
                    let r = if sub_flag { a - b } else { a + b };
                    println!(
                        "{:02x} {:02x} {:02x} {:02x} {:01x} {:02x} {:02x}",
                        af as u8,
                        ae as u8,
                        bf as u8,
                        ambig as u8,
                        sub_flag as u8,
                        r.fraction as u8,
                        r.exponent as u8
                    );
                    count += 1;
                }
            }
        }
    }

    eprintln!("Generated {} vectors", count);
}
