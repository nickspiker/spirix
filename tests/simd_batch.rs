//! Every `spirix::simd` batch function must return results bit-identical to the scalar operator applied element-wise.
//!
//! Inputs cover raw random bit patterns (every class: normal, non-canonical normal, Zero, Infinity, exploded, vanished, undefined), an edge cross product of fractions and exponents at the cycle extremes, and near-equal exponent pairs that exercise the aligned subtract and normalize path.
//!
//! ScalarF4E4 is the only type width the batch API supports today.
//!
//! Run under both `cargo test --test simd_batch` and `RUSTFLAGS="-C target-feature=+avx2" cargo test --test simd_batch`; the x86_64 kernels are also called directly whenever the CPU reports AVX2 at runtime.

use spirix::*;

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn raw(fraction: u16, exponent: u16) -> ScalarF4E4 {
    let mut s = ScalarF4E4::ZERO;
    s.fraction = fraction as i16;
    s.exponent = exponent as i16;
    s
}

fn bits(s: &ScalarF4E4) -> (u16, u16) {
    (s.fraction as u16, s.exponent as u16)
}

const EDGE_FRACTIONS: [u16; 18] = [
    0x0000, 0x0001, 0x2000, 0x3FFF, 0x4000, 0x4001, 0x5555, 0x7FFE, 0x7FFF, 0x8000, 0x8001, 0xAAAA,
    0xBFFF, 0xC000, 0xC001, 0xE000, 0xFFFE, 0xFFFF,
];

const EDGE_EXPONENTS: [u16; 14] = [
    0x0000, 0x0001, 0x0002, 0x000F, 0x0010, 0x7FFE, 0x7FFF, 0x8000, 0x8001, 0x800E, 0x800F, 0xFFF1,
    0xFFFE, 0xFFFF,
];

fn edge_values() -> Vec<ScalarF4E4> {
    let mut v = Vec::new();
    for &e in EDGE_EXPONENTS.iter() {
        for &f in EDGE_FRACTIONS.iter() {
            v.push(raw(f, e));
        }
    }
    v
}

/// Builds the operand pairs: full edge cross product, random raw bits, and near-equal exponents.
fn operand_pairs() -> (Vec<ScalarF4E4>, Vec<ScalarF4E4>) {
    let mut a = Vec::new();
    let mut b = Vec::new();
    let edges = edge_values();
    for x in edges.iter() {
        for y in edges.iter() {
            a.push(*x);
            b.push(*y);
        }
    }
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    for _ in 0..200_000 {
        let r = rng.next();
        a.push(raw(r as u16, (r >> 16) as u16));
        b.push(raw((r >> 32) as u16, (r >> 48) as u16));
    }
    // Near-equal exponents, including pairs that straddle the cycle ends so the normalize step can explode or vanish.
    for _ in 0..400_000 {
        let r = rng.next();
        let base = match r & 3 {
            0 => (r >> 2) as u16 & 0x1F,
            1 => 0xFFFFu16.wrapping_sub((r >> 2) as u16 & 0x1F),
            _ => (r >> 2) as u16,
        };
        let delta = ((r >> 20) % 33) as u16;
        let eb = if r & (1 << 30) == 0 {
            base.wrapping_add(delta)
        } else {
            base.wrapping_sub(delta)
        };
        a.push(raw((r >> 32) as u16, base));
        b.push(raw((r >> 48) as u16, eb));
    }
    // Equal magnitudes so the difference is exactly Zero, and neighbours one ULP apart.
    for _ in 0..20_000 {
        let r = rng.next();
        let x = raw(r as u16, (r >> 16) as u16 | 1);
        a.push(x);
        b.push(x);
        a.push(x);
        b.push(raw((r as u16).wrapping_add(1), (r >> 16) as u16 | 1));
    }
    (a, b)
}

fn expected(a: &[ScalarF4E4], b: &[ScalarF4E4]) -> Vec<ScalarF4E4> {
    a.iter().zip(b.iter()).map(|(x, y)| *x - *y).collect()
}

fn check(name: &str, a: &[ScalarF4E4], b: &[ScalarF4E4], got: &[ScalarF4E4]) {
    let want = expected(a, b);
    let mut failures = 0usize;
    for i in 0..a.len() {
        if bits(&got[i]) != bits(&want[i]) {
            if failures < 10 {
                eprintln!(
                    "{name}[{i}]: a={:04X?} b={:04X?} got={:04X?} want={:04X?}",
                    bits(&a[i]),
                    bits(&b[i]),
                    bits(&got[i]),
                    bits(&want[i])
                );
            }
            failures += 1;
        }
    }
    assert_eq!(
        failures,
        0,
        "{name}: {failures} of {} lanes differ from element-wise scalar subtract",
        a.len()
    );
}

#[test]
fn subtract_batch_matches_scalar() {
    let (a, b) = operand_pairs();
    let mut got = vec![raw(0x1234, 0x5678); a.len()];
    spirix::simd::scalar_subtract_batch(&a, &b, &mut got);
    check("scalar_subtract_batch", &a, &b, &got);
}

#[test]
fn subtract_batch_every_length() {
    // Every length up to a few vectors wide so chunk boundaries and remainder tails are all covered.
    let (a, b) = operand_pairs();
    for len in 0..=40 {
        for start in [0usize, 1, 7, 333, 60_000] {
            let a = &a[start..start + len];
            let b = &b[start..start + len];
            let mut got = vec![raw(0x1234, 0x5678); len];
            spirix::simd::scalar_subtract_batch(a, b, &mut got);
            check("scalar_subtract_batch len sweep", a, b, &got);
        }
    }
}

#[test]
#[should_panic]
fn subtract_batch_length_mismatch_panics() {
    let a = vec![ScalarF4E4::ONE; 9];
    let b = vec![ScalarF4E4::ONE; 8];
    let mut r = vec![ScalarF4E4::ZERO; 9];
    spirix::simd::scalar_subtract_batch(&a, &b, &mut r);
}

#[cfg(target_arch = "x86_64")]
#[test]
fn x86_64_kernels_match_scalar() {
    use spirix::simd::x86_64::*;
    let (a, b) = operand_pairs();
    if std::is_x86_feature_detected!("avx2") {
        let mut got = vec![raw(0x1234, 0x5678); a.len()];
        unsafe { scalar_subtract_batch_avx2(&a, &b, &mut got) };
        check("scalar_subtract_batch_avx2", &a, &b, &got);

        let mut got = vec![raw(0x1234, 0x5678); a.len()];
        unsafe { scalar_subtract_batch_avx2_15op_shift(&a, &b, &mut got) };
        check("scalar_subtract_batch_avx2_15op_shift", &a, &b, &got);

        let chunks = a.len() / 8;
        let mut got = vec![raw(0x1234, 0x5678); chunks * 8];
        for i in 0..chunks {
            let ac: &[ScalarF4E4; 8] = a[i * 8..i * 8 + 8].try_into().unwrap();
            let bc: &[ScalarF4E4; 8] = b[i * 8..i * 8 + 8].try_into().unwrap();
            let rc: &mut [ScalarF4E4; 8] = (&mut got[i * 8..i * 8 + 8]).try_into().unwrap();
            unsafe { scalar_subtract_batch_avx2_8op(ac, bc, rc) };
        }
        check(
            "scalar_subtract_batch_avx2_8op",
            &a[..chunks * 8],
            &b[..chunks * 8],
            &got,
        );

        if std::is_x86_feature_detected!("bmi2") {
            let mut got = vec![raw(0x1234, 0x5678); a.len()];
            unsafe { scalar_subtract_batch_avx2_15op_bmi2(&a, &b, &mut got) };
            check("scalar_subtract_batch_avx2_15op_bmi2", &a, &b, &got);
        }
    }
    if std::is_x86_feature_detected!("sse4.2") {
        let mut got = vec![raw(0x1234, 0x5678); a.len()];
        unsafe { scalar_subtract_batch_sse42(&a, &b, &mut got) };
        check("scalar_subtract_batch_sse42", &a, &b, &got);
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn x86_64_runtime_detection_matches_std() {
    assert_eq!(
        spirix::simd::x86_64::has_avx2(),
        std::is_x86_feature_detected!("avx2")
    );
}
