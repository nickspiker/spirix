/// Bit-accurate Rust port of spirix_add_unified.v (FRAC=25, EXP=8) compared against native f32 addition (IEEE 754 binary32, RNE).
///
/// Converts f32 pairs → Spirix N1 format, adds with the Verilog algorithm (close/far split, shared barrel, banker's rounding), converts back to f32, and checks against hardware f32 addition.
///
/// Run with:  cargo run --example ieee_add_f32 --release

const FRAC: i32 = 25;
const EXP: i32 = 8;
const INT_BITS: i32 = FRAC + 3; // 28
const AMB_EXP: i8 = i8::MAX; // 127 — matches Rust src `ambiguous_exponent() = E::max_value()`

// ── f32 ↔ Spirix conversion (lossless for normal f32) ──────────────────────

/// Convert a normal, finite, nonzero f32 to Spirix N1 (i32 frac, i8 exp). f32 significand is 24 bits (1 implicit + 23 stored). Spirix FRAC=25: sign + 24 magnitude bits → exact match.
fn f32_to_spirix(v: f32) -> (i32, i8) {
    let bits = v.to_bits();
    let sign = (bits >> 31) != 0;
    let biased_exp = ((bits >> 23) & 0xFF) as i32;
    let mantissa = (bits & 0x7F_FFFF) as i32;

    if biased_exp == 0 || biased_exp == 255 {
        // subnormal, zero, inf, nan — not tested
        return (0, AMB_EXP);
    }

    // IEEE significand = 1.mantissa = (2^23 + mantissa)
    let sig = (1 << 23) | mantissa; // 24-bit unsigned, in [2^23, 2^24-1]

    // Spirix fraction: signed 25-bit, N1 = top 2 bits differ positive: 01xxx...x, negative: two's complement
    let frac: i32 = if sign { -sig } else { sig };

    // Spirix exponent: value = frac * 2^(exp - 24) IEEE:  value = sig * 2^(biased_exp - 150) So: exp - 24 = biased_exp - 150  →  exp = biased_exp - 126
    let exp = (biased_exp - 126) as i8;

    (frac, exp)
}

/// Convert Spirix N1 (i32 frac, i8 exp) back to f32.
fn spirix_to_f32(frac: i32, exp: i8) -> f32 {
    if frac == 0 || exp == AMB_EXP {
        return 0.0;
    }

    let sign = frac < 0;
    let mag = if sign { (-frac) as u32 } else { frac as u32 };

    // mag should be N1: bit 23 set, bits 24+ clear (for FRAC=25) Find leading bit position
    let lz = mag.leading_zeros(); // e.g. 8 means bit 23 is MSB
    let shift = lz as i32 - 8; // how many bits to shift to get bit 23 as MSB

    let normalized_mag = if shift > 0 {
        mag << shift
    } else if shift < 0 {
        mag >> (-shift)
    } else {
        mag
    };

    let mantissa = (normalized_mag & 0x7F_FFFF) as u32;
    let biased_exp = (exp as i32 + 126 - shift) as u32;

    if biased_exp == 0 || biased_exp >= 255 {
        // overflow/underflow
        if biased_exp >= 255 {
            return if sign {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
        }
        return 0.0; // subnormal → flush to zero
    }

    let bits = ((sign as u32) << 31) | (biased_exp << 23) | mantissa;
    f32::from_bits(bits)
}

// ── Bit-accurate port of spirix_add_unified.v ──────────────────────────────

fn spirix_add(a_frac: i32, a_exp: i8, b_frac: i32, b_exp: i8) -> (i32, i8) {
    // Step 1: exponent difference + swap
    let raw_diff = (a_exp as i16) - (b_exp as i16);
    let a_is_big = raw_diff >= 0;

    let (big_frac, big_exp, small_frac) = if a_is_big {
        (a_frac, a_exp, b_frac)
    } else {
        (b_frac, b_exp, a_frac)
    };

    let exp_diff = raw_diff.unsigned_abs() as i32;
    let negligible = exp_diff >= FRAC;

    if negligible {
        return (big_frac, big_exp);
    }

    let is_close = exp_diff <= 1;

    // Step 2: extend to INT_BITS (28-bit) with 2-bit left shift
    let big_ext = (big_frac as i64) << 2;
    let small_ext = (small_frac as i64) << 2;

    // We use i64 thruout to avoid overflow in intermediate calculations. The Verilog uses INT_BITS=28 bit wires; we mask to 28 bits where needed.

    // Mask to simulate INT_BITS-bit signed arithmetic
    let mask = (1i64 << INT_BITS) - 1; // 0x0FFF_FFFF (28 bits)
    let sign_bit = 1i64 << (INT_BITS - 1); // bit 27

    // Sign-extend a value to i64 from INT_BITS bits
    let sext = |v: i64| -> i64 {
        let v = v & mask;
        if v & sign_bit != 0 {
            v | !mask
        } else {
            v
        }
    };

    let big_ext = sext(big_ext);
    let small_ext = sext(small_ext);

    if is_close {
        // Step 3: Close path
        let close_small = if (exp_diff & 1) != 0 {
            sext(small_ext >> 1)
        } else {
            small_ext
        };
        let close_align_sticky = ((exp_diff & 1) != 0) && ((small_ext & 1) != 0);
        let close_sum = sext(big_ext + close_small);
        let close_is_zero = close_sum == 0;

        if close_is_zero {
            return (0, AMB_EXP);
        }

        // CLZ: count leading redundant sign bits In Verilog we use XOR-adjacent. In Rust, simpler: For a signed INT_BITS-bit number, leading = max(leading_zeros, leading_ones) after masking to INT_BITS bits.
        let ubits = (close_sum & mask) as u32;
        let top_bit = (close_sum >> (INT_BITS - 1)) & 1;

        let leading = if top_bit != 0 {
            // negative: count leading 1s (invert and count leading 0s)
            let inverted = (!ubits) & (mask as u32);
            inverted.leading_zeros() as i32 - (32 - INT_BITS)
        } else {
            // positive: count leading 0s
            ubits.leading_zeros() as i32 - (32 - INT_BITS)
        };

        let close_norm_shift = leading - 1;

        // Normalize via left shift
        let close_normalized = sext(close_sum << close_norm_shift);

        // Extract fraction
        let out_frac_raw = (close_normalized >> (INT_BITS - FRAC)) as i32;

        // Rounding bits
        let guard = ((close_normalized >> (INT_BITS - 1 - FRAC)) & 1) != 0;
        let lsb = ((close_normalized >> (INT_BITS - FRAC)) & 1) != 0;
        let round_bit = ((close_normalized >> (INT_BITS - 2 - FRAC)) & 1) != 0;
        let ext_sticky_mask = (1i64 << (INT_BITS - 3 - FRAC)) - 1;
        let ext_sticky = (close_normalized & ext_sticky_mask) != 0;
        let sticky = ext_sticky || close_align_sticky;
        let round_up = guard && (round_bit || sticky || lsb);

        // Rounding overflow detection (early, from pre-round signals)
        let pos_all_ones = (out_frac_raw & ((1 << (FRAC - 1)) - 1)) == ((1 << (FRAC - 1)) - 1);
        let rovf_pos = (out_frac_raw >> (FRAC - 1)) & 1 == 0 && pos_all_ones && round_up;

        // rovf_neg: frac = 10111...1 and round_up
        let rovf_neg = ((out_frac_raw >> (FRAC - 1)) & 1 == 1)
            && ((out_frac_raw >> (FRAC - 2)) & 1 == 0)
            && ((out_frac_raw & ((1 << (FRAC - 2)) - 1)) == (1 << (FRAC - 2)) - 1)
            && round_up;

        let pos_half: i32 = 1 << (FRAC - 2); // 01000...0
        let neg_one: i32 = -(1 << (FRAC - 1)); // 10000...0

        let out_frac = if rovf_pos {
            pos_half
        } else if rovf_neg {
            neg_one
        } else {
            out_frac_raw + if round_up { 1 } else { 0 }
        };

        // Exponent
        let exp_wide = (big_exp as i16) + 2 - (leading as i16) + if rovf_pos { 1 } else { 0 }
            - if rovf_neg { 1 } else { 0 };
        let out_exp = exp_wide as i8;

        // Underflow check
        let offset = out_exp.wrapping_sub(1);
        let underflow = big_exp < 0 && offset >= 0;
        if underflow {
            let uf_frac = ((close_normalized >> (INT_BITS - 1)) << (FRAC - 1))
                | (((close_normalized >> (INT_BITS - FRAC)) & ((1 << (FRAC - 1)) - 1)) as i64);
            return (uf_frac as i32, AMB_EXP);
        }

        (out_frac, out_exp)
    } else {
        // Step 4+5: Far path Barrel shift for alignment
        let far_shift = if exp_diff >= INT_BITS {
            (INT_BITS - 1) as u32
        } else {
            exp_diff as u32
        };

        // Arithmetic right shift with sticky
        let mut shifted = small_ext;
        let mut barrel_sticky = false;
        for i in 0..5u32 {
            if (far_shift >> i) & 1 != 0 {
                let amount = 1u32 << i;
                let lost_mask = (1i64 << amount) - 1;
                barrel_sticky = barrel_sticky || (shifted & lost_mask) != 0;
                shifted = sext(shifted >> amount);
            }
        }

        let far_aligned = shifted;
        let far_sum = sext(big_ext + far_aligned);
        let far_is_zero = far_sum == 0;

        if far_is_zero {
            return (0, AMB_EXP);
        }

        // Bounded normalize (0, 1, or 2 bit shift)
        let bit_n1 = (far_sum >> (INT_BITS - 1)) & 1;
        let bit_n2 = (far_sum >> (INT_BITS - 2)) & 1;
        let bit_n3 = (far_sum >> (INT_BITS - 3)) & 1;
        let far_d0 = bit_n1 != bit_n2;
        let far_d1 = bit_n2 != bit_n3;
        let far_norm_shift: i32 = if far_d0 {
            0
        } else if far_d1 {
            1
        } else {
            2
        };
        let far_leading = far_norm_shift + 1;

        let far_normalized = sext(far_sum << far_norm_shift);

        // Select path
        let normalized = far_normalized;
        let leading = far_leading;

        // Extract fraction
        let out_frac_raw = (normalized >> (INT_BITS - FRAC)) as i32;

        // Rounding bits
        let guard = ((normalized >> (INT_BITS - 1 - FRAC)) & 1) != 0;
        let lsb = ((normalized >> (INT_BITS - FRAC)) & 1) != 0;
        let round_bit = ((normalized >> (INT_BITS - 2 - FRAC)) & 1) != 0;
        let ext_sticky_mask = (1i64 << (INT_BITS - 3 - FRAC)) - 1;
        let ext_sticky = if INT_BITS - 3 - FRAC > 0 {
            (normalized & ext_sticky_mask) != 0
        } else {
            false
        };
        let sticky = ext_sticky || barrel_sticky;
        let round_up = guard && (round_bit || sticky || lsb);

        // Rovf detection
        let pos_all_ones = (out_frac_raw & ((1 << (FRAC - 1)) - 1)) == ((1 << (FRAC - 1)) - 1);
        let rovf_pos = (out_frac_raw >> (FRAC - 1)) & 1 == 0 && pos_all_ones && round_up;

        let rovf_neg = ((out_frac_raw >> (FRAC - 1)) & 1 == 1)
            && ((out_frac_raw >> (FRAC - 2)) & 1 == 0)
            && ((out_frac_raw & ((1 << (FRAC - 2)) - 1)) == (1 << (FRAC - 2)) - 1)
            && round_up;

        let pos_half: i32 = 1 << (FRAC - 2);
        let neg_one: i32 = -(1 << (FRAC - 1));

        let out_frac = if rovf_pos {
            pos_half
        } else if rovf_neg {
            neg_one
        } else {
            out_frac_raw + if round_up { 1 } else { 0 }
        };

        // Exponent: big_exp + 2 - leading + rovf adjustments Far path uses big_exp (not sExpB)
        let exp_wide = (big_exp as i16) + 2 - (leading as i16) + if rovf_pos { 1 } else { 0 }
            - if rovf_neg { 1 } else { 0 };
        let out_exp = exp_wide as i8;

        // Underflow check
        let offset = out_exp.wrapping_sub(1);
        let underflow = big_exp < 0 && offset >= 0;
        if underflow {
            let sign_bit_val = (normalized >> (INT_BITS - 1)) & 1;
            let next_bits = (normalized >> (INT_BITS - FRAC)) & ((1 << (FRAC - 1)) - 1);
            let uf_frac = (sign_bit_val << (FRAC - 1)) | next_bits;
            return (uf_frac as i32, AMB_EXP);
        }

        (out_frac, out_exp)
    }
}

// ── Deterministic LCG RNG ──────────────────────────────────────────────────

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    /// Generate a random normal f32 (no subnormals, no inf/nan, no zero)
    fn next_normal_f32(&mut self) -> f32 {
        loop {
            let bits = self.next();
            let exp = (bits >> 23) & 0xFF;
            if exp != 0 && exp != 255 {
                // Also avoid exponents near boundaries that would overflow Spirix i8 exponent range: keep biased_exp in [2, 253]
                if exp >= 2 && exp <= 253 {
                    return f32::from_bits(bits);
                }
            }
        }
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    const N: usize = 10_000_000;

    println!("Spirix unified adder (FRAC={FRAC}, EXP={EXP}) vs IEEE f32 — {N} trials\n");

    let mut rng = Lcg(0xDEAD_BEEF_CAFE_1234);
    let mut exact = 0u64;
    let mut off_by_1 = 0u64;
    let mut off_by_more = 0u64;
    let mut max_ulp_err: u64 = 0;
    let mut spirix_zero_ieee_not = 0u64;
    let mut ieee_zero_spirix_not = 0u64;
    let mut worst_a = 0.0f32;
    let mut worst_b = 0.0f32;

    for _ in 0..N {
        let a = rng.next_normal_f32();
        let b = rng.next_normal_f32();

        // IEEE reference
        let ieee_result = a + b;

        // Spirix path: convert → add → convert back
        let (a_frac, a_exp) = f32_to_spirix(a);
        let (b_frac, b_exp) = f32_to_spirix(b);
        let (r_frac, r_exp) = spirix_add(a_frac, a_exp, b_frac, b_exp);
        let spirix_result = spirix_to_f32(r_frac, r_exp);

        // Compare
        if ieee_result == 0.0 && spirix_result == 0.0 {
            exact += 1;
            continue;
        }
        if ieee_result == 0.0 {
            spirix_zero_ieee_not += 0;
            ieee_zero_spirix_not += 0;
            // One is zero, other isn't — count as mismatch
            if spirix_result != 0.0 {
                ieee_zero_spirix_not += 1;
            }
            continue;
        }
        if spirix_result == 0.0 && ieee_result != 0.0 {
            spirix_zero_ieee_not += 1;
            continue;
        }

        // ULP comparison
        let ieee_bits = ieee_result.to_bits();
        let spirix_bits = spirix_result.to_bits();

        // Same sign?
        if (ieee_bits ^ spirix_bits) >> 31 != 0 {
            // Different signs — big error
            off_by_more += 1;
            continue;
        }

        let ulp_diff = if ieee_bits > spirix_bits {
            (ieee_bits - spirix_bits) as u64
        } else {
            (spirix_bits - ieee_bits) as u64
        };

        if ulp_diff == 0 {
            exact += 1;
        } else if ulp_diff == 1 {
            off_by_1 += 1;
        } else {
            off_by_more += 1;
            if ulp_diff > max_ulp_err {
                max_ulp_err = ulp_diff;
                worst_a = a;
                worst_b = b;
            }
        }
    }

    let total = exact + off_by_1 + off_by_more + spirix_zero_ieee_not + ieee_zero_spirix_not;

    println!("Results:");
    println!(
        "  Exact match:    {exact:>10} ({:.4}%)",
        exact as f64 / N as f64 * 100.0
    );
    println!(
        "  Off by 1 ULP:   {off_by_1:>10} ({:.4}%)",
        off_by_1 as f64 / N as f64 * 100.0
    );
    println!(
        "  Off by >1 ULP:  {off_by_more:>10} ({:.4}%)",
        off_by_more as f64 / N as f64 * 100.0
    );
    if spirix_zero_ieee_not > 0 {
        println!("  Spirix→0, IEEE≠0: {spirix_zero_ieee_not}");
    }
    if ieee_zero_spirix_not > 0 {
        println!("  IEEE→0, Spirix≠0: {ieee_zero_spirix_not}");
    }
    println!("  Max ULP error:  {max_ulp_err}");
    if max_ulp_err > 1 {
        println!("  Worst case: a={worst_a:e}, b={worst_b:e}");
        let (af, ae) = f32_to_spirix(worst_a);
        let (bf, be) = f32_to_spirix(worst_b);
        let (rf, re) = spirix_add(af, ae, bf, be);
        let ieee = worst_a + worst_b;
        println!(
            "    Spirix: frac={rf}, exp={re} → {}",
            spirix_to_f32(rf, re)
        );
        println!("    IEEE:   {ieee}");
    }
    println!("  Total tested:   {total}");
}
