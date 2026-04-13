/// Generate test vectors for spirix_cmp, spirix_neg, spirix_floor, spirix_abs.
///
/// Covers all 16 frac×exp width combos (F3-F6 × E3-E6).
/// Output: hex file with lines:
///   op fw ew a_frac a_exp b_frac b_exp r_frac r_exp
///
/// Op: 05=CMP 06=NEG 07=FLOOR 08=ABS
/// CMP encodes flags in r_frac: bit0=lt, bit1=eq, bit2=gt, bit3=unord
/// Unary ops (NEG/FLOOR/ABS): b_frac=0 b_exp=0
///
/// Fractions MSB-aligned to 64 bits, exponents LSB-aligned with universal AMBIG.
use spirix::Scalar;
use std::cmp::Ordering;

const OP_CMP: u8 = 5;
const OP_NEG: u8 = 6;
const OP_FLOOR: u8 = 7;
const OP_ABS: u8 = 8;

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

/// Trait to abstract over width-specific operations
trait TestWidth: Copy {
    fn msb_frac_64(self) -> u64;
    fn lsb_exp_64(self) -> u64;
    fn frac_bits() -> u32;
    fn exp_bits() -> u32;
    fn fw() -> u8;
    fn ew() -> u8;
}

macro_rules! impl_test_width {
    ($f:ty, $e:ty, $fw:expr, $ew:expr, $fshift:expr) => {
        impl TestWidth for Scalar<$f, $e> {
            fn msb_frac_64(self) -> u64 {
                let raw = self.fraction as u64;
                let mask = if <$f>::BITS == 64 {
                    u64::MAX
                } else {
                    (1u64 << <$f>::BITS) - 1
                };
                (raw & mask) << $fshift
            }
            fn lsb_exp_64(self) -> u64 {
                if self.exponent == <$e>::MIN {
                    0x8000_0000_0000_0000u64 // universal AMBIG
                } else {
                    self.exponent as i64 as u64 // sign-extend
                }
            }
            fn frac_bits() -> u32 {
                <$f>::BITS
            }
            fn exp_bits() -> u32 {
                <$e>::BITS
            }
            fn fw() -> u8 {
                $fw
            }
            fn ew() -> u8 {
                $ew
            }
        }
    };
}

// F3=i8(shift 56), F4=i16(shift 48), F5=i32(shift 32), F6=i64(shift 0)
impl_test_width!(i8, i8, 0, 0, 56);
impl_test_width!(i8, i16, 0, 1, 56);
impl_test_width!(i8, i32, 0, 2, 56);
impl_test_width!(i8, i64, 0, 3, 56);
impl_test_width!(i16, i8, 1, 0, 48);
impl_test_width!(i16, i16, 1, 1, 48);
impl_test_width!(i16, i32, 1, 2, 48);
impl_test_width!(i16, i64, 1, 3, 48);
impl_test_width!(i32, i8, 2, 0, 32);
impl_test_width!(i32, i16, 2, 1, 32);
impl_test_width!(i32, i32, 2, 2, 32);
impl_test_width!(i32, i64, 2, 3, 32);
impl_test_width!(i64, i8, 3, 0, 0);
impl_test_width!(i64, i16, 3, 1, 0);
impl_test_width!(i64, i32, 3, 2, 0);
impl_test_width!(i64, i64, 3, 3, 0);

fn emit_cmp<S: TestWidth>(a: S, b: S, ord: Option<Ordering>) {
    let flags: u64 = match ord {
        Some(Ordering::Less) => 1,
        Some(Ordering::Equal) => 2,
        Some(Ordering::Greater) => 4,
        None => 8,
    };
    println!(
        "{:02x} {} {} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
        OP_CMP,
        S::fw(),
        S::ew(),
        a.msb_frac_64(),
        a.lsb_exp_64(),
        b.msb_frac_64(),
        b.lsb_exp_64(),
        flags << 56, // flags in MSB position for consistent format
        0u64,
    );
}

fn emit_unary<S: TestWidth>(op: u8, a: S, r: S) {
    println!(
        "{:02x} {} {} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
        op,
        S::fw(),
        S::ew(),
        a.msb_frac_64(),
        a.lsb_exp_64(),
        0u64,
        0u64,
        r.msb_frac_64(),
        r.lsb_exp_64(),
    );
}

macro_rules! gen_for_width {
    ($f:ty, $e:ty, $rng:expr, $count:expr) => {{
        type S = Scalar<$f, $e>;
        let ambig: $e = <$e>::MIN;

        // Edge values for this width
        let edge_fracs: Vec<$f> = vec![
            0,                      // zero frac
            <$f>::MIN,              // neg_one / infinity frac
            (<$f>::MIN >> 1),       // pos_half (N1 positive)
            ((<$f>::MIN >> 1) + 1).wrapping_neg(), // N1 negative (~-65 for i8)
            (<$f>::MIN >> 2),       // N2 positive (vanished)
            (<$f>::MIN >> 2).wrapping_neg().wrapping_neg().wrapping_add(<$f>::MIN >> 2).wrapping_neg(), // N2 negative
            (<$f>::MIN >> 3),       // N3 positive (undefined)
            (<$f>::MIN >> 3).wrapping_neg(), // N3 negative (undefined)
        ];
        // Simpler: use known patterns
        let edge_values: Vec<($f, $e)> = vec![
            (0, ambig),             // zero
            (<$f>::MIN, ambig),     // infinity (NEG_ONE frac)
            // Exploded positive (N1 with AMBIG)
            (((1 as $f) << (<$f>::BITS - 2)), ambig),
            // Exploded negative (N1 with AMBIG)
            (((<$f>::MIN) >> 1).wrapping_add((<$f>::MIN) >> 2), ambig),
            // Vanished positive
            (((1 as $f) << (<$f>::BITS - 3)), ambig),
            // Vanished negative
            ((-1 as $f) << (<$f>::BITS - 3), ambig),
            // Undefined positive
            (((1 as $f) << (<$f>::BITS - 4)), ambig),
            // Undefined negative
            ((-1 as $f) << (<$f>::BITS - 4), ambig),
        ];

        // --- Edge × Edge: CMP, NEG, FLOOR, ABS ---
        for &(af, ae) in &edge_values {
            let a = S::new(af, ae);

            // Unary ops
            emit_unary(OP_NEG, a, -a);
            emit_unary(OP_FLOOR, a, a.floor());
            emit_unary(OP_ABS, a, a.magnitude());
            *$count += 3;

            // CMP against all edge values
            for &(bf, be) in &edge_values {
                let b = S::new(bf, be);
                let ord = a.partial_cmp(&b);
                emit_cmp(a, b, ord);
                *$count += 1;
            }
        }

        // --- Normal values: unary ops + CMP ---
        let normal_exps: Vec<$e> = {
            let max_e = <$e>::MAX;
            let min_e = <$e>::MIN.wrapping_add(1); // avoid AMBIG
            vec![min_e, -1 as $e, 0 as $e, 1 as $e, max_e,
                 (max_e >> 1), (min_e >> 1)]
        };

        for &ae in &normal_exps {
            // N1 fracs: several interesting values
            let test_fracs: Vec<$f> = vec![
                ((1 as $f) << (<$f>::BITS - 2)),         // POS_HALF
                <$f>::MIN,                                // NEG_ONE
                ((1 as $f) << (<$f>::BITS - 2)) | 1,     // POS_HALF + 1
                <$f>::MIN | ((1 as $f) << (<$f>::BITS - 2)), // -POS_HALF
                <$f>::MAX,                                // max positive
            ];
            for &af in &test_fracs {
                let a = S::new(af, ae);
                emit_unary(OP_NEG, a, -a);
                emit_unary(OP_FLOOR, a, a.floor());
                emit_unary(OP_ABS, a, a.magnitude());
                *$count += 3;
            }
        }

        // --- Random normal pairs: CMP ---
        for _ in 0..200 {
            let af = loop {
                let f = $rng.next() as $f;
                if ((f >> (<$f>::BITS - 1)) & 1) != ((f >> (<$f>::BITS - 2)) & 1) { break f; }
            };
            let bf = loop {
                let f = $rng.next() as $f;
                if ((f >> (<$f>::BITS - 1)) & 1) != ((f >> (<$f>::BITS - 2)) & 1) { break f; }
            };
            let ae = loop {
                let e = $rng.next() as $e;
                if e != ambig { break e; }
            };
            // Mix of exp proximity modes
            let be = match $rng.next() % 4 {
                0 => ae,
                1 => { let d = ($rng.next() % 7) as $e - 3; let e = ae.wrapping_add(d); if e == ambig { ae } else { e } }
                2 => { let e = loop { let e = $rng.next() as $e; if e != ambig { break e; } }; e }
                _ => ae,
            };

            let a = S::new(af, ae);
            let b = S::new(bf, be);
            let ord = a.partial_cmp(&b);
            emit_cmp(a, b, ord);
            *$count += 1;

            // Also test unary on a
            emit_unary(OP_NEG, a, -a);
            emit_unary(OP_FLOOR, a, a.floor());
            emit_unary(OP_ABS, a, a.magnitude());
            *$count += 3;
        }
    }};
}

fn main() {
    let mut rng = Rng(0xCAFEBABE12345678);
    let mut count = 0u64;

    // Generate for all 16 width combos
    gen_for_width!(i8, i8, &mut rng, &mut count);
    gen_for_width!(i16, i8, &mut rng, &mut count);
    gen_for_width!(i32, i8, &mut rng, &mut count);
    gen_for_width!(i64, i8, &mut rng, &mut count);
    gen_for_width!(i8, i16, &mut rng, &mut count);
    gen_for_width!(i16, i16, &mut rng, &mut count);
    gen_for_width!(i32, i16, &mut rng, &mut count);
    gen_for_width!(i64, i16, &mut rng, &mut count);
    gen_for_width!(i8, i32, &mut rng, &mut count);
    gen_for_width!(i16, i32, &mut rng, &mut count);
    gen_for_width!(i32, i32, &mut rng, &mut count);
    gen_for_width!(i64, i32, &mut rng, &mut count);
    gen_for_width!(i8, i64, &mut rng, &mut count);
    gen_for_width!(i16, i64, &mut rng, &mut count);
    gen_for_width!(i32, i64, &mut rng, &mut count);
    gen_for_width!(i64, i64, &mut rng, &mut count);

    eprintln!("Generated {} vectors across 16 width combos", count);
}
