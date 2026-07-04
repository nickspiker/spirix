use crate::core::integer::*;
use crate::core::undefined::*;
use crate::{Integer, Scalar, ScalarConstants};
use core::ops::*;
use i256::I256;
use num_traits::{AsPrimitive, WrappingAdd, WrappingMul, WrappingNeg, WrappingSub};
#[allow(private_bounds)]
impl<
        F: Integer
            + FullInt
            + Shl<isize, Output = F>
            + Shr<isize, Output = F>
            + Shl<F, Output = F>
            + Shr<F, Output = F>
            + Shl<E, Output = F>
            + Shr<E, Output = F>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
        E: Integer
            + FullInt
            + Shl<isize, Output = E>
            + Shr<isize, Output = E>
            + Shl<E, Output = E>
            + Shr<E, Output = E>
            + Shl<F, Output = E>
            + Shr<F, Output = E>
            + WrappingNeg
            + WrappingAdd
            + WrappingMul
            + WrappingSub,
    > Scalar<F, E>
where
    Scalar<F, E>: ScalarConstants,
    u8: AsPrimitive<F>,
    u16: AsPrimitive<F>,
    u32: AsPrimitive<F>,
    u64: AsPrimitive<F>,
    u128: AsPrimitive<F>,
    usize: AsPrimitive<F>,
    i8: AsPrimitive<F>,
    i16: AsPrimitive<F>,
    i32: AsPrimitive<F>,
    i64: AsPrimitive<F>,
    i128: AsPrimitive<F>,
    isize: AsPrimitive<F>,
    I256: From<F>,
    u8: AsPrimitive<E>,
    u16: AsPrimitive<E>,
    u32: AsPrimitive<E>,
    u64: AsPrimitive<E>,
    u128: AsPrimitive<E>,
    usize: AsPrimitive<E>,
    i8: AsPrimitive<E>,
    i16: AsPrimitive<E>,
    i32: AsPrimitive<E>,
    i64: AsPrimitive<E>,
    i128: AsPrimitive<E>,
    isize: AsPrimitive<E>,
    I256: From<E>,
{
    /// Multiplies this Scalar by another Scalar
    ///
    /// # Description
    ///
    /// Performs multiplication between two Scalars, handling special cases according to mathematical principles. For normal values, this produces the expected mathematical product. Special states follow special rules to maintain mathematical continuity even when results exceed representable ranges.
    ///
    /// Multiplication process:
    /// 0. Checks for escaped values (undefined, exploded, vanished) and applies special case handling
    /// 1. Uses wider integer types for fraction multiplication as product lands in the high half
    /// ```txt
    ///    □□□□□□□ ■■■■■■■ = Multiplier □□□□□□□ ■■■■■■■ = Multiplicand ⤪⤪⤪⤪⤪⤪      Multiply! ■■■■■■■ □□□□□□□ = Intermediate 2x-bit product space ↘↘↘↘↘↘↘ ■■■■■■■ = High half kept, low bits discarded to floor
    /// 2. Calculates leading Zeros/Ones to determine normalization shift and exponent nudge
    /// 3. Adds exponents and adjusts by normalization shift (exponent_result = self.exponent + other.exponent - shift)
    /// 4. Handles special cases where exponent exceeds MAX_EXPONENT (explode) or falls below MIN_EXPONENT (vanish)
    /// 5. For escaped values, preserves sign and phase information while following mathematical convention
    ///
    /// # Returns
    ///
    /// - `[℘ ]` ➔ `[℘ ]` First undefined state encountered
    /// - `[0]` × `[#]` or `[#]` × `[0]` ➔ `[0]` Zero (multiplicative annihilation)
    /// - `[↑]` × `[↓]` ➔ `[℘ ↑×↓]` Undefined state (magnitude indeterminate)
    /// - `[↓]` × `[↑]` ➔ `[℘ ↓×↑]` Undefined state (magnitude indeterminate)
    /// - `[↑]` × `[#]` or `[#]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[#]` or `[#]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[↑]` × `[↑]` ➔ `[↑]` Exploded with sign/phase following multiplication rule
    /// - `[↓]` × `[↓]` ➔ `[↓]` Vanished with sign/phase following multiplication rule
    /// - `[#]` × `[#]` ➔ `[#]` or `[↑]` or `[↓]` A finite, exploded or vanished Scalar
    ///
    /// # Examples
    ///
    /// ```rust
    /// use spirix::{Scalar, ScalarF6E4};
    ///
    /// // Multiplying finite Scalars let eight = Scalar::<i64, i16>::from(8); let eigth = ScalarF6E4::ONE / 8; assert!(eight * eigth == 1); // Restores unity thru multiplicative inverse
    ///
    /// // Multiplying near boundaries let large = ScalarF6E4::from(64) * ScalarF6E4::MAX_NEG; let small = ScalarF6E4::from(1) / large; assert!(large * small == 1);
    ///
    /// // Multiplication preserves sign according to mathematical rule let negative = ScalarF6E4::from(-1.5); assert!((eight * negative).is_negative()); // Positive × Negative = Negative assert!((negative * negative).is_positive()); // Negative × Negative = Positive
    ///
    /// // Vanished values maintain sign thru multiplication let tiny = ScalarF6E4::MIN_POS / ScalarF6E4::from(11); assert!(tiny.vanished()); let neg_tiny = tiny * ScalarF6E4::NEG_ONE; assert!(neg_tiny.vanished() && neg_tiny.is_negative());
    ///
    /// // Exploded values interact consistently with finite values let huge = ScalarF6E4::MAX * 42; assert!(huge.exploded()); assert!((1 / huge).vanished()); // Inverse of exploded is vanished assert!(((-1) / huge).is_negative()); // Signs are propogated following multiplication rule assert!((huge * ScalarF6E4::NEG_ONE).exploded()); assert!((huge * -1).is_negative());
    ///
    /// // Multiplying by zero always produces zero, even with escaped values assert!((huge * 0).is_zero()); assert!((tiny * 0).is_zero());
    ///
    /// // Vanished × Exploded yields an undefined state (magnitude indeterminate) let undefined_product = tiny * huge; assert!(undefined_product.is_undefined());
    /// ```
    pub(crate) fn scalar_multiply_scalar(&self, other: &Self) -> Self {
        if !self.is_normal() || !other.is_normal() {
            if self.is_undefined() {
                return *self;
            }
            if other.is_undefined() {
                return *other;
            }
            if self.is_infinite() && other.is_zero() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_zero() && other.is_infinite() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.is_infinite() || other.is_infinite() {
                return Self::INFINITY;
            }
            if self.is_zero() || other.is_zero() {
                return Self::ZERO;
            }
            if self.exploded() && other.vanished() {
                return Self {
                    fraction: TRANSFINITE_MULTIPLY_NEGLIGIBLE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if self.vanished() && other.exploded() {
                return Self {
                    fraction: NEGLIGIBLE_MULTIPLY_TRANSFINITE.prefix.sa(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            // Escaped × escaped/normal: the result class is fixed by magnitude-class dominance (exploded if either operand exploded, else vanished), but we carry the SIGNIFICAND thru so orientation survives — an escaped value's phase is the only information it still holds, and Circle complex orientation is built on it.
            // An escaped fraction stores its significand shifted by class (exploded `01mmm` = normal `1mmm` >> 1, vanished `001mm` = >> 2), so |a|·|b| of the raw fractions, renormalized by leading-zero count, recovers the significand product regardless of each operand's class scale.
            // Unlike the old formula-extraction (which recomputed the class from the wrapped signed product and could land on N3+ undefined), the result class here is predetermined and the fixed shift stamps the `01`/`001` prefix, so the value can never wander out of its class.
            let result_exploded = self.exploded() || other.exploded();
            let result_negative = self.is_negative() != other.is_negative();
            // Significand mantissa of each operand, as an unsigned magnitude with the leading 1 at bit FRAC-1. magnitude() folds sign (its phase survives negation), then `cycle_widen` zero-extends into the wide type and the class shift restores the significand scale: escaped classes store the significand shifted down (exploded `01mmm` = >> 1 of the normal `1mmm`, vanished `001mm` = >> 2), so shifting back up re-aligns all classes to a common leading-1 position.
            // NOTE: do NOT use inflate() here — inflate(true) applies the N0 sign decode, which is only correct for normal fractions; escaped fractions carry an explicit sign bit and would be corrupted.
            let class_shift = |v: &Self| -> isize {
                if v.exploded() { 1 } else if v.vanished() { 2 } else { 0 }
            };
            let mant_a = self.magnitude().fraction.cycle_widen().w_shl(class_shift(self));
            let mant_b = other.magnitude().fraction.cycle_widen().w_shl(class_shift(other));
            let u_product = mant_a.w_mul(mant_b);
            let leading = u_product.w_leading_zeros();
            let fb = Self::fraction_bits();
            // Extract the positive-magnitude escaped fraction: exploded places the leading 1 at bit FRAC-2 (`01`), vanished at FRAC-3 (`001`). Reading u_product as unsigned (w_leading_zeros / w_shr_logical) means the wide product wrapping into the sign bit is harmless.
            let k = if result_exploded {
                fb.wrapping_sub(leading).wrapping_add(1)
            } else {
                fb.wrapping_sub(leading).wrapping_add(2)
            };
            let pos_frac = if k >= 0 {
                u_product.w_shr_logical(k).deflate()
            } else {
                u_product.w_shl(k.wrapping_neg()).deflate()
            };
            // Apply sign via the library's own two's-complement negation. Negatives are the two's-complement of positives across ALL bits (not a top-bit XOR flip), so delegating to scalar_negate keeps escaped phase consistent with `-x`, magnitude(), and the number-line continuity, and guarantees a*(-b) == -(a*b). (A top-bit flip only agrees at phase 0, which is why the old canonical-only path never exposed the difference.)
            let mut result = Self { fraction: pos_frac, exponent: Self::ambiguous_exponent() };
            if result_negative {
                result.scalar_negate();
            }
            return result;
        }

        // AMBIG=0 native: view stored exponents as UNSIGNED cycle positions (1..2^N - 1 = normal, 0 = AMBIG). `cycle_widen` zero-extends each stored exponent into <E as Inflate>::Wide (i8→i16, ..., i128→I256) — enough headroom to detect wrap regardless of E's width. Arithmetic uses WideOps' wrapping ops. `bo` is subtracted once to undo the doubled bias from adding two biased operands.
        let pa = self.exponent.cycle_widen();
        let pb = other.exponent.cycle_widen();
        let bo = Self::binade_origin().cycle_widen();
        let max_pos = Self::max_exponent().cycle_widen();
        let min_pos = Self::min_exponent().cycle_widen();
        let w_one = E::one().cycle_widen();

        // Fast-path neg_one_normal × neg_one_normal: both inflate to -2^FRAC, their product = 2^(2*FRAC) overflows the 2*FRAC-bit Wide and wraps to 0. Compute via exponent arithmetic only. Cycle math: stored_pos = pa + pb - bo + 2 (per-op +1 plus +1 from -2^(e+1) × -2^(e+1)).
        if self.fraction == Self::neg_one_normal() && other.fraction == Self::neg_one_normal() {
            let stored_pos = w_one.w_add(w_one).w_add(pa).w_add(pb).w_sub(bo);
            if stored_pos > max_pos {
                return Self {
                    fraction: Self::pos_one_exploded(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            if stored_pos < min_pos {
                return Self {
                    fraction: Self::pos_one_vanished(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: Self::pos_one_normal(),
                exponent: stored_pos.deflate(),
            };
        }
        // x86 implementation note: Why this code is NOT signless Spirix's fundamental multiply is signless: two's-complement inflated fractions multiplied with arithmetic shift right for floor rounding, no magnitude/sign decomposition anywhere. In the Verilog target this is exactly how it's implemented — one multiplier, one barrel shifter, no sign-separation datapath, no cmov analog. Here in Rust/x86 we're forced into a hybrid because of a platform bit-width accident. Inflated fractions occupy FRAC+1 bits signed (magnitude reaches 2^FRAC at the ±1.0 boundary), so their product needs 2*FRAC+2 bits. Our `Wide` type is only 2*FRAC bits (i16 for i8 stored, i32 for i16, ..., I256 for i128 — Rust has no 2N+2-bit primitive at any width). That leaves us exactly one bit short in the worst case, and the signed multiply wraps. The wrap happens to be INVISIBLE for the main-path byte extraction (shift = FRAC - leading ≤ FRAC, so the 2^W wrap correction is 0 mod 2^FRAC), which lets us keep signed arithmetic + arith shr = floor for that path. But for exploded/vanished (shift > FRAC), the wrap correction doesn't vanish mod 2^FRAC, so we reluctantly fall back to the magnitude-dance (compute |p|, logical shr, XOR a sign-flip mask). Those paths represent "too big/small to represent normally" so the magnitude precision is already lossy — rounding mode is moot there. In Verilog all of this dissolves: hardware arith shr is free, register width is whatever we declare, and signed/unsigned interpretation is just wire routing. The code below is overhead paid for x86 ISA quirks, not inherent algorithmic cost.
        let p_signed = self
            .fraction
            .inflate(true)
            .w_mul(other.fraction.inflate(true));
        let fb = Self::fraction_bits();

        // u_product = |p_signed|. LLVM compiles this conditional to `neg + cmov` (branchless at asm level) — no wider bit-ops formulation beats that.
        let expect_negative = self.is_negative() != other.is_negative();
        let u_product = if expect_negative {
            p_signed.w_neg()
        } else {
            p_signed
        };
        let leading = u_product.w_leading_zeros();

        // Exploded/vanished extract: u_product (magnitude) >> k logical gives the positive escaped fraction. Magnitude-based so wrap doesn't bite; rounding mode is moot here.
        // Sign is applied by the library's two's-complement negation (scalar_negate), NOT a top-bit XOR flip: negatives are the two's-complement of positives across all bits, so a XOR flip only agrees at phase 0. Matching scalar_negate keeps escaped phase consistent with -x / magnitude() and the escaped-operand path above.
        let extract_escaped = |k: isize| -> F {
            let pos = if k >= 0 {
                u_product.w_shr_logical(k).deflate()
            } else {
                u_product.w_shl(k.wrapping_neg()).deflate()
            };
            if expect_negative {
                let mut s = Self { fraction: pos, exponent: Self::ambiguous_exponent() };
                s.scalar_negate();
                s.fraction
            } else {
                pos
            }
        };

        // AMBIG=0 native wrap detection: stored_pos = pa + pb - bo + 1 - leading. Single compare against max_pos / min_pos catches overflow and underflow (including the tail cases where the cycle position would land on AMBIG = 0). `leading` is isize from w_leading_zeros — funnel thru E to cycle_widen for the type-correct Wide value.
        let leading_e: E = leading.as_();
        let w_leading = leading_e.cycle_widen();
        let stored_pos = pa.w_add(pb).w_sub(bo).w_add(w_one).w_sub(w_leading);
        if stored_pos > max_pos {
            let k = fb.wrapping_sub(leading).wrapping_add(1);
            return Self {
                fraction: extract_escaped(k),
                exponent: Self::ambiguous_exponent(),
            };
        }
        if stored_pos < min_pos {
            let k = fb.wrapping_sub(leading).wrapping_add(2);
            return Self {
                fraction: extract_escaped(k),
                exponent: Self::ambiguous_exponent(),
            };
        }
        let exponent: E = stored_pos.deflate();

        // Main path: signed arith shr of p_signed. For k <= FRAC (always true here since k_main = FRAC - leading ≤ FRAC), the wrap correction vanishes mod 2^FRAC, so the byte is correct. Arith shr = floor rounding.
        let k_main = fb.wrapping_sub(leading);
        let fraction = if k_main >= 0 {
            p_signed.w_shr(k_main).deflate()
        } else {
            p_signed.w_shl(k_main.wrapping_neg()).deflate()
        };
        // Boundary: signed extract of magnitude 2^(FRAC-1) (= POS_ONE_NORMAL) with negative result lands at the asymmetric two's complement edge. Reroute to NEG_ONE_NORMAL at exp-1 (with AMBIGUOUS check).
        if expect_negative && fraction == Self::pos_one_normal() {
            let exp_m1 = exponent.wrapping_sub(&1u8.as_());
            if exp_m1 == Self::ambiguous_exponent() {
                return Self {
                    fraction: Self::neg_one_vanished(),
                    exponent: Self::ambiguous_exponent(),
                };
            }
            return Self {
                fraction: Self::neg_one_normal(),
                exponent: exp_m1,
            };
        }
        Self { fraction, exponent }
    }
}
