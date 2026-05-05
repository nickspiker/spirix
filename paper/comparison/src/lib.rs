//! Shared infrastructure for IEEE 754 binary32 comparison harnesses.
//!
//! Each per-operation binary in `src/bin/` provides a single bit-accurate Spirix algorithm; everything else (Spirix↔IEEE conversion, state classification, comparison categorization, phase orchestration) lives here. The bit-accurate algorithm sits in the binary so a paper reader can verify the implementation in one file without crossing module boundaries.

use spirix::Scalar;

// ── Format constants (Spirix v0.1 N0 at FRAC=24, EXP=8) ─────────────────────

/// Storage fraction width — 24 bits, "wonky" non-power-of-2 width chosen for bit-equivalence with IEEE 754 binary32 (32 total storage bits, 24 effective precision bits).
pub const FRAC: i32 = 24;
/// Compute fraction width — 25 bits = FRAC + implicit complement bit reconstructed at compute. Same as v0.0 N1's stored width; the algorithm transfers verbatim between v0.0 and v0.1, with only the conversion functions and AMB sentinel position differing.
pub const COMPUTE_FRAC: i32 = 25;
/// Internal arithmetic width — compute width + 3 bits of guard/round/sticky headroom = 28.
pub const INT_BITS: i32 = COMPUTE_FRAC + 3;
/// Ambiguous exponent sentinel under v0.1 — the single reserved exponent value used for all non-normal Spirix states (zero, infinity, exploded, vanished, undefined). Both overflow past i8::MAX-1 and underflow past i8::MIN saturate here.
pub const AMB_EXP: i8 = i8::MAX;

// ── Spirix state enum and operand ───────────────────────────────────────────

/// State of a Spirix Scalar — every bit pattern maps to exactly one of these by design.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpirixState {
    Normal,
    Zero,
    Vanished,
    Exploded,
    Infinity,
    Undefined,
}

/// Operand carried through bit-accurate Spirix algorithms. For Normal state, `q` is the compute-form Q (sign-extended 25-bit two's complement) and `exp` is the unbiased exponent. For non-normal states (exp == AMB_EXP), `q` carries the storage pattern in its lower 24 bits.
#[derive(Debug, Clone, Copy)]
pub struct Operand {
    pub q: i32,
    pub exp: i8,
    pub state: SpirixState,
}

/// IEEE 754 binary32 result kind for categorization purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IeeeKind {
    Normal,
    Zero,
    Denormal,
    Inf,
    Nan,
}

// ── Classifiers ─────────────────────────────────────────────────────────────

/// Classify a Spirix Scalar from its (q, exp) storage. Used after IEEE→Spirix conversion and at the output of arithmetic to decide which truth-table branch or category applies.
pub fn classify_state(q: i32, exp: i8) -> SpirixState {
    if exp != AMB_EXP {
        return SpirixState::Normal;
    }
    let stored = (q as u32) & 0xFF_FFFF;
    if stored == 0 {
        return SpirixState::Zero;
    }
    if stored == 0xFF_FFFF {
        return SpirixState::Infinity;
    }
    let lsbc = leading_same_bit_count(stored);
    match lsbc {
        1 => SpirixState::Exploded,
        2 => SpirixState::Vanished,
        _ => SpirixState::Undefined,
    }
}

pub fn classify_ieee(v: f32) -> IeeeKind {
    if v.is_nan() {
        IeeeKind::Nan
    } else if v.is_infinite() {
        IeeeKind::Inf
    } else if v == 0.0 {
        IeeeKind::Zero
    } else if is_denormal(v) {
        IeeeKind::Denormal
    } else {
        IeeeKind::Normal
    }
}

/// Returns true if `v` is a denormal (subnormal) IEEE 754 binary32 value.
pub fn is_denormal(v: f32) -> bool {
    let bits = v.to_bits();
    let biased_exp = (bits >> 23) & 0xFF;
    let mantissa = bits & 0x7F_FFFF;
    biased_exp == 0 && mantissa != 0
}

/// Count the leading same-bit run length in the lower 24 bits of `stored`. Used to classify Spirix non-normal states at the ambiguous exponent: 1 = exploded, 2 = vanished, 3+ = undefined. Returns FRAC for the uniform patterns (zero, infinity).
pub fn leading_same_bit_count(stored: u32) -> i32 {
    let stored = stored & 0xFF_FFFF;
    if stored == 0 || stored == 0xFF_FFFF {
        return FRAC;
    }
    let msb = (stored >> (FRAC - 1)) & 1;
    let mut count = 1;
    for i in (0..FRAC - 1).rev() {
        let bit = (stored >> i) & 1;
        if bit == msb {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Count the leading same-bit run length of `value`, examining the top `width` bits. Returns `width` for uniform patterns. Used during the IEEE→Spirix conversion bridge to classify the lib's 32-bit storage form.
pub fn leading_same_bit_count_u32(value: u32, width: u32) -> u32 {
    debug_assert!(width <= 32);
    let mask = if width == 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    };
    let masked = value & mask;
    if masked == 0 || masked == mask {
        return width;
    }
    let msb = (masked >> (width - 1)) & 1;
    let mut count = 1u32;
    for i in (0..width - 1).rev() {
        let bit = (masked >> i) & 1;
        if bit == msb {
            count += 1;
        } else {
            break;
        }
    }
    count
}

// ── Conversions ─────────────────────────────────────────────────────────────

/// Convert an f32 to a Spirix Operand at FRAC=24, using the spirix lib for the IEEE→Scalar mapping (so denormal handling, ±∞ → exploded, NaN → undefined, ±0 → zero, and the high-end conversion-loss-to-exploded boundary all match the lib's authoritative behavior).
///
/// The lib produces a `Scalar<i32, i8>` at FRAC=32. We bridge to our FRAC=24 compute form by truncating the lib's stored 32-bit fraction down 8 bits — bit-exact for f32 inputs because IEEE binary32 has only 24 effective fraction bits, so the lib's lower 8 stored bits are always zero for f32-sourced normals.
pub fn f32_to_spirix_via_lib(v: f32) -> Operand {
    let s: Scalar<i32, i8> = v.into();

    if s.exponent == AMB_EXP {
        let stored_32 = s.fraction as u32;
        let stored_24 = (stored_32 >> 8) as i32;
        let state = if stored_32 == 0 {
            SpirixState::Zero
        } else if stored_32 == 0xFFFF_FFFF {
            SpirixState::Infinity
        } else {
            let lsbc = leading_same_bit_count_u32(stored_32, 32);
            match lsbc {
                1 => SpirixState::Exploded,
                2 => SpirixState::Vanished,
                _ => SpirixState::Undefined,
            }
        };
        return Operand { q: stored_24, exp: AMB_EXP, state };
    }

    let stored_24 = (s.fraction as u32) >> 8;
    let msb = (stored_24 >> 23) & 1;
    let prefix = 1 - msb;
    let q_25bit = (prefix << 24) | stored_24;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    Operand { q, exp: s.exponent, state: SpirixState::Normal }
}

/// Convert Spirix v0.1 N0 (Q, exp) back to f32 using the lib's to_f32. Bridges our FRAC=24 form to the lib's Scalar<i32, i8> at FRAC=32 by left-shifting the 24-bit storage to occupy the upper 24 bits of the 32-bit fraction (bit-exact since FRAC=24 has no info below those bits).
///
/// **Spirix-design override**: per Spirix's "no signed zero" principle, any zero result from the lib (which would be -0.0 for negative-vanished inputs) is normalized to +0.0. This makes the IEEE-side reference consistent with Spirix's signless treatment of zero.
pub fn spirix_to_f32(q: i32, exp: i8) -> f32 {
    let stored_24 = (q as u32) & 0xFF_FFFF;
    let stored_32 = if exp == AMB_EXP && stored_24 == 0xFF_FFFF {
        0xFFFF_FFFFu32
    } else {
        stored_24 << 8
    };
    let s = Scalar::<i32, i8> {
        fraction: stored_32 as i32,
        exponent: exp,
    };
    let result = s.to_f32();
    if result == 0.0 {
        0.0
    } else {
        result
    }
}

/// Negate a Spirix Operand using the lib's `Neg` impl. Handles all states correctly (normal sign flip with renormalization for boundary cases like ±1.0; signless states stay signless; phase flip for exploded/vanished; undefined propagates).
pub fn spirix_negate(op: Operand) -> Operand {
    let stored_24 = (op.q as u32) & 0xFF_FFFF;
    let stored_32 = if op.exp == AMB_EXP && stored_24 == 0xFF_FFFF {
        0xFFFF_FFFFu32
    } else {
        stored_24 << 8
    };
    let s = Scalar::<i32, i8> {
        fraction: stored_32 as i32,
        exponent: op.exp,
    };
    let neg_s = -s;

    if neg_s.exponent == AMB_EXP {
        let neg_stored_32 = neg_s.fraction as u32;
        let neg_stored_24 = (neg_stored_32 >> 8) as i32;
        let state = if neg_stored_32 == 0 {
            SpirixState::Zero
        } else if neg_stored_32 == 0xFFFF_FFFF {
            SpirixState::Infinity
        } else {
            let lsbc = leading_same_bit_count_u32(neg_stored_32, 32);
            match lsbc {
                1 => SpirixState::Exploded,
                2 => SpirixState::Vanished,
                _ => SpirixState::Undefined,
            }
        };
        return Operand { q: neg_stored_24, exp: AMB_EXP, state };
    }

    let neg_stored_24 = (neg_s.fraction as u32) >> 8;
    let msb = (neg_stored_24 >> 23) & 1;
    let prefix = 1 - msb;
    let q_25bit = (prefix << 24) | neg_stored_24;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    Operand { q, exp: neg_s.exponent, state: SpirixState::Normal }
}

// ── Random generators ───────────────────────────────────────────────────────

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
}

/// Generate a random valid Spirix operand. Random 24-bit fraction + random 8-bit exponent — every bit pattern is a valid Spirix state by design.
pub fn random_spirix_operand(rng: &mut Lcg) -> Operand {
    let stored_24 = (rng.next_u32() & 0xFF_FFFF) as i32;
    let exp_byte = rng.next_u32() as u8;
    let exp = exp_byte as i8;

    if exp == AMB_EXP {
        let state = classify_state(stored_24, AMB_EXP);
        return Operand { q: stored_24, exp: AMB_EXP, state };
    }

    let stored_24_u = stored_24 as u32 & 0xFF_FFFF;
    let msb = (stored_24_u >> 23) & 1;
    let prefix = 1 - msb;
    let q_25bit = (prefix << 24) | stored_24_u;
    let q = if (q_25bit >> 24) & 1 != 0 {
        (q_25bit | !0x1FF_FFFF_u32) as i32
    } else {
        q_25bit as i32
    };
    Operand { q, exp, state: SpirixState::Normal }
}

// ── Counters and categorization ─────────────────────────────────────────────

/// Counters tabulating per-category comparison results.
///
/// Categories are organized by Spirix output state, then sub-divided by what IEEE produced. The "architectural" categories are honest-by-design mismatches between IEEE 754 and Spirix v0.1 semantics; "off_by_more" should always be zero for a correct implementation.
#[derive(Default, Debug)]
pub struct Counters {
    pub exact: u64,
    pub one_ulp: u64,
    pub spirix_normal_arch_drift: u64,
    pub off_by_more: u64,
    pub max_ulp_diff: u64,
    pub worst_a: f32,
    pub worst_b: f32,

    pub spirix_zero_ieee_zero: u64,
    pub spirix_zero_ieee_other: u64,

    pub spirix_vanished_ieee_zero: u64,
    pub spirix_vanished_ieee_denormal: u64,
    pub spirix_vanished_ieee_other: u64,

    pub spirix_exploded_ieee_inf: u64,
    pub spirix_exploded_ieee_finite: u64,
    pub spirix_exploded_ieee_other: u64,

    pub spirix_undefined_ieee_nan: u64,
    pub spirix_undefined_ieee_inf: u64,
    pub spirix_undefined_ieee_finite: u64,
    pub spirix_undefined_ieee_zero: u64,

    pub conversion_loss_to_exploded: u64,
    pub conversion_loss_to_vanished: u64,
}

impl Counters {
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

/// Compute ULP difference between two f32 values of the same sign. Returns u64::MAX for sign mismatches or NaN inputs.
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

/// Categorize a single (a, b) → result pair against IEEE and update counters. Used by both Phase A (IEEE-driven) and Phase B (Spirix-driven) — categorization is symmetric in input direction. Caller provides the Spirix operation result `(r_q, r_exp)` and the IEEE result `ieee_result`; this function classifies output states and buckets accordingly.
pub fn categorize_and_record(
    a: f32,
    b: f32,
    ieee_result: f32,
    a_op: Operand,
    b_op: Operand,
    r_q: i32,
    r_exp: i8,
    counters: &mut Counters,
) {
    let r_state = classify_state(r_q, r_exp);
    let ieee_kind = classify_ieee(ieee_result);

    let inputs_had_arch_input = a_op.state != SpirixState::Normal
        || b_op.state != SpirixState::Normal
        || classify_ieee(a) != IeeeKind::Normal
        || classify_ieee(b) != IeeeKind::Normal;

    match r_state {
        SpirixState::Normal => {
            let spirix_result = spirix_to_f32(r_q, r_exp);
            match ieee_kind {
                IeeeKind::Normal => {
                    if ieee_result == 0.0 && spirix_result == 0.0 {
                        counters.exact += 1;
                    } else if ieee_result == 0.0 || spirix_result == 0.0 {
                        counters.record_off_by_more(u64::MAX, a, b);
                    } else {
                        let diff = ulp_diff_same_sign(ieee_result, spirix_result);
                        if diff == 0 {
                            counters.exact += 1;
                        } else if inputs_had_arch_input {
                            counters.spirix_normal_arch_drift += 1;
                        } else if diff == 1 {
                            counters.one_ulp += 1;
                        } else {
                            counters.record_off_by_more(diff, a, b);
                        }
                    }
                }
                IeeeKind::Zero | IeeeKind::Denormal => {
                    counters.spirix_normal_arch_drift += 1;
                }
                IeeeKind::Inf | IeeeKind::Nan => {
                    counters.record_off_by_more(u64::MAX, a, b);
                }
            }
        }
        SpirixState::Zero => match ieee_kind {
            IeeeKind::Zero => counters.spirix_zero_ieee_zero += 1,
            _ => counters.spirix_zero_ieee_other += 1,
        },
        SpirixState::Vanished => match ieee_kind {
            IeeeKind::Zero => counters.spirix_vanished_ieee_zero += 1,
            IeeeKind::Denormal => counters.spirix_vanished_ieee_denormal += 1,
            _ => counters.spirix_vanished_ieee_other += 1,
        },
        SpirixState::Exploded => match ieee_kind {
            IeeeKind::Inf => counters.spirix_exploded_ieee_inf += 1,
            IeeeKind::Normal => counters.spirix_exploded_ieee_finite += 1,
            _ => counters.spirix_exploded_ieee_other += 1,
        },
        SpirixState::Infinity => match ieee_kind {
            IeeeKind::Nan => counters.spirix_undefined_ieee_nan += 1,
            _ => counters.record_off_by_more(u64::MAX, a, b),
        },
        SpirixState::Undefined => match ieee_kind {
            IeeeKind::Nan => counters.spirix_undefined_ieee_nan += 1,
            IeeeKind::Inf => counters.spirix_undefined_ieee_inf += 1,
            IeeeKind::Normal | IeeeKind::Denormal => counters.spirix_undefined_ieee_finite += 1,
            IeeeKind::Zero => counters.spirix_undefined_ieee_zero += 1,
        },
    }
}

// ── Phase orchestration ─────────────────────────────────────────────────────

/// Run a Phase A (IEEE-driven random) test. Generates random IEEE bit patterns, applies the IEEE op for the reference, and the Spirix op on the lib-converted operands.
pub fn run_phase_a<I, S>(
    op_name: &str,
    n: u64,
    seed: u64,
    ieee_op: I,
    spirix_op: S,
) -> Counters
where
    I: Fn(f32, f32) -> f32,
    S: Fn(Operand, Operand) -> (i32, i8),
{
    println!("== Phase A: IEEE-driven random — {n} trials ({op_name}) ==");
    println!("Inputs: full random IEEE 754 binary32 bit patterns (every f32 bit pattern is a valid IEEE value). Lib's from(f32) maps each to its corresponding valid Spirix state.");
    println!();

    let mut rng = Lcg::new(seed);
    let mut counters = Counters::default();
    for _ in 0..n {
        let a = f32::from_bits(rng.next_u32());
        let b = f32::from_bits(rng.next_u32());
        let a_op = f32_to_spirix_via_lib(a);
        let b_op = f32_to_spirix_via_lib(b);
        for op in [a_op, b_op] {
            match op.state {
                SpirixState::Exploded => counters.conversion_loss_to_exploded += 1,
                SpirixState::Vanished => counters.conversion_loss_to_vanished += 1,
                _ => {}
            }
        }
        let ieee_result = ieee_op(a, b);
        let (r_q, r_exp) = spirix_op(a_op, b_op);
        categorize_and_record(a, b, ieee_result, a_op, b_op, r_q, r_exp, &mut counters);
    }
    counters.print_summary(n);
    counters
}

/// Run a Phase B (Spirix-driven random) test. Generates random Spirix bit patterns directly, then converts each operand to f32 via the lib (with Spirix's signless-zero principle enforced) for the IEEE-side reference.
pub fn run_phase_b<I, S>(
    op_name: &str,
    n: u64,
    seed: u64,
    ieee_op: I,
    spirix_op: S,
) -> Counters
where
    I: Fn(f32, f32) -> f32,
    S: Fn(Operand, Operand) -> (i32, i8),
{
    println!("== Phase B: Spirix-driven random — {n} trials ({op_name}) ==");
    println!("Inputs: full random Spirix bit patterns (random 24-bit fraction + random i8 exponent — every bit pattern is a valid Spirix state by design). IEEE-side reference computed via the lib's to_f32, with Spirix's signless-zero principle enforced (vanished → +0; zero → +0; signless infinity → NaN; undefined → NaN; exploded → ±∞ with phase; otherwise precise).");
    println!();

    let mut rng = Lcg::new(seed);
    let mut counters = Counters::default();
    for _ in 0..n {
        let a_op = random_spirix_operand(&mut rng);
        let b_op = random_spirix_operand(&mut rng);
        let a = spirix_to_f32(a_op.q, a_op.exp);
        let b = spirix_to_f32(b_op.q, b_op.exp);
        let ieee_result = ieee_op(a, b);
        let (r_q, r_exp) = spirix_op(a_op, b_op);
        categorize_and_record(a, b, ieee_result, a_op, b_op, r_q, r_exp, &mut counters);
    }
    counters.print_summary(n);
    counters
}
