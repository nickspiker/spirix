//! Representable-value distribution at F5E3.
//!
//! At each normal exponent e (range [-127, 127], 255 exponents total): positives cover magnitudes [2^(e-1), 2^e)   — 2^31 patterns negatives cover magnitudes (2^(e-1), 2^e]   — 2^31 patterns (neg_one_normal @ e hits the top exactly)
//!
//! So exp=0 covers magnitudes [0.5, 1.0] ENTIRELY (negatives include -1, positives stop just below +1). exp=1 covers [1.0, 2.0] ENTIRELY (positives include +1). Exponents 0 and below are ALL |v| ≤ 1; exponents 1 and above are ALL |v| ≥ 1.
//!
//! With exp in [-127, 127]: "≤ 1" exponents: e in [-127, 0] → 128 exponents "≥ 1" exponents: e in [1, 127]  → 127 exponents
//!
//! That's the asymmetry: one more exponent-worth of values (2^32 patterns) live at
//! |v| ≤ 1 than at |v| ≥ 1.

fn main() {
    const FRAC_BITS: u32 = 32;
    const EXP_MIN: i32 = -127;
    const EXP_MAX: i32 = 127;

    let per_exp: u64 = 1u64 << FRAC_BITS; // 2^32 patterns per exponent
    let half_per_exp: u64 = per_exp >> 1; // 2^31 positives, 2^31 negatives

    // Below 1 strictly (|v| < 1): exp=-127..-1 (127 exps): all 2^32 patterns each exp=0 : all 2^32 except neg_one (which is = 1)
    let below_1: u64 = 127 * per_exp + (per_exp - 1);

    // Equal to 1: neg_one_normal @ exp=0 = -1 pos_one_normal @ exp=1 = +1
    let eq_1: u64 = 2;

    // Above 1 strictly (|v| > 1): exp=1 : all 2^32 except pos_one (which is = 1) exp=2..127 (126 exps)
    let above_1: u64 = (per_exp - 1) + 126 * per_exp;

    let total: u64 = (EXP_MAX - EXP_MIN + 1) as u64 * per_exp;

    println!("=== F5E3 representable normals ({total} total) ===");
    println!();
    println!(
        "  |v| < 1:  {below_1:>14} ({:.4}%)",
        100.0 * below_1 as f64 / total as f64
    );
    println!(
        "  |v| = 1:  {eq_1:>14} ({:.8}%) — just -1 and +1",
        100.0 * eq_1 as f64 / total as f64
    );
    println!(
        "  |v| > 1:  {above_1:>14} ({:.4}%)",
        100.0 * above_1 as f64 / total as f64
    );
    println!("  verify:   {}", below_1 + eq_1 + above_1);
    println!();
    println!(
        "Difference: {} patterns more below 1 than above 1",
        (below_1 as i128) - (above_1 as i128)
    );
    println!("            = 2^32 (one whole exponent bucket).");
    println!();
    println!("Per-exponent breakdown (magnitude range):");
    println!(
        "  exp={:>4}: magnitudes [2^{:>4}, 2^{:>4}) pos, (2^{:>4}, 2^{:>4}] neg",
        EXP_MIN,
        EXP_MIN - 1,
        EXP_MIN,
        EXP_MIN - 1,
        EXP_MIN
    );
    println!("   ...");
    println!("  exp=   0: magnitudes [2^-1, 2^0) pos = [0.5, 1)");
    println!("            magnitudes (2^-1, 2^0] neg = (0.5, 1]  ← includes -1");
    println!("  exp=   1: magnitudes [2^0,  2^1) pos = [1, 2)    ← includes +1");
    println!("            magnitudes (2^0,  2^1] neg = (1, 2]");
    println!("   ...");
    println!();
    println!("Exponent partition around 1:");
    println!(
        "  exps with |v|≤1:  e in [-127, 0]  → 128 exponents × 2^32 = {}",
        128_u64 * per_exp
    );
    println!(
        "  exps with |v|≥1:  e in [1, 127]   → 127 exponents × 2^32 = {}",
        127_u64 * per_exp
    );

    let _ = half_per_exp;
}
