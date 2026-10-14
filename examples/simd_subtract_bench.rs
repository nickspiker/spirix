//! Times `simd::scalar_subtract_batch` and the AVX2 kernel against element-wise ScalarF4E4 subtract, and checks they agree bit for bit.
//!
//! Run with `cargo run --release --example simd_subtract_bench`, optionally under `RUSTFLAGS="-C target-feature=+avx2"` so the public dispatcher picks the AVX2 path.

use spirix::*;
use std::hint::black_box;
use std::time::Instant;

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

fn time(label: &str, reps: usize, n: usize, mut f: impl FnMut()) -> f64 {
    f();
    let start = Instant::now();
    for _ in 0..reps {
        f();
    }
    let ns = start.elapsed().as_nanos() as f64 / (reps * n) as f64;
    println!("  {label:<34} {ns:6.2} ns/elem");
    ns
}

fn run(name: &str, a: &[ScalarF4E4], b: &[ScalarF4E4]) {
    let n = a.len();
    let reps = 20;
    let want: Vec<ScalarF4E4> = a.iter().zip(b).map(|(x, y)| *x - *y).collect();
    let mut out = vec![ScalarF4E4::ZERO; n];
    println!("{name} ({n} elements)");
    let scalar = time("element-wise a - b", reps, n, || {
        for i in 0..n {
            out[i] = black_box(a[i]) - black_box(b[i]);
        }
        black_box(&out);
    });
    let batch = time("simd::scalar_subtract_batch", reps, n, || {
        spirix::simd::scalar_subtract_batch(black_box(a), black_box(b), &mut out);
        black_box(&out);
    });
    for i in 0..n {
        assert_eq!(
            (out[i].fraction, out[i].exponent),
            (want[i].fraction, want[i].exponent),
            "batch lane {i}"
        );
    }
    println!("  speedup {:.2}x", scalar / batch);
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        let avx2 = time("x86_64::scalar_subtract_batch_avx2", reps, n, || {
            unsafe {
                spirix::simd::x86_64::scalar_subtract_batch_avx2(
                    black_box(a),
                    black_box(b),
                    &mut out,
                )
            };
            black_box(&out);
        });
        for i in 0..n {
            assert_eq!(
                (out[i].fraction, out[i].exponent),
                (want[i].fraction, want[i].exponent),
                "avx2 lane {i}"
            );
        }
        println!("  speedup {:.2}x", scalar / avx2);
    }
}

fn main() {
    let n = 1 << 20;
    let mut rng = XorShift(0x2545_F491_4F6C_DD1D);

    // Ordinary data: values of similar magnitude, as produced by real workloads.
    let mut a = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    for _ in 0..n {
        let r = rng.next();
        let x = ((r & 0xFFFF_FFFF) as f64 / 4294967296.0 - 0.5) * 2000.0;
        let y = ((r >> 32) as f64 / 4294967296.0 - 0.5) * 2000.0;
        a.push(ScalarF4E4::from(x));
        b.push(ScalarF4E4::from(y));
    }
    run("similar magnitudes", &a, &b);

    // Raw random bits: mostly widely separated exponents, plus about 3% non-normal lanes per operand that take the scalar fallback.
    let mut a = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    for _ in 0..n {
        let r = rng.next();
        let ea = if (r >> 58) == 0 { 0 } else { (r >> 16) as u16 };
        let eb = if (r >> 58) == 1 { 0 } else { (r >> 48) as u16 };
        a.push(raw(r as u16, ea));
        b.push(raw((r >> 32) as u16, eb));
    }
    run("raw random bits", &a, &b);

    // Soak: many more near-equal exponent pairs at the cycle ends, checked bit for bit.
    let mut a = Vec::with_capacity(n);
    let mut b = Vec::with_capacity(n);
    for round in 0..64 {
        a.clear();
        b.clear();
        for _ in 0..n {
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
        let mut out = vec![ScalarF4E4::ZERO; n];
        spirix::simd::scalar_subtract_batch(&a, &b, &mut out);
        #[cfg(target_arch = "x86_64")]
        let mut out2 = vec![ScalarF4E4::ZERO; n];
        #[cfg(target_arch = "x86_64")]
        if std::is_x86_feature_detected!("avx2") {
            unsafe { spirix::simd::x86_64::scalar_subtract_batch_avx2(&a, &b, &mut out2) };
        } else {
            out2.copy_from_slice(&out);
        }
        for i in 0..n {
            let want = a[i] - b[i];
            assert_eq!(
                (out[i].fraction, out[i].exponent),
                (want.fraction, want.exponent),
                "soak round {round} lane {i}"
            );
            #[cfg(target_arch = "x86_64")]
            assert_eq!(
                (out2[i].fraction, out2[i].exponent),
                (want.fraction, want.exponent),
                "soak avx2 round {round} lane {i}"
            );
        }
    }
    println!("soak: {} near-equal-exponent pairs bit-identical", 64 * n);
}
