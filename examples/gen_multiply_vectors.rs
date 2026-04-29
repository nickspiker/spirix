/// Generate test vectors for minimal multiply core validation. Outputs: a_frac a_exp b_frac b_exp result_frac result_exp (hex, signed)
use spirix::{Scalar, ScalarF3E3};

fn main() {
    let mut lfsr: u64 = 0xDEADBEEFCAFE1234;
    let ambig: i8 = -128;
    let mut count = 0;

    for _ in 0..100000 {
        lfsr = lfsr.wrapping_shl(1)
            ^ if (lfsr >> 63) ^ (lfsr >> 62) ^ (lfsr >> 60) ^ (lfsr >> 59) & 1 != 0 {
                1
            } else {
                0
            };

        // Generate N1-normalized fractions (top 2 bits differ)
        let bit7 = ((lfsr >> 7) & 1) as u8;
        let a_frac = ((bit7 << 7) | ((!bit7 & 1) << 6) | ((lfsr as u8) & 0x3F)) as i8;

        let bit15 = ((lfsr >> 15) & 1) as u8;
        let b_frac = ((bit15 << 7) | ((!bit15 & 1) << 6) | (((lfsr >> 8) as u8) & 0x3F)) as i8;

        let mut a_exp = ((lfsr >> 16) & 0xFF) as i8;
        if a_exp == ambig {
            a_exp = ambig + 1;
        }
        let mut b_exp = ((lfsr >> 24) & 0xFF) as i8;
        if b_exp == ambig {
            b_exp = ambig + 1;
        }

        let a = Scalar::<i8, i8> {
            fraction: a_frac,
            exponent: a_exp,
        };
        let b = Scalar::<i8, i8> {
            fraction: b_frac,
            exponent: b_exp,
        };

        let result = a * b;

        println!(
            "{:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            a_frac as u8,
            a_exp as u8,
            b_frac as u8,
            b_exp as u8,
            result.fraction as u8,
            result.exponent as u8,
        );
        count += 1;
    }
    eprintln!("Generated {} vectors", count);
}
