// Generate test vectors for Spirix F5E4 converter verification Run: cd /mnt/Octopus/Code/spirix && cargo run --example gen_calc_vectors 2>/dev/null
use spirix::Scalar;
type S = Scalar<i32, i16>;

fn main() {
    let values: Vec<(&str, S)> = vec![
        ("zero", S::from(0u8)),
        ("one", S::from(1u8)),
        ("two", S::from(2u8)),
        ("three", S::from(3u8)),
        ("seven", S::from(7u8)),
        ("eleven", S::from(11u8)),
        ("twelve", S::from(12u8)),
        ("twenty_four", S::from(24u8)),
        ("one_forty_four", S::from(144u16)),
        ("neg_one", -S::from(1u8)),
        ("neg_three", -S::from(3u8)),
        ("neg_twelve", -S::from(12u8)),
    ];

    println!("// Spirix F5E4 test vectors (Scalar<i32, i16>)");
    println!("// Format: frac_hex exp_decimal label");
    for (label, val) in &values {
        let frac = val.fraction as u32;
        let exp = val.exponent;
        println!("// {}: frac=0x{:08X} exp={}", label, frac, exp);
    }
}
