/// Bit-accurate Rust port of spirix_fma.v (FRAC=25, EXP=8)
/// compared against f64 reference (exact for f32 operands).
///
/// Run with:  cargo run --example fma_f32 --release

const FRAC: i32 = 25;
const PROD_BITS: i32 = 2 * FRAC - 1; // 49
const INT_BITS: i32 = PROD_BITS + 3; // 52
const AMB_EXP: i8 = i8::MIN; // -128

// ── f32 ↔ Spirix conversion ────────────────────────────────────────────────

fn f32_to_spirix(v: f32) -> (i32, i8) {
    let bits = v.to_bits();
    let sign = (bits >> 31) != 0;
    let biased_exp = ((bits >> 23) & 0xFF) as i32;
    let mantissa = (bits & 0x7F_FFFF) as i32;

    if biased_exp == 0 || biased_exp == 255 {
        return (0, AMB_EXP);
    }

    let sig = (1 << 23) | mantissa;
    let frac: i32 = if sign { -sig } else { sig };
    let exp = (biased_exp - 126) as i8;
    (frac, exp)
}

fn spirix_to_f32(frac: i32, exp: i8) -> f32 {
    if frac == 0 || exp == AMB_EXP {
        return 0.0;
    }

    let sign = frac < 0;
    let mag = if sign { (-frac) as u32 } else { frac as u32 };
    let lz = mag.leading_zeros();
    let shift = lz as i32 - 8;

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
        if biased_exp >= 255 {
            return if sign {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            };
        }
        return 0.0;
    }

    let bits = ((sign as u32) << 31) | (biased_exp << 23) | mantissa;
    f32::from_bits(bits)
}

// ── Bit-accurate port of spirix_fma.v ──────────────────────────────────────

fn spirix_fma(
    a_frac: i32,
    a_exp: i8,
    b_frac: i32,
    b_exp: i8,
    c_frac: i32,
    c_exp: i8,
    sub: bool,
) -> (i32, i8) {
    // Use i128 for wide intermediates (INT_BITS=52 fits comfortably)
    let mask = (1i128 << INT_BITS) - 1;
    let sign_bit = 1i128 << (INT_BITS - 1);
    let sext = |v: i128| -> i128 {
        let v = v & mask;
        if v & sign_bit != 0 {
            v | !mask
        } else {
            v
        }
    };

    // Step 1: DSP multiply
    let product = (a_frac as i64) * (b_frac as i64); // PROD_BITS wide

    // Bounded normalize (0-1 bit)
    let prod_is_n1 = ((product >> (PROD_BITS - 1)) & 1) != ((product >> (PROD_BITS - 2)) & 1);
    let norm_shift: i32 = if prod_is_n1 { 0 } else { 1 };
    let prod_normalized = if norm_shift != 0 {
        product << 1
    } else {
        product
    };

    // Zero product when either input is ambiguous
    let prod_is_zero = a_exp == AMB_EXP || b_exp == AMB_EXP;
    let prod_safe = if prod_is_zero { 0i64 } else { prod_normalized };

    // Product exponent (parallel with DSP)
    let prod_exp_raw = (a_exp as i16) + (b_exp as i16);
    let prod_exp = prod_exp_raw - (norm_shift as i16);
    let prod_exp_safe: i16 = if prod_is_zero {
        i8::MIN as i16
    } else {
        prod_exp
    };

    // Widen c to PROD_BITS: c_frac in upper bits, zeros below
    let c_wide = (c_frac as i64) << (FRAC - 1);

    // Exponent difference + swap
    let raw_diff: i32 = (prod_exp_safe as i32) - (c_exp as i32);
    let prod_is_big = raw_diff >= 0;

    let (big_frac, big_exp, small_frac) = if prod_is_big {
        (prod_safe as i128, prod_exp_safe, c_wide as i128)
    } else {
        (c_wide as i128, c_exp as i16, prod_safe as i128)
    };

    let exp_diff = raw_diff.unsigned_abs() as i32;
    let negligible = exp_diff >= PROD_BITS;

    // Subtraction: negate c operand when sub=true
    let negate_small = sub && prod_is_big;
    let negate_big = sub && !prod_is_big;

    // Negligible bypass — only when c is big (already FRAC-wide, no rounding needed).
    // When product is big, it needs rounding from PROD_BITS to FRAC, so let the
    // far path handle it.
    if negligible && !negate_big && !prod_is_big {
        return (c_frac, big_exp as i8);
    }

    let is_close = exp_diff <= 1 && !negligible;

    // Extend to internal width
    let big_ext = sext(big_frac << 2);
    let small_ext = sext(small_frac << 2);

    // Apply subtraction via one's complement + carry-in
    let negate_b = if negate_big { mask } else { 0 };
    let negate_s = if negate_small { mask } else { 0 };
    let carry_in: i128 = if sub { 1 } else { 0 };

    if is_close {
        // Close path
        let close_small = if (exp_diff & 1) != 0 {
            sext(small_ext >> 1)
        } else {
            small_ext
        };
        let close_align_sticky = ((exp_diff & 1) != 0) && ((small_ext & 1) != 0);
        let close_sum = sext((big_ext ^ negate_b) + (close_small ^ negate_s) + carry_in);
        let close_is_zero = close_sum == 0;

        if close_is_zero {
            return (0, AMB_EXP);
        }

        // CLZ via XOR-adjacent priority encode
        let ubits = (close_sum & mask) as u64;
        let top_bit = (close_sum >> (INT_BITS - 1)) & 1;
        let leading = if top_bit != 0 {
            let inverted = (!ubits) & (mask as u64);
            inverted.leading_zeros() as i32 - (64 - INT_BITS)
        } else {
            ubits.leading_zeros() as i32 - (64 - INT_BITS)
        };
        let close_norm_shift = leading - 1;
        let close_normalized = sext(close_sum << close_norm_shift);

        return round_and_output(
            close_normalized,
            leading,
            false,
            close_align_sticky,
            big_exp,
            sext,
            mask,
        );
    }

    // Far path
    let far_shift = if exp_diff >= INT_BITS {
        INT_BITS - 1
    } else {
        exp_diff
    };

    // Barrel shift with sticky
    let mut shifted = small_ext;
    let mut barrel_sticky = false;
    for i in 0..6u32 {
        if ((far_shift as u32) >> i) & 1 != 0 {
            let amount = 1u32 << i;
            let lost_mask = (1i128 << amount) - 1;
            barrel_sticky = barrel_sticky || (shifted & lost_mask) != 0;
            shifted = sext(shifted >> amount);
        }
    }

    let far_sum = sext((big_ext ^ negate_b) + (shifted ^ negate_s) + carry_in);
    let far_is_zero = far_sum == 0;

    if far_is_zero {
        return (0, AMB_EXP);
    }

    // Bounded normalize (0-2 bit shift)
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

    round_and_output(
        far_normalized,
        far_leading,
        barrel_sticky,
        false,
        big_exp,
        sext,
        mask,
    )
}

fn round_and_output(
    normalized: i128,
    leading: i32,
    barrel_sticky: bool,
    close_align_sticky: bool,
    big_exp: i16,
    sext: impl Fn(i128) -> i128,
    _mask: i128,
) -> (i32, i8) {
    let _ = sext; // used implicitly via normalized already being sign-extended

    // Extract FRAC_BITS fraction from top of normalized
    let out_frac_raw = (normalized >> (INT_BITS - FRAC)) as i32;

    // Rounding bits
    let guard = ((normalized >> (INT_BITS - 1 - FRAC)) & 1) != 0;
    let lsb = ((normalized >> (INT_BITS - FRAC)) & 1) != 0;
    let round_bit = ((normalized >> (INT_BITS - 2 - FRAC)) & 1) != 0;
    let ext_sticky = if INT_BITS - 3 - FRAC > 0 {
        let ext_sticky_mask = (1i128 << (INT_BITS - 3 - FRAC)) - 1;
        (normalized & ext_sticky_mask) != 0
    } else {
        false
    };
    let sticky = ext_sticky || barrel_sticky || close_align_sticky;
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

    // Exponent: big_exp + 2 - leading + rovf
    // The +2 comes from the <<< 2 extension, same as addsub.
    // Width of INT_BITS doesn't matter — we always extract top FRAC_BITS.
    let exp_wide = (big_exp as i32) + 2 - (leading as i32) + if rovf_pos { 1 } else { 0 }
        - if rovf_neg { 1 } else { 0 };
    let out_exp = exp_wide as i8;

    // Underflow check
    let offset = out_exp.wrapping_sub(1);
    let underflow = (big_exp as i8) < 0 && offset >= 0;
    if underflow {
        let sign_bit_val = ((normalized >> (INT_BITS - 1)) & 1) as i32;
        let next_bits = ((normalized >> (INT_BITS - FRAC)) & ((1 << (FRAC - 1)) - 1)) as i32;
        let uf_frac = (sign_bit_val << (FRAC - 1)) | next_bits;
        return (uf_frac, AMB_EXP);
    }

    (out_frac, out_exp)
}

// ── RNG ────────────────────────────────────────────────────────────────────

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn next_normal_f32(&mut self) -> f32 {
        loop {
            let bits = self.next();
            let exp = (bits >> 23) & 0xFF;
            if exp >= 2 && exp <= 253 {
                return f32::from_bits(bits);
            }
        }
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    const N: usize = 10_000_000;
    println!("Spirix FMA (FRAC={FRAC}, EXP=8) vs f64 reference — {N} trials\n");

    let mut rng = Lcg(0xDEAD_BEEF_CAFE_1234);
    let mut exact = 0u64;
    let mut off_by_1 = 0u64;
    let mut off_by_more = 0u64;
    let mut max_ulp_err: u64 = 0;
    let mut zero_mismatches = 0u64;
    let mut sign_mismatches = 0u64;
    let mut inf_mismatches = 0u64;

    for _ in 0..N {
        let a = rng.next_normal_f32();
        let b = rng.next_normal_f32();
        let c = rng.next_normal_f32();
        let sub = (rng.next() & 1) != 0;

        // f64 reference (exact for f32 operands, single rounding at f32 conversion)
        let ref_f64 = if sub {
            (a as f64) * (b as f64) - (c as f64)
        } else {
            (a as f64) * (b as f64) + (c as f64)
        };
        let ref_f32 = ref_f64 as f32;

        // Spirix FMA
        let (af, ae) = f32_to_spirix(a);
        let (bf, be) = f32_to_spirix(b);
        let (cf, ce) = f32_to_spirix(c);
        let (rf, re) = spirix_fma(af, ae, bf, be, cf, ce, sub);
        let spirix_result = spirix_to_f32(rf, re);

        // Compare
        if ref_f32 == 0.0 && spirix_result == 0.0 {
            exact += 1;
            continue;
        }
        if ref_f32 == 0.0 || spirix_result == 0.0 {
            zero_mismatches += 1;
            continue;
        }
        if ref_f32.is_infinite() || spirix_result.is_infinite() {
            if ref_f32 == spirix_result {
                exact += 1;
            } else {
                inf_mismatches += 1;
            }
            continue;
        }

        let ref_bits = ref_f32.to_bits();
        let spx_bits = spirix_result.to_bits();

        if (ref_bits ^ spx_bits) >> 31 != 0 {
            sign_mismatches += 1;
            if sign_mismatches <= 3 {
                let op = if sub { "-" } else { "+" };
                println!("SIGN MISMATCH: a={a:e} b={b:e} c={c:e} op={op}");
                println!("  ref={ref_f32:e} (bits={ref_bits:#010x}), spirix={spirix_result:e} (bits={spx_bits:#010x})");
                println!("  fma_out: frac={rf} exp={re}");
            }
            continue;
        }

        let ulp_diff = ref_bits.abs_diff(spx_bits) as u64;
        match ulp_diff {
            0 => exact += 1,
            1 => off_by_1 += 1,
            _ => {
                off_by_more += 1;
                if ulp_diff > max_ulp_err {
                    max_ulp_err = ulp_diff;
                }
                if off_by_more <= 5 {
                    let op = if sub { "-" } else { "+" };
                    println!("MISMATCH: a={a:e} b={b:e} c={c:e} op={op}");
                    println!("  a_spirix: frac={af} exp={ae}");
                    println!("  b_spirix: frac={bf} exp={be}");
                    println!("  c_spirix: frac={cf} exp={ce}");
                    println!("  fma_out:  frac={rf} exp={re} → {spirix_result:e}");
                    println!("  ref_f64:  {ref_f64:e} → f32={ref_f32:e}");
                    println!("  ulp_diff: {ulp_diff}");
                }
            }
        }
    }

    let total = exact + off_by_1 + off_by_more + zero_mismatches;
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
    if zero_mismatches > 0 {
        println!("  Zero mismatches: {zero_mismatches}");
    }
    println!("  Sign mismatch:  {sign_mismatches:>10}");
    println!("  Inf mismatch:   {inf_mismatches:>10}");
    println!("  Max ULP error:  {max_ulp_err}");
    println!("  Total tested:   {total}");
}
