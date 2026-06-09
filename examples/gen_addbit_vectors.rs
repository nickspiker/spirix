/// Generate test vectors for spirix_alu_addbit — all 16 frac×exp width combos.
///
/// Input distribution per width: 25% edge cases (at least one input is non-normal) 75% normal×normal (N1 fracs, mixed exponent proximity)
///
/// Output: hex lines "op fw ew a_frac a_exp b_frac b_exp r_frac r_exp" Fractions MSB-aligned to 64 bits, exponents LSB-aligned with universal AMBIG. Op: 0=ADD 1=SUB 2=AND 3=OR 4=XOR
use spirix::Scalar;

/// Simple deterministic PRNG (xorshift64)
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn msb_frac_64<F: Copy + Into<i64>>(v: F, bits: u32) -> u64 {
    let raw = v.into() as u64;
    let mask = if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    (raw & mask) << (64 - bits)
}

fn lsb_exp_64<E: Copy + Into<i64> + PartialEq>(v: E, min: E) -> u64 {
    if v == min {
        0x8000_0000_0000_0000u64
    } else {
        v.into() as u64
    }
}

macro_rules! gen_width {
    ($f:ty, $e:ty, $fw:expr, $ew:expr, $rng:expr, $count:expr) => {{
        type S = Scalar<$f, $e>;
        let ambig: $e = <$e>::MIN;
        let fbits: u32 = <$f>::BITS;

        let emit = |op: u8, a: S, b: S, r: S| {
            println!(
                "{:02x} {} {} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                op, $fw, $ew,
                msb_frac_64(a.fraction, fbits), lsb_exp_64(a.exponent, ambig),
                msb_frac_64(b.fraction, fbits), lsb_exp_64(b.exponent, ambig),
                msb_frac_64(r.fraction, fbits), lsb_exp_64(r.exponent, ambig),
            );
        };

        let compute = |op: u8, a: S, b: S| -> S {
            // F6 (64-bit frac): pre-align operands to match hardware barrel truncation. Verilog shifts small RIGHT (drops bits); Rust uses i128 and shifts big LEFT (no loss). Pre-truncating the small operand makes both paths produce identical results.
            let (pa, pb) = if $fw == 3 && a.exponent != ambig && b.exponent != ambig {
                let ae = a.exponent as i128;
                let be = b.exponent as i128;
                if ae >= be {
                    let d = ae - be;
                    if d > 0 && d < fbits as i128 {
                        (a, S::new(b.fraction >> d as u32, a.exponent))
                    } else { (a, b) }
                } else {
                    let d = be - ae;
                    if d > 0 && d < fbits as i128 {
                        (S::new(a.fraction >> d as u32, b.exponent), b)
                    } else { (a, b) }
                }
            } else { (a, b) };
            match op {
                0 => pa + pb,
                1 => pa - pb,
                2 => pa & pb,
                3 => pa | pb,
                4 => pa ^ pb,
                _ => unreachable!(),
            }
        };

        // Edge values for this width
        let edge_values: Vec<($f, $e)> = vec![
            (0, ambig),                                                         // zero
            (<$f>::MIN, ambig),                                                 // infinity
            ((1 as $f) << (fbits - 2), ambig),                                 // exploded+
            ((((<$f>::MIN) >> 1) as $f).wrapping_add(((<$f>::MIN) >> 2) as $f), ambig), // exploded-
            ((1 as $f) << (fbits - 3), ambig),                                 // vanished+
            ((-1 as $f) << (fbits - 3), ambig),                                // vanished-
            ((1 as $f) << (fbits - 4), ambig),                                 // undefined+
            ((-1 as $f) << (fbits - 4), ambig),                                // undefined-
        ];

        // Sample normal exponents for edge×normal
        let sample_exps: Vec<$e> = {
            let max_e = <$e>::MAX;
            let min_e = <$e>::MIN.wrapping_add(1 as $e);
            vec![min_e, -1 as $e, 0 as $e, 1 as $e, max_e,
                 max_e >> 1, min_e >> 1]
        };
        // Sample N1 fractions for edge×normal
        let sample_fracs: Vec<$f> = vec![
            (1 as $f) << (fbits - 2),              // POS_HALF
            <$f>::MIN,                              // NEG_ONE
            ((1 as $f) << (fbits - 2)) | 1,        // POS_HALF+1
            <$f>::MAX,                              // max positive N1
            (<$f>::MIN >> 1).wrapping_add(1 as $f), // just past N1 boundary neg
        ];

        // --- Part 1: Edge cases ---
        for op in 0u8..5 {
            // Edge × edge
            for &(af, ae) in &edge_values {
                for &(bf, be) in &edge_values {
                    let a = S::new(af, ae);
                    let b = S::new(bf, be);
                    let r = compute(op, a, b);
                    emit(op, a, b, r);
                    *$count += 1;
                }
            }

            // Edge × normal
            for &(af, ae) in &edge_values {
                for &be in &sample_exps {
                    for &bf in &sample_fracs {
                        let a = S::new(af, ae);
                        let b = S::new(bf, be);
                        let r = compute(op, a, b);
                        emit(op, a, b, r);
                        *$count += 1;
                    }
                }
            }

            // Normal × edge
            for &(bf, be) in &edge_values {
                for &ae in &sample_exps {
                    for &af in &sample_fracs {
                        let a = S::new(af, ae);
                        let b = S::new(bf, be);
                        let r = compute(op, a, b);
                        emit(op, a, b, r);
                        *$count += 1;
                    }
                }
            }
        }

        let edge_count = *$count;

        // --- Part 2: Normal × normal with exponent proximity ---
        let normal_target = edge_count * 3;
        let per_op = normal_target / 5;

        for op in 0u8..5 {
            for i in 0u64..per_op {
                let af: $f = loop {
                    let f = $rng.next() as $f;
                    if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
                };
                let bf: $f = loop {
                    let f = $rng.next() as $f;
                    if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
                };
                let ae: $e = loop {
                    let e = $rng.next() as $e;
                    if e != ambig { break e; }
                };
                let be: $e = match i % 4 {
                    0 => ae,
                    1 => {
                        let d = ($rng.next() % 7) as $e - 3;
                        let e = ae.wrapping_add(d);
                        if e == ambig { ae } else { e }
                    }
                    2 => {
                        let d = ($rng.next() % 31) as $e - 15;
                        let e = ae.wrapping_add(d);
                        if e == ambig { ae } else { e }
                    }
                    3 => loop {
                        let e = $rng.next() as $e;
                        if e != ambig { break e; }
                    },
                    _ => unreachable!(),
                };

                let a = S::new(af, ae);
                let b = S::new(bf, be);
                let r = compute(op, a, b);
                emit(op, a, b, r);
                *$count += 1;
            }
        }

        // --- Part 3: Normal power-of-two pairs --- POS_HALF and NEG_ONE fractions trigger special negate handling in SUB. These are never hit by PRNG (too specific) so test them explicitly.
        let po2_fracs: Vec<$f> = vec![
            (1 as $f) << (fbits - 2),              // POS_HALF (fraction of ONE, HALF, TWO)
            <$f>::MIN,                              // NEG_ONE fraction
            <$f>::MAX,                              // EFFECTIVELY_POS_ONE fraction
            (<$f>::MIN).wrapping_add(1 as $f),      // EFFECTIVELY_NEG_ONE fraction
        ];
        let po2_exps: Vec<$e> = vec![-1 as $e, 0 as $e, 1 as $e, 2 as $e];
        for op in 0u8..5 {
            for &af in &po2_fracs {
                for &ae in &po2_exps {
                    for &bf in &po2_fracs {
                        for &be in &po2_exps {
                            let a = S::new(af, ae);
                            let b = S::new(bf, be);
                            let r = compute(op, a, b);
                            emit(op, a, b, r);
                            *$count += 1;
                        }
                    }
                }
            }
        }
    }};
}

fn main() {
    let mut rng = Rng(0xDEADBEEFCAFEBABE);
    let mut total = 0u64;

    // All 16 width combos
    let mut w_count = 0u64;
    gen_width!(i8, i8, 0, 0, &mut rng, &mut w_count);
    let c00 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i8, 1, 0, &mut rng, &mut w_count);
    let c10 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i8, 2, 0, &mut rng, &mut w_count);
    let c20 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i8, 3, 0, &mut rng, &mut w_count);
    let c30 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i16, 0, 1, &mut rng, &mut w_count);
    let c01 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i16, 1, 1, &mut rng, &mut w_count);
    let c11 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i16, 2, 1, &mut rng, &mut w_count);
    let c21 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i16, 3, 1, &mut rng, &mut w_count);
    let c31 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i32, 0, 2, &mut rng, &mut w_count);
    let c02 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i32, 1, 2, &mut rng, &mut w_count);
    let c12 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i32, 2, 2, &mut rng, &mut w_count);
    let c22 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i32, 3, 2, &mut rng, &mut w_count);
    let c32 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i64, 0, 3, &mut rng, &mut w_count);
    let c03 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i64, 1, 3, &mut rng, &mut w_count);
    let c13 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i64, 2, 3, &mut rng, &mut w_count);
    let c23 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i64, 3, 3, &mut rng, &mut w_count);
    let c33 = w_count;
    total += w_count;

    eprintln!("Generated {} total vectors across 16 width combos", total);
    eprintln!("  F3E3={} F4E3={} F5E3={} F6E3={}", c00, c10, c20, c30);
    eprintln!("  F3E4={} F4E4={} F5E4={} F6E4={}", c01, c11, c21, c31);
    eprintln!("  F3E5={} F4E5={} F5E5={} F6E5={}", c02, c12, c22, c32);
    eprintln!("  F3E6={} F4E6={} F5E6={} F6E6={}", c03, c13, c23, c33);
}
