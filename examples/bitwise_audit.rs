// Audit AND / OR / XOR on ScalarF4E3 vs native i16 over a wide pair sweep. Each scalar op should agree with the native int op when both operands are integer-valued and the result is representable. Mismatches expose renormalization bugs in `bitwise_normal`.

use spirix::ScalarF4E3;
use std::collections::BTreeMap;

fn run(label: &str, pairs: &[(i16, i16)]) {
    let mut and_miss = 0u64;
    let mut or_miss = 0u64;
    let mut xor_miss = 0u64;
    let mut and_examples = BTreeMap::<(i16, i16, i16, i16), u64>::new();
    let mut or_examples = BTreeMap::<(i16, i16, i16, i16), u64>::new();
    let mut xor_examples = BTreeMap::<(i16, i16, i16, i16), u64>::new();

    for &(l, r) in pairs {
        let ls = ScalarF4E3::from(l as f32);
        let rs = ScalarF4E3::from(r as f32);

        let to_i16 = |x: ScalarF4E3| -> i16 {
            let f: f32 = (&x).into();
            f.round().clamp(i16::MIN as f32, i16::MAX as f32) as i16
        };

        let s_and = to_i16(ls & rs);
        let s_or = to_i16(ls | rs);
        let s_xor = to_i16(ls ^ rs);
        let n_and = l & r;
        let n_or = l | r;
        let n_xor = l ^ r;

        if s_and != n_and {
            and_miss += 1;
            if and_examples.len() < 6 {
                *and_examples.entry((l, r, n_and, s_and)).or_insert(0) += 1;
            }
        }
        if s_or != n_or {
            or_miss += 1;
            if or_examples.len() < 6 {
                *or_examples.entry((l, r, n_or, s_or)).or_insert(0) += 1;
            }
        }
        if s_xor != n_xor {
            xor_miss += 1;
            if xor_examples.len() < 6 {
                *xor_examples.entry((l, r, n_xor, s_xor)).or_insert(0) += 1;
            }
        }
    }

    let n = pairs.len() as u64;
    println!(
        "{:<14}  AND: {:>6} mismatch / {}   OR: {:>6}   XOR: {:>6}",
        label, and_miss, n, or_miss, xor_miss
    );
    let dump = |op: &str, m: &BTreeMap<(i16, i16, i16, i16), u64>| {
        if !m.is_empty() {
            println!("  {} examples:", op);
            for &(l, r, native, spirix) in m.keys() {
                println!(
                    "    {:>6} {} {:>6}: native={:>6}  spirix={:>6}",
                    l, op, r, native, spirix
                );
            }
        }
    };
    dump("AND", &and_examples);
    dump(" OR", &or_examples);
    dump("XOR", &xor_examples);
}

fn main() {
    // 1) Tiny exhaustive sweep
    let mut pairs = Vec::new();
    for l in -64i16..=64 {
        for r in -64i16..=64 {
            pairs.push((l, r));
        }
    }
    run("[-64..64]^2", &pairs);

    // 2) Power-of-two pairs and near-neighbors
    let mut p2 = Vec::new();
    for k in 0..15 {
        let v = 1i16 << k;
        for d in -3..=3i16 {
            for s1 in [1i16, -1] {
                for s2 in [1i16, -1] {
                    p2.push((s1 * v, s2 * (v + d)));
                }
            }
        }
    }
    run("pow2 +/- 3", &p2);

    // 3) Random pairs
    use rand::{rngs::StdRng, Rng, SeedableRng};
    let mut rng = StdRng::seed_from_u64(42);
    let mut rand_pairs = Vec::with_capacity(200_000);
    for _ in 0..200_000 {
        rand_pairs.push((
            rng.random_range(i16::MIN..=i16::MAX),
            rng.random_range(i16::MIN..=i16::MAX),
        ));
    }
    run("random 200k", &rand_pairs);
}
