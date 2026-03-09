/// Generate test vectors for spirix_alu_addbit (F3E3).
///
/// Input distribution:
///   75% normal×normal (N1 fracs, mixed exponent proximity)
///   25% edge cases (at least one input is non-normal):
///     - zero, infinity, exploded±, vanished±, undefined
///
/// Output: hex file with lines "op a_frac a_exp b_frac b_exp r_frac r_exp"
/// Fractions MSB-aligned, exponents LSB-aligned (sign-extended i8 → u64).
/// Op: 0=ADD 1=SUB 2=AND 3=OR 4=XOR
use spirix::Scalar;

type S = Scalar<i8, i8>;

const AMBIG: i8 = -128; // 0x80

fn msb_frac(v: i8) -> u64 {
    ((v as u8) as u64) << 56
}

fn lsb_exp(v: i8) -> u64 {
    v as i64 as u64  // sign-extend to 64 bits
}

fn emit(op: u8, a: S, b: S, r: S) {
    println!(
        "{:02x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
        op,
        msb_frac(a.fraction), lsb_exp(a.exponent),
        msb_frac(b.fraction), lsb_exp(b.exponent),
        msb_frac(r.fraction), lsb_exp(r.exponent),
    );
}

fn compute(op: u8, a: S, b: S) -> S {
    match op {
        0 => a + b,
        1 => a - b,
        2 => a & b,
        3 => a | b,
        4 => a ^ b,
        _ => unreachable!(),
    }
}

/// Simple deterministic PRNG (xorshift64)
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn next_i8(&mut self) -> i8 {
        self.next() as i8
    }
    /// Random N1 fraction: top 2 bits differ (01... or 10...)
    fn random_n1_frac(&mut self) -> i8 {
        loop {
            let f = self.next_i8();
            // N1: bit7 != bit6
            if ((f >> 7) & 1) != ((f >> 6) & 1) {
                return f;
            }
        }
    }
    /// Random normal exponent (anything except AMBIG)
    fn random_exp(&mut self) -> i8 {
        loop {
            let e = self.next_i8();
            if e != AMBIG {
                return e;
            }
        }
    }
}

fn main() {
    let mut rng = Rng(0xDEADBEEFCAFEBABE);
    let mut count = 0u64;

    // --- Edge case categories ---
    let edge_values: Vec<(&str, i8, i8)> = vec![
        ("zero",        0,    AMBIG),
        ("infinity",   -128,  AMBIG),  // NEG_ONE with AMBIG = infinity
        ("exploded+",   64,   AMBIG),  // POS_HALF + AMBIG
        ("exploded-",  -65,   AMBIG),  // N1 negative + AMBIG
        ("vanished+",   32,   AMBIG),  // non-N1 positive + AMBIG
        ("vanished-",  -33,   AMBIG),  // non-N1 negative + AMBIG
        ("undefined+",  1,    AMBIG),  // non-N1 small + AMBIG
        ("undefined-", -2,    AMBIG),  // non-N1 small neg + AMBIG
    ];

    // --- Part 1: Edge cases (25%) — all pairs of edge values × all ops ---
    // Plus edge × normal and normal × edge
    for op in 0u8..5 {
        // Edge × edge
        for (_aname, af, ae) in &edge_values {
            for (_bname, bf, be) in &edge_values {
                let a = S::new(*af, *ae);
                let b = S::new(*bf, *be);
                let r = compute(op, a, b);
                emit(op, a, b, r);
                count += 1;
            }
        }

        // Edge × normal (sampled exponents)
        for (_aname, af, ae) in &edge_values {
            for &be in &[-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for &bf in &[64i8, 65, 96, 127, -65, -96, -128] {
                    let a = S::new(*af, *ae);
                    let b = S::new(bf, be);
                    let r = compute(op, a, b);
                    emit(op, a, b, r);
                    count += 1;
                }
            }
        }

        // Normal × edge
        for (_bname, bf, be) in &edge_values {
            for &ae in &[-127i8, -64, -1, 0, 1, 64, 126, 127] {
                for &af in &[64i8, 65, 96, 127, -65, -96, -128] {
                    let a = S::new(af, ae);
                    let b = S::new(*bf, *be);
                    let r = compute(op, a, b);
                    emit(op, a, b, r);
                    count += 1;
                }
            }
        }
    }

    // --- Part 2: Normal × normal (75%) with exponent proximity ---
    // 4 modes: identical, close (±3), moderate (±15), random
    let normal_count = count * 3; // target 3× the edge count
    let tests_per_op = normal_count / 5;

    for op in 0u8..5 {
        for i in 0..tests_per_op {
            let af = rng.random_n1_frac();
            let bf = rng.random_n1_frac();
            let ae = rng.random_exp();

            let mode = i % 4;
            let be = match mode {
                0 => ae,                                              // identical
                1 => {                                                // close ±3
                    let delta = ((rng.next() % 7) as i8) - 3;
                    let e = ae.wrapping_add(delta);
                    if e == AMBIG { ae } else { e }
                }
                2 => {                                                // moderate ±15
                    let delta = ((rng.next() % 31) as i8) - 15;
                    let e = ae.wrapping_add(delta);
                    if e == AMBIG { ae } else { e }
                }
                3 => rng.random_exp(),                                // fully random
                _ => unreachable!(),
            };

            let a = S::new(af, ae);
            let b = S::new(bf, be);
            let r = compute(op, a, b);
            emit(op, a, b, r);
            count += 1;
        }
    }

    eprintln!("Generated {} vectors ({} edge, {} normal)",
              count, count - tests_per_op * 5, tests_per_op * 5);
}
