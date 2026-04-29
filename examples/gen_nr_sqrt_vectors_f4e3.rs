/// Generate F4E3 (i16, i8) test vectors for NR sqrt core validation. Outputs: a_frac a_exp result_frac result_exp (hex, signed)
use spirix::Scalar;

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

        // Generate N1-normalized 16-bit fractions (top 2 bits differ)
        let bit15 = ((lfsr >> 15) & 1) as u16;
        let a_frac = ((bit15 << 15) | ((!bit15 & 1) << 14) | ((lfsr as u16) & 0x3FFF)) as i16;

        let mut a_exp = ((lfsr >> 16) & 0xFF) as i8;
        if a_exp == ambig {
            a_exp = ambig + 1;
        }

        let a = Scalar::<i16, i8> {
            fraction: a_frac,
            exponent: a_exp,
        };

        let result = a.sqrt();

        println!(
            "{:04x} {:02x} {:04x} {:02x}",
            a_frac as u16, a_exp as u8, result.fraction as u16, result.exponent as u8,
        );
        count += 1;
    }
    eprintln!("Generated {} vectors", count);
}
