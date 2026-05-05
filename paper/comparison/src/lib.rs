//! Shared utilities for IEEE 754 binary32 comparison harnesses.
//!
//! These utilities are common to all per-operation binaries in `src/bin/`. The bit-accurate Spirix algorithms themselves live in each binary, intentionally not factored out, so a reader can see the entire implementation in one file without indirection.

/// Deterministic LCG random number generator. Reproduces the same sequence across runs given the same seed, so reported numbers are auditable.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    /// Generate a random normal f32 — no subnormals, no inf, no nan, no zero. Restricts biased exponent to [2, 253] to keep generated values comfortably inside the Spirix exponent range with margin for arithmetic to overflow into exploded or underflow into vanished without immediately leaving i8 representable space.
    pub fn next_normal_f32(&mut self) -> f32 {
        loop {
            let bits = self.next_u32();
            let biased_exp = (bits >> 23) & 0xFF;
            if (2..=253).contains(&biased_exp) {
                return f32::from_bits(bits);
            }
        }
    }
}

/// Counters tabulating per-category comparison results. See README.md for category definitions.
#[derive(Default, Debug)]
pub struct Counters {
    pub exact: u64,
    pub one_ulp: u64,
    pub spirix_vanished_ieee_zero: u64,
    pub spirix_vanished_ieee_denormal: u64,
    pub spirix_exploded_ieee_inf: u64,
    pub spirix_exploded_ieee_finite: u64,
    pub off_by_more: u64,
    pub max_ulp_diff: u64,
    pub worst_a: f32,
    pub worst_b: f32,
}

impl Counters {
    pub fn record_exact(&mut self) {
        self.exact += 1;
    }

    pub fn record_one_ulp(&mut self) {
        self.one_ulp += 1;
    }

    pub fn record_spirix_vanished_ieee_zero(&mut self) {
        self.spirix_vanished_ieee_zero += 1;
    }

    pub fn record_spirix_vanished_ieee_denormal(&mut self) {
        self.spirix_vanished_ieee_denormal += 1;
    }

    pub fn record_spirix_exploded_ieee_inf(&mut self) {
        self.spirix_exploded_ieee_inf += 1;
    }

    pub fn record_spirix_exploded_ieee_finite(&mut self) {
        self.spirix_exploded_ieee_finite += 1;
    }

    pub fn record_off_by_more(&mut self, ulp_diff: u64, a: f32, b: f32) {
        self.off_by_more += 1;
        if ulp_diff > self.max_ulp_diff {
            self.max_ulp_diff = ulp_diff;
            self.worst_a = a;
            self.worst_b = b;
        }
    }

    pub fn print_summary(&self, n: u64) {
        let pct = |c: u64| c as f64 / n as f64 * 100.0;
        println!("Results ({} trials):", n);
        println!(
            "  Exact match:                       {:>10}  ({:.4}%)",
            self.exact,
            pct(self.exact)
        );
        println!(
            "  1 ULP (valid rounding choice):     {:>10}  ({:.4}%)",
            self.one_ulp,
            pct(self.one_ulp)
        );
        println!(
            "  Spirix vanished, IEEE -> 0:        {:>10}  ({:.4}%)",
            self.spirix_vanished_ieee_zero,
            pct(self.spirix_vanished_ieee_zero)
        );
        println!(
            "  Spirix vanished, IEEE -> denormal: {:>10}  ({:.4}%)",
            self.spirix_vanished_ieee_denormal,
            pct(self.spirix_vanished_ieee_denormal)
        );
        println!(
            "  Spirix exploded, IEEE -> +/-inf:   {:>10}  ({:.4}%)",
            self.spirix_exploded_ieee_inf,
            pct(self.spirix_exploded_ieee_inf)
        );
        println!(
            "  Spirix exploded, IEEE -> finite:   {:>10}  ({:.4}%)",
            self.spirix_exploded_ieee_finite,
            pct(self.spirix_exploded_ieee_finite)
        );
        println!(
            "  Off by more (unexpected):          {:>10}  ({:.4}%)",
            self.off_by_more,
            pct(self.off_by_more)
        );
        if self.off_by_more > 0 {
            println!();
            println!("  Max ULP error: {}", self.max_ulp_diff);
            println!("  Worst-case operand pair: a = {:e}, b = {:e}", self.worst_a, self.worst_b);
        }
    }
}

/// Compute ULP difference between two f32 values of the same sign. Returns u64::MAX for sign mismatches or NaN inputs (treated as max-error).
pub fn ulp_diff_same_sign(a: f32, b: f32) -> u64 {
    if a.is_nan() || b.is_nan() {
        return u64::MAX;
    }
    let abits = a.to_bits();
    let bbits = b.to_bits();
    if (abits ^ bbits) >> 31 != 0 {
        return u64::MAX;
    }
    if abits > bbits {
        (abits - bbits) as u64
    } else {
        (bbits - abits) as u64
    }
}
