//! Audit the `exp_diff >= FRAC` truncation boundary in scalar_modulus_scalar.
//! The code currently returns ZERO when a.exp - b.exp >= FRAC because the shift
//! wouldn't fit in 2*FRAC-bit wide. This audits how often that short-circuit
//! produces 0 when the true floor-mod is non-zero.
use spirix::*;

type S = ScalarF3E3;

fn main() {
    let mut ec_ok = 0usize;         // spirix=0 and f64≈0
    let mut ec_lossy = 0usize;      // spirix=0 but f64 says nonzero (precision loss)
    let mut ec_nonzero = 0usize;    // spirix returned nonzero
    let mut ec_total = 0usize;
    let mut by_diff: std::collections::BTreeMap<i32, usize> = std::collections::BTreeMap::new();
    let mut samples: Vec<(S, S, f64)> = Vec::new();

    for f1 in -128i8..=127 {
        for e1 in -128i8..=127 {
            let a = unsafe { std::mem::transmute::<[i8; 2], S>([f1, e1]) };
            if !a.is_normal() { continue; }
            for f2 in -128i8..=127 {
                for e2 in -128i8..=127 {
                    let b = unsafe { std::mem::transmute::<[i8; 2], S>([f2, e2]) };
                    if !b.is_normal() { continue; }

                    let exp_diff = (e1 as i32) - (e2 as i32);
                    if exp_diff < 8 { continue; } // only the truncation regime

                    let a_f64: f64 = a.into();
                    let b_f64: f64 = b.into();
                    // f64-reliable window
                    if b_f64 == 0.0 { continue; }
                    let ratio = (a_f64 / b_f64).abs();
                    if !ratio.is_finite() || ratio >= (1u64 << 52) as f64 { continue; }

                    let true_mod = a_f64 - (a_f64 / b_f64).floor() * b_f64;
                    let spirix_mod: S = a % b;
                    let spirix_f64: f64 = spirix_mod.into();

                    ec_total += 1;
                    *by_diff.entry(exp_diff).or_insert(0) += 1;

                    if spirix_mod.is_zero() {
                        // Is the true answer also ~0 (up to f64 noise)?
                        if true_mod.abs() < b_f64.abs() * 1e-14 {
                            ec_ok += 1;
                        } else {
                            ec_lossy += 1;
                            if samples.len() < 5 {
                                samples.push((a, b, true_mod));
                            }
                        }
                    } else {
                        ec_nonzero += 1;
                        // For large exp_diff the current code returns ZERO,
                        // so reaching here means we're already past the fix.
                        let diff = (spirix_f64 - true_mod).abs();
                        let tol = b_f64.abs() * 1e-10;
                        if diff > tol {
                            if samples.len() < 5 {
                                samples.push((a, b, true_mod));
                            }
                        }
                    }
                }
            }
        }
    }

    println!("=== Mod truncation audit (exp_diff >= FRAC=8, f64-reliable) ===");
    println!("Total pairs sampled:     {}", ec_total);
    println!("  spirix=0 and true≈0:   {} (correct short-circuit)", ec_ok);
    println!("  spirix=0 but true≠0:   {} (LOSSY — true info discarded)", ec_lossy);
    println!("  spirix returned ≠0:    {} (reached computation path)", ec_nonzero);
    println!();
    println!("Lossy fraction of audited: {:.2}%",
             100.0 * ec_lossy as f64 / ec_total.max(1) as f64);
    println!();
    println!("Sample cases (a, b, true floor_mod):");
    for (a, b, tm) in &samples {
        let af: f64 = (*a).into();
        let bf: f64 = (*b).into();
        let sm: f64 = (*a % *b).into();
        println!("  a={:e} b={:e} -> spirix={:e} true={:e}", af, bf, sm, tm);
    }
    println!();
    let mut diffs: Vec<_> = by_diff.iter().collect();
    diffs.sort_by_key(|&(d, _)| *d);
    print!("pairs per exp_diff:");
    for (d, n) in &diffs { print!(" {}→{}", d, n); if **d > 16 { print!("...");break; } }
    println!();
}
