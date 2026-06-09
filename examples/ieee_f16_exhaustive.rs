/// Exhaustive IEEE f16 → Spirix comparison for +, -, *, /, sqrt
///
/// Both paths start and end at f16: IEEE:   f16 → f32 → op → f32 → f16 Spirix: f16 → f32 → F4E3 → op → F4E3 → f32 → f16
///
/// Any difference is purely the arithmetic, since both paths eat the same conversion losses at the endpoints.
///
/// Run with: cargo run --example ieee_f16_exhaustive --release
use spirix::ScalarF4E4;

// ── f16 ↔ f32 conversion ──────────────────────────────────────────────────

/// IEEE binary16 bit pattern → f32 (lossless)
fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits >> 15) & 1) as u32;
    let exp = ((bits >> 10) & 0x1F) as u32;
    let frac = (bits & 0x3FF) as u32;

    if exp == 0 {
        if frac == 0 {
            // ±zero
            return f32::from_bits(sign << 31);
        }
        // Denormal: value = (-1)^s × 2^(-14) × 0.frac
        let mut f = frac;
        let mut e = 0i32;
        // Normalize: shift until bit 10 is set
        while (f & 0x400) == 0 {
            f <<= 1;
            e += 1;
        }
        f &= 0x3FF; // strip implicit bit
        let f32_exp = (127 - 15 - e) as u32;
        return f32::from_bits((sign << 31) | (f32_exp << 23) | (f << 13));
    }
    if exp == 31 {
        if frac == 0 {
            // ±infinity
            return f32::from_bits((sign << 31) | (0xFF << 23));
        }
        // NaN — preserve payload
        return f32::from_bits((sign << 31) | (0xFF << 23) | (frac << 13));
    }
    // Normal: rebias exponent from f16 bias (15) to f32 bias (127)
    let f32_exp = exp + 127 - 15;
    f32::from_bits((sign << 31) | (f32_exp << 23) | (frac << 13))
}

/// f32 → IEEE binary16 bit pattern (round-to-nearest-even)
fn f32_to_f16(v: f32) -> u16 {
    let bits = v.to_bits();
    let sign = ((bits >> 31) & 1) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let frac = bits & 0x7F_FFFF;

    // NaN
    if exp == 255 && frac != 0 {
        return (sign << 15) | 0x7E00; // quiet NaN
    }
    // Infinity
    if exp == 255 {
        return (sign << 15) | 0x7C00;
    }

    // Rebias: f32 bias 127 → f16 bias 15
    let new_exp = exp - 127 + 15;

    if new_exp >= 31 {
        // Overflow → infinity
        return (sign << 15) | 0x7C00;
    }

    if new_exp <= 0 {
        // Subnormal or underflow
        if new_exp < -10 {
            // Too small → zero
            return sign << 15;
        }
        // Denormal: shift mantissa with implicit bit
        let full_frac = frac | 0x80_0000; // add implicit 1
        let shift = (1 - new_exp) as u32 + 13; // total right-shift into 10-bit field
        let half = 1u32 << (shift - 1); // round bit
        let remainder = full_frac & ((1 << shift) - 1);
        let mut result = (full_frac >> shift) as u16;
        // Round-to-nearest-even
        if remainder > half || (remainder == half && (result & 1) != 0) {
            result += 1;
        }
        return (sign << 15) | result;
    }

    // Normal
    let half = 1u32 << 12; // bit 12 = round bit (bit below 13-bit shift)
    let remainder = frac & 0x1FFF; // bottom 13 bits
    let mut f16_frac = (frac >> 13) as u16;
    let mut f16_exp = new_exp as u16;

    // Round-to-nearest-even
    if remainder > half || (remainder == half && (f16_frac & 1) != 0) {
        f16_frac += 1;
        if f16_frac >= 0x400 {
            f16_frac = 0;
            f16_exp += 1;
            if f16_exp >= 31 {
                return (sign << 15) | 0x7C00; // overflow to inf
            }
        }
    }

    (sign << 15) | (f16_exp << 10) | f16_frac
}

// ── f16 classification ─────────────────────────────────────────────────────

fn f16_is_nan(bits: u16) -> bool {
    let exp = (bits >> 10) & 0x1F;
    let frac = bits & 0x3FF;
    exp == 31 && frac != 0
}

fn f16_is_inf(bits: u16) -> bool {
    let exp = (bits >> 10) & 0x1F;
    let frac = bits & 0x3FF;
    exp == 31 && frac == 0
}

fn f16_is_zero(bits: u16) -> bool {
    (bits & 0x7FFF) == 0
}

fn f16_is_neg_zero(bits: u16) -> bool {
    bits == 0x8000
}

fn f16_is_denormal(bits: u16) -> bool {
    let exp = (bits >> 10) & 0x1F;
    let frac = bits & 0x3FF;
    exp == 0 && frac != 0
}

fn f16_sign(bits: u16) -> bool {
    (bits >> 15) != 0
}

// ── Result tracking ────────────────────────────────────────────────────────

#[derive(Default)]
struct OpStats {
    total: u64,
    exact: u64,
    off_by_1: u64,
    off_by_gt1: u64,
    // Expected disagreements
    both_nan: u64,           // IEEE NaN, Spirix NaN → both become NaN f16
    both_inf: u64,           // both overflow to inf
    both_zero: u64,          // both underflow to zero
    neg_zero_vs_zero: u64,   // IEEE -0 vs Spirix +0 (or vice versa)
    neg_inf_vs_pos_inf: u64, // IEEE -inf vs Spirix +inf (singular infinity)
    nan_vs_nan_payload: u64, // both NaN but different payloads
    spirix_undefined: u64,   // Spirix undefined where IEEE has a value (0/0, inf/inf, etc.)
    ieee_nan_only: u64,      // IEEE NaN but Spirix has a value
    other_disagree: u64,     // unexpected
    max_ulp: u64,
    worst_a: u16,
    worst_b: u16,
}

impl OpStats {
    fn classify(&mut self, a_bits: u16, b_bits: u16, ieee_f16: u16, spirix_f16: u16) {
        self.total += 1;

        // Exact bit match — done
        if ieee_f16 == spirix_f16 {
            self.exact += 1;
            return;
        }

        let ieee_nan = f16_is_nan(ieee_f16);
        let spirix_nan = f16_is_nan(spirix_f16);
        let ieee_inf = f16_is_inf(ieee_f16);
        let spirix_inf = f16_is_inf(spirix_f16);
        let ieee_zero = f16_is_zero(ieee_f16);
        let spirix_zero = f16_is_zero(spirix_f16);

        // Both NaN (possibly different payloads) — expected
        if ieee_nan && spirix_nan {
            self.nan_vs_nan_payload += 1;
            return;
        }

        // -0 vs +0 — expected (Spirix has single zero)
        if ieee_zero && spirix_zero {
            self.neg_zero_vs_zero += 1;
            return;
        }

        // Both inf but different signs — expected (Spirix singular infinity)
        if ieee_inf && spirix_inf {
            if f16_sign(ieee_f16) != f16_sign(spirix_f16) {
                self.neg_inf_vs_pos_inf += 1;
            } else {
                self.both_inf += 1; // same sign inf, shouldn't differ but just in case
            }
            return;
        }

        // IEEE NaN but Spirix produced a value (or vice versa)
        if ieee_nan && !spirix_nan {
            self.ieee_nan_only += 1;
            return;
        }
        if spirix_nan && !ieee_nan {
            self.spirix_undefined += 1;
            return;
        }

        // Both are finite, non-NaN, non-zero: compare ULPs
        if !ieee_nan && !spirix_nan && !ieee_inf && !spirix_inf {
            // Same sign?
            if f16_sign(ieee_f16) != f16_sign(spirix_f16) {
                // Different signs — big disagreement
                self.other_disagree += 1;
                return;
            }
            // ULP distance
            let ieee_mag = ieee_f16 & 0x7FFF;
            let spirix_mag = spirix_f16 & 0x7FFF;
            let ulp = if ieee_mag > spirix_mag {
                (ieee_mag - spirix_mag) as u64
            } else {
                (spirix_mag - ieee_mag) as u64
            };

            if ulp == 1 {
                self.off_by_1 += 1;
            } else {
                self.off_by_gt1 += 1;
                if ulp > self.max_ulp {
                    self.max_ulp = ulp;
                    self.worst_a = a_bits;
                    self.worst_b = b_bits;
                }
            }
            return;
        }

        // One is inf, other isn't; or other mismatches
        self.other_disagree += 1;
    }

    fn print(&self, name: &str) {
        let pct = |n: u64| n as f64 / self.total as f64 * 100.0;
        println!("  {name}  ({} pairs)", self.total);
        println!(
            "    Exact match:      {:>12} ({:.4}%)",
            self.exact,
            pct(self.exact)
        );
        println!(
            "    Off by 1 ULP:     {:>12} ({:.6}%)",
            self.off_by_1,
            pct(self.off_by_1)
        );
        if self.off_by_gt1 > 0 {
            println!(
                "    Off by >1 ULP:    {:>12} ({:.6}%)  *** CHECK ***",
                self.off_by_gt1,
                pct(self.off_by_gt1)
            );
            println!("      Max ULP error:  {}", self.max_ulp);
            println!(
                "      Worst pair:     a=0x{:04X}, b=0x{:04X}",
                self.worst_a, self.worst_b
            );
            let a = f16_to_f32(self.worst_a);
            let b = f16_to_f32(self.worst_b);
            println!("                      a={a}, b={b}");
        }
        println!("    — Expected disagreements —");
        if self.neg_zero_vs_zero > 0 {
            println!("    -0 vs +0:         {:>12}", self.neg_zero_vs_zero);
        }
        if self.neg_inf_vs_pos_inf > 0 {
            println!(
                "    -inf vs +inf:     {:>12} (Spirix singular infinity)",
                self.neg_inf_vs_pos_inf
            );
        }
        if self.nan_vs_nan_payload > 0 {
            println!("    NaN payloads:     {:>12}", self.nan_vs_nan_payload);
        }
        if self.spirix_undefined > 0 {
            println!(
                "    Spirix undefined: {:>12} (0/0, inf/inf, sqrt(-), etc.)",
                self.spirix_undefined
            );
        }
        if self.ieee_nan_only > 0 {
            println!("    IEEE NaN only:    {:>12}", self.ieee_nan_only);
        }
        if self.other_disagree > 0 {
            println!(
                "    Other disagree:   {:>12}  *** CHECK ***",
                self.other_disagree
            );
        }
    }
}

// ── Binary op test ─────────────────────────────────────────────────────────

fn test_binary_op(
    name: &str,
    ieee_op: fn(f32, f32) -> f32,
    spirix_op: fn(ScalarF4E4, ScalarF4E4) -> ScalarF4E4,
) {
    let mut stats = OpStats::default();
    let total = 65536u64 * 65536;

    for a_bits in 0u16..=u16::MAX {
        if a_bits % 4096 == 0 {
            eprint!("\r  {name}: {:.1}%", a_bits as f64 / 65536.0 * 100.0);
        }

        let a_f32 = f16_to_f32(a_bits);

        for b_bits in 0u16..=u16::MAX {
            // IEEE path: f16 → f32 → op → f32 → f16
            let ieee_result = ieee_op(a_f32, f16_to_f32(b_bits));
            let ieee_f16 = f32_to_f16(ieee_result);

            // Spirix path: f16 → f32 → F4E3 → op → F4E3 → f32 → f16
            let sa = ScalarF4E4::from(a_f32);
            let sb = ScalarF4E4::from(f16_to_f32(b_bits));
            let spirix_result = spirix_op(sa, sb);
            let spirix_f32: f32 = (&spirix_result).into();
            let spirix_f16 = f32_to_f16(spirix_f32);

            stats.classify(a_bits, b_bits, ieee_f16, spirix_f16);
        }
    }
    eprintln!("\r  {name}: done     ");
    stats.print(name);
    let agree = stats.exact + stats.off_by_1;
    let expected = stats.neg_zero_vs_zero
        + stats.neg_inf_vs_pos_inf
        + stats.nan_vs_nan_payload
        + stats.spirix_undefined
        + stats.ieee_nan_only;
    println!("    ─────────────────");
    println!(
        "    Agree (exact+1ULP): {:>12} ({:.4}%)",
        agree,
        agree as f64 / total as f64 * 100.0
    );
    println!(
        "    Expected disagree:  {:>12} ({:.4}%)",
        expected,
        expected as f64 / total as f64 * 100.0
    );
    println!();
}

// ── Unary op test (sqrt) ───────────────────────────────────────────────────

fn test_unary_op(name: &str, ieee_op: fn(f32) -> f32, spirix_op: fn(&ScalarF4E4) -> ScalarF4E4) {
    let mut stats = OpStats::default();
    let total = 65536u64;

    for a_bits in 0u16..=u16::MAX {
        let a_f32 = f16_to_f32(a_bits);

        let ieee_result = ieee_op(a_f32);
        let ieee_f16 = f32_to_f16(ieee_result);

        let sa = ScalarF4E4::from(a_f32);
        let spirix_result = spirix_op(&sa);
        let spirix_f32: f32 = (&spirix_result).into();
        let spirix_f16 = f32_to_f16(spirix_f32);

        stats.classify(a_bits, 0, ieee_f16, spirix_f16);
    }
    stats.print(name);
    let agree = stats.exact + stats.off_by_1;
    let expected = stats.neg_zero_vs_zero
        + stats.neg_inf_vs_pos_inf
        + stats.nan_vs_nan_payload
        + stats.spirix_undefined
        + stats.ieee_nan_only;
    println!("    ─────────────────");
    println!(
        "    Agree (exact+1ULP): {:>12} ({:.4}%)",
        agree,
        agree as f64 / total as f64 * 100.0
    );
    println!(
        "    Expected disagree:  {:>12} ({:.4}%)",
        expected,
        expected as f64 / total as f64 * 100.0
    );
    println!();
}

// ── Main ───────────────────────────────────────────────────────────────────

fn main() {
    println!("Exhaustive IEEE f16 → Spirix F4E3 comparison");
    println!("  IEEE path:   f16 → f32 → op → f32 → f16");
    println!("  Spirix path: f16 → f32 → F4E3 → op → F4E3 → f32 → f16");
    println!("  Binary ops: all 65536 × 65536 = 4,294,967,296 pairs");
    println!("  Unary ops:  all 65536 values");
    println!();

    test_binary_op("Add", |a, b| a + b, |a, b| a + b);
    test_binary_op("Sub", |a, b| a - b, |a, b| a - b);
    test_binary_op("Mul", |a, b| a * b, |a, b| a * b);
    test_binary_op("Div", |a, b| a / b, |a, b| a / b);
    test_unary_op("Sqrt", f32::sqrt, ScalarF4E4::sqrt);
}
