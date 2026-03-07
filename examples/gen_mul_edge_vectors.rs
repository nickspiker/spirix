/// Generate exhaustive multiply edge-case test vectors for F3E3 (FRAC=8, EXP=8).
/// Covers all pairs where at least one input has AMBIG exponent.
/// Output: hex file with lines "a_frac a_exp b_frac b_exp r_frac r_exp"
use spirix::{Scalar, ScalarF3E3};

fn main() {
    let ambig: i8 = -128; // AMBIGUOUS_EXPONENT for 8-bit
    let mut count = 0u64;

    // All pairs where both have AMBIG exponent (65536 pairs)
    for af in -128i8..=127 {
        for bf in -128i8..=127 {
            let a = Scalar::<i8, i8>::new(af, ambig);
            let b = Scalar::<i8, i8>::new(bf, ambig);
            let r = a * b;
            println!(
                "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                af as u8, ambig as u8, bf as u8, ambig as u8,
                r.fraction as u8, r.exponent as u8
            );
            count += 1;
        }
    }

    // All pairs where a has AMBIG, b is normal (sampled exponents)
    for ae in [ambig] {
        for af in -128i8..=127 {
            for be in [-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for bf in -128i8..=127 {
                    let a = Scalar::<i8, i8>::new(af, ae);
                    let b = Scalar::<i8, i8>::new(bf, be);
                    let r = a * b;
                    println!(
                        "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                        af as u8, ae as u8, bf as u8, be as u8,
                        r.fraction as u8, r.exponent as u8
                    );
                    count += 1;
                }
            }
        }
    }

    // All pairs where b has AMBIG, a is normal (sampled exponents)
    for be in [ambig] {
        for bf in -128i8..=127 {
            for ae in [-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for af in -128i8..=127 {
                    let a = Scalar::<i8, i8>::new(af, ae);
                    let b = Scalar::<i8, i8>::new(bf, be);
                    let r = a * b;
                    println!(
                        "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                        af as u8, ae as u8, bf as u8, be as u8,
                        r.fraction as u8, r.exponent as u8
                    );
                    count += 1;
                }
            }
        }
    }

    eprintln!("Generated {} vectors", count);
}
