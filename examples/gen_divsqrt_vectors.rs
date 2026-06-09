/// Generate test vectors for spirix_alu_divsqrt — all 16 frac×exp width combos.
///
/// Output: hex lines "op fw ew a_frac a_exp b_frac b_exp r_frac r_exp" op: 0=DIV, 1=SQRT, 2=MOD Fractions MSB-aligned to 64 bits, exponents LSB-aligned with universal AMBIG.
///
/// Edge case outputs match the HARDWARE convention (UNDEF_GENERAL for non-normal cases that Rust would compute Euclidean), not the Rust model.
use spirix::Scalar;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn msb_frac_64<F: Copy + Into<i64>>(v: F, bits: u32) -> u64 {
    let raw = v.into() as u64;
    let mask = if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    (raw & mask) << (64 - bits)
}

fn lsb_exp_64<E: Copy + Into<i64> + PartialEq>(v: E, min: E) -> u64 {
    if v == min {
        0x8000_0000_0000_0000u64
    } else {
        v.into() as u64
    }
}

macro_rules! gen_width {
    ($f:ty, $e:ty, $fw:expr, $ew:expr, $rng:expr, $count:expr) => {{
        type S = Scalar<$f, $e>;
        let ambig: $e = <$e>::MIN;
        let fbits: u32 = <$f>::BITS;
        let ebits: u32 = <$e>::BITS;
        let shift_to_64: u32 = 64 - fbits;
        let frac_mask_64: u64 = if fbits >= 64 { u64::MAX } else {
            ((1u64 << fbits) - 1) << shift_to_64
        };

        // Hardware edge case constants (prefix in top 8 bits)
        let undef_neg_div_neg: $f = ((0xE9_u64) << (fbits - 8)) as $f;
        let undef_tf_div_tf: $f   = ((0x16_u64) << (fbits - 8)) as $f;
        let undef_general: $f     = ((0xFE_u64) << (fbits - 8)) as $f;
        let undef_sqrt_neg: $f    = ((0xF6_u64) << (fbits - 8)) as $f;
        let undef_sqrt_explod: $f = ((0x08_u64) << (fbits - 8)) as $f;
        let undef_sqrt_vanish: $f = ((0xF7_u64) << (fbits - 8)) as $f;

        let infinity_frac: $f = <$f>::MIN.wrapping_add(<$f>::MAX); // -1 = all ones

        let hw_div = |a: S, b: S| -> S {
            if !a.is_normal() || !b.is_normal() {
                if a.is_undefined() { return a; }
                if b.is_undefined() { return b; }
                if b.is_zero() {
                    return if a.is_zero() {
                        S::new(undef_neg_div_neg, ambig)
                    } else {
                        S::new(infinity_frac, ambig)
                    };
                }
                if a.is_infinite() {
                    return if b.is_infinite() {
                        S::new(undef_tf_div_tf, ambig)
                    } else {
                        S::new(infinity_frac, ambig)
                    };
                }
                if a.is_zero() || b.is_infinite() {
                    return S::ZERO;
                }
                if a.exploded() && b.exploded() {
                    return S::new(undef_tf_div_tf, ambig);
                }
                if a.vanished() && b.vanished() {
                    return S::new(undef_neg_div_neg, ambig);
                }
                // Remaining non-normal → UNDEF_GENERAL (hardware shortcut)
                return S::new(undef_general, ambig);
            }
            // Normal ÷ normal → use Rust division
            a / b
        };

        let hw_sqrt = |a: S| -> S {
            if !a.is_normal() {
                if a.is_undefined() { return a; }
                if a.is_zero() || a.is_infinite() { return a; } // N0 passthrough
                if a.vanished() { return S::new(undef_sqrt_vanish, ambig); }
                // exploded
                return S::new(undef_sqrt_explod, ambig);
            }
            if a.fraction < (0 as $f) {
                return S::new(undef_sqrt_neg, ambig);
            }
            a.sqrt()
        };

        let emit_div = |a: S, b: S| {
            let r = hw_div(a, b);
            println!(
                "0 {} {} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                $fw, $ew,
                msb_frac_64(a.fraction, fbits), lsb_exp_64(a.exponent, ambig),
                msb_frac_64(b.fraction, fbits), lsb_exp_64(b.exponent, ambig),
                msb_frac_64(r.fraction, fbits), lsb_exp_64(r.exponent, ambig),
            );
        };

        let emit_sqrt = |a: S| {
            let r = hw_sqrt(a);
            println!(
                "1 {} {} {:016x} {:016x} 0000000000000000 0000000000000000 {:016x} {:016x}",
                $fw, $ew,
                msb_frac_64(a.fraction, fbits), lsb_exp_64(a.exponent, ambig),
                msb_frac_64(r.fraction, fbits), lsb_exp_64(r.exponent, ambig),
            );
        };

        // Hardware MOD edge cases
        let undef_tf_mod: $f      = ((0x15_u64) << (fbits - 8)) as $f;
        let undef_mod_vanish: $f  = ((0x14_u64) << (fbits - 8)) as $f;
        let undef_mod_explod: $f  = ((0x13_u64) << (fbits - 8)) as $f;

        let hw_mod = |a: S, b: S| -> S {
            if !a.is_normal() || !b.is_normal() {
                if a.is_undefined() { return a; }
                if b.is_undefined() { return b; }
                if a.is_zero() || b.is_zero() { return S::ZERO; }
                if a.is_transfinite() {
                    return S::new(undef_tf_mod, ambig);
                }
                if b.is_infinite() { return a; }
                if b.vanished() {
                    return S::new(undef_mod_vanish, ambig);
                }
                // Remaining non-normal: same signs → passthrough a, diff signs → undef
                if (a.fraction < (0 as $f)) == (b.fraction < (0 as $f)) {
                    return a;
                } else {
                    return S::new(undef_mod_explod, ambig);
                }
            }

            // Normal mod: exact restoring-divider algorithm (matches hardware)
            //
            // All magnitude values use the hardware's 64-bit register layout: abs_X = a_frac[62:0] (63 bits, MSB at bit 62, bit 63 always 0) This mirrors the Verilog's MAG = MAX_FRAC-1 = 63 bit abs extraction.
            let sign_a = a.fraction < (0 as $f);
            let sign_b = b.fraction < (0 as $f);
            let same_sign = sign_a == sign_b;

            let a_is_neg_one = a.fraction == <$f>::MIN;
            let b_is_neg_one = b.fraction == <$f>::MIN;

            // Hardware: abs_a = a_is_neg_one ? POS_HALF[62:0] : (a_frac[63] ? (~a_frac[62:0]+1) : a_frac[62:0]) In u64: bit 63 = 0, magnitude in bits 62:0
            let pos_half_64: u64 = 1u64 << 62; // always 0x4000000000000000
            let neg_one_64: u64 = 1u64 << 63;  // always 0x8000000000000000

            let frac_a_64 = msb_frac_64(a.fraction, fbits);
            let frac_b_64 = msb_frac_64(b.fraction, fbits);

            let abs_a: u64 = if a_is_neg_one {
                pos_half_64
            } else if sign_a {
                let lower63 = frac_a_64 & 0x7FFFFFFFFFFFFFFF;
                ((!lower63) & 0x7FFFFFFFFFFFFFFF).wrapping_add(1) & 0x7FFFFFFFFFFFFFFF
            } else {
                frac_a_64 // bit 63 already 0 for positive N1
            };

            let abs_b: u64 = if b_is_neg_one {
                pos_half_64
            } else if sign_b {
                let lower63 = frac_b_64 & 0x7FFFFFFFFFFFFFFF;
                ((!lower63) & 0x7FFFFFFFFFFFFFFF).wrapping_add(1) & 0x7FFFFFFFFFFFFFFF
            } else {
                frac_b_64
            };

            // Active magnitude mask (top mag_bits of the 63-bit field)
            let mag_bits = fbits - 1;
            let mag_mask: u64 = if mag_bits >= 63 { 0x7FFFFFFFFFFFFFFF } else {
                ((1u64 << mag_bits) - 1) << (63 - mag_bits)
            };

            // Exponent adjustment for NEG_ONE (use i128 to match hardware's 65-bit signed)
            let a_exp_val: i128 = Into::<i64>::into(a.exponent) as i128 + if a_is_neg_one { 1 } else { 0 };
            let b_exp_val: i128 = Into::<i64>::into(b.exponent) as i128 + if b_is_neg_one { 1 } else { 0 };
            let d: i128 = a_exp_val - b_exp_val;

            // Helper: finalize mod result from magnitude (63-bit, MSB at bit 62)
            let exp_max: i128 = (1i128 << (ebits - 1)) - 1;
            let exp_min: i128 = -(1i128 << (ebits - 1)) + 1;
            let finalize_mod = |mod_mag: u64, b_exp: i128, b_is_neg: bool| -> S {
                if mod_mag == 0 {
                    return S::ZERO;
                }
                // CLZ on 63-bit value (bit 62 is MSB) u64 leading_zeros counts from bit 63, so hw_clz = u64_clz - 1
                let rust_clz = mod_mag.leading_zeros();
                let hw_clz = rust_clz - 1; // since bit 63 is always 0
                // Normalize: shift so MSB is at bit 62
                let norm = mod_mag << hw_clz; // bit 62 = 1, bit 63 = 0
                // mod_pos_frac = {1'b0, mod_norm} = norm (bit 63 already 0)
                let pos_frac = norm;

                let frac_val: u64 = if b_is_neg {
                    if pos_frac == pos_half_64 {
                        neg_one_64 // NEG_ONE
                    } else {
                        (!pos_frac).wrapping_add(1) // 64-bit negate
                    }
                } else {
                    pos_frac
                };
                let frac_out = frac_val & frac_mask_64;

                let mut exp_result: i128 = b_exp - hw_clz as i128;
                if b_is_neg && pos_frac == pos_half_64 {
                    exp_result -= 1; // NEG_ONE exp adjustment
                }
                if exp_result > exp_max {
                    // Exp overflow → exploded: masked frac, AMBIG exp
                    return S::new((frac_out >> shift_to_64) as $f, ambig);
                }
                if exp_result < exp_min {
                    // Exp underflow → vanished: arithmetic right shift by 1, then mask Hardware: {frac[63], frac[63:1]} & mask
                    let sign_bit = frac_out >> 63;
                    let small_frac = (sign_bit << 63) | (frac_out >> 1);
                    let small_out = small_frac & frac_mask_64;
                    return S::new((small_out >> shift_to_64) as $f, ambig);
                }
                S::new((frac_out >> shift_to_64) as $f, (exp_result as i64 as u64) as $e)
            };

            if d < 0 {
                if same_sign {
                    return a; // passthrough
                } else {
                    // |b| - (|a| >> |d|), masked to active magnitude bits
                    let shift_amt = (-d).min(63) as u32;
                    let a_shifted = if shift_amt >= 63 { 0u64 } else {
                        (abs_a >> shift_amt) & mag_mask
                    };
                    let mod_mag = abs_b.wrapping_sub(a_shifted);
                    return finalize_mod(mod_mag, b_exp_val, sign_b);
                }
            }

            if d > fbits as i128 {
                // d > active_frac: cannot compute exact remainder
                return S::ZERO;
            }

            // Restoring binary division for d iterations Mirrors hardware: S_IDLE does first trial (unshifted), S_COMPUTE shifts+trials Iteration 0 (S_IDLE): trial on {1'b0, abs_a} vs {2'b00, abs_b}
            let mut r: u64 = abs_a;
            if r >= abs_b {
                r = r.wrapping_sub(abs_b);
            }
            // Iterations 1..d (S_COMPUTE): shift left, trial subtract
            for _ in 0..d as u64 {
                r = r << 1;
                if r >= abs_b {
                    r = r.wrapping_sub(abs_b);
                }
            }

            // r = R_d (Euclidean remainder) Floored-mod sign correction: diff signs and R≠0 → complement
            let mod_mag = if !same_sign && r != 0 {
                abs_b.wrapping_sub(r)
            } else {
                r
            };

            finalize_mod(mod_mag, b_exp_val, sign_b)
        };

        let emit_mod = |a: S, b: S| {
            let r = hw_mod(a, b);
            println!(
                "2 {} {} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}",
                $fw, $ew,
                msb_frac_64(a.fraction, fbits), lsb_exp_64(a.exponent, ambig),
                msb_frac_64(b.fraction, fbits), lsb_exp_64(b.exponent, ambig),
                msb_frac_64(r.fraction, fbits), lsb_exp_64(r.exponent, ambig),
            );
        };

        // Edge values
        let edge_values: Vec<($f, $e)> = vec![
            (0, ambig),                                                         // zero
            (<$f>::MIN.wrapping_add(<$f>::MAX), ambig),                         // infinity (-1)
            ((1 as $f) << (fbits - 2), ambig),                                 // exploded+
            ((((<$f>::MIN) >> 1) as $f).wrapping_add(((<$f>::MIN) >> 2) as $f), ambig), // exploded-
            ((1 as $f) << (fbits - 3), ambig),                                 // vanished+
            ((-1 as $f) << (fbits - 3), ambig),                                // vanished-
            ((1 as $f) << (fbits - 4), ambig),                                 // undefined+
            ((-1 as $f) << (fbits - 4), ambig),                                // undefined-
        ];

        let sample_exps: Vec<$e> = {
            let max_e = <$e>::MAX;
            let min_e = <$e>::MIN.wrapping_add(1 as $e);
            vec![min_e, -2 as $e, -1 as $e, 0 as $e, 1 as $e, 2 as $e,
                 max_e, max_e >> 1, min_e >> 1]
        };

        let sample_fracs: Vec<$f> = vec![
            (1 as $f) << (fbits - 2),              // POS_HALF
            <$f>::MIN,                              // NEG_ONE
            ((1 as $f) << (fbits - 2)) | 1,        // POS_HALF+1
            <$f>::MAX,                              // max positive N1
            (<$f>::MIN >> 1).wrapping_add(1 as $f), // just past N1 boundary neg
        ];

        // ===== DIVISION =====

        // Edge × edge (8×8 = 64)
        for &(af, ae) in &edge_values {
            for &(bf, be) in &edge_values {
                let a = S::new(af, ae);
                let b = S::new(bf, be);
                emit_div(a, b);
                *$count += 1;
            }
        }

        // Edge × normal
        for &(af, ae) in &edge_values {
            for &be in &sample_exps {
                for &bf in &sample_fracs {
                    let a = S::new(af, ae);
                    let b = S::new(bf, be);
                    emit_div(a, b);
                    *$count += 1;
                }
            }
        }

        // Normal × edge
        for &(bf, be) in &edge_values {
            for &ae in &sample_exps {
                for &af in &sample_fracs {
                    let a = S::new(af, ae);
                    let b = S::new(bf, be);
                    emit_div(a, b);
                    *$count += 1;
                }
            }
        }

        // Normal × normal with exponent proximity
        for _ in 0..800 {
            let af: $f = loop {
                let f = $rng.next() as $f;
                if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
            };
            let bf: $f = loop {
                let f = $rng.next() as $f;
                if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
            };
            let ae: $e = loop {
                let e = $rng.next() as $e;
                if e != ambig { break e; }
            };
            let be: $e = match $rng.next() % 4 {
                0 => ae,
                1 => {
                    let d = ($rng.next() % 7) as $e - 3;
                    let e = ae.wrapping_add(d);
                    if e == ambig { ae } else { e }
                }
                2 => loop {
                    let e = $rng.next() as $e;
                    if e != ambig { break e; }
                },
                _ => {
                    let e = ($rng.next() % 5) as $e - 2;
                    if e == ambig { 0 as $e } else { e }
                }
            };

            let a = S::new(af, ae);
            let b = S::new(bf, be);
            emit_div(a, b);
            *$count += 1;
        }

        // Power-of-two boundary pairs
        let po2_fracs: Vec<$f> = vec![
            (1 as $f) << (fbits - 2),
            <$f>::MIN,
            <$f>::MAX,
        ];
        let po2_exps: Vec<$e> = {
            let max_e = <$e>::MAX;
            let min_e = <$e>::MIN.wrapping_add(1 as $e);
            vec![min_e, -1 as $e, 0 as $e, 1 as $e, max_e, max_e >> 1, min_e >> 1]
        };
        for &af in &po2_fracs {
            for &ae in &po2_exps {
                for &bf in &po2_fracs {
                    for &be in &po2_exps {
                        let a = S::new(af, ae);
                        let b = S::new(bf, be);
                        emit_div(a, b);
                        *$count += 1;
                    }
                }
            }
        }

        // ===== SQRT =====

        // Edge values
        for &(af, ae) in &edge_values {
            let a = S::new(af, ae);
            emit_sqrt(a);
            *$count += 1;
        }

        // Sample normal values (positive and negative)
        for &ae in &sample_exps {
            for &af in &sample_fracs {
                let a = S::new(af, ae);
                emit_sqrt(a);
                *$count += 1;
            }
        }

        // Random normal values for sqrt
        for _ in 0..400 {
            let af: $f = loop {
                let f = $rng.next() as $f;
                if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
            };
            let ae: $e = loop {
                let e = $rng.next() as $e;
                if e != ambig { break e; }
            };
            let a = S::new(af, ae);
            emit_sqrt(a);
            *$count += 1;
        }

        // ===== MODULO =====

        // Edge × edge (8×8 = 64)
        for &(af, ae) in &edge_values {
            for &(bf, be) in &edge_values {
                let a = S::new(af, ae);
                let b = S::new(bf, be);
                emit_mod(a, b);
                *$count += 1;
            }
        }

        // Edge × normal
        for &(af, ae) in &edge_values {
            for &be in &sample_exps {
                for &bf in &sample_fracs {
                    let a = S::new(af, ae);
                    let b = S::new(bf, be);
                    emit_mod(a, b);
                    *$count += 1;
                }
            }
        }

        // Normal × edge
        for &(bf, be) in &edge_values {
            for &ae in &sample_exps {
                for &af in &sample_fracs {
                    let a = S::new(af, ae);
                    let b = S::new(bf, be);
                    emit_mod(a, b);
                    *$count += 1;
                }
            }
        }

        // Normal × normal with exponent proximity (mod-specific: focus on d values)
        for _ in 0..800 {
            let af: $f = loop {
                let f = $rng.next() as $f;
                if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
            };
            let bf: $f = loop {
                let f = $rng.next() as $f;
                if ((f >> (fbits - 1)) & 1) != ((f >> (fbits - 2)) & 1) { break f; }
            };
            let ae: $e = loop {
                let e = $rng.next() as $e;
                if e != ambig { break e; }
            };
            // Bias toward small d values (important for mod correctness)
            let be: $e = match $rng.next() % 6 {
                0 => ae,                           // d=0
                1 => {                             // d=1..3
                    let d = ($rng.next() % 3) as $e + 1;
                    let e = ae.wrapping_sub(d);
                    if e == ambig { ae } else { e }
                }
                2 => {                             // d=-1..-3 (|a|<|b|)
                    let d = ($rng.next() % 3) as $e + 1;
                    let e = ae.wrapping_add(d);
                    if e == ambig { ae } else { e }
                }
                3 => {                             // d near frac boundary
                    let fbits_e = fbits as $e;
                    let d = ($rng.next() % 5) as $e;
                    let e = ae.wrapping_sub(fbits_e).wrapping_add(d);
                    if e == ambig { ae } else { e }
                }
                4 => loop {                        // random exp
                    let e = $rng.next() as $e;
                    if e != ambig { break e; }
                },
                _ => {                             // small exp
                    let e = ($rng.next() % 5) as $e - 2;
                    if e == ambig { 0 as $e } else { e }
                }
            };

            let a = S::new(af, ae);
            let b = S::new(bf, be);
            emit_mod(a, b);
            *$count += 1;
        }

        // Same-sign and diff-sign pairs (important for floored mod correction) Only N1 fractions: positive 01xxxxxx, negative 10xxxxxx
        let sign_fracs: Vec<$f> = vec![
            (1 as $f) << (fbits - 2),              // POS_HALF = 0.5 (+)
            <$f>::MIN,                              // NEG_ONE = -1.0 (-)
            <$f>::MAX,                              // max positive N1 = 0.9921... (+)
            (<$f>::MIN >> 1),                       // most negative N1 = -0.5 (10...0) (-)
        ];
        let sign_exps: Vec<$e> = vec![0 as $e, 1 as $e, 2 as $e, -1 as $e, -2 as $e];
        for &af in &sign_fracs {
            for &ae in &sign_exps {
                for &bf in &sign_fracs {
                    for &be in &sign_exps {
                        let a = S::new(af, ae);
                        let b = S::new(bf, be);
                        emit_mod(a, b);
                        *$count += 1;
                    }
                }
            }
        }
    }};
}

fn main() {
    let mut rng = Rng(0xD1A5_5047_0001u64.wrapping_mul(0xCAFEBABE));
    let mut total = 0u64;

    let mut w_count = 0u64;
    gen_width!(i8, i8, 0, 0, &mut rng, &mut w_count);
    let c00 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i8, 1, 0, &mut rng, &mut w_count);
    let c10 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i8, 2, 0, &mut rng, &mut w_count);
    let c20 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i8, 3, 0, &mut rng, &mut w_count);
    let c30 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i16, 0, 1, &mut rng, &mut w_count);
    let c01 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i16, 1, 1, &mut rng, &mut w_count);
    let c11 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i16, 2, 1, &mut rng, &mut w_count);
    let c21 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i16, 3, 1, &mut rng, &mut w_count);
    let c31 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i32, 0, 2, &mut rng, &mut w_count);
    let c02 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i32, 1, 2, &mut rng, &mut w_count);
    let c12 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i32, 2, 2, &mut rng, &mut w_count);
    let c22 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i32, 3, 2, &mut rng, &mut w_count);
    let c32 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i8, i64, 0, 3, &mut rng, &mut w_count);
    let c03 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i16, i64, 1, 3, &mut rng, &mut w_count);
    let c13 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i32, i64, 2, 3, &mut rng, &mut w_count);
    let c23 = w_count;
    total += w_count;
    w_count = 0;
    gen_width!(i64, i64, 3, 3, &mut rng, &mut w_count);
    let c33 = w_count;
    total += w_count;

    eprintln!("Generated {} total vectors across 16 width combos", total);
    eprintln!("  F3E3={} F4E3={} F5E3={} F6E3={}", c00, c10, c20, c30);
    eprintln!("  F3E4={} F4E4={} F5E4={} F6E4={}", c01, c11, c21, c31);
    eprintln!("  F3E5={} F4E5={} F5E5={} F6E5={}", c02, c12, c22, c32);
    eprintln!("  F3E6={} F4E6={} F5E6={} F6E6={}", c03, c13, c23, c33);
}
