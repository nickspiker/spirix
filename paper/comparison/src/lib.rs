//! Spirix vs IEEE 754 binary32 add/sub comparison support.
//!
//! Spirix N0 at FRAC=24, EXP=8 has the same effective precision as binary32
//! (24 bits vs binary32's 23+1 implicit). The comparison gold model is direct
//! IEEE arithmetic, with state mapping between IEEE specials and Spirix
//! specials applied at the boundary.
//!
//! Exponent encoding (AMBIG = 0 convention, matches patent line 252):
//!   The exponent field is an unsigned modular integer in Z/2^EXP Z. The
//!   stored bit pattern 0 = AMBIGUOUS_EXPONENT (sentinel for non-normal
//!   states). Stored values 1..2^EXP-1 represent normal binades arranged
//!   monotonically around the cycle. With bias = 2^(EXP-1) - 1 = 127 for
//!   8-bit exponents, the unit binades containing +1.0 and -1.0 sit at
//!   stored = 0x80 and 0x7F respectively, straddling the cycle's middle.
//!   Both overflow past stored=255 and underflow past stored=1 reach
//!   AMBIG=0 by opposite traversals of the modular cycle, collapsing
//!   saturation detection to one equality check against zero.
//!
//! Value formula (Normal):
//!   value = compute_q * 2^(internal_exp - FRAC)
//!   where internal_exp = stored_exp - BIAS, compute_q = inflate(storage).
//!
//! State mapping IEEE → Spirix:
//!   IEEE  NaN          → Spirix Undefined (canonical pattern)
//!   IEEE  ±Infinity    → Spirix Exploded (sign-preserved)
//!   IEEE  ±0           → Spirix Zero (signless)
//!   IEEE  subnormal    → Spirix Vanished (new spirix internal_exp range
//!                        [-126, 128] is too small to hold f32 subnormals;
//!                        the smallest f32 normal at 2^-126 maps to spirix
//!                        stored=1 = internal_exp -126, exactly at the
//!                        boundary).
//!   IEEE  normal       → Spirix Normal at (frac=24, exp=8) if in range,
//!                        else Exploded (sign-preserved).

pub const FRAC: u32 = 24;
pub const EXP_BITS: u32 = 8;

// Unsigned modular exponent. AMBIG = 0 is the sentinel; stored values 1..255
// are normal binades. Overflow past 255 and underflow past 1 both wrap to 0
// via natural unsigned modular arithmetic.
pub const AMBIG_EXP: u8 = 0;
pub const MIN_EXP: u8 = 1;
pub const MAX_EXP: u8 = 255;

// Exponent bias. Internal (true) exponent = stored - BIAS. With BIAS=127:
// stored=0x80 → internal=1 (binade containing +1.0); stored=0x7F → internal=0
// (binade containing -1.0 via N0 canonical form).
pub const BIAS: i32 = 127;

/// N0 storage boundary patterns (24-bit).
pub const POS_ONE_NORMAL: u32   = 0x80_0000;
pub const NEG_ONE_NORMAL: u32   = 0x00_0000;
pub const POS_ONE_EXPLODED: u32 = 0x40_0000;
pub const NEG_ONE_EXPLODED: u32 = 0x80_0000; // same bit pattern as POS_ONE_NORMAL,
                                              // distinguished by AMBIG_EXP
pub const POS_ONE_VANISHED: u32 = 0x20_0000;
pub const NEG_ONE_VANISHED: u32 = 0xC0_0000;

/// Canonical "undefined" storage used by the gold model when IEEE produces
/// NaN. The DUT will produce its own cause-encoding (UNDEF_TF_P_FIN etc.);
/// matching is done by classifying the *state*, not the exact bit pattern.
pub const UNDEF_CANONICAL: u32 = 0x10_0000; // top 3 bits 0_0_0_1 → LSBC ≥ 3 → Undefined

/// Spirix state classification for a (storage, exp) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpirixState {
    Normal,
    Zero,
    PosVanished,
    NegVanished,
    PosExploded,
    NegExploded,
    Infinity,
    Undefined,
}

pub fn classify(storage: u32, exp: u8) -> SpirixState {
    let storage = storage & 0x00FF_FFFF;
    if exp != AMBIG_EXP {
        return SpirixState::Normal;
    }
    if storage == 0 {
        return SpirixState::Zero;
    }
    if storage == 0x00FF_FFFF {
        return SpirixState::Infinity;
    }
    let lsbc = leading_same_count(storage);
    let msb = (storage >> (FRAC - 1)) & 1;
    match lsbc {
        1 => if msb == 0 { SpirixState::PosExploded } else { SpirixState::NegExploded },
        2 => if msb == 0 { SpirixState::PosVanished } else { SpirixState::NegVanished },
        _ => SpirixState::Undefined,
    }
}

fn leading_same_count(storage: u32) -> u32 {
    let storage = storage & 0x00FF_FFFF;
    let msb = (storage >> (FRAC - 1)) & 1;
    let mut count = 1;
    for i in (0..FRAC - 1).rev() {
        if ((storage >> i) & 1) == msb {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// Convert IEEE binary32 → Spirix N0 (storage 24-bit, exp 8-bit).
/// Returns the (storage, exp) pair plus the classified state of the result.
pub fn f32_to_spirix(v: f32) -> (u32, u8, SpirixState) {
    let bits = v.to_bits();
    let sign = (bits >> 31) & 1;
    let biased_exp = (bits >> 23) & 0xFF;
    let mantissa = bits & 0x007F_FFFF;

    // NaN
    if biased_exp == 0xFF && mantissa != 0 {
        return (UNDEF_CANONICAL, AMBIG_EXP, SpirixState::Undefined);
    }
    // ±Inf
    if biased_exp == 0xFF {
        let s = if sign == 0 { POS_ONE_EXPLODED } else { NEG_ONE_EXPLODED };
        let st = if sign == 0 { SpirixState::PosExploded } else { SpirixState::NegExploded };
        return (s, AMBIG_EXP, st);
    }
    // ±0
    if biased_exp == 0 && mantissa == 0 {
        return (0, AMBIG_EXP, SpirixState::Zero);
    }
    // Unified normalize for normals and subnormals.
    let abs_mantissa: u32 = if biased_exp == 0 { mantissa } else { mantissa | (1 << 23) };
    let significant = 32 - abs_mantissa.leading_zeros() as i32;
    let eff_exp: i32 = if biased_exp == 0 { 1 } else { biased_exp as i32 };
    let mut internal_exp = eff_exp + significant - 150;

    // Shift leading 1 to bit 23 (FRAC-1).
    let shift = 24 - significant;
    let compute_q_pos: u32 = if shift >= 0 {
        abs_mantissa << shift
    } else {
        abs_mantissa >> (-shift)
    };

    // Negative boundary: -1/2 × 2^exp canonicalizes to NEG_ONE_NORMAL × 2^(exp-1).
    let neg_boundary = sign == 1 && compute_q_pos == POS_ONE_NORMAL;
    if neg_boundary {
        internal_exp -= 1;
    }

    // Saturate against Spirix stored exp range [1, 255]. stored = internal + BIAS.
    let stored = internal_exp + BIAS;
    if stored > MAX_EXP as i32 {
        let s = if sign == 0 { POS_ONE_EXPLODED } else { NEG_ONE_EXPLODED };
        let st = if sign == 0 { SpirixState::PosExploded } else { SpirixState::NegExploded };
        return (s, AMBIG_EXP, st);
    }
    if stored < MIN_EXP as i32 {
        let s = if sign == 0 { POS_ONE_VANISHED } else { NEG_ONE_VANISHED };
        let st = if sign == 0 { SpirixState::PosVanished } else { SpirixState::NegVanished };
        return (s, AMBIG_EXP, st);
    }

    let storage = if neg_boundary {
        NEG_ONE_NORMAL
    } else if sign == 0 {
        compute_q_pos
    } else {
        (1u32 << 24).wrapping_sub(compute_q_pos) & 0x00FF_FFFF
    };
    (storage, stored as u8, SpirixState::Normal)
}

/// Convert Spirix N0 (storage, exp) → IEEE binary64 (lossless for Normal).
pub fn spirix_to_f64(storage: u32, exp: u8) -> f64 {
    let storage = storage & 0x00FF_FFFF;
    match classify(storage, exp) {
        SpirixState::Zero => 0.0,
        SpirixState::Infinity => f64::INFINITY,
        SpirixState::PosExploded => f64::INFINITY,
        SpirixState::NegExploded => f64::NEG_INFINITY,
        SpirixState::PosVanished => f64::from_bits(1), // smallest positive denormal
        SpirixState::NegVanished => f64::from_bits(1 | (1u64 << 63)),
        SpirixState::Undefined => f64::NAN,
        SpirixState::Normal => {
            let inflated_high = (!(storage >> 23) & 1) << 24;
            let compute_q_unsigned = inflated_high | storage;
            let compute_q = if (compute_q_unsigned >> 24) & 1 != 0 {
                (compute_q_unsigned | 0xFE00_0000) as i32
            } else {
                compute_q_unsigned as i32
            };
            let internal_exp = exp as i32 - BIAS;
            (compute_q as f64) * (internal_exp as f64 - FRAC as f64).exp2()
        }
    }
}

/// Convert IEEE f64 → Spirix N0, banker's rounding to 24-bit precision.
pub fn f64_to_spirix(v: f64) -> (u32, u8, SpirixState) {
    if v.is_nan() {
        return (UNDEF_CANONICAL, AMBIG_EXP, SpirixState::Undefined);
    }
    if v.is_infinite() {
        let s = if v > 0.0 { POS_ONE_EXPLODED } else { NEG_ONE_EXPLODED };
        let st = if v > 0.0 { SpirixState::PosExploded } else { SpirixState::NegExploded };
        return (s, AMBIG_EXP, st);
    }
    if v == 0.0 {
        return (0, AMBIG_EXP, SpirixState::Zero);
    }
    let bits = v.to_bits();
    let sign = (bits >> 63) & 1;
    let biased_exp = ((bits >> 52) & 0x7FF) as i32;
    let mantissa_52 = bits & 0x000F_FFFF_FFFF_FFFF;

    let (effective_mantissa_53, true_exp): (u64, i32) = if biased_exp == 0 {
        if mantissa_52 == 0 { return (0, AMBIG_EXP, SpirixState::Zero); }
        let lz = mantissa_52.leading_zeros() as i32 - (64 - 53);
        let m_normalized = (mantissa_52 << (lz + 1)) & 0x001F_FFFF_FFFF_FFFF;
        let m_53 = (1u64 << 52) | m_normalized;
        let exp = -1022 - lz;
        (m_53, exp)
    } else {
        let m_53 = (1u64 << 52) | mantissa_52;
        let exp = biased_exp - 1023;
        (m_53, exp)
    };

    let compute_q_mag_floor = (effective_mantissa_53 >> 29) as u32;
    let guard = ((effective_mantissa_53 >> 28) & 1) != 0;
    let round_bit = ((effective_mantissa_53 >> 27) & 1) != 0;
    let sticky = (effective_mantissa_53 & ((1u64 << 27) - 1)) != 0;
    let lsb = (compute_q_mag_floor & 1) != 0;
    let round_up = guard && (round_bit | sticky | lsb);

    let mut compute_q_mag = compute_q_mag_floor + (round_up as u32);
    let mut internal_exp = true_exp + 1;

    if compute_q_mag >= (1u32 << 24) {
        compute_q_mag >>= 1;
        internal_exp += 1;
    }

    // Negative-boundary canonicalization (must run BEFORE saturation).
    if sign == 1 && compute_q_mag == (1u32 << 23) {
        let canon_internal = internal_exp - 1;
        let canon_stored = canon_internal + BIAS;
        if canon_stored < MIN_EXP as i32 {
            return (NEG_ONE_VANISHED, AMBIG_EXP, SpirixState::NegVanished);
        }
        if canon_stored > MAX_EXP as i32 {
            return (NEG_ONE_EXPLODED, AMBIG_EXP, SpirixState::NegExploded);
        }
        return (NEG_ONE_NORMAL, canon_stored as u8, SpirixState::Normal);
    }

    let stored = internal_exp + BIAS;
    if stored > MAX_EXP as i32 {
        let s = if sign == 0 { POS_ONE_EXPLODED } else { NEG_ONE_EXPLODED };
        let st = if sign == 0 { SpirixState::PosExploded } else { SpirixState::NegExploded };
        return (s, AMBIG_EXP, st);
    }
    if stored < MIN_EXP as i32 {
        let s = if sign == 0 { POS_ONE_VANISHED } else { NEG_ONE_VANISHED };
        let st = if sign == 0 { SpirixState::PosVanished } else { SpirixState::NegVanished };
        return (s, AMBIG_EXP, st);
    }

    let storage = if sign == 0 {
        compute_q_mag
    } else {
        (1u32 << 24).wrapping_sub(compute_q_mag) & 0x00FF_FFFF
    };

    (storage, stored as u8, SpirixState::Normal)
}

/// "Equal under Spirix semantics".
pub fn spirix_eq(a_storage: u32, a_exp: u8, b_storage: u32, b_exp: u8) -> bool {
    let a_state = classify(a_storage, a_exp);
    let b_state = classify(b_storage, b_exp);
    if a_state == SpirixState::Undefined && b_state == SpirixState::Undefined {
        return true;
    }
    let a_storage = a_storage & 0x00FF_FFFF;
    let b_storage = b_storage & 0x00FF_FFFF;
    a_state == b_state && a_storage == b_storage && a_exp == b_exp
}

/// Simple LFSR-based pseudo-random generator for reproducible test vectors.
pub struct Lfsr64(pub u64);
impl Lfsr64 {
    pub fn new(seed: u64) -> Self { Self(if seed == 0 { 1 } else { seed }) }
    pub fn next(&mut self) -> u64 {
        let lsb = self.0 & 1;
        self.0 >>= 1;
        if lsb != 0 { self.0 ^= 0xD800_0000_0000_0000; }
        self.0
    }
    pub fn next_f32(&mut self) -> f32 { f32::from_bits(self.next() as u32) }
}
