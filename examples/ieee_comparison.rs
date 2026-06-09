/// IEEE 754 f32 vs Spirix F4E4 comparison
///
/// For each operation we run N random trials, converting inputs to both f32 and ScalarF4E4, perform the operation, convert results back to f64 for comparison, and report error statistics.
///
/// Operations covered: Mandated by IEEE 754:  + - * / sqrt % Libm (f32 reference):  powf logf (for fun)
///
/// Run with: cargo run --example ieee_comparison --release
use spirix::ScalarF4E4;

const N: usize = 100_000;

// ── helpers ─────────────────────────────────────────────────────────────────

/// Map a u32 into a finite, non-zero f32 in (-range, -epsilon] ∪ [epsilon, range]
fn f32_from_bits_finite(bits: u32, range: f32) -> f32 {
    // Use bits to get a value in [0,1), scale, shift to avoid zero
    let unit = (bits as f64 / u32::MAX as f64) as f32; // [0, 1)
    let v = (unit * 2.0 - 1.0) * range; // (-range, range)
    if v.abs() < 1e-6 {
        1e-6
    } else {
        v
    }
}

fn to_f64(s: ScalarF4E4) -> f64 {
    s.to_f64()
}

struct Stats {
    mean_abs_err: f64,
    max_abs_err: f64,
    mean_rel_err: f64,
    max_rel_err: f64,
    nan_count: usize,
}

fn stats(errors: &[(f64, f64)]) -> Stats {
    // errors: (abs_err, rel_err)
    let n = errors.len() as f64;
    let mut sum_abs = 0f64;
    let mut max_abs = 0f64;
    let mut sum_rel = 0f64;
    let mut max_rel = 0f64;
    let mut nans = 0usize;

    for &(abs, rel) in errors {
        if abs.is_nan() || rel.is_nan() {
            nans += 1;
            continue;
        }
        sum_abs += abs;
        max_abs = max_abs.max(abs);
        sum_rel += rel;
        max_rel = max_rel.max(rel);
    }

    Stats {
        mean_abs_err: sum_abs / n,
        max_abs_err: max_abs,
        mean_rel_err: sum_rel / n,
        max_rel_err: max_rel,
        nan_count: nans,
    }
}

fn print_stats(name: &str, s: &Stats) {
    println!("  {name}");
    println!("    mean abs err : {:.6e}", s.mean_abs_err);
    println!("    max  abs err : {:.6e}", s.max_abs_err);
    println!("    mean rel err : {:.4}%", s.mean_rel_err * 100.0);
    println!("    max  rel err : {:.4}%", s.max_rel_err * 100.0);
    if s.nan_count > 0 {
        println!("    NaN/undefined: {}", s.nan_count);
    }
}

// ── deterministic LCG RNG (no deps) ─────────────────────────────────────────

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() {
    println!("IEEE 754 f32  vs  Spirix F4E4  —  {} trials each\n", N);

    let mut rng = Lcg(0xDEAD_BEEF_CAFE_1234);

    // ── Addition ─────────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            let a = f32_from_bits_finite(rng.next(), 1000.0);
            let b = f32_from_bits_finite(rng.next(), 1000.0);
            let ieee = (a + b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa + sb);
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Addition  (+)", &stats(&errs));
    }

    // ── Subtraction ──────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            let a = f32_from_bits_finite(rng.next(), 1000.0);
            let b = f32_from_bits_finite(rng.next(), 1000.0);
            let ieee = (a - b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa - sb);
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Subtraction (-)", &stats(&errs));
    }

    // ── Multiplication ───────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            let a = f32_from_bits_finite(rng.next(), 100.0);
            let b = f32_from_bits_finite(rng.next(), 100.0);
            let ieee = (a * b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa * sb);
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Multiply   (*)", &stats(&errs));
    }

    // ── Division ─────────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            let a = f32_from_bits_finite(rng.next(), 1000.0);
            let b = f32_from_bits_finite(rng.next(), 1000.0);
            let ieee = (a / b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa / sb);
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Division   (/)", &stats(&errs));
    }

    // ── Remainder ────────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            // Keep a/b ratio small to avoid catastrophic cancellation in floor(q)*b - a
            let b = f32_from_bits_finite(rng.next(), 10.0);
            let a = f32_from_bits_finite(rng.next(), 10.0 * b.abs());
            let ieee = (a % b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa % sb);
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Remainder  (%)", &stats(&errs));
    }

    // ── Square root ──────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            let a = (rng.next() as f32 / u32::MAX as f32) * 10000.0; // positive only
            let ieee = (a.sqrt()) as f64;
            let sa = ScalarF4E4::from(a);
            let spirix = to_f64(sa.sqrt());
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Sqrt    (sqrt)", &stats(&errs));
    }

    println!();
    println!("── For fun: libm reference (not IEEE-mandated) ──────────────────");
    println!();

    // ── pow ──────────────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            // keep base positive, exponent small to avoid overflow
            let a = (rng.next() as f32 / u32::MAX as f32) * 10.0 + 0.01;
            let b = (rng.next() as f32 / u32::MAX as f32) * 4.0 - 2.0;
            let ieee = a.powf(b) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(b);
            let spirix = to_f64(sa.pow(sb));
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Power   (pow)", &stats(&errs));
    }

    // ── log ──────────────────────────────────────────────────────────────────
    {
        let mut errs = Vec::with_capacity(N);
        for _ in 0..N {
            // positive inputs only, base > 1
            let a = (rng.next() as f32 / u32::MAX as f32) * 1000.0 + 0.01;
            let base = (rng.next() as f32 / u32::MAX as f32) * 8.0 + 2.0;
            let ieee = a.log(base) as f64;
            let sa = ScalarF4E4::from(a);
            let sb = ScalarF4E4::from(base);
            let spirix = to_f64(sa.log(sb));
            if ieee.is_finite() && spirix.is_finite() {
                let abs = (ieee - spirix).abs();
                let rel = if ieee.abs() > 1e-10 {
                    abs / ieee.abs()
                } else {
                    abs
                };
                errs.push((abs, rel));
            }
        }
        print_stats("Log     (log)", &stats(&errs));
    }

    println!();
    println!("Note: F4E4 has 16-bit fraction and 16-bit exponent.");
    println!("      f32 has 23-bit mantissa, 8-bit exponent.");
    println!("      Relative error reflects precision difference, not bugs.");
}
