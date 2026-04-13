/// Generate edge-case test vectors for spirix_alu_unified (F3E3).
///
/// Tests all 13 ops across all combinations of input categories:
///   zero, infinity, exploded+, exploded-, vanished+, vanished-,
///   undefined, normal+, normal-, pos_half, neg_one, pos_small, neg_small
///
/// Output: hex file with lines "op a_frac a_exp b_frac b_exp r_frac r_exp [cmp_flags]"
/// where cmp_flags = "lt eq gt un" (4 hex digits, only for CMP op)
///
/// For MSB-aligned 64-bit output, F3E3 values are left-shifted:
///   frac_msb = (frac as u8) << 56, exp_msb = (exp as u8) << 56
use spirix::Scalar;
use std::cmp::Ordering;

type S = Scalar<i8, i8>;

const AMBIG: i8 = -128;

fn main() {
    // Build representative values for each category.
    // Each entry: (name, frac, exp)
    let categories: Vec<(&str, i8, i8)> = vec![
        // Special states (exp = AMBIG)
        ("zero", 0, AMBIG),
        ("infinity", -128, AMBIG), // NEG_ONE = -128 for i8
        ("exploded+", 64, AMBIG),  // POS_HALF = 0x40, N1 positive
        ("exploded-", -65, AMBIG), // N1 negative (0xBF)
        ("vanished+", 32, 0),      // POS_SMALL = 0x20, N2 positive
        ("vanished-", -33, 0),     // N2 negative (0xDF)
        ("undefined+", 1, 0),      // top3 same (000..001), not N0
        ("undefined-", -2, 0),     // top3 same (111..110), not N0
        // Normal values at various exponents
        ("pos_half", 64, 0),  // +0.5 * 2^0
        ("neg_one", -128, 1), // -1.0 * 2^1 (note: NEG_ONE frac at normal exp)
        ("pos_small", 32, 0), // smallest N1... wait, 32=0x20 is N2
        // Actually for i8: POS_HALF=0x40=64, N1 means bit[7]!=bit[6]
        // 64 = 0b01000000 → bit7=0, bit6=1 → N1 ✓
        // -65 = 0b10111111 → bit7=1, bit6=0 → N1 ✓
        // -128 = 0b10000000 → bit7=1, bit6=0 → N1 ✓ (but this is NEG_ONE)

        // More normal values
        ("norm+_1", 64, 1),       // +0.5 * 2^1
        ("norm+_2", 64, -1),      // +0.5 * 2^-1
        ("norm+_3", 65, 0),       // slightly > +0.5
        ("norm+_4", 96, 5),       // +0.75 * 2^5
        ("norm+_max", 64, 127),   // +0.5 * 2^127
        ("norm+_min", 64, -127),  // +0.5 * 2^-127
        ("norm-_1", -65, 1),      // ≈ -0.5 * 2^1
        ("norm-_2", -65, -1),     // ≈ -0.5 * 2^-1
        ("norm-_3", -128, 0),     // -1.0 * 2^0
        ("norm-_4", -96, 5),      // -0.75 * 2^5
        ("norm-_max", -65, 127),  // negative large
        ("norm-_min", -65, -127), // negative tiny
    ];

    let mut count = 0u64;

    // Two-input ops: ADD(0), SUB(1), MUL(2), AND(3), OR(4), XOR(5)
    for op in 0u8..=5 {
        for (aname, af, ae) in &categories {
            for (bname, bf, be) in &categories {
                let a = S::new(*af, *ae);
                let b = S::new(*bf, *be);

                let r = match op {
                    0 => a + b,
                    1 => a - b,
                    2 => a * b,
                    3 => a & b,
                    4 => a | b,
                    5 => a ^ b,
                    _ => unreachable!(),
                };

                // MSB-align to 64-bit: shift left by 56 bits
                let af_msb = (*af as u8) as u64;
                let ae_msb = (*ae as u8) as u64;
                let bf_msb = (*bf as u8) as u64;
                let be_msb = (*be as u8) as u64;
                let rf_msb = (r.fraction as u8) as u64;
                let re_msb = (r.exponent as u8) as u64;

                println!(
                    "{:02x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {} {} {} {}",
                    op,
                    af_msb << 56,
                    ae_msb << 56,
                    bf_msb << 56,
                    be_msb << 56,
                    rf_msb << 56,
                    re_msb << 56,
                    0,
                    0,
                    0,
                    0,
                );
                count += 1;
            }
        }
    }

    // One-input ops: NOT(6), NEG(10), MAG(11), SIGN(12)
    for op in [6u8, 10, 11, 12] {
        for (aname, af, ae) in &categories {
            let a = S::new(*af, *ae);

            let r = match op {
                6 => !a,
                10 => -a,
                11 => a.magnitude(),
                12 => a.sign(),
                _ => unreachable!(),
            };

            let af_msb = (*af as u8) as u64;
            let ae_msb = (*ae as u8) as u64;
            let rf_msb = (r.fraction as u8) as u64;
            let re_msb = (r.exponent as u8) as u64;

            // b = 0 for one-input ops
            println!(
                "{:02x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {} {} {} {}",
                op,
                af_msb << 56,
                ae_msb << 56,
                0u64,
                0u64,
                rf_msb << 56,
                re_msb << 56,
                0,
                0,
                0,
                0,
            );
            count += 1;
        }
    }

    // SHL(7), SHR(8): a is the value, b_exp is the shift amount
    for op in [7u8, 8] {
        let shift_amounts: Vec<i8> = vec![-127, -64, -1, 0, 1, 5, 64, 127, AMBIG];
        for (aname, af, ae) in &categories {
            for &shift in &shift_amounts {
                let a = S::new(*af, *ae);

                let r = match op {
                    7 => a.scalar_shl_integer(&shift),
                    8 => a.scalar_shr_integer(&shift),
                    _ => unreachable!(),
                };

                let af_msb = (*af as u8) as u64;
                let ae_msb = (*ae as u8) as u64;
                let shift_msb = (shift as u8) as u64;
                let rf_msb = (r.fraction as u8) as u64;
                let re_msb = (r.exponent as u8) as u64;

                // b_frac = 0, b_exp = shift amount (MSB-aligned)
                println!(
                    "{:02x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {} {} {} {}",
                    op,
                    af_msb << 56,
                    ae_msb << 56,
                    0u64,
                    shift_msb << 56,
                    rf_msb << 56,
                    re_msb << 56,
                    0,
                    0,
                    0,
                    0,
                );
                count += 1;
            }
        }
    }

    // CMP(9): need lt/eq/gt/unord flags
    for (aname, af, ae) in &categories {
        for (bname, bf, be) in &categories {
            let a = S::new(*af, *ae);
            let b = S::new(*bf, *be);

            // Use partial_cmp for comparison (handles unordered)
            let (lt, eq, gt, unord) = match a.partial_cmp(&b) {
                Some(Ordering::Less) => (1u8, 0u8, 0u8, 0u8),
                Some(Ordering::Equal) => (0, 1, 0, 0),
                Some(Ordering::Greater) => (0, 0, 1, 0),
                None => (0, 0, 0, 1),
            };

            let af_msb = (*af as u8) as u64;
            let ae_msb = (*ae as u8) as u64;
            let bf_msb = (*bf as u8) as u64;
            let be_msb = (*be as u8) as u64;

            // For CMP, result_frac/exp = 0/AMBIG, flags in extra field
            println!(
                "{:02x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {} {} {} {}",
                9u8,
                af_msb << 56,
                ae_msb << 56,
                bf_msb << 56,
                be_msb << 56,
                0u64,
                (0x80u64) << 56, // result = zero (frac=0, exp=AMBIG)
                lt,
                eq,
                gt,
                unord,
            );
            count += 1;
        }
    }

    eprintln!("Generated {} vectors", count);
}
