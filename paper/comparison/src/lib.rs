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
///
/// Categories are organized by Spirix output state, then sub-divided by what IEEE produced. The "architectural" categories are honest-by-design mismatches between IEEE 754 and Spirix v0.1 semantics; "off_by_more" should always be zero for a correct implementation.
#[derive(Default, Debug)]
pub struct Counters {
    // ── Spirix Normal output ──────────────────────────────────────────────
    pub exact: u64,
    pub one_ulp: u64,
    /// Spirix Normal output with > 1 ULP diff from IEEE, where at least one input was non-normal Spirix (e.g., a vanished input was dropped per truth table while IEEE kept its denormal contribution), or where IEEE produced a denormal/zero result. Architectural — not a bug.
    pub spirix_normal_arch_drift: u64,
    /// Spirix Normal output with > 1 ULP diff from IEEE, with both inputs Normal Spirix — should be 0 for a correct algorithm.
    pub off_by_more: u64,
    pub max_ulp_diff: u64,
    pub worst_a: f32,
    pub worst_b: f32,

    // ── Spirix Zero output ────────────────────────────────────────────────
    /// Both produced zero (Spirix singleton ↔ IEEE ±0). Match.
    pub spirix_zero_ieee_zero: u64,
    /// Spirix produced zero, IEEE produced something else — unexpected.
    pub spirix_zero_ieee_other: u64,

    // ── Spirix Vanished output ────────────────────────────────────────────
    /// Spirix vanished, IEEE flushed to ±0 (architectural — Spirix preserves direction).
    pub spirix_vanished_ieee_zero: u64,
    /// Spirix vanished, IEEE produced a denormal (architectural — IEEE has reduced precision).
    pub spirix_vanished_ieee_denormal: u64,
    /// Spirix vanished, IEEE produced something else — unexpected.
    pub spirix_vanished_ieee_other: u64,

    // ── Spirix Exploded output ────────────────────────────────────────────
    /// Spirix exploded, IEEE produced ±∞ (architectural — Spirix preserves direction beyond magnitude limit).
    pub spirix_exploded_ieee_inf: u64,
    /// Spirix exploded, IEEE produced a finite normal (architectural — Spirix's narrower max-positive-exp range).
    pub spirix_exploded_ieee_finite: u64,
    /// Spirix exploded, IEEE produced something else — unexpected.
    pub spirix_exploded_ieee_other: u64,

    // ── Spirix Undefined output ───────────────────────────────────────────
    /// Spirix undefined, IEEE produced NaN. Architectural match — both signal "undefined".
    pub spirix_undefined_ieee_nan: u64,
    /// Spirix undefined, IEEE produced ±∞ (e.g., Spirix exploded+normal=undefined, IEEE ∞+normal=∞).
    pub spirix_undefined_ieee_inf: u64,
    /// Spirix undefined, IEEE produced a finite value (e.g., Spirix vanished+vanished=undefined, IEEE produces a tiny normal).
    pub spirix_undefined_ieee_finite: u64,
    /// Spirix undefined, IEEE produced ±0 (e.g., Spirix vanished+vanished=undefined cancellation, IEEE flushed to 0).
    pub spirix_undefined_ieee_zero: u64,

    // ── Input-side conversion tracking (per-input, informational) ─────────
    /// Inputs that mapped to Spirix exploded during IEEE→Spirix conversion (e.g., IEEE biased_exp=254 inputs that exceed Spirix's max-normal-exp of 126).
    pub conversion_loss_to_exploded: u64,
    /// Inputs that mapped to Spirix vanished during IEEE→Spirix conversion (denormals below Spirix's smallest normal).
    pub conversion_loss_to_vanished: u64,
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

    pub fn record_conversion_loss_to_exploded(&mut self) {
        self.conversion_loss_to_exploded += 1;
    }

    pub fn record_conversion_loss_to_vanished(&mut self) {
        self.conversion_loss_to_vanished += 1;
    }

    pub fn record_spirix_normal_arch_drift(&mut self) { self.spirix_normal_arch_drift += 1; }
    pub fn record_spirix_zero_ieee_zero(&mut self) { self.spirix_zero_ieee_zero += 1; }
    pub fn record_spirix_zero_ieee_other(&mut self) { self.spirix_zero_ieee_other += 1; }
    pub fn record_spirix_vanished_ieee_other(&mut self) { self.spirix_vanished_ieee_other += 1; }
    pub fn record_spirix_exploded_ieee_other(&mut self) { self.spirix_exploded_ieee_other += 1; }
    pub fn record_spirix_undefined_ieee_nan(&mut self) { self.spirix_undefined_ieee_nan += 1; }
    pub fn record_spirix_undefined_ieee_inf(&mut self) { self.spirix_undefined_ieee_inf += 1; }
    pub fn record_spirix_undefined_ieee_finite(&mut self) { self.spirix_undefined_ieee_finite += 1; }
    pub fn record_spirix_undefined_ieee_zero(&mut self) { self.spirix_undefined_ieee_zero += 1; }

    pub fn print_summary(&self, n: u64) {
        let pct = |c: u64| c as f64 / n as f64 * 100.0;
        let row = |label: &str, count: u64| {
            println!("  {:42} {:>10}  ({:.4}%)", label, count, pct(count));
        };

        println!("Results ({} trials):", n);
        println!();
        println!("Spirix Normal output:");
        row("exact match", self.exact);
        row("1 ULP (valid rounding choice)", self.one_ulp);
        row("architectural drift (non-normal input or IEEE denormal/zero)", self.spirix_normal_arch_drift);
        row("off by more (REAL BUG — should be 0)", self.off_by_more);

        let zero_total = self.spirix_zero_ieee_zero + self.spirix_zero_ieee_other;
        if zero_total > 0 {
            println!();
            println!("Spirix Zero output:");
            row("IEEE -> 0 (match)", self.spirix_zero_ieee_zero);
            row("IEEE -> other (UNEXPECTED)", self.spirix_zero_ieee_other);
        }

        let vanished_total = self.spirix_vanished_ieee_zero
            + self.spirix_vanished_ieee_denormal
            + self.spirix_vanished_ieee_other;
        if vanished_total > 0 {
            println!();
            println!("Spirix Vanished output (architectural — Spirix preserves direction below precision):");
            row("IEEE -> 0", self.spirix_vanished_ieee_zero);
            row("IEEE -> denormal", self.spirix_vanished_ieee_denormal);
            row("IEEE -> other (UNEXPECTED)", self.spirix_vanished_ieee_other);
        }

        let exploded_total = self.spirix_exploded_ieee_inf
            + self.spirix_exploded_ieee_finite
            + self.spirix_exploded_ieee_other;
        if exploded_total > 0 {
            println!();
            println!("Spirix Exploded output (architectural — Spirix preserves direction beyond magnitude limit):");
            row("IEEE -> +/-inf", self.spirix_exploded_ieee_inf);
            row("IEEE -> finite (exp-range delta)", self.spirix_exploded_ieee_finite);
            row("IEEE -> other (UNEXPECTED)", self.spirix_exploded_ieee_other);
        }

        let undefined_total = self.spirix_undefined_ieee_nan
            + self.spirix_undefined_ieee_inf
            + self.spirix_undefined_ieee_finite
            + self.spirix_undefined_ieee_zero;
        if undefined_total > 0 {
            println!();
            println!("Spirix Undefined output (architectural — Spirix flags more cases as undefined than IEEE):");
            row("IEEE -> NaN  (architectural match)", self.spirix_undefined_ieee_nan);
            row("IEEE -> +/-inf  (Spirix more conservative)", self.spirix_undefined_ieee_inf);
            row("IEEE -> finite  (Spirix more conservative)", self.spirix_undefined_ieee_finite);
            row("IEEE -> 0  (Spirix more conservative)", self.spirix_undefined_ieee_zero);
        }

        if self.conversion_loss_to_exploded > 0 || self.conversion_loss_to_vanished > 0 {
            println!();
            println!("Input-side IEEE->Spirix conversion loss (informational, per input):");
            row("IEEE input -> Spirix exploded", self.conversion_loss_to_exploded);
            row("IEEE input -> Spirix vanished", self.conversion_loss_to_vanished);
        }

        if self.off_by_more > 0 || self.spirix_zero_ieee_other > 0 || self.spirix_vanished_ieee_other > 0 || self.spirix_exploded_ieee_other > 0 {
            println!();
            println!("  Max ULP error (Normal+Normal): {}", self.max_ulp_diff);
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
